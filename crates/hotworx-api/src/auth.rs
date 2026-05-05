//! Authentication helpers.

use sha2::{Digest, Sha256};

/// Hash a plaintext password the way the HOTWORX API expects.
///
/// HOTWORX's login endpoint accepts the SHA-256 hex digest of the password,
/// **not** the plaintext. This matches what the official Android app sends
/// over the wire. The hash is unsalted by design — that's a property of
/// HOTWORX's login protocol, not a choice of this crate.
///
/// ```
/// assert_eq!(
///     hotworx_api::password_hash("password"),
///     "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8"
/// );
/// ```
pub fn password_hash(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}
