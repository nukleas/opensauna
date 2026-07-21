//! Login, OTP, and the user-profile payload returned after authentication.

use serde::{Deserialize, Serialize};

/// Response from `loginwithpassword` and `verifyOtp`.
///
/// On a successful first-factor login the response carries a `token` and may
/// also set `two_factor` to `"yes"` to indicate that an OTP step is still
/// required (see [`requires_otp`](Self::requires_otp)). After successful
/// OTP verification the same shape comes back populated with `data`.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LoginResponse {
    /// Human-readable status from the server (e.g. `"success"`).
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub msg: Option<String>,
    /// Bearer token to attach to subsequent requests via
    /// [`HotworxClient::with_token`](crate::HotworxClient::with_token).
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub token: Option<String>,
    /// `"yes"` when the account requires OTP verification before the token
    /// is fully usable.
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub two_factor: Option<String>,
    /// Server-supplied error message, if any. HOTWORX sometimes sends this
    /// as a nested object (e.g. field-level validation errors) instead of a
    /// plain string; that shape is coerced to a best-effort string too.
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub error: Option<String>,
    /// Coarse-grained status string. Often `"ok"` or `"error"`.
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub status: Option<String>,
    /// User profile, populated after a fully successful login.
    pub data: Option<UserProfile>,
}

impl LoginResponse {
    /// `true` when a usable token was issued.
    pub fn is_success(&self) -> bool {
        self.token.is_some() && self.error.is_none()
    }

    /// `true` when the server is asking for an OTP before the token is
    /// considered fully authorized.
    pub fn requires_otp(&self) -> bool {
        self.two_factor.as_deref() == Some("yes")
    }
}

/// Subset of user-profile fields HOTWORX returns alongside a successful
/// login. The richer shape is exposed by
/// [`view_profile`](crate::HotworxClient::view_profile).
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct UserProfile {
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub user_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub first_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub last_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub email: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub phone: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub location_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub gender: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub dob: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub height: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub weight: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub image_url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub full_name: Option<String>,
}

/// Coerce a wire value that's usually a string into `Option<String>`.
///
/// HOTWORX has been observed sending numbers, booleans, and (for `error` in
/// particular) nested objects where the Android client — and this crate —
/// expect a plain string. Objects are flattened to a best-effort message by
/// picking the first `message`/`msg`/`error` string key found, since that
/// covers the common "structured validation error" shape; anything else
/// falls back to a compact JSON rendering so the information isn't lost.
fn deserialize_flexible_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(v.and_then(|val| match val {
        serde_json::Value::String(s) if !s.is_empty() => Some(s),
        serde_json::Value::Null | serde_json::Value::String(_) => None,
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        serde_json::Value::Object(map) => ["message", "msg", "error"]
            .iter()
            .find_map(|key| map.get(*key).and_then(|v| v.as_str()))
            .map(str::to_string)
            .or_else(|| serde_json::to_string(&map).ok()),
        serde_json::Value::Array(_) => serde_json::to_string(&val).ok(),
    }))
}
