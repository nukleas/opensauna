//! Typed request and response shapes for the HOTWORX API.
//!
//! The HOTWORX backend isn't strictly typed on the wire — fields can come
//! back as strings, numbers, booleans, or omitted entirely depending on the
//! endpoint and account state. The deserializers below (kept private to
//! each submodule) normalize that drift so callers see consistent
//! `Option<String>` / `Option<i32>` / `Option<bool>` values.

pub mod activity;
pub mod auth;
pub mod booking;
pub mod dashboard;
pub mod location;
pub mod profile;

pub use activity::*;
pub use auth::*;
pub use booking::*;
pub use dashboard::*;
pub use location::*;
pub use profile::*;
