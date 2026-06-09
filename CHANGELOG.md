# Changelog

All notable changes to BookWorx are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project aims
to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Per-release binaries and auto-generated notes also live on the
[GitHub Releases](../../releases) page.

## [Unreleased]

### Fixed

- **OTP login.** The two-factor step sent the plaintext password where the
  HOTWORX `verifyOtp` endpoint expects the SHA-256 digest, causing verification
  to fail. The password is now hashed before sending, matching the login step.
- **`delete_session` no longer reports false success.** HOTWORX returns HTTP 200
  with an `error` body on a failed cancellation; the SDK now surfaces that
  instead of silently returning `Ok`.
- **Active-session timer leak.** The 1 Hz timer was leaked with `mem::forget`
  and kept firing after unmount; it is now cleared on cleanup.
- **End-session failures** no longer leave the overlay stuck on "Ending…".

### Changed

- Auth errors are differentiated: an expired session and "not logged in" now
  show distinct messages.
- Targeted HOTWORX app version bumped to **6.6.3** (was 6.5.5).
- Booking and Quick Book now confirm success with a toast.
- App version unified to `0.3.1` across `tauri.conf.json` and `Cargo.toml`.

### Security

- The pending-login password is now **encrypted at rest** during the OTP step
  (previously written in plaintext) and cleared on abandon.
- The bearer token is no longer logged to the JS console during OTP.

### Added

- Android release pipeline: tag-driven, signed universal APK/AAB published to
  GitHub Releases, with a monotonic `versionCode` for reliable in-place updates.
- `PRIVACY.md`, this changelog, and release/signing docs in `CONTRIBUTING.md`.
- OTP input accessibility attributes (`aria-label`, `inputmode`,
  `autocomplete="one-time-code"`).

## Released

Earlier releases — `v0.3.0` (2026-03-11), `v0.2.2`, `v0.2.1`, `v0.2.0`
(2026-03-03), and `v0.1.0` (2026-02-21) — predate this changelog; see their
notes on the [GitHub Releases](../../releases) page.
