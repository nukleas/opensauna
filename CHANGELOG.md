# Changelog

All notable changes to OpenSauna are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project aims
to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Per-release binaries and auto-generated notes also live on the
[GitHub Releases](../../releases) page.

## [0.4.3] - 2026-06-13

### Fixed

- **macOS DMG is now notarized + stapled.** Tauri notarizes the `.app` but not
  the `.dmg` wrapper, so the downloaded DMG still tripped Gatekeeper ("Apple
  could not verify it is free of malware") on open. CI now submits the DMG to
  Apple's notary and staples the ticket, so the downloaded file opens clean.

## [0.4.2] - 2026-06-13

Hotfix for 0.4.1, which tagged but failed to publish: the Windows bundle step
broke, blocking the release. Ships the 0.4.1 signing work below plus the fix.

### Fixed

- **Windows desktop build.** The `NO_COLOR=false` prefix on the Trunk
  `beforeBuildCommand` is Unix-only shell syntax and failed on the Windows
  runner; reverted to a plain `trunk build` so the bundle step succeeds.

## [0.4.1] - 2026-06-13

Release-engineering hardening — the macOS build is now properly signed and
notarized, so the downloaded DMG opens without Gatekeeper warnings.

### Changed

- **macOS releases are signed with a Developer ID Application certificate and
  notarized + stapled** in CI, so a downloaded DMG is accepted by Gatekeeper
  with no "unidentified developer" prompt. The signing identity is auto-detected
  from the imported certificate, so there's no brittle hand-set identity string.
- Dev/build commands force colored Trunk output (`NO_COLOR=false`).

### Added

- `docs/SIGNING.md` — per-platform signing reference (macOS Developer ID,
  Android keystore) and the deliberate choice to ship Windows **unsigned** (a
  one-click SmartScreen "More info → Run anyway") rather than pay for an
  Authenticode certificate.

## [0.4.0] - 2026-06-08

The OpenSauna rebrand release: renamed and re-iconed, with a round of feature
polish, interaction feedback, and bug fixes.

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
- **"Completed Today" mismatch.** It read from `getDashboard`'s often-empty
  `todays_completed_sessions` while the history clearly showed today's sessions;
  it's now derived from the same activity history, so the two always agree.
- **Session-history date duplication** ("…19:33:25 at 19:33:25") — the time is
  only appended when the date string doesn't already include it.
- A leftover dev-server rebuild/reload loop (trunk now ignores build/tooling
  scratch dirs).

### Changed

- **Rebranded BookWorx → OpenSauna.** Product name, app/window title, Cargo
  packages, and the Android/Apple bundle identifier are now `opensauna`
  (`com.nukleas.opensauna`). The `hotworx-api`/`hotworx-mcp` crates keep their
  descriptive names (they're clients *for* the HOTWORX API). Removed the
  hardcoded Apple development team ID.
- Auth errors are differentiated: an expired session and "not logged in" now
  show distinct messages.
- Targeted HOTWORX app version bumped to **6.6.3** (was 6.5.5).
- Booking and Quick Book now confirm success with a toast.
- **New app icon** — stacked sauna stones with rising steam (warm ember
  palette); login logo now matches it (was the old flame). The macOS icon is a
  rounded squircle with margins (macOS icon grid).
- **Interaction feedback:** "Set as Favorite" confirms with a toast and a
  "★ Saved" state; cancelling a session toasts; Profile/Goals saves use the
  global toasts; tappable cards/chips/buttons have press (`:active`) states.
- **Logout** is now a two-step confirm.
- **OTP entry** is six digit boxes; the dead "Resend" button became a hint.
- **Date selection** is a Today/Tomorrow/<day> pill row instead of a native
  date picker (booking is a 3-day window anyway).
- Calories and session counts over 1,000 now show **thousands separators**
  (e.g. `12,500`).
- App version is `0.4.0` across `tauri.conf.json` and `Cargo.toml`.

### Security

- The pending-login password is now **encrypted at rest** during the OTP step
  (previously written in plaintext) and cleared on abandon.
- The bearer token is no longer logged to the JS console during OTP.

### Added

- Android release pipeline: tag-driven, signed universal APK/AAB published to
  GitHub Releases, with a monotonic `versionCode` for reliable in-place updates.
- `PRIVACY.md`, this changelog, and release/signing docs in `CONTRIBUTING.md`.
- `docs/why-opensauna.md` (the project's rationale and documented critiques of
  the official app) and a top-level `NOTICE` (trademark/interoperability/legal).
- OTP input accessibility attributes (`aria-label`, `inputmode`,
  `autocomplete="one-time-code"`).
- The OTP screen prefills `123456` — HOTWORX's one-time code is that constant
  value every time, so prefilling it is correct (the field stays editable).

## Released

Earlier releases — `v0.3.0` (2026-03-11), `v0.2.2`, `v0.2.1`, `v0.2.0`
(2026-03-03), and `v0.1.0` (2026-02-21) — predate this changelog; see their
notes on the [GitHub Releases](../../releases) page.
