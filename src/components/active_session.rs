use crate::components::icons::IconSize;
use crate::components::{IconClock, IconFlame, IconMapPin, SessionTimer};
use crate::models::session_tracking::TrackedSession;
use crate::state::use_session_tracking_state;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    fn invoke(cmd: &str, args: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_name = "Date.now")]
    fn date_now() -> f64;
}

/// Get current time in milliseconds
fn now_ms() -> i64 {
    date_now() as i64
}

/// Full-screen view for an in-progress workout session, showing timer, calories, and location.
#[component]
pub fn ActiveSessionView(
    /// The active session being tracked
    session: TrackedSession,
) -> impl IntoView {
    let session_state = use_session_tracking_state();
    let confirming_end = RwSignal::new(false);
    let ending = RwSignal::new(false);

    // Timer tick - updates every second
    let elapsed_seconds = RwSignal::new(0i64);
    let total_seconds = session.total_seconds();
    let started_at = session.started_at.unwrap_or_else(now_ms);
    let session_name = session.session_name.clone();
    let location_name = session.location_name.clone();
    let duration_minutes = session.duration_minutes;

    // Update elapsed time every second using gloo-timers
    Effect::new(move |_| {
        use gloo_timers::callback::Interval;

        // Initial update
        let now = now_ms();
        let elapsed = ((now - started_at) / 1000).max(0);
        elapsed_seconds.set(elapsed);

        let interval = Interval::new(1000, move || {
            let now = now_ms();
            let elapsed = ((now - started_at) / 1000).max(0);
            elapsed_seconds.set(elapsed);
        });

        // Store interval to prevent it from being dropped
        // When the effect is cleaned up, the interval will be dropped and stop
        std::mem::forget(interval);
    });

    // Estimate calories burned (rough estimate: ~10 cal/min for hot yoga)
    let estimated_calories = move || {
        let mins = elapsed_seconds.get() / 60;
        mins * 10
    };

    let end_session = move || {
        ending.set(true);

        wasm_bindgen_futures::spawn_local(async move {
            log("[ActiveSession] Ending session via state");
            let _ = session_state.end_session().await;
        });
    };

    view! {
        <div class="active-session-overlay">
            <div class="active-session-content">
                // Session icon with glow
                <div class="active-session-icon">
                    <IconFlame size=IconSize::Xl />
                </div>

                // Session info
                <h1 class="active-session-name">{session_name}</h1>
                <div class="active-session-location">
                    <IconMapPin size=IconSize::Sm />
                    <span>{location_name}</span>
                </div>

                // Timer (centered, large)
                <div class="active-session-timer">
                    <SessionTimer
                        elapsed_seconds=Signal::derive(move || elapsed_seconds.get())
                        total_seconds=Signal::derive(move || total_seconds)
                        show_countdown=true
                        size="large".to_string()
                    />
                </div>

                // Stats row
                <div class="active-session-stats">
                    <div class="stat-item">
                        <IconClock size=IconSize::Sm />
                        <span class="stat-value">{duration_minutes}" min"</span>
                        <span class="stat-label">"Planned"</span>
                    </div>
                    <div class="stat-item">
                        <IconFlame size=IconSize::Sm />
                        <span class="stat-value">{estimated_calories}" cal"</span>
                        <span class="stat-label">"Est. Burned"</span>
                    </div>
                </div>

                // End session button area
                {move || {
                    if confirming_end.get() {
                        view! {
                            <div class="end-session-confirm">
                                <p class="confirm-text">"End this session?"</p>
                                <div class="confirm-actions">
                                    <button
                                        class="button button-danger"
                                        disabled=move || ending.get()
                                        on:click={
                                            let end_session = end_session.clone();
                                            move |_| end_session()
                                        }
                                    >
                                        {move || if ending.get() { "Ending..." } else { "Yes, End Session" }}
                                    </button>
                                    <button
                                        class="button button-secondary"
                                        disabled=move || ending.get()
                                        on:click=move |_| confirming_end.set(false)
                                    >
                                        "Continue"
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <button
                                class="end-session-btn"
                                on:click=move |_| confirming_end.set(true)
                            >
                                "End Session"
                            </button>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
