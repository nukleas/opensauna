use serde::{Deserialize, Serialize};

/// Login request body for password-based login
#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest {
    pub email_address: String,
    pub password: String, // SHA-256 hashed
    pub device_id: String,
}

/// Login request for OTP-based login
#[derive(Debug, Clone, Serialize)]
pub struct OtpLoginRequest {
    pub email_address: String,
    pub phone_number: String,
    pub device_id: String,
}

/// OTP verification request
#[derive(Debug, Clone, Serialize)]
pub struct VerifyOtpRequest {
    pub email_address: String,
    pub password: String,
    pub phone_number: String,
    pub device_id: String,
    pub otp: String,
    #[serde(rename = "type")]
    pub login_type: String,
}

/// Generic API response wrapper
#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse<T> {
    pub status: Option<String>,
    pub msg: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
    pub data: Option<T>,
}

/// Login response data (works for both initial login and OTP verification)
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LoginResponse {
    pub msg: Option<String>,
    pub token: Option<String>,
    pub two_factor: Option<String>,
    pub error: Option<String>,
    pub status: Option<String>,
    pub data: Option<UserProfile>, // Present after successful OTP verification
}

/// User profile data returned after successful login
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

impl LoginResponse {
    /// Returns `true` if the response contains a token and no error.
    pub fn is_success(&self) -> bool {
        self.token.is_some() && self.error.is_none()
    }

    /// Returns `true` if the server is requesting OTP verification.
    pub fn requires_otp(&self) -> bool {
        self.two_factor.as_deref() == Some("yes")
    }
}
