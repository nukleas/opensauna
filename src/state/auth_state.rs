use leptos::prelude::*;
use crate::api::client::{get_auth_token, store_auth_token, clear_auth_token, ApiClient};
use crate::models::auth::UserProfile;

/// Global authentication state
#[derive(Clone, Copy)]
pub struct AuthState {
    /// The current auth token (if authenticated)
    pub token: RwSignal<Option<String>>,
    /// Current user profile
    pub user: RwSignal<Option<UserProfile>>,
    /// Loading state for auth operations
    pub loading: RwSignal<bool>,
    /// Error message for auth operations
    pub error: RwSignal<Option<String>>,
}

impl AuthState {
    /// Create a new AuthState
    pub fn new() -> Self {
        Self {
            token: RwSignal::new(None),
            user: RwSignal::new(None),
            loading: RwSignal::new(true), // Start true until restore_session completes
            error: RwSignal::new(None),
        }
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> Signal<bool> {
        let token = self.token;
        Signal::derive(move || token.get().is_some())
    }

    /// Get an API client with the current token
    pub fn api_client(&self) -> ApiClient {
        match self.token.get() {
            Some(token) => ApiClient::with_token(token),
            None => ApiClient::new(),
        }
    }

    /// Set the auth token (and persist it)
    pub async fn set_token(&self, token: String) {
        self.token.set(Some(token.clone()));
        if let Err(e) = store_auth_token(&token).await {
            leptos::logging::error!("Failed to persist token: {}", e);
        }
    }

    /// Clear the auth token (logout)
    pub async fn logout(&self) {
        self.token.set(None);
        self.user.set(None);
        if let Err(e) = clear_auth_token().await {
            leptos::logging::error!("Failed to clear token: {}", e);
        }
    }

    /// Try to restore token from storage
    pub async fn restore_session(&self) {
        self.loading.set(true);
        match get_auth_token().await {
            Ok(Some(token)) => {
                self.token.set(Some(token));
            }
            Ok(None) => {
                // No stored token
            }
            Err(e) => {
                leptos::logging::error!("Failed to restore token: {}", e);
            }
        }
        self.loading.set(false);
    }

    /// Set an error message
    pub fn set_error(&self, error: String) {
        self.error.set(Some(error));
    }

    /// Clear any error
    pub fn clear_error(&self) {
        self.error.set(None);
    }
}

impl Default for AuthState {
    fn default() -> Self {
        Self::new()
    }
}

/// Provide AuthState context at the app root
pub fn provide_auth_state() -> AuthState {
    let auth_state = AuthState::new();
    provide_context(auth_state.clone());
    auth_state
}

/// Use AuthState from context
pub fn use_auth_state() -> AuthState {
    use_context::<AuthState>().expect("AuthState must be provided")
}

/// Pending login data for OTP flow
#[derive(Clone)]
pub struct PendingLogin {
    pub email: String,
    pub password: String,
    pub token: String,
}

/// Provide PendingLogin context at the app root
pub fn provide_pending_login() -> RwSignal<Option<PendingLogin>> {
    let pending = RwSignal::new(None);
    provide_context(pending);
    pending
}

/// Use PendingLogin from context
pub fn use_pending_login() -> RwSignal<Option<PendingLogin>> {
    use_context::<RwSignal<Option<PendingLogin>>>().expect("PendingLogin must be provided")
}
