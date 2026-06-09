use crate::components::toast::use_toast;
use crate::components::{BottomNav, EmptySessionList, NavItem, PageLoading, SessionCard};
use crate::models::api::ApiEnvelope;
use crate::models::dashboard::{DashboardData, PendingSession};
use crate::state::{handle_invoke_error, use_auth_state};
use crate::utils::dates::today as get_today_date;
use crate::utils::tauri::{invoke, log};
use leptos::prelude::*;
use serde::Deserialize;
use wasm_bindgen_futures::JsFuture;

/// `{ data: { upcoming: [...] } }` from `api_get_upcoming_sessions`.
#[derive(Debug, Clone, Deserialize)]
struct UpcomingEnvelope {
    data: UpcomingInner,
}

#[derive(Debug, Clone, Deserialize)]
struct UpcomingInner {
    #[serde(default)]
    upcoming: Vec<PendingSession>,
}

/// Activity history page is `{ activities: [...], no_of_pages: N, ... }`.
#[derive(Debug, Clone, Deserialize)]
struct ActivityHistoryResponse {
    #[serde(default)]
    activities: Vec<serde_json::Value>,
}

/// True if an activity entry's start date falls on `today_ymd` (`YYYY-MM-DD`).
/// `start_date_time` arrives as e.g. `"2026-06-08 19:33:25"`.
fn entry_is_today(entry: &serde_json::Value, today_ymd: &str) -> bool {
    let starts_today = |key: &str| {
        entry
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.starts_with(today_ymd))
            .unwrap_or(false)
    };
    starts_today("start_date_time") || starts_today("date")
}

/// Render one activity-history entry as a card. Shared by "Completed Today"
/// (history filtered to today) and the full history list, so the two can never
/// disagree about what happened today.
fn history_card(entry: &serde_json::Value) -> impl IntoView {
    let get_str = |key: &str| entry.get(key).and_then(|v| v.as_str());
    let session_name = get_str("workout_type")
        .or_else(|| get_str("session_name"))
        .unwrap_or("Session")
        .to_string();
    let location = get_str("location_name").unwrap_or("").to_string();
    let calories = get_str("total_burnt")
        .or_else(|| get_str("cal_burnt"))
        .map(|c| {
            let c = c.trim();
            if c.is_empty() {
                "--".to_string()
            } else if c.contains(char::is_alphabetic) {
                c.to_string() // already has units, e.g. "192 Cal"
            } else {
                format!("{} cal", c)
            }
        })
        .unwrap_or_else(|| "--".to_string());

    let raw_date = get_str("display_date")
        .or_else(|| get_str("date"))
        .unwrap_or("")
        .to_string();
    let time = get_str("start_date_time")
        .and_then(|dt| dt.split(' ').nth(1))
        .unwrap_or("")
        .to_string();
    // `display_date` often already includes the time (e.g. "...19:33:25"), so
    // only append the time when the date string doesn't already carry one.
    let date_line = if raw_date.contains(':') || time.is_empty() {
        raw_date
    } else {
        format!("{} at {}", raw_date, time)
    };

    view! {
        <div class="history-card">
            <div class="history-card-header">
                <span class="history-session-name">{session_name}</span>
                <span class="history-calories">{calories}</span>
            </div>
            <div class="history-card-details">
                <span class="history-location">{location}</span>
                <span class="history-date">{date_line}</span>
            </div>
        </div>
    }
}

