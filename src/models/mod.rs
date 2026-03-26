//! Data models for HOTWORX API requests and responses.
//!
//! Many API fields arrive as inconsistent types (numbers as strings, bools as ints, etc.),
//! so most models use custom deserializers to normalize the data.

/// Login, OTP, and authentication response types.
pub mod auth;
/// Time slots, session booking, and cancellation types.
pub mod booking;
/// Dashboard summary and pending/completed session types.
pub mod dashboard;
/// Studio location and session type data.
pub mod location;
/// User profile, stats, goals, and weight tracking types.
pub mod profile;
/// Client-side session timer state machine.
pub mod session_tracking;
