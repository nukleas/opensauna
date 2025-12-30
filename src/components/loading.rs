use leptos::prelude::*;

#[component]
pub fn LoadingSpinner(
    #[prop(optional)] size: Option<String>,
) -> impl IntoView {
    let size = size.unwrap_or_else(|| "medium".to_string());
    let class = format!("loading-spinner loading-spinner-{}", size);

    view! {
        <div class=class>
            <div class="spinner"></div>
        </div>
    }
}

#[component]
pub fn LoadingOverlay(
    #[prop(optional)] message: Option<String>,
) -> impl IntoView {
    view! {
        <div class="loading-overlay">
            <div class="loading-content">
                <LoadingSpinner size="large".to_string() />
                {message.map(|m| view! { <p class="loading-message">{m}</p> })}
            </div>
        </div>
    }
}

#[component]
pub fn PageLoading() -> impl IntoView {
    view! {
        <div class="page-loading">
            <LoadingSpinner size="large".to_string() />
        </div>
    }
}