/// Activity history page with upcoming and completed sessions, filterable by date range.
#[component]
pub fn SessionsPage() -> impl IntoView {
    let auth = use_auth_state();
    let toast = use_toast();
    let pending_sessions: RwSignal<Vec<PendingSession>> = RwSignal::new(Vec::new());
    let session_history: RwSignal<Vec<serde_json::Value>> = RwSignal::new(Vec::new());
    // Distinguishes "history still loading" from "genuinely no history".
    let history_loaded = RwSignal::new(false);
    let history_filter = RwSignal::new("7days".to_string()); // "7days", "30days", "all"
    let loading = RwSignal::new(true);
    let error: RwSignal<Option<String>> = RwSignal::new(None);

    // Remove a cancelled session from the upcoming list.
    let on_session_cancelled = Callback::new(move |session_id: String| {
        log(&format!(
            "[Sessions] Removing cancelled session: {}",
            session_id
        ));
        pending_sessions.update(|sessions| {
            sessions.retain(|s| s.session_record_id.as_deref() != Some(&session_id));
        });
    });

    // Fetch sessions on mount via Tauri backend
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            log("[Sessions] Fetching sessions data...");

            // "Completed Today" is derived from the activity history below, so
            // it can never disagree with the history list. Here we only need
            // upcoming bookings.
            let upcoming_promise = invoke("api_get_upcoming_sessions", crate::json_args!({}));

            // Upcoming bookings across the next few days.
            match JsFuture::from(upcoming_promise).await {
                Ok(result) => {
                    if let Ok(env) = serde_wasm_bindgen::from_value::<UpcomingEnvelope>(result) {
                        log(&format!(
                            "[Sessions] {} upcoming sessions",
                            env.data.upcoming.len()
                        ));
                        pending_sessions.set(env.data.upcoming);
                    }

                    // Fallback for empty multi-day list — pull today's
                    // pending from the dashboard.
                    if pending_sessions.get_untracked().is_empty() {
                        let fallback_args = serde_wasm_bindgen::to_value(
                            &serde_json::json!({ "currentDate": get_today_date() }),
                        )
                        .unwrap();
                        if let Ok(result) =
                            JsFuture::from(invoke("api_get_dashboard", fallback_args)).await
                        {
                            if let Ok(env) =
                                serde_wasm_bindgen::from_value::<ApiEnvelope<DashboardData>>(result)
                            {
                                if let Some(pending) =
                                    env.data.and_then(|d| d.todays_pending_sessions)
                                {
                                    pending_sessions.set(pending);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[Sessions] Upcoming error: {:?}", e));
                    if handle_invoke_error(&e, auth, toast).await {
                        loading.set(false);
                        return;
                    }
                    error.set(Some("Failed to load sessions.".to_string()));
                }
            }

            loading.set(false);
        });
    });

    // Fetch session history from API (ActivityByLifeTime endpoint)
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            log("[Sessions] Fetching activity history from API...");

            // Pass empty session_type for all types (Android app uses "" not "all")
            let args = crate::json_args!({
                "pageNo": 1,
                "pageLimit": 100,
            });
            let promise = invoke("api_get_activity_history", args);

            match JsFuture::from(promise).await {
                Ok(result) => {
                    match serde_wasm_bindgen::from_value::<ActivityHistoryResponse>(result) {
                        Ok(resp) => {
                            log(&format!(
                                "[Sessions] {} activity entries from API",
                                resp.activities.len()
                            ));
                            session_history.set(resp.activities);
                        }
                        Err(e) => {
                            log(&format!("[Sessions] Activity parse error: {:?}", e));
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[Sessions] Activity history error: {:?}", e));
                    if handle_invoke_error(&e, auth, toast).await {
                        return;
                    }
                    // Fall back to local storage
                    let args = crate::json_args!({});
                    let promise = invoke("get_session_history", args);
                    if let Ok(result) = JsFuture::from(promise).await {
                        if let Ok(history) =
                            serde_wasm_bindgen::from_value::<Vec<serde_json::Value>>(result)
                        {
                            log(&format!(
                                "[Sessions] Fallback: {} local history entries",
                                history.len()
                            ));
                            session_history.set(history);
                        }
                    }
                }
            }
            history_loaded.set(true);
        });
    });

    view! {
        <div class="sessions-page">
            {move || loading.get().then(|| view! { <PageLoading /> })}

            <div class="sessions-header">
                <h1 class="page-title">"My Sessions"</h1>
            </div>

            <div class="sessions-content">
                // Pending sessions
                <div class="section">
                    <h2 class="section-title">"Upcoming"</h2>
                    {move || {
                        let sessions = pending_sessions.get();
                        if sessions.is_empty() {
                            view! {
                                <EmptySessionList message="No upcoming sessions".to_string() />
                            }.into_any()
                        } else {
                            view! {
                                <div class="session-list">
                                    {sessions.into_iter().map(|session| {
                                        view! {
                                            <SessionCard
                                                session=session
                                                on_cancel=on_session_cancelled
                                            />
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>

                // Completed today — derived from the activity history so it
                // always agrees with the history list below.
                <div class="section">
                    <h2 class="section-title">"Completed Today"</h2>
                    {move || {
                        if !history_loaded.get() {
                            return view! {
                                <EmptySessionList message="Loading today's sessions...".to_string() />
                            }.into_any();
                        }
                        let today = get_today_date();
                        let today_sessions: Vec<_> = session_history.get()
                            .into_iter()
                            .filter(|e| entry_is_today(e, &today))
                            .collect();
                        if today_sessions.is_empty() {
                            view! {
                                <EmptySessionList message="No completed sessions today".to_string() />
                            }.into_any()
                        } else {
                            view! {
                                <div class="history-list">
                                    {today_sessions.iter().map(history_card).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>

                // Session history
                <div class="section">
                    <h2 class="section-title">"Session History"</h2>
                    <div class="history-filters">
                        <button
                            class=move || if history_filter.get() == "7days" { "filter-btn active" } else { "filter-btn" }
                            on:click=move |_| history_filter.set("7days".to_string())
                        >
                            "Last 7 Days"
                        </button>
                        <button
                            class=move || if history_filter.get() == "30days" { "filter-btn active" } else { "filter-btn" }
                            on:click=move |_| history_filter.set("30days".to_string())
                        >
                            "Last 30 Days"
                        </button>
                        <button
                            class=move || if history_filter.get() == "all" { "filter-btn active" } else { "filter-btn" }
                            on:click=move |_| history_filter.set("all".to_string())
                        >
                            "All"
                        </button>
                    </div>
                    {move || {
                        let filter = history_filter.get();
                        let history = session_history.get();

                        if !history_loaded.get() && history.is_empty() {
                            view! {
                                <EmptySessionList message="Loading activity history...".to_string() />
                            }.into_any()
                        } else {
                            // Client-side window over the loaded entries until the
                            // endpoint supports true date-range filtering.
                            let filtered: Vec<_> = history.into_iter()
                                .take(match filter.as_str() {
                                    "7days" => 20,   // Approx 2-3 sessions per day
                                    "30days" => 100, // Approx 3 sessions per day
                                    _ => 1000,       // All
                                })
                                .collect();

                            if filtered.is_empty() {
                                view! {
                                    <EmptySessionList message="No session history yet".to_string() />
                                }.into_any()
                            } else {
                                view! {
                                    <div class="history-list">
                                        {filtered.iter().map(history_card).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }
                    }}
                </div>

                {move || error.get().map(|e| view! {
                    <div class="error-message">{e}</div>
                })}
            </div>

            <BottomNav active=Signal::derive(|| NavItem::Sessions) />
        </div>
    }
}
