//! Full-page route components. Each page handles its own data fetching via Tauri IPC.

/// Session booking flow: pick a date, session type, and time slot.
pub mod booking;
/// Home screen with today's sessions, summary stats, and quick actions.
pub mod dashboard;
/// Studio location picker for booking.
pub mod locations;
/// Email + password login form.
pub mod login;
/// OTP verification step after password login.
pub mod otp;
/// User profile, calorie stats, goals, and weight management.
pub mod profile;
/// One-tap rebooking using saved location and session type preferences.
pub mod quick_book;
/// Activity history with date-filtered completed sessions.
pub mod sessions;

pub use booking::*;
pub use dashboard::*;
pub use locations::*;
pub use login::*;
pub use otp::*;
pub use profile::*;
pub use quick_book::*;
pub use sessions::*;
