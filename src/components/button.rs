use leptos::prelude::*;

/// Primary action button with optional loading and disabled states.
#[component]
pub fn Button(
    #[prop(into)] label: String,
    #[prop(optional)] disabled: Option<Signal<bool>>,
    #[prop(optional)] loading: Option<Signal<bool>>,
    #[prop(optional)] variant: Option<String>,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    let disabled_signal = disabled.unwrap_or_else(|| Signal::derive(|| false));
    let loading_signal = loading.unwrap_or_else(|| Signal::derive(|| false));
    let variant = variant.unwrap_or_else(|| "primary".to_string());

    let button_class = move || {
        let base = "button";
        let variant_class = match variant.as_str() {
            "secondary" => "button-secondary",
            "danger" => "button-danger",
            _ => "button-primary",
        };
        let disabled_class = if disabled_signal.get() || loading_signal.get() {
            "button-disabled"
        } else {
            ""
        };
        format!("{} {} {}", base, variant_class, disabled_class)
    };

    view! {
        <button
            class=button_class
            disabled=move || disabled_signal.get() || loading_signal.get()
            on:click=move |_| {
                if !disabled_signal.get() && !loading_signal.get() {
                    on_click();
                }
            }
        >
            {move || {
                if loading_signal.get() {
                    view! { <span class="loading-spinner">"..."</span> }.into_any()
                } else {
                    view! { <span>{label.clone()}</span> }.into_any()
                }
            }}
        </button>
    }
}

/// Compact button showing an icon with an optional text label.
#[component]
pub fn IconButton(
    #[prop(into)] icon: String,
    #[prop(optional)] label: Option<String>,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    view! {
        <button class="icon-button" on:click=move |_| on_click()>
            <span class="icon">{icon}</span>
            {label.map(|l| view! { <span class="label">{l}</span> })}
        </button>
    }
}
