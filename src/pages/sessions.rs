use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use crate::components::{BottomNav, NavItem, SessionCard, EmptySessionList, PageLoading};
use crate::models::dashboard::PendingSession;

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
    let loading = RwSignal::new(true);
    let error: RwSignal<Option<String>> = RwSignal::new(None);

    // Fetch sessions on mount via Tauri backend
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            log("[Sessions] Fetching dashboard data...");

            let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).unwrap();
            let promise = invoke("api_get_dashboard", args);

            match JsFuture::from(promise).await {
                Ok(result) => {
                    if let Ok(response) = serde_wasm_bindgen::from_value::<serde_json::Value>(result) {
                        log("[Sessions] Got dashboard response");
                        if let Some(data) = response.get("data") {
                            if let Some(pending_json) = data.get("todays_pending_sessions") {
                                if let Ok(pending) = serde_json::from_value::<Vec<PendingSession>>(pending_json.clone()) {
                                    log(&format!("[Sessions] {} pending sessions", pending.len()));
                                    pending_sessions.set(pending);
                                }
                            }
                            if let Some(completed_json) = data.get("todays_completed_sessions") {
                                if let Ok(completed) = serde_json::from_value::<Vec<PendingSession>>(completed_json.clone()) {
                                    log(&format!("[Sessions] {} completed sessions", completed.len()));
                                    completed_sessions.set(completed);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[Sessions] Error: {:?}", e));
                    let err_str = js_sys::JSON::stringify(&e)
                        .map(|s| s.as_string().unwrap_or_default())
                        .unwrap_or_else(|_| format!("{:?}", e));
                    error.set(Some(format!("Failed to load sessions: {}", err_str)));
                }
            }

            loading.set(false);
        });
    });

    let on_cancel_session = move |session_record_id: String, lead_record_id: String| {
        wasm_bindgen_futures::spawn_local(async move {
            log(&format!("[Sessions] Cancelling session {}", session_record_id));

            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "sessionRecordId": session_record_id,
                "leadRecordId": lead_record_id
            })).unwrap();

            let promise = invoke("api_delete_session", args);

            match JsFuture::from(promise).await {
                Ok(_) => {
                    log("[Sessions] Session cancelled successfully");
                    // Remove session from the list
                    pending_sessions.update(|sessions| {
                        sessions.retain(|s| s.session_record_id.as_deref() != Some(&session_record_id));
                    });
                }
                Err(e) => {
                    log(&format!("[Sessions] Cancel error: {:?}", e));
                    let err_str = js_sys::JSON::stringify(&e)
                        .map(|s| s.as_string().unwrap_or_default())
                        .unwrap_or_else(|_| format!("{:?}", e));
                    error.set(Some(format!("Failed to cancel session: {}", err_str)));
                }
            }
        });
    };

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
                                        let session_id = session.session_record_id.clone().unwrap_or_default();
                                        let lead_id = session.lead_record_id.clone().unwrap_or_default();
                                        let on_cancel = on_cancel_session.clone();
                                        view! {
                                            <SessionCard
                                                session=session
                                                show_cancel=true
                                                on_cancel=Box::new(move || on_cancel(session_id.clone(), lead_id.clone()))
                                            />
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>

                // Completed sessions
                <div class="section">
                    <h2 class="section-title">"Completed"</h2>
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

                {move || error.get().map(|e| view! {
                    <div class="error-message">{e}</div>
                })}
            </div>

            <BottomNav active=Signal::derive(|| NavItem::Sessions) />
        </div>
    }
}
