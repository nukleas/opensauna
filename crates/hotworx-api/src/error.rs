//! Errors returned by [`crate::HotworxClient`].

use thiserror::Error;

/// Convenience alias for `Result<T, HotworxError>`.
pub type Result<T> = std::result::Result<T, HotworxError>;

/// Anything that can go wrong while talking to HOTWORX.
///
/// Callers typically care most about [`HotworxError::AuthExpired`]: it tells
/// you the stored token is no longer accepted and the user must log in
/// again. Everything else is best treated as a transient or programmer error
/// and surfaced verbatim.
#[derive(Debug, Error)]
pub enum HotworxError {
    /// The server rejected the request with HTTP 401 or 403, or the client
    /// was constructed without a token for an endpoint that needs one.
    /// Callers should clear any stored token and prompt the user to sign in.
    #[error("authentication required or token expired")]
    AuthExpired,

    /// The server returned a non-success HTTP status that wasn't an
    /// authentication failure. The raw body is included for diagnostics.
    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },

    /// The HOTWORX request layer failed (DNS, TLS, timeout, etc.).
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// The response body was a 2xx but couldn't be deserialized.
    #[error("could not decode response: {0}")]
    Decode(#[from] serde_json::Error),
}

impl HotworxError {
    /// Build an [`HotworxError::Http`] from a status code and an arbitrary
    /// body, mapping 401 and 403 to [`HotworxError::AuthExpired`].
    pub(crate) fn from_status(status: u16, body: String) -> Self {
        if status == 401 || status == 403 {
            HotworxError::AuthExpired
        } else {
            HotworxError::Http { status, body }
        }
    }
}
