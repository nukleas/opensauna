use leptos::prelude::*;
use crate::models::dashboard::PendingSession;

#[component]
pub fn SessionCard(
    #[prop(into)] session: PendingSession,
    #[prop(optional)] show_cancel: Option<bool>,
    #[prop(optional)] on_cancel: Option<Box<dyn Fn()>>,
) -> impl IntoView {
    let show_cancel = show_cancel.unwrap_or(false);

    view! {
        <div class="session-card">
            <div class="session-header">
                <h3 class="session-name">{session.display_name()}</h3>
                <span class="session-time">{session.display_time()}</span>
            </div>
            <div class="session-details">
                <p class="session-location">{session.display_location()}</p>
                {session.display_date.clone().map(|d| view! { <p class="session-date">{d}</p> })}
                {session.duration.clone().map(|d| view! { <p class="session-duration">{d} " mins"</p> })}
            </div>
            {show_cancel.then(|| {
                view! {
                    <button
                        class="session-cancel-btn"
                        on:click=move |_| {
                            if let Some(ref cb) = on_cancel {
                                cb();
                            }
                        }
                    >
                        "Cancel Session"
                    </button>
                }
            })}
        </div>
    }
}

#[component]
pub fn EmptySessionList(
    #[prop(into)] message: String,
) -> impl IntoView {
    view! {
        <div class="empty-session-list">
            <p class="empty-message">{message}</p>
        </div>
    }
}
