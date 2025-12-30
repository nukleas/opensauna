use leptos::prelude::*;
use leptos::web_sys;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use crate::components::{BottomNav, NavItem, SessionCard, EmptySessionList, PageLoading, Button};
use crate::state::use_auth_state;
use crate::models::dashboard::DashboardData;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    fn invoke(cmd: &str, args: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn navigate_to(path: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(path);
    }
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let auth = use_auth_state();

    let dashboard_data: RwSignal<Option<DashboardData>> = RwSignal::new(None);
    let loading = RwSignal::new(true);
    let error: RwSignal<Option<String>> = RwSignal::new(None);

    // Fetch dashboard data on mount via Tauri backend
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            log("[Dashboard] Fetching dashboard data...");

            let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).unwrap();
            let promise = invoke("api_get_dashboard", args);

            match JsFuture::from(promise).await {
                Ok(result) => {
                    // Parse the response - it's a DashboardResponse with data field
                    match serde_wasm_bindgen::from_value::<serde_json::Value>(result) {
                        Ok(response) => {
                            log(&format!("[Dashboard] Response: {}", serde_json::to_string_pretty(&response).unwrap_or_default()));

                            if let Some(data) = response.get("data") {
                                match serde_json::from_value::<DashboardData>(data.clone()) {
                                    Ok(dashboard) => {
                                        log(&format!("[Dashboard] Parsed: sessions={:?}, summary={:?}",
                                            dashboard.todays_pending_sessions.as_ref().map(|s| s.len()),
                                            dashboard.summary));
                                        dashboard_data.set(Some(dashboard));
                                    }
                                    Err(e) => {
                                        log(&format!("[Dashboard] Parse DashboardData error: {}", e));
                                        error.set(Some(format!("Parse error: {}", e)));
                                    }
                                }
                            } else {
                                log("[Dashboard] No 'data' field in response");
                                error.set(Some("No data in response".to_string()));
                            }
                        }
                        Err(e) => {
                            log(&format!("[Dashboard] Deserialize error: {:?}", e));
                            error.set(Some(format!("Deserialize error: {:?}", e)));
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[Dashboard] Error: {:?}", e));
                    let err_str = js_sys::JSON::stringify(&e)
                        .map(|s| s.as_string().unwrap_or_default())
                        .unwrap_or_else(|_| format!("{:?}", e));
                    error.set(Some(format!("Failed to load dashboard: {}", err_str)));
                }
            }

            loading.set(false);
        });
    });

    let on_book_now = move || {
        navigate_to("/book");
    };

    let on_logout = move || {
        wasm_bindgen_futures::spawn_local(async move {
            auth.logout().await;
            navigate_to("/login");
        });
    };

    view! {
        <div class="dashboard-page">
            {move || loading.get().then(|| view! { <PageLoading /> })}

            <div class="dashboard-header">
                <h1 class="dashboard-title">"Welcome Back!"</h1>
                <button class="logout-btn" on:click=move |_| on_logout()>
                    "Logout"
                </button>
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

                // Quick book button
                <div class="quick-book-section">
                    <Button
                        label="Book a Session"
                        on_click=on_book_now
                    />
                </div>

                // Today's pending sessions
                <div class="section">
                    <h2 class="section-title">"Upcoming Sessions"</h2>
                    {move || {
                        match dashboard_data.get() {
                            Some(data) => {
                                match data.todays_pending_sessions {
                                    Some(sessions) if !sessions.is_empty() => {
                                        view! {
                                            <div class="session-list">
                                                {sessions.into_iter().map(|session| {
                                                    view! { <SessionCard session=session show_cancel=true /> }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }.into_any()
                                    }
                                    _ => view! {
                                        <EmptySessionList message="No upcoming sessions. Book one now!".to_string() />
                                    }.into_any()
                                }
                            }
                            None => view! {
                                <EmptySessionList message="Loading...".to_string() />
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
