//! BookWorx frontend — a Leptos 0.7 SPA compiled to WASM, rendered inside a Tauri webview.
//!
//! All API calls go through the Tauri backend via IPC commands. The WASM frontend
//! never contacts the HOTWORX API directly; the typed shapes the backend returns
//! come from the workspace's `hotworx-api` crate.

/// Root application component and route definitions.
pub mod app;
/// Reusable UI components (buttons, icons, loading states, toasts, etc.).
pub mod components;
/// Data types matching the HOTWORX API request/response shapes.
pub mod models;
/// Full-page route components (login, dashboard, booking, sessions, profile).
pub mod pages;
/// Reactive application state (auth, session tracking).
pub mod state;
/// Shared helpers — Tauri IPC bindings, date utilities, navigation.
pub mod utils;
