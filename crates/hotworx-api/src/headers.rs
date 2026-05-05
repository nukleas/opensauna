//! HTTP headers the HOTWORX backend expects.
//!
//! The public HOTWORX API only accepts requests that look like they're
//! coming from the official Android app. We send the same `User-Agent`,
//! `application-version`, and platform headers the app uses.

/// Base URL for the HOTWORX REST API.
pub const BASE_URL: &str = "https://sailposapi.hotworx.net/api/v1";

/// HOTWORX Android app version this crate targets. Bumping this when the
/// upstream app updates is the most common reason to publish a new release
/// of `hotworx-api`.
pub const APP_VERSION: &str = "6.5.5";

/// User-Agent sent on every request (matches the Android `okhttp` build).
pub const USER_AGENT: &str = "okhttp/4.12.0";

/// Platform string the API sniffs when routing requests.
pub const PLATFORM: &str = "Android";

/// Apply the standard HOTWORX-app headers to a [`reqwest::RequestBuilder`].
///
/// The caller supplies the per-request `device_id` (an unguessable, stable
/// identifier — typically a UUID generated once per install).
pub(crate) fn apply_app_headers(
    builder: reqwest::RequestBuilder,
    device_id: &str,
) -> reqwest::RequestBuilder {
    builder
        .header("User-Agent", USER_AGENT)
        .header("sec-ch-ua-platform", PLATFORM)
        .header("application-version", APP_VERSION)
        .header("device-id", device_id)
}
