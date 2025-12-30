use leptos::prelude::*;

/// Format seconds into MM:SS or HH:MM:SS display
fn format_time(total_seconds: i64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

#[component]
pub fn SessionTimer(
    /// Elapsed time in seconds
    #[prop(into)] elapsed_seconds: Signal<i64>,
    /// Total planned duration in seconds
    #[prop(into)] total_seconds: Signal<i64>,
    /// Whether to show countdown (remaining) or elapsed time
    #[prop(optional)] show_countdown: Option<bool>,
    /// Size variant: "large" for active session view, "small" for cards
    #[prop(optional)] size: Option<String>,
) -> impl IntoView {
    let show_countdown = show_countdown.unwrap_or(true);
    let size = size.unwrap_or_else(|| "large".to_string());

    // Calculate remaining time
    let remaining_seconds = move || {
        let total = total_seconds.get();
        let elapsed = elapsed_seconds.get();
        (total - elapsed).max(0)
    };

    // Calculate progress percentage (0.0 to 1.0+)
    let progress = move || {
        let total = total_seconds.get();
        let elapsed = elapsed_seconds.get();
        if total > 0 {
            elapsed as f64 / total as f64
        } else {
            0.0
        }
    };

    // Progress clamped to 100% for the bar
    let progress_clamped = move || progress().min(1.0);

    // Check if overtime
    let is_overtime = move || elapsed_seconds.get() > total_seconds.get();

    // Display time (either remaining or elapsed based on show_countdown)
    let display_time = move || {
        if show_countdown {
            format_time(remaining_seconds())
        } else {
            format_time(elapsed_seconds.get())
        }
    };

    // Timer class based on overtime status and size
    let timer_class = move || {
        let base = format!("session-timer session-timer-{}", size);
        if is_overtime() {
            format!("{} session-timer-overtime", base)
        } else {
            base
        }
    };

    view! {
        <div class=timer_class>
            // Main time display
            <div class="timer-time">
                {display_time}
            </div>

            // Progress bar
            <div class="timer-progress-container">
                <div
                    class="timer-progress-bar"
                    style=move || format!("width: {}%", progress_clamped() * 100.0)
                />
            </div>

            // Labels row
            <div class="timer-labels">
                <span class="timer-label-elapsed">
                    {move || format!("Elapsed: {}", format_time(elapsed_seconds.get()))}
                </span>
                <span class="timer-label-remaining">
                    {move || {
                        if is_overtime() {
                            let overtime = elapsed_seconds.get() - total_seconds.get();
                            format!("+{} overtime", format_time(overtime))
                        } else {
                            format!("Remaining: {}", format_time(remaining_seconds()))
                        }
                    }}
                </span>
            </div>

            // Overtime badge
            {move || is_overtime().then(|| view! {
                <div class="timer-overtime-badge">
                    "OVERTIME"
                </div>
            })}
        </div>
    }
}
