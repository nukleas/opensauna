use leptos::prelude::*;
use leptos_router::components::{Router, Route, Routes, Redirect};
use leptos_router::path;

use crate::state::{provide_auth_state, provide_pending_login, provide_session_tracking_state};
use crate::pages::{LoginPage, OtpPage, DashboardPage, LocationsPage, BookingPage, SessionsPage, QuickBookPage};

#[component]
pub fn App() -> impl IntoView {
    // Provide auth state at the app root
    let auth_state = provide_auth_state();
    // Provide pending login state for OTP flow
    let _ = provide_pending_login();
    // Provide session tracking state
    let _ = provide_session_tracking_state();

    // Restore session on mount
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            auth_state.restore_session().await;
        });
    });

    view! {
        <Router>
            <Routes fallback=|| view! { <Redirect path="/" /> }>
                // Public routes
                <Route path=path!("/login") view=LoginPage />
                <Route path=path!("/otp") view=OtpPage />

                // Protected routes - auth check happens inside each page
                <Route path=path!("/") view=AuthenticatedDashboard />
                <Route path=path!("/book") view=AuthenticatedLocations />
                <Route path=path!("/book/:location_id") view=AuthenticatedBooking />
                <Route path=path!("/quick-book") view=AuthenticatedQuickBook />
                <Route path=path!("/sessions") view=AuthenticatedSessions />
            </Routes>
        </Router>
    }
}

#[component]
fn AuthenticatedDashboard() -> impl IntoView {
    let auth = crate::state::use_auth_state();
    let is_authenticated = auth.is_authenticated();
    let is_loading = auth.loading;

    view! {
        <Show
            when=move || !is_loading.get()
            fallback=|| view! {
                <div class="auth-loading">
                    <div class="spinner"></div>
                </div>
            }
        >
            <Show
                when=move || is_authenticated.get()
                fallback=|| view! { <Redirect path="/login" /> }
            >
                <DashboardPage />
            </Show>
        </Show>
    }
}

#[component]
fn AuthenticatedLocations() -> impl IntoView {
    let auth = crate::state::use_auth_state();
    let is_authenticated = auth.is_authenticated();
    let is_loading = auth.loading;

    view! {
        <Show
            when=move || !is_loading.get()
            fallback=|| view! {
                <div class="auth-loading">
                    <div class="spinner"></div>
                </div>
            }
        >
            <Show
                when=move || is_authenticated.get()
                fallback=|| view! { <Redirect path="/login" /> }
            >
                <LocationsPage />
            </Show>
        </Show>
    }
}

#[component]
fn AuthenticatedBooking() -> impl IntoView {
    let auth = crate::state::use_auth_state();
    let is_authenticated = auth.is_authenticated();
    let is_loading = auth.loading;

    view! {
        <Show
            when=move || !is_loading.get()
            fallback=|| view! {
                <div class="auth-loading">
                    <div class="spinner"></div>
                </div>
            }
        >
            <Show
                when=move || is_authenticated.get()
                fallback=|| view! { <Redirect path="/login" /> }
            >
                <BookingPage />
            </Show>
        </Show>
    }
}

#[component]
fn AuthenticatedSessions() -> impl IntoView {
    let auth = crate::state::use_auth_state();
    let is_authenticated = auth.is_authenticated();
    let is_loading = auth.loading;

    view! {
        <Show
            when=move || !is_loading.get()
            fallback=|| view! {
                <div class="auth-loading">
                    <div class="spinner"></div>
                </div>
            }
        >
            <Show
                when=move || is_authenticated.get()
                fallback=|| view! { <Redirect path="/login" /> }
            >
                <SessionsPage />
            </Show>
        </Show>
    }
}

#[component]
fn AuthenticatedQuickBook() -> impl IntoView {
    let auth = crate::state::use_auth_state();
    let is_authenticated = auth.is_authenticated();
    let is_loading = auth.loading;

    view! {
        <Show
            when=move || !is_loading.get()
            fallback=|| view! {
                <div class="auth-loading">
                    <div class="spinner"></div>
                </div>
            }
        >
            <Show
                when=move || is_authenticated.get()
                fallback=|| view! { <Redirect path="/login" /> }
            >
                <QuickBookPage />
            </Show>
        </Show>
    }
}
