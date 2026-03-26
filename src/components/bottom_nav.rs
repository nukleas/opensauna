use crate::components::icons::{IconCalendar, IconCalendarPlus, IconHome, IconUser};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// Tabs in the bottom navigation bar.
#[derive(Clone, PartialEq)]
pub enum NavItem {
    Home,
    Book,
    Sessions,
    Profile,
}

/// Bottom tab bar with Home, Book, Sessions, and Profile tabs.
#[component]
pub fn BottomNav(#[prop(into)] active: Signal<NavItem>) -> impl IntoView {
    let navigate = use_navigate();

    let nav_item_class = move |item: NavItem| {
        if active.get() == item {
            "nav-item nav-item-active"
        } else {
            "nav-item"
        }
    };

    view! {
        <nav class="bottom-nav">
            <button
                class=move || nav_item_class(NavItem::Home)
                on:click={
                    let navigate = navigate.clone();
                    move |_| navigate("/", Default::default())
                }
            >
                <span class="nav-icon">
                    <IconHome />
                </span>
                <span class="nav-label">"Home"</span>
            </button>
            <button
                class=move || nav_item_class(NavItem::Book)
                on:click={
                    let navigate = navigate.clone();
                    move |_| navigate("/book", Default::default())
                }
            >
                <span class="nav-icon">
                    <IconCalendarPlus />
                </span>
                <span class="nav-label">"Book"</span>
            </button>
            <button
                class=move || nav_item_class(NavItem::Sessions)
                on:click={
                    let navigate = navigate.clone();
                    move |_| navigate("/sessions", Default::default())
                }
            >
                <span class="nav-icon">
                    <IconCalendar />
                </span>
                <span class="nav-label">"Sessions"</span>
            </button>
            <button
                class=move || nav_item_class(NavItem::Profile)
                on:click={
                    let navigate = navigate.clone();
                    move |_| navigate("/profile", Default::default())
                }
            >
                <span class="nav-icon">
                    <IconUser />
                </span>
                <span class="nav-label">"Profile"</span>
            </button>
        </nav>
    }
}
