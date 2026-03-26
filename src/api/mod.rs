//! Frontend API layer — thin wrappers around Tauri `invoke()` calls.
//!
//! Each function serializes its arguments, calls the corresponding Tauri backend command,
//! and deserializes the response into typed models.

/// Login and OTP verification commands.
pub mod auth;
/// Location, slot, and session booking commands.
pub mod booking;
/// HTTP client that routes requests through Tauri's native backend.
pub mod client;
/// Dashboard data fetching.
pub mod dashboard;
/// Profile, stats, goals, and weight commands.
pub mod profile;

pub use client::ApiClient;
