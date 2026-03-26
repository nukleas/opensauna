use gloo_timers::callback::Timeout;
use leptos::prelude::*;

/// A single toast notification to display.
#[derive(Clone, Debug)]
pub struct Toast {
    pub message: String,
    pub variant: ToastVariant,
}

/// Visual style for a toast notification.
#[derive(Clone, Debug, PartialEq)]
pub enum ToastVariant {
    Success,
    Error,
    Info,
}

/// Reactive state for showing toast notifications. Provided via Leptos context.
#[derive(Clone, Copy)]
pub struct ToastState {
    pub toast: RwSignal<Option<Toast>>,
}

impl ToastState {
    /// Create a new toast state with no active toast.
    pub fn new() -> Self {
        Self {
            toast: RwSignal::new(None),
        }
    }

    /// Show a toast with the given message and variant. Auto-dismisses after 4 seconds.
    pub fn show(&self, message: impl Into<String>, variant: ToastVariant) {
        let toast_signal = self.toast;
        toast_signal.set(Some(Toast {
            message: message.into(),
            variant,
        }));
        // Auto-dismiss after 4 seconds
        Timeout::new(4_000, move || {
            toast_signal.set(None);
        })
        .forget();
    }

    /// Show a success toast.
    pub fn success(&self, message: impl Into<String>) {
        self.show(message, ToastVariant::Success);
    }

    /// Show an error toast.
    pub fn error(&self, message: impl Into<String>) {
        self.show(message, ToastVariant::Error);
    }

    /// Show an informational toast.
    pub fn info(&self, message: impl Into<String>) {
        self.show(message, ToastVariant::Info);
    }
}

impl Default for ToastState {
    fn default() -> Self {
        Self::new()
    }
}

/// Provide [`ToastState`] in the current Leptos context. Call once at app root.
pub fn provide_toast_state() -> ToastState {
    let state = ToastState::new();
    provide_context(state);
    state
}

/// Get the [`ToastState`] from the current Leptos context.
pub fn use_toast() -> ToastState {
    use_context::<ToastState>().expect("ToastState must be provided")
}

/// Renders the currently active toast notification, if any.
#[component]
pub fn ToastContainer() -> impl IntoView {
    let toast_state = use_toast();

    view! {
        {move || {
            toast_state.toast.get().map(|toast| {
                let class = match toast.variant {
                    ToastVariant::Success => "toast toast-success",
                    ToastVariant::Error => "toast toast-error",
                    ToastVariant::Info => "toast toast-info",
                };
                view! {
                    <div class=class>
                        <span class="toast-message">{toast.message}</span>
                        <button
                            class="toast-close"
                            on:click=move |_| toast_state.toast.set(None)
                        >
                            "\u{2715}"
                        </button>
                    </div>
                }
            })
        }}
    }
}
