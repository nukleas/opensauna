use crate::components::toast::use_toast;
use crate::components::{
    ActiveSessionView, BottomNav, Button, EmptySessionList, IconLogOut, NavItem, PageLoading,
    SessionCard,
};
use crate::models::api::ApiEnvelope;
use crate::models::dashboard::{DashboardData, PendingSession};
use crate::state::{handle_invoke_error, use_auth_state, use_session_tracking_state};
use crate::utils::dates::today as get_today_date;
use crate::utils::nav::go as navigate_to;
use crate::utils::tauri::{invoke, log};
use leptos::prelude::*;
use serde::Deserialize;
use wasm_bindgen_futures::JsFuture;

/// Inner shape of `api_get_upcoming_sessions` — `{ data: { upcoming: [...] } }`.
#[derive(Debug, Clone, Deserialize)]
struct UpcomingEnvelope {
    data: UpcomingInner,
}

#[derive(Debug, Clone, Deserialize)]
struct UpcomingInner {
    #[serde(default)]
    upcoming: Vec<PendingSession>,
}

/// Home screen showing today's sessions, summary stats, and an active session banner.
#[component]
pub fn DashboardPage() -> impl IntoView {
    let auth = use_auth_state();
    let toast = use_toast();
    let session_state = use_session_tracking_state();

    let dashboard_data: RwSignal<Option<DashboardData>> = RwSignal::new(None);
    let all_upcoming_sessions: RwSignal<Vec<PendingSession>> = RwSignal::new(Vec::new());
    let loading = RwSignal::new(true);
    let error: RwSignal<Option<String>> = RwSignal::new(None);

    // Quick book preferences
    let has_quick_book_prefs = RwSignal::new(false);
    let quick_book_session_type = RwSignal::new(String::new());

    // Restore active session on mount
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            session_state.restore_session().await;
        });
    });

    // Drop a cancelled session from the displayed dashboard.
    let on_session_cancelled = Callback::new(move |session_id: String| {
        log(&format!(
            "[Dashboard] Removing cancelled session: {}",
            session_id
        ));
        dashboard_data.update(|data| {
            if let Some(ref mut d) = data {
                if let Some(ref mut sessions) = d.todays_pending_sessions {
                    sessions.retain(|s| s.session_record_id.as_deref() != Some(&session_id));
                }
            }
        });
    });

    // Begin tracking a session the user just hit "Start" on.
    let on_session_started = Callback::new(move |pending: PendingSession| {
        log(&format!(
            "[Dashboard] Starting session: {:?}",
            pending.session_record_id
        ));
        wasm_bindgen_futures::spawn_local(async move {
            match session_state.start_session(&pending).await {
                Ok(()) => log("[Dashboard] Session started successfully"),
                Err(e) => log(&format!("[Dashboard] Failed to start session: {}", e)),
            }
        });
    });

    // Fetch dashboard data on mount via Tauri backend
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            log("[Dashboard] Fetching dashboard data...");

            let args = serde_wasm_bindgen::to_value(
                &serde_json::json!({ "currentDate": get_today_date() }),
            )
            .unwrap();
            let promise = invoke("api_get_dashboard", args);

            match JsFuture::from(promise).await {
                Ok(result) => {
                    match serde_wasm_bindgen::from_value::<ApiEnvelope<DashboardData>>(result) {
                        Ok(env) => match env.data {
                            Some(dashboard) => {
                                log(&format!(
                                    "[Dashboard] Parsed: sessions={:?}",
                                    dashboard.todays_pending_sessions.as_ref().map(|s| s.len())
                                ));
                                dashboard_data.set(Some(dashboard));
                            }
                            None => {
                                log("[Dashboard] No `data` field in response");
                                error.set(Some(
                                    "Failed to load dashboard. Pull down to refresh.".to_string(),
                                ));
                            }
                        },
                        Err(e) => {
                            log(&format!("[Dashboard] Deserialize error: {:?}", e));
                            error.set(Some(
                                "Failed to load dashboard. Pull down to refresh.".to_string(),
                            ));
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[Dashboard] Error: {:?}", e));
                    if handle_invoke_error(&e, auth, toast).await {
                        loading.set(false);
                        return;
                    }
                    error.set(Some(
                        "Failed to load dashboard. Pull down to refresh.".to_string(),
                    ));
                }
            }

            loading.set(false);
        });
    });

    // Fetch all upcoming sessions (not just today)
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            log("[Dashboard] Fetching all upcoming sessions...");

            let args = crate::json_args!({});
            let promise = invoke("api_get_upcoming_sessions", args);

            match JsFuture::from(promise).await {
                Ok(result) => {
                    if let Ok(env) = serde_wasm_bindgen::from_value::<UpcomingEnvelope>(result) {
                        log(&format!(
                            "[Dashboard] Got {} upcoming sessions",
                            env.data.upcoming.len()
                        ));
                        all_upcoming_sessions.set(env.data.upcoming);
                    }
                }
                Err(e) => {
                    log(&format!(
                        "[Dashboard] Upcoming sessions error (non-fatal): {:?}",
                        e
                    ));
                    let _ = handle_invoke_error(&e, auth, toast).await;
                }
            }
        });
    });

    // Check for quick book preferences
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            let args = crate::json_args!({});

            // Check for preferred location
            let loc_promise = invoke("get_preferred_location", args.clone());
            let type_promise = invoke("get_preferred_session_type", args);

            let has_location = match JsFuture::from(loc_promise).await {
                Ok(result) => serde_wasm_bindgen::from_value::<Option<(String, String)>>(result)
                    .ok()
                    .flatten()
                    .is_some(),
                Err(_) => false,
            };

            let session_type_display = match JsFuture::from(type_promise).await {
                Ok(result) => serde_wasm_bindgen::from_value::<Option<(String, String)>>(result)
                    .ok()
                    .flatten()
                    .map(|(_, display)| display),
                Err(_) => None,
            };

            if has_location && session_type_display.is_some() {
                has_quick_book_prefs.set(true);
                quick_book_session_type.set(session_type_display.unwrap_or_default());
                log("[Dashboard] Quick book preferences found");
            }
        });
    });

    let on_book_now = move || {
        navigate_to("/book");
    };

    // Two-step logout so a stray tap doesn't end the session.
    let confirming_logout = RwSignal::new(false);
    let on_logout = move || {
        wasm_bindgen_futures::spawn_local(async move {
            auth.logout().await;
            navigate_to("/login");
        });
    };

    view! {
        <div class="dashboard-page">
            // Active session overlay (takes over the screen when there's an active session)
            {move || {
                session_state.active_session.get().map(|session| {
                    view! {
                        <ActiveSessionView
                            session=session
                        />
                    }
                })
            }}

            {move || loading.get().then(|| view! { <PageLoading /> })}

            <div class="dashboard-header">
                <h1 class="dashboard-title">"Welcome Back!"</h1>
                {move || if confirming_logout.get() {
                    view! {
                        <div class="logout-confirm">
                            <button class="logout-btn logout-btn-danger" on:click=move |_| on_logout()>
                                "Log out?"
                            </button>
                            <button class="logout-btn" on:click=move |_| confirming_logout.set(false)>
                                "Cancel"
                            </button>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <button class="logout-btn" on:click=move |_| confirming_logout.set(true)>
                            <IconLogOut size=crate::components::icons::IconSize::Sm />
                            "Logout"
                        </button>
                    }.into_any()
                }}
            </div>

            <div class="dashboard-content">
                // Summary stats
                <div class="summary-section">
                    {move || {
                        dashboard_data.get().map(|data| {
                            let summary = data.summary.unwrap_or_default();
                            view! {
                                <div class="stats-grid">
                                    <div class="stat-card">
                                        <div class="stat-value">{summary.sessions_count()}</div>
                                        <div class="stat-label">"Total Sessions"</div>
                                    </div>
                                    <div class="stat-card">
                                        <div class="stat-value">{summary.calories_burned()}</div>
                                        <div class="stat-label">"Calories Burned"</div>
                                    </div>
                                    <div class="stat-card">
                                        <div class="stat-value">{summary.streak()}</div>
                                        <div class="stat-label">"Day Streak"</div>
                                    </div>
                                </div>
                            }
                        })
                    }}
                </div>

                // Quick book buttons
                <div class="quick-book-section">
                    <Button
                        label="Book a Session"
                        on_click=on_book_now
                    />
                    {move || {
                        let has_prefs = has_quick_book_prefs.get();
                        let session_type = quick_book_session_type.get();
                        has_prefs.then(|| {
                            let label = format!("Quick Book {}", session_type);
                            view! {
                                <Button
                                    label=label
                                    variant="secondary".to_string()
                                    on_click=move || navigate_to("/quick-book")
                                />
                            }
                        })
                    }}
                </div>

                // All upcoming sessions (today + future)
                <div class="section">
                    <h2 class="section-title">"Upcoming Sessions"</h2>
                    {move || {
                        // First try all_upcoming_sessions, fall back to today's from dashboard
                        let upcoming = all_upcoming_sessions.get();
                        let today_sessions = dashboard_data.get()
                            .and_then(|d| d.todays_pending_sessions)
                            .unwrap_or_default();

                        // Use all upcoming if available, otherwise fall back to today's
                        let sessions = if !upcoming.is_empty() {
                            upcoming
                        } else {
                            today_sessions
                        };

                        if sessions.is_empty() {
                            view! {
                                <EmptySessionList message="No upcoming sessions. Book one now!".to_string() />
                            }.into_any()
                        } else {
                            // Group sessions by date
                            let mut grouped: std::collections::BTreeMap<String, Vec<PendingSession>> = std::collections::BTreeMap::new();
                            for session in sessions {
                                let date = session.date.clone().unwrap_or_else(|| "Unknown Date".to_string());
                                grouped.entry(date).or_default().push(session);
                            }

                            view! {
                                <div class="session-list">
                                    {grouped.into_iter().map(|(date, sessions)| {
                                        view! {
                                            <div class="session-date-group">
                                                <h3 class="session-date-header">{date}</h3>
                                                {sessions.into_iter().map(|session| {
                                                    view! {
                                                        <SessionCard
                                                            session=session
                                                            on_cancel=on_session_cancelled
                                                            on_start=on_session_started
                                                        />
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>

                // Completed sessions
                <div class="section">
                    <h2 class="section-title">"Completed Today"</h2>
                    {move || {
                        match dashboard_data.get() {
                            Some(data) => {
                                match data.todays_completed_sessions {
                                    Some(sessions) if !sessions.is_empty() => {
                                        view! {
                                            <div class="session-list">
                                                {sessions.into_iter().map(|session| {
                                                    view! { <SessionCard session=session /> }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }.into_any()
                                    }
                                    _ => view! {
                                        <EmptySessionList message="No completed sessions today".to_string() />
                                    }.into_any()
                                }
                            }
                            None => view! { <div></div> }.into_any()
                        }
                    }}
                </div>

                {move || error.get().map(|e| view! {
                    <div class="error-message">{e}</div>
                })}
            </div>

            <BottomNav active=Signal::derive(|| NavItem::Home) />
        </div>
    }
}
