# Privacy Policy

BookWorx is an unofficial desktop and mobile client for HOTWORX. It is not
affiliated with HOTWORX. This document describes what the app does with your
data. Short version: it talks only to HOTWORX, stores as little as possible
locally, and collects no analytics.

## What the app stores on your device

- **Bearer token.** After you log in, HOTWORX issues a session token. BookWorx
  stores it **encrypted at rest** (AES-256-GCM, keyed per install) and uses it
  to authenticate subsequent requests. It is cleared on logout.
- **Pending login (during OTP only).** If your account requires a one-time code,
  the email and password you entered are held **encrypted at rest** between the
  password step and the code step, and are **deleted** as soon as login
  completes or you leave the screen.
- **Device ID.** A random UUID generated once per install, sent with requests so
  HOTWORX can recognize the device. It is not derived from any hardware
  identifier.
- **Preferences.** Local UI preferences such as your preferred studio and
  session type for Quick Book.

Your password is **SHA-256 hashed before it leaves the device** — the HOTWORX
login protocol expects the hash, so the plaintext password is never sent over
the network and is never written to disk in plaintext.

## What the app sends, and to whom

All network traffic goes to HOTWORX's own API (`sailposapi.hotworx.net`) to
perform the actions you request: logging in, booking and cancelling sessions,
and reading your dashboard, profile, and stats. The app sends the same
app-identifying headers the official HOTWORX Android app sends.

## What the app does **not** do

- No third-party analytics, telemetry, ads, or crash reporting.
- No selling or sharing of data with anyone other than HOTWORX itself.
- No background data collection.

## Your control

Logging out clears the stored token. Uninstalling the app removes all locally
stored data. Your HOTWORX account data is governed by HOTWORX's own privacy
policy.

## Contact

Questions or concerns: open an issue on the project repository.
