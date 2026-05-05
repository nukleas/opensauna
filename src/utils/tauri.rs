//! Single source of truth for the Tauri IPC bindings the WASM frontend uses.
//!
//! Every page used to declare its own `#[wasm_bindgen]` block re-binding
//! `__TAURI__.core.invoke` and `console.log`. This module exports those
//! bindings once so pages can `use crate::utils::tauri::{invoke, log}` and
//! get on with their work.

use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    /// Call a Tauri command. Returns a `Promise` that resolves to the
    /// command's return value (already deserialized from JSON by Tauri).
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    pub fn invoke(cmd: &str, args: JsValue) -> js_sys::Promise;

    /// `console.log`. Use [`log_str`] / [`log_fmt`] for typed call-sites.
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

/// `console.log(s)`.
pub fn log_str(s: &str) {
    log(s);
}

/// `console.log(format!(...))` shorthand.
#[macro_export]
macro_rules! console_log {
    ($($arg:tt)*) => {
        $crate::utils::tauri::log_str(&format!($($arg)*))
    };
}

/// Convert a Tauri-rejection [`JsValue`] into a string. Tauri commands
/// that return `Err(String)` reject with that string as a JS primitive,
/// so the typical case is just `as_string()`.
pub fn js_error_string(err: &JsValue) -> String {
    err.as_string().unwrap_or_else(|| format!("{:?}", err))
}

/// Convert a serializable Rust value into the [`JsValue`] Tauri expects
/// for command arguments. Wraps the underlying serializer error in a
/// `String` so call sites don't need a custom error type.
pub fn to_args<T: Serialize>(value: &T) -> Result<JsValue, String> {
    serde_wasm_bindgen::to_value(value).map_err(|e| format!("Failed to serialize args: {:?}", e))
}

/// Invoke a Tauri command with already-serialized arguments. Returns the
/// command's return value parsed into `R`.
///
/// Use this instead of calling [`invoke`] directly when you want the
/// promise/await/deserialize boilerplate handled in one place.
pub async fn invoke_typed<R: DeserializeOwned>(cmd: &str, args: JsValue) -> Result<R, JsValue> {
    let promise = invoke(cmd, args);
    let result = JsFuture::from(promise).await?;
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| JsValue::from_str(&format!("Failed to deserialize response: {:?}", e)))
}

// ─── Auth-token storage helpers (formerly src/api/client.rs) ─────────────

/// Persist an auth token in encrypted storage on the native side.
pub async fn store_auth_token(token: &str) -> Result<(), String> {
    let args = to_args(&serde_json::json!({ "token": token }))?;
    JsFuture::from(invoke("store_auth_token", args))
        .await
        .map_err(|e| js_error_string(&e))?;
    Ok(())
}

/// Read the persisted auth token, if any.
pub async fn get_auth_token() -> Result<Option<String>, String> {
    let args = to_args(&serde_json::json!({}))?;
    let result = JsFuture::from(invoke("get_auth_token", args))
        .await
        .map_err(|e| js_error_string(&e))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Failed to parse token: {:?}", e))
}

/// Wipe the persisted auth token.
pub async fn clear_auth_token() -> Result<(), String> {
    let args = to_args(&serde_json::json!({}))?;
    JsFuture::from(invoke("clear_auth_token", args))
        .await
        .map_err(|e| js_error_string(&e))?;
    Ok(())
}
