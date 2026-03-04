use crate::components::{BottomNav, EmptySessionList, NavItem, PageLoading, SessionCard};
use crate::models::dashboard::PendingSession;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    fn invoke(cmd: &str, args: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[component]
pub fn SessionsPage() -> impl IntoView {
    let pending_sessions: RwSignal<Vec<PendingSession>> = RwSignal::new(Vec::new());
    let completed_sessions: RwSignal<Vec<PendingSession>> = RwSignal::new(Vec::new());
    let session_history: RwSignal<Vec<serde_json::Value>> = RwSignal::new(Vec::new());
    let history_filter = RwSignal::new("7days".to_string()); // "7days", "30days", "all"
    let loading = RwSignal::new(true);
    let error: RwSignal<Option<String>> = RwSignal::new(None);

    // Signal to track which session was cancelled (set by SessionCard)
    let (cancelled_session, set_cancelled_session) = signal::<Option<String>>(None);

    // Remove cancelled session from list when signal changes
    Effect::new(move |_| {
        if let Some(session_id) = cancelled_session.get() {
            log(&format!(
                "[Sessions] Removing cancelled session: {}",
                session_id
            ));
            pending_sessions.update(|sessions| {
                sessions.retain(|s| s.session_record_id.as_deref() != Some(&session_id));
            });
            set_cancelled_session.set(None);
        }
    });

    // Fetch sessions on mount via Tauri backend
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            log("[Sessions] Fetching sessions data...");

            let dashboard_args =
                serde_wasm_bindgen::to_value(&serde_json::json!({ "currentDate": get_today_date() }))
                    .unwrap();
            let empty_args = serde_wasm_bindgen::to_value(&serde_json::json!({})).unwrap();

            // Fetch dashboard data for today's sessions
            let dashboard_promise = invoke("api_get_dashboard", dashboard_args.clone());
            // Fetch all upcoming sessions
            let upcoming_promise = invoke("api_get_upcoming_sessions", empty_args);

            // Process dashboard response
            match JsFuture::from(dashboard_promise).await {
                Ok(result) => {
                    if let Ok(response) =
                        serde_wasm_bindgen::from_value::<serde_json::Value>(result)
                    {
                        log("[Sessions] Got dashboard response");
                        if let Some(data) = response.get("data") {
                            if let Some(completed_json) = data.get("todays_completed_sessions") {
                                if let Ok(completed) = serde_json::from_value::<Vec<PendingSession>>(
                                    completed_json.clone(),
                                ) {
                                    log(&format!(
                                        "[Sessions] {} completed sessions today",
                                        completed.len()
                                    ));
                                    completed_sessions.set(completed);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[Sessions] Dashboard error: {:?}", e));
                }
            }

            // Process upcoming sessions
            match JsFuture::from(upcoming_promise).await {
                Ok(result) => {
                    if let Ok(response) =
                        serde_wasm_bindgen::from_value::<serde_json::Value>(result)
                    {
                        if let Some(data) = response.get("data") {
                            if let Some(upcoming_json) = data.get("upcoming") {
                                if let Ok(upcoming) = serde_json::from_value::<Vec<PendingSession>>(
                                    upcoming_json.clone(),
                                ) {
                                    log(&format!(
                                        "[Sessions] {} upcoming sessions",
                                        upcoming.len()
                                    ));
                                    pending_sessions.set(upcoming);
                                }
                            }
                        }
                        // Fallback: if no upcoming in response, check for today's pending from earlier
                        if pending_sessions.get().is_empty() {
                            let fallback_args =
                                serde_wasm_bindgen::to_value(&serde_json::json!({ "currentDate": get_today_date() })).unwrap();
                            if let Ok(result) =
                                JsFuture::from(invoke("api_get_dashboard", fallback_args)).await
                            {
                                if let Ok(resp) =
                                    serde_wasm_bindgen::from_value::<serde_json::Value>(result)
                                {
                                    if let Some(data) = resp.get("data") {
                                        if let Some(pending_json) =
                                            data.get("todays_pending_sessions")
                                        {
                                            if let Ok(pending) =
                                                serde_json::from_value::<Vec<PendingSession>>(
                                                    pending_json.clone(),
                                                )
                                            {
                                                pending_sessions.set(pending);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[Sessions] Upcoming error: {:?}", e));
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

            // Request up to 100 activities, all session types
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "pageNo": 1,
                "pageLimit": 100,
                "sessionType": "all"
            }))
            .unwrap();
            let promise = invoke("api_get_activity_history", args);

            match JsFuture::from(promise).await {
                Ok(result) => {
                    if let Ok(response) =
                        serde_wasm_bindgen::from_value::<serde_json::Value>(result)
                    {
                        log(&format!(
                            "[Sessions] Activity history response: {:?}",
                            response
                        ));

                        // The API returns data in a specific structure, extract the activities
                        if let Some(data) = response.get("data") {
                            if let Some(activities) = data.as_array() {
                                log(&format!(
                                    "[Sessions] {} activity entries from API",
                                    activities.len()
                                ));
                                session_history.set(activities.clone());
                            }
                        } else if let Some(activities) = response.as_array() {
                            // Maybe it's a direct array
                            log(&format!(
                                "[Sessions] {} activity entries (direct array)",
                                activities.len()
                            ));
                            session_history.set(activities.clone());
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[Sessions] Activity history error: {:?}", e));
                    // Fall back to local storage
                    let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).unwrap();
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
                                                show_cancel=true
                                                on_cancelled=set_cancelled_session
                                            />
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>

                // Completed sessions today
                <div class="section">
                    <h2 class="section-title">"Completed Today"</h2>
                    {move || {
                        let sessions = completed_sessions.get();
                        if sessions.is_empty() {
                            view! {
                                <EmptySessionList message="No completed sessions today".to_string() />
                            }.into_any()
                        } else {
                            view! {
                                <div class="session-list">
                                    {sessions.into_iter().map(|session| {
                                        view! { <SessionCard session=session /> }
                                    }).collect::<Vec<_>>()}
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
                        let history = session_history.get();
                        let filter = history_filter.get();

                        if history.is_empty() {
                            view! {
                                <EmptySessionList message="Loading activity history...".to_string() />
                            }.into_any()
                        } else {
                            // For now, show all and let API handle pagination
                            // We'll filter client-side by keeping only recent entries
                            let filtered: Vec<_> = history.into_iter()
                                .take(match filter.as_str() {
                                    "7days" => 20,   // Approx 2-3 sessions per day
                                    "30days" => 100, // Approx 3 sessions per day
                                    _ => 1000,       // All
                                })
                                .collect();

                            if filtered.is_empty() {
                                view! {
                                    <EmptySessionList message="No sessions in this time period".to_string() />
                                }.into_any()
                            } else {
                                view! {
                                    <div class="history-list">
                                        {filtered.into_iter().map(|entry| {
                                            // API fields: workout_type, total_burnt, display_date, location_name, start_date_time, end_date_time
                                            let session_name = entry.get("workout_type")
                                                .and_then(|v| v.as_str())
                                                .or_else(|| entry.get("session_name").and_then(|v| v.as_str()))
                                                .unwrap_or("Session")
                                                .to_string();
                                            let location = entry.get("location_name")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string();
                                            // API returns total_burnt as calories (string)
                                            let calories = entry.get("total_burnt")
                                                .and_then(|v| v.as_str())
                                                .or_else(|| entry.get("total_burnt").and_then(|v| v.as_i64()).map(|_| ""))
                                                .map(|c| if c.is_empty() { "--".to_string() } else { format!("{} cal", c) })
                                                .unwrap_or_else(|| "--".to_string());
                                            let date = entry.get("display_date")
                                                .and_then(|v| v.as_str())
                                                .or_else(|| entry.get("date").and_then(|v| v.as_str()))
                                                .unwrap_or("")
                                                .to_string();
                                            // Get start time for display
                                            let time = entry.get("start_date_time")
                                                .and_then(|v| v.as_str())
                                                .map(|dt| {
                                                    // Extract time portion if full datetime
                                                    if let Some(time_part) = dt.split(' ').nth(1) {
                                                        time_part.to_string()
                                                    } else {
                                                        dt.to_string()
                                                    }
                                                })
                                                .unwrap_or_default();

                                            view! {
                                                <div class="history-card">
                                                    <div class="history-card-header">
                                                        <span class="history-session-name">{session_name}</span>
                                                        <span class="history-calories">{calories}</span>
                                                    </div>
                                                    <div class="history-card-details">
                                                        <span class="history-location">{location}</span>
                                                        <span class="history-date">{date}{if !time.is_empty() { format!(" at {}", time) } else { String::new() }}</span>
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
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

/// Get today's date in YYYY-MM-DD format
fn get_today_date() -> String {
    let now = js_sys::Date::new_0();
    let year = now.get_full_year();
    let month = now.get_month() + 1; // 0-indexed
    let day = now.get_date();
    format!("{:04}-{:02}-{:02}", year, month, day)
}
