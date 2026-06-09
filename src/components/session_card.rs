use crate::components::toast::use_toast;
use crate::models::dashboard::PendingSession;
use crate::state::{handle_invoke_error, use_auth_state};
use crate::utils::tauri::{invoke, log};
use leptos::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// Card displaying a single session.
///
/// Buttons are rendered conditionally on which callbacks the parent
/// passes in:
///
/// - `on_cancel: Some(...)` adds a "Cancel" button that walks the user
///   through a confirm prompt; the callback fires once the API
///   confirms cancellation, with the session record ID as its argument.
/// - `on_start: Some(...)` adds a "Start Session" button that fires
///   the callback with the full session payload.
///
/// Pass neither for a read-only card (e.g. completed sessions).
#[component]
pub fn SessionCard(
    session: PendingSession,
    #[prop(optional)] on_cancel: Option<Callback<String>>,
    #[prop(optional)] on_start: Option<Callback<PendingSession>>,
) -> impl IntoView {
    let auth = use_auth_state();
    let toast = use_toast();
    let session_for_start = session.clone();
    let confirming = RwSignal::new(false);
    let cancelling = RwSignal::new(false);
    let error_msg: RwSignal<Option<String>> = RwSignal::new(None);

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
                } else {
                    calories
                        .as_ref()
                        .map(|cal| view! { <span class="session-time">{cal.clone()}</span> })
                }}
            </div>
            <div class="session-details">
                {(!location.is_empty()).then(|| view! { <p class="session-location">{location}</p> })}
                {session.display_date.clone().map(|d| view! { <p class="session-date">{d}</p> })}
                {session.duration.clone().map(|d| view! { <p class="session-duration">{d} " mins"</p> })}
            </div>

            {move || error_msg.get().map(|e| view! {
                <div class="error-message session-card-error">{e}</div>
            })}

            // Cancel-confirm prompt
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

                                                    let args = crate::json_args!({
                                                        "sessionRecordId": sid.clone(),
                                                        "leadRecordId": lid,
                                                    });

                                                    match JsFuture::from(invoke("api_delete_session", args)).await {
                                                        Ok(_) => {
                                                            log("[SessionCard] Session cancelled successfully");
                                                            toast.success("Session cancelled");
                                                            if let Some(cb) = on_cancel {
                                                                cb.run(sid);
                                                            }
                                                        }
                                                        Err(e) => {
                                                            log(&format!("[SessionCard] Cancel error: {:?}", e));
                                                            if !handle_invoke_error(&e, auth, toast).await {
                                                                error_msg.set(Some("Failed to cancel session.".to_string()));
                                                            }
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

            // Action buttons (start, cancel)
            {
                let session_clone = session_for_start.clone();
                move || {
                    let session_clone = session_clone.clone();
                    (!confirming.get()).then(move || {
                        let session_clone = session_clone.clone();
                        view! {
                            <div class="session-actions">
                                {on_start.map(|cb| {
                                    let session_clone = session_clone.clone();
                                    view! {
                                        <button
                                            class="session-start-btn"
                                            on:click=move |_| cb.run(session_clone.clone())
                                        >
                                            "Start Session"
                                        </button>
                                    }
                                })}

                                {on_cancel.map(|_| view! {
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
