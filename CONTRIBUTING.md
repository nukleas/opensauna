# Contributing to BookWorx

Thanks for your interest. Here's how to get involved.

## Setup

Follow the [Getting Started](README.md#getting-started) section in the README.

## What to work on

- Check [Issues](../../issues) for open bugs and feature requests.
- Before starting a big PR, open an issue to discuss the approach. Saves everyone time.
- Small fixes (typos, UI tweaks, bug fixes) are always welcome — just open a PR.

## Code style

Before submitting, make sure your code passes these checks (CI runs them too):

```bash
# Format
cargo fmt --all

# Lint the backend
cargo clippy -p bookworx -- -D warnings

# Run backend tests
cargo test -p bookworx
```

If `cargo fmt` changes anything, commit those changes. CI will reject unformatted code.

## Pull request process

1. Fork the repo and create a branch from `main`.
2. Make your changes. Keep PRs focused — one feature or fix per PR.
3. Write a short description of what you changed and why.
4. Open a PR against `main`.

If your change touches the API layer or encryption, call that out in the PR description so reviewers know to look closely.

## Releasing

Releases are tag-driven. Pushing a `v*` tag triggers CI to build the desktop
bundles and a signed Android APK/AAB and publish them to a GitHub Release.

1. Bump the version in **both** `src-tauri/tauri.conf.json` and
   `src-tauri/Cargo.toml` to the new `X.Y.Z` (keep them identical). Optionally
   bump `tauri.android.versionName` in `src-tauri/gen/android/app/tauri.properties`
   to match — CI overrides it at build time, but it keeps local builds honest.
2. Commit, then tag and push:

   ```bash
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

3. CI publishes the GitHub Release. The Android `versionCode` comes from the
   monotonic CI run number, so it always strictly increases — **never** hand-set
   it, and **never** regenerate the signing keystore (doing either breaks
   in-place updates on already-installed phones).

Pre-release tags (`-alpha`/`-beta`/`-rc`) are marked as GitHub pre-releases and
are ignored by Obtainium by default.

## Android signing (one-time, maintainers only)

The release job signs the APK with a keystore injected via repo secrets. To set
it up once:

```bash
# Generate the upload keystore (back it up offline — losing it is unrecoverable)
keytool -genkey -v -keystore upload-keystore.jks \
  -keyalg RSA -keysize 2048 -validity 10000 -alias upload

# Base64-encode it for the GitHub secret
base64 -i upload-keystore.jks | pbcopy   # macOS (Linux: base64 -w0 upload-keystore.jks)
```

Then add four repository secrets (Settings → Secrets and variables → Actions):

| Secret | Value |
|---|---|
| `ANDROID_KEYSTORE_BASE64` | the base64 blob from above |
| `ANDROID_KEY_ALIAS` | `upload` |
| `ANDROID_KEYSTORE_PASSWORD` | the store password you set |
| `ANDROID_KEY_PASSWORD` | the key password (set equal to the store password if you didn't pick a separate one) |

The keystore and `keystore.properties` are `.gitignore`d and must never be
committed. On a `v*` tag with no keystore secret, the build fails fast rather
than shipping an unsigned release.

## Questions?

Open an issue. If you use HOTWORX and something about this app bugs you, we'd love to hear about it.
