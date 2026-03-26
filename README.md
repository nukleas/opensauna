# BookWorx

A desktop app for managing your HOTWORX sessions. Book faster, track better, skip the phone.

## What is this

BookWorx is a third-party desktop app for HOTWORX members. It talks to the same API as the official mobile app — you get the same data, just on a real screen with a keyboard.

It's open source. Every line of code that touches your credentials is readable right here. Not affiliated with HOTWORX.

## Screenshots

| Dashboard | Quick Book |
|:---------:|:----------:|
| <!-- screenshot: dashboard --> | <!-- screenshot: quick-book --> |

| Session Timer | Sessions History |
|:-------------:|:----------------:|
| <!-- screenshot: session-timer --> | <!-- screenshot: sessions --> |

## Features

**What you get:**

- Desktop booking — pick your studio, workout, and time slot without squinting at your phone
- Quick Book — one tap to rebook your usual session
- Multi-slot selection — book several time slots in one go
- Live session timer with calorie and heart rate tracking
- Activity history with date filters
- Profile and goals editing
- Encrypted token storage (AES-256-GCM)
- Cross-platform: macOS, Windows, Linux, Android

**What you don't get:**

- Ads
- Upsells
- A 200 MB download for a booking form

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- WASM target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install trunk`
- [Tauri CLI v2](https://v2.tauri.app/): `cargo install tauri-cli`
- Platform dependencies — on Ubuntu/Debian:
  ```
  sudo apt-get install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
  ```
  macOS and Windows just need Rust and the tools above.

### Development

```
cargo tauri dev
```

This starts Trunk on port 1420 and opens BookWorx with hot reload.

### Production build

```
cargo tauri build
```

### Android

CI builds signed APKs on pushes to `main` and on tagged releases. See [`.github/workflows/ci.yml`](.github/workflows/ci.yml) for the full pipeline. To build locally, you'll also need the Android SDK, NDK 27, and the `aarch64-linux-android` Rust target.

## How it's built

**Frontend:** Leptos 0.7 compiled to WASM, running client-side in a Tauri webview. No JS framework — it's all Rust.

**Backend:** Tauri 2.0 native process. Handles every API call, token encryption, and local session tracking. The WASM frontend never talks to HOTWORX directly.

**Communication:** Tauri IPC commands. The frontend calls `invoke("command_name", args)` and the backend returns typed responses.

### Directory layout

```
src/                        # Frontend (Leptos → WASM)
├── api/                    # Tauri invoke wrappers
├── components/             # Reusable UI (session timer, toast, nav, etc.)
├── models/                 # Shared types (auth, booking, dashboard, profile)
├── pages/                  # Route pages (login, dashboard, booking, sessions, profile)
├── state/                  # Reactive state (auth, session tracking)
├── app.rs                  # Router + layout
└── main.rs                 # Entry point

src-tauri/                  # Backend (native Rust)
└── src/
    ├── lib.rs              # 26 Tauri commands + encryption + API client
    └── main.rs             # Tauri bootstrap
```

### Key decisions

- **API calls go through the backend, not WASM.** Browser-like WASM can't set arbitrary headers, and we need to spoof the Android app's `User-Agent` for API compatibility. The native backend has no such restrictions.
- **AES-256-GCM token encryption.** Auth tokens are encrypted before they hit disk via `tauri-plugin-store`. The key derives from a device-specific salt.
- **Local session tracking.** Active session state lives on the client so the timer keeps running even if the API is slow.
- **Android app header spoofing.** The HOTWORX API expects mobile app headers (version, device info). The backend mimics what the official Android app sends.

## Disclaimer

This is an unofficial project. It is not affiliated with, endorsed by, or connected to HOTWORX in any way. Use at your own risk.

Your password is SHA-256 hashed before it leaves your machine — the same way the official app handles it. BookWorx never sees or stores your plaintext password.

## License

[MIT](LICENSE)
