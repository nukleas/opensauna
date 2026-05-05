use crate::components::toast::use_toast;
use crate::models::dashboard::PendingSession;
use crate::state::{handle_invoke_error, use_auth_state};
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

/// Card displaying a single session (pending or completed) with optional cancel/start actions.
#[component]
pub fn SessionCard(
    session: PendingSession,
    #[prop(optional)] show_cancel: bool,
    #[prop(optional)] show_start: bool,
    #[prop(optional)] on_cancelled: Option<WriteSignal<Option<String>>>,
    #[prop(optional)] on_start: Option<WriteSignal<Option<PendingSession>>>,
) -> impl IntoView {
    let auth = use_auth_state();
    let toast = use_toast();
    // Clone session for the start button callback
    let session_for_start = session.clone();
    let confirming = RwSignal::new(false);
    let cancelling = RwSignal::new(false);
    let error_msg: RwSignal<Option<String>> = RwSignal::new(None);

    // Store IDs for the cancel action - clone for use in multiple closures
    let session_id = session.session_record_id.clone().unwrap_or_default();
    let lead_id = session.lead_record_id.clone().unwrap_or_default();

    let time = session.display_time();
    let location = session.display_location();
    let calories = session.display_calories();

    view! {
        <div class="session-card">
            <div class="session-header">
                <h3 class="session-name">{session.display_name()}</h3>
                {if !time.is_empty() {
                    Some(view! { <span class="session-time">{time}</span> })
                } else if let Some(ref cal) = calories {
                    Some(view! { <span class="session-time">{cal.clone()}</span> })
                } else {
                    None
                }}
            </div>
            <div class="session-details">
                {(!location.is_empty()).then(|| view! { <p class="session-location">{location}</p> })}
                {session.display_date.clone().map(|d| view! { <p class="session-date">{d}</p> })}
                {session.duration.clone().map(|d| view! { <p class="session-duration">{d} " mins"</p> })}
            </div>

            // Error message
            {move || error_msg.get().map(|e| view! {
                <div class="error-message session-card-error">{e}</div>
            })}

            // Show confirm buttons when confirming
            {
                let sid = session_id.clone();
                let lid = lead_id.clone();
                move || {
                    let sid = sid.clone();
                    let lid = lid.clone();
                    confirming.get().then(move || {
                        let sid = sid.clone();
                        let lid = lid.clone();
                        view! {
                            <div class="cancel-confirm">
                                <p class="confirm-text">"Cancel this session?"</p>
                                <div class="confirm-actions">
                                    <button
                                        class="button button-danger"
                                        disabled=move || cancelling.get()
                                        on:click={
                                            let sid = sid.clone();
                                            let lid = lid.clone();
                                            move |_| {
                                                let sid = sid.clone();
                                                let lid = lid.clone();
                                                cancelling.set(true);
                                                error_msg.set(None);

                                                wasm_bindgen_futures::spawn_local(async move {
                                                    log(&format!("[SessionCard] Cancelling session {}", sid));

                                                    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                                                        "sessionRecordId": sid.clone(),
                                                        "leadRecordId": lid
                                                    })).unwrap();

                                                    let promise = invoke("api_delete_session", args);

                                                    match JsFuture::from(promise).await {
                                                        Ok(_) => {
                                                            log("[SessionCard] Session cancelled successfully");
                                                            if let Some(signal) = on_cancelled {
                                                                signal.set(Some(sid));
                                                            }
                                                        }
                                                        Err(e) => {
                                                            log(&format!("[SessionCard] Cancel error: {:?}", e));
                                                            if handle_invoke_error(&e, auth, toast).await {
                                                                cancelling.set(false);
                                                                confirming.set(false);
                                                                return;
                                                            }
                                                            error_msg.set(Some("Failed to cancel session.".to_string()));
                                                            cancelling.set(false);
                                                            confirming.set(false);
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                    >
                                        {move || if cancelling.get() { "Cancelling..." } else { "Yes, Cancel" }}
                                    </button>
                                    <button
                                        class="button button-secondary"
                                        disabled=move || cancelling.get()
                                        on:click=move |_| confirming.set(false)
                                    >
                                        "No, Keep"
                                    </button>
                                </div>
                            </div>
                        }
                    })
                }
            }

            // Session action buttons
            {
                let session_clone = session_for_start.clone();
                move || {
                    let session_clone = session_clone.clone();
                    (!confirming.get()).then(move || {
                        let session_clone = session_clone.clone();
                        view! {
                            <div class="session-actions">
                                // Start button
                                {show_start.then(|| {
                                    let session_clone = session_clone.clone();
                                    view! {
                                        <button
                                            class="session-start-btn"
                                            on:click=move |_| {
                                                if let Some(signal) = on_start {
                                                    signal.set(Some(session_clone.clone()));
                                                }
                                            }
                                        >
                                            "Start Session"
                                        </button>
                                    }
                                })}

                                // Cancel button
                                {show_cancel.then(|| view! {
                                    <button
                                        class="session-cancel-btn"
                                        on:click=move |_| confirming.set(true)
                                    >
                                        "Cancel"
                                    </button>
                                })}
                            </div>
                        }
                    })
                }
            }
        </div>
    }
}

/// Placeholder shown when a session list has no items.
#[component]
pub fn EmptySessionList(#[prop(into)] message: String) -> impl IntoView {
    view! {
        <div class="empty-session-list">
            <p class="empty-message">{message}</p>
        </div>
    }
}
