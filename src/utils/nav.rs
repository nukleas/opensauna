//! Imperative navigation helper.
//!
//! Pages historically did `window.location.set_href(path)` directly. This
//! wrapper centralises that so a future move to `leptos_router::use_navigate`
//! is a single edit.

use leptos::web_sys;

/// Navigate to `path` by setting the window location.
pub fn go(path: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(path);
    }
}
