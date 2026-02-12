use crate::api::client::{get_device_id, hash_password, ApiClient, ApiError};
use crate::models::auth::{LoginRequest, LoginResponse, OtpLoginRequest, VerifyOtpRequest};

impl ApiClient {
    /// Login with email and password
    pub async fn login_with_password(
        &self,
        email: &str,
        password: &str,
    ) -> Result<LoginResponse, ApiError> {
        // Hash the password
        let hashed_password = hash_password(password).await?;

        // Get device ID
        let device_id = get_device_id().await?;

        let request = LoginRequest {
            email_address: email.to_string(),
            password: hashed_password,
            device_id,
        };

        // Use form-urlencoded POST (required by Hotworx API)
        self.post_form("loginwithpassword", &request).await
    }

    /// Request OTP for login
    pub async fn request_otp(&self, email: &str, phone: &str) -> Result<LoginResponse, ApiError> {
        let device_id = get_device_id().await?;

        let request = OtpLoginRequest {
            email_address: email.to_string(),
            phone_number: phone.to_string(),
            device_id,
        };

        self.post_form("login", &request).await
    }

    /// Verify OTP and complete login
    /// The `login_type` parameter should be "password" for password-based login
    pub async fn verify_otp(
        &self,
        email: &str,
        password_hash: &str, // Already hashed password
        phone: &str,
        otp: &str,
    ) -> Result<LoginResponse, ApiError> {
        let device_id = get_device_id().await?;

        let request = VerifyOtpRequest {
            email_address: email.to_string(),
            password: password_hash.to_string(),
            phone_number: phone.to_string(),
            device_id,
            otp: otp.to_string(),
            login_type: "password".to_string(), // "password" for password-based login
        };

        self.post_form("verifyOtp", &request).await
    }
}
