use leptos::prelude::*;

/// Labeled text input with optional error display.
#[component]
pub fn TextInput(
    #[prop(into)] placeholder: String,
    #[prop(into)] value: RwSignal<String>,
    #[prop(optional)] input_type: Option<String>,
    #[prop(optional)] label: Option<String>,
    #[prop(optional)] error: Option<Signal<Option<String>>>,
) -> impl IntoView {
    let input_type = input_type.unwrap_or_else(|| "text".to_string());
    let error_signal = error.unwrap_or_else(|| Signal::derive(|| None));

    let input_class = move || {
        if error_signal.get().is_some() {
            "input input-error"
        } else {
            "input"
        }
    };

    view! {
        <div class="input-group">
            {label.map(|l| view! { <label class="input-label">{l}</label> })}
            <input
                type=input_type
                class=input_class
                placeholder=placeholder
                prop:value=move || value.get()
                on:input=move |ev| {
                    value.set(event_target_value(&ev));
                }
            />
            {move || {
                error_signal.get().map(|e| view! { <span class="input-error-text">{e}</span> })
            }}
        </div>
    }
}

/// Numeric code input for OTP verification (defaults to 6 digits).
#[component]
pub fn OtpInput(
    #[prop(into)] value: RwSignal<String>,
    #[prop(optional)] length: Option<usize>,
) -> impl IntoView {
    let length = length.unwrap_or(6);

    view! {
        // A single (visually hidden) input overlays the box row and owns the
        // value; the boxes below just render its digits. This gets the 6-box
        // look without juggling focus across six separate inputs.
        <div class="otp-boxes">
            <input
                class="otp-hidden-input"
                type="text"
                maxlength=length
                aria-label="6-digit verification code"
                inputmode="numeric"
                autocomplete="one-time-code"
                prop:value=move || value.get()
                on:input=move |ev| {
                    let digits: String = event_target_value(&ev)
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .take(length)
                        .collect();
                    value.set(digits);
                }
            />
            <div class="otp-box-row" aria-hidden="true">
                {(0..length).map(|i| {
                    view! {
                        <div class=move || {
                            let len = value.get().chars().count();
                            if i < len { "otp-box filled" }
                            else if i == len { "otp-box active" }
                            else { "otp-box" }
                        }>
                            {move || value.get().chars().nth(i).map(|c| c.to_string()).unwrap_or_default()}
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
