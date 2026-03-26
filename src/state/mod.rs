//! Reactive application state, provided at the app root via Leptos context.

/// Authentication state: token management, login/logout, session restore.
pub mod auth_state;
/// Active session tracking: start, stop, persist, and restore the workout timer.
pub mod session_state;

pub use auth_state::*;
pub use session_state::*;
