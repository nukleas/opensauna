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
    pub msg: Option<String>,
    /// Bearer token to attach to subsequent requests via
    /// [`HotworxClient::with_token`](crate::HotworxClient::with_token).
    pub token: Option<String>,
    /// `"yes"` when the account requires OTP verification before the token
    /// is fully usable.
    pub two_factor: Option<String>,
    /// Server-supplied error message, if any.
    pub error: Option<String>,
    /// Coarse-grained status string. Often `"ok"` or `"error"`.
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
    pub user_id: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub location_id: Option<String>,
    pub gender: Option<String>,
    pub dob: Option<String>,
    pub height: Option<String>,
    pub weight: Option<String>,
    pub image_url: Option<String>,
    pub full_name: Option<String>,
}
