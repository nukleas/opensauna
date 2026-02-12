use leptos::prelude::*;

/// Icon size variants
#[derive(Clone, Copy, Default)]
pub enum IconSize {
    Sm, // 16px
    #[default]
    Md, // 20px
    Lg, // 24px
    Xl, // 32px
}

impl IconSize {
    fn size(&self) -> &'static str {
        match self {
            IconSize::Sm => "16",
            IconSize::Md => "20",
            IconSize::Lg => "24",
            IconSize::Xl => "32",
        }
    }
}

// ===== Navigation Icons =====

#[component]
pub fn IconHome(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/>
            <path d="M3 10a2 2 0 0 1 .709-1.528l7-5.999a2 2 0 0 1 2.582 0l7 5.999A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
        </svg>
    }
}

#[component]
pub fn IconCalendar(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <path d="M8 2v4"/>
            <path d="M16 2v4"/>
            <rect width="18" height="18" x="3" y="4" rx="2"/>
            <path d="M3 10h18"/>
        </svg>
    }
}

#[component]
pub fn IconCalendarPlus(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <path d="M8 2v4"/>
            <path d="M16 2v4"/>
            <path d="M21 13V6a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h8"/>
            <path d="M3 10h18"/>
            <path d="M16 19h6"/>
            <path d="M19 16v6"/>
        </svg>
    }
}

#[component]
pub fn IconMapPin(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <path d="M20 10c0 4.993-5.539 10.193-7.399 11.799a1 1 0 0 1-1.202 0C9.539 20.193 4 14.993 4 10a8 8 0 0 1 16 0"/>
            <circle cx="12" cy="10" r="3"/>
        </svg>
    }
}

#[component]
pub fn IconUser(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <circle cx="12" cy="8" r="5"/>
            <path d="M20 21a8 8 0 0 0-16 0"/>
        </svg>
    }
}

// ===== Action Icons =====

#[component]
pub fn IconChevronLeft(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <path d="m15 18-6-6 6-6"/>
        </svg>
    }
}

#[component]
pub fn IconChevronRight(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <path d="m9 18 6-6-6-6"/>
        </svg>
    }
}

#[component]
pub fn IconArrowLeft(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <path d="m12 19-7-7 7-7"/>
            <path d="M19 12H5"/>
        </svg>
    }
}

#[component]
pub fn IconPlus(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <path d="M5 12h14"/>
            <path d="M12 5v14"/>
        </svg>
    }
}

#[component]
pub fn IconX(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <path d="M18 6 6 18"/>
            <path d="m6 6 12 12"/>
        </svg>
    }
}

#[component]
pub fn IconCheck(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <path d="M20 6 9 17l-5-5"/>
        </svg>
    }
}

// ===== Status Icons =====

#[component]
pub fn IconSearch(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <circle cx="11" cy="11" r="8"/>
            <path d="m21 21-4.3-4.3"/>
        </svg>
    }
}

#[component]
pub fn IconLogOut(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
            <polyline points="16 17 21 12 16 7"/>
            <line x1="21" x2="9" y1="12" y2="12"/>
        </svg>
    }
}

#[component]
pub fn IconClock(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <circle cx="12" cy="12" r="10"/>
            <polyline points="12 6 12 12 16 14"/>
        </svg>
    }
}

#[component]
pub fn IconFlame(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <path d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z"/>
        </svg>
    }
}

#[component]
pub fn IconZap(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <path d="M4 14a1 1 0 0 1-.78-1.63l9.9-10.2a.5.5 0 0 1 .86.46l-1.92 6.02A1 1 0 0 0 13 10h7a1 1 0 0 1 .78 1.63l-9.9 10.2a.5.5 0 0 1-.86-.46l1.92-6.02A1 1 0 0 0 11 14z"/>
        </svg>
    }
}

#[component]
pub fn IconTrendingUp(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <polyline points="22 7 13.5 15.5 8.5 10.5 2 17"/>
            <polyline points="16 7 22 7 22 13"/>
        </svg>
    }
}

#[component]
pub fn IconLoader(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=format!("animate-spin {}", class)
        >
            <path d="M12 2v4"/>
            <path d="m16.2 7.8 2.9-2.9"/>
            <path d="M18 12h4"/>
            <path d="m16.2 16.2 2.9 2.9"/>
            <path d="M12 18v4"/>
            <path d="m4.9 19.1 2.9-2.9"/>
            <path d="M2 12h4"/>
            <path d="m4.9 4.9 2.9 2.9"/>
        </svg>
    }
}

#[component]
pub fn IconAlertCircle(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" x2="12" y1="8" y2="12"/>
            <line x1="12" x2="12.01" y1="16" y2="16"/>
        </svg>
    }
}

#[component]
pub fn IconCheckCircle(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <circle cx="12" cy="12" r="10"/>
            <path d="m9 12 2 2 4-4"/>
        </svg>
    }
}

#[component]
pub fn IconTrash(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <path d="M3 6h18"/>
            <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
        </svg>
    }
}

#[component]
pub fn IconMail(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <rect width="20" height="16" x="2" y="4" rx="2"/>
            <path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/>
        </svg>
    }
}

#[component]
pub fn IconLock(
    #[prop(optional)] size: IconSize,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let s = size.size();
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=s
            height=s
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            <rect width="18" height="11" x="3" y="11" rx="2" ry="2"/>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
        </svg>
    }
}
