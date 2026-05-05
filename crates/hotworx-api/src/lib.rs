//! Unofficial Rust client for the HOTWORX member API.
//!
//! `hotworx-api` is **not** affiliated with, endorsed by, or connected to
//! HOTWORX in any way. It speaks the same wire protocol as the official
//! HOTWORX Android app and is intended for personal use by HOTWORX members
//! who want to build their own tools.
//!
//! # Quick start
//!
//! ```no_run
//! use hotworx_api::{HotworxClient, password_hash};
//!
//! # async fn run() -> hotworx_api::Result<()> {
//! let client = HotworxClient::new("my-stable-device-id");
//! let login = client
//!     .login_with_password("me@example.com", "my-plaintext-password")
//!     .await?;
//!
//! if let Some(token) = login.token {
//!     let dashboard = HotworxClient::new("my-stable-device-id")
//!         .with_token(token)
//!         .get_dashboard(None)
//!         .await?;
//!     println!("today: {:?}", dashboard.todays_pending_sessions);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Devices and tokens
//!
//! Every request carries a stable per-device identifier (the `device_id`).
//! HOTWORX uses this to bind sessions to a single device — you should pick
//! one identifier per install (a UUID is fine) and reuse it across calls.
//! Authentication tokens come back from [`HotworxClient::login_with_password`]
//! (or [`HotworxClient::verify_otp`] for two-factor accounts) and should be
//! persisted by the caller; the client itself is stateless beyond what you
//! pass into it.

pub mod auth;
pub mod client;
pub mod error;
pub mod headers;
pub mod models;

pub use auth::password_hash;
pub use client::HotworxClient;
pub use error::{HotworxError, Result};
pub use models::*;
