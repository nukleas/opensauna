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
    /// The server rejected the request as unauthenticated (HOTWORX uses its
    /// own HTTP 551 for a missing or expired token; 401 and 403 are also
    /// accepted), or the client was constructed without a token for an
    /// endpoint that needs one.
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

    /// The response body was a 2xx but couldn't be deserialized. `body` is
    /// the start of the raw response (at most [`DECODE_BODY_EXCERPT`]
    /// bytes) so a shape change on HOTWORX's side is diagnosable from the
    /// error alone.
    #[error("could not decode response: {source}; body: {body}")]
    Decode {
        source: serde_json::Error,
        body: String,
    },
}

/// Maximum number of bytes of the raw body kept in [`HotworxError::Decode`].
pub const DECODE_BODY_EXCERPT: usize = 600;

impl HotworxError {
    /// Build an [`HotworxError::Http`] from a status code and an arbitrary
    /// body, mapping 401, 403 and HOTWORX's custom 551 ("token not provided"
    /// / "provided token is expired") to [`HotworxError::AuthExpired`].
    pub(crate) fn from_status(status: u16, body: String) -> Self {
        if matches!(status, 401 | 403 | 551) {
            HotworxError::AuthExpired
        } else {
            HotworxError::Http { status, body }
        }
    }

    /// Build an [`HotworxError::Decode`] keeping only the first
    /// [`DECODE_BODY_EXCERPT`] bytes of `body` (cut on a char boundary).
    pub(crate) fn decode(source: serde_json::Error, body: &str) -> Self {
        let mut end = body.len().min(DECODE_BODY_EXCERPT);
        while !body.is_char_boundary(end) {
            end -= 1;
        }
        let mut excerpt = body[..end].to_string();
        if end < body.len() {
            excerpt.push('…');
        }
        HotworxError::Decode {
            source,
            body: excerpt,
        }
    }
}
