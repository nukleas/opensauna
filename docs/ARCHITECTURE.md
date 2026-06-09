# Architecture

How BookWorx fits together and why.

## Workspace layout

```
HotWorx/                     ← repo root
├── src/                     ── BookWorx frontend (Leptos → WASM)
├── src-tauri/               ── BookWorx native backend (Tauri 2)
├── crates/hotworx-api/      ── reusable HOTWORX SDK (this is where the
│                                wire protocol lives)
└── hotworx-mcp/             ── MCP server for Claude Code, also a
                                consumer of the SDK
```

Three packages, one repository. The SDK was extracted from the desktop
app's backend so a second consumer (the MCP server) could use it without
re-implementing the HOTWORX wire format. See
[Why a separate crate?](#why-a-separate-crate) below.

## Data flow

```
┌──────────────────────────┐        ┌──────────────────────────────┐
│ WASM frontend (src/)     │        │ Native backend (src-tauri/)  │
│                          │ Tauri  │                              │
│   pages → invoke("…")    │ <───►  │   #[tauri::command]          │
│                          │  IPC   │   hotworx_api::HotworxClient │
└──────────────────────────┘        └──────────────────────────────┘
                                                  │
                                                  │ HTTPS
                                                  ▼
                                       ┌──────────────────────┐
                                       │ HOTWORX API          │
                                       │ sailposapi.hotworx.net│
                                       └──────────────────────┘
```

Frontend pages call Tauri commands by name (`api_get_dashboard`,
`api_book_session`, …). Each command on the backend pulls the persisted
auth token, builds a `HotworxClient`, calls one method, and returns the
typed result back across the IPC boundary.

The MCP server has the same shape minus the frontend — Claude Code calls
MCP tools, the server pulls config from `~/.hotworx-mcp/config.json`,
calls `HotworxClient`, returns the result.

## Key decisions

### Why a separate crate?

`hotworx-api` exists because we needed two consumers — the desktop app
and the MCP server. Pulling the wire protocol into a workspace crate
gave us:

1. **One source of truth for the API surface.** When HOTWORX bumps the
   Android app version (which we have to spoof), there's one place to
   change it.
2. **Validation against two real consumers.** If the SDK's API was ever
   awkward to use, we'd notice from one of the two callers immediately.
3. **An option to publish.** The crate is built publish-clean (full
   rustdoc, no leaky deps, semver-conscious surface) but kept
   `publish = false` until we're confident in the API. Anyone else who
   wants a Rust HOTWORX client today can add it as a path dependency
   from this repo; tomorrow we can move it to crates.io with no rewrite.

### Why does the WASM frontend talk to the backend, not the API?

WASM running in a Tauri webview *can* make `fetch()` calls, but with
the same restrictions a regular browser has — most importantly, it
can't send a custom `User-Agent`. The HOTWORX API requires the
`okhttp/4.12.0` UA + `application-version: 6.6.3` headers that mark the
request as coming from the official Android app. Setting those from
WASM isn't possible.

The native backend has none of those restrictions, so all HOTWORX
traffic goes through Tauri IPC commands. As a side benefit this also
keeps the bearer token out of the browser environment entirely.

### AES-256-GCM token-at-rest encryption

Bearer tokens are persisted via `tauri-plugin-store` in `auth.json`
under the OS-conventional app data directory. Before being written they
are encrypted with AES-256-GCM. Each ciphertext starts with a fresh
12-byte random nonce so repeat-encryption of the same token produces
different output — preventing simple correlation.

The encryption key is derived deterministically per-install:

```
key = SHA-256(device_id ‖ "bookworx-token-encryption-salt")
```

The `device_id` is a UUID generated once on first run and stored in
`settings.json` next to the encrypted token. The threat model here
isn't "stop a determined attacker who has full disk access" — they can
reproduce the key from the same `settings.json`. It is "stop a passing
glance at `auth.json` from leaking a working bearer token," which is
roughly what file permissions-only protection achieves on every other
desktop app, with a meaningful improvement at the cost of one
additional file read.

If decryption ever fails (corruption, the salt changing in a future
version, partial migration), we treat the stored token as junk and
return `None`. The frontend's auth-expired flow then handles the rest.

### Local session tracking

When the user starts a workout in the app, we don't expect the HOTWORX
server to have a real-time session-state machine. The active-session
timer state lives entirely in `sessions.json` on the local machine,
updated and persisted by the backend. This means the timer keeps
running across app restarts and remains accurate even if the API is
slow or briefly unreachable. The MCP server doesn't have this concept;
it's purely a transactional client.

### Android-app header spoofing

The HOTWORX API ignores requests that don't claim to be the Android
app. We send the same set of headers the official app does:

| Header | Value | Why |
|---|---|---|
| `User-Agent` | `okhttp/4.12.0` | Android client signature |
| `application-version` | `6.6.3` | App version compatibility |
| `sec-ch-ua-platform` | `Android` | Platform routing |
| `device-id` | UUID per install | Required field |

These constants live in `crates/hotworx-api/src/headers.rs`. When
HOTWORX rolls out a new app version that changes the protocol, that
file is the single edit needed.

### Auth-expiry detection

When the bearer token is no longer accepted (HTTP 401/403, or the
backend has nothing stored), the SDK returns
`HotworxError::AuthExpired`. The Tauri layer prefixes the error string
sent across IPC with `AUTH_EXPIRED:`. The frontend's
`handle_invoke_error` sees that prefix, calls `auth.logout()`, shows a
toast, and the existing `<Show>`/`<Redirect>` guards in `app.rs` send
the user back to `/login`.

This survives concurrent failures gracefully — the helper checks
whether logout has already happened before showing a duplicate toast —
and gives every page a single error path to wire up. A future
refactor will replace the string sentinel with a typed IPC error code,
but the current scheme is robust enough to not block external
publishing.

## Things this design is *not* trying to solve

- **Offline mode.** Booking, dashboard, and profile views all require
  the API. If you're offline, you see "Session expired" or a network
  error. The app remembers your active session timer locally; that's
  the only offline-capable surface.
- **Multi-account support.** One HOTWORX account per install. Adding a
  second would require namespacing every key in `auth.json` and is
  outside scope.
- **Reverse-engineering protection.** This is an unofficial client
  using the same protocol as the public Android app. We don't try to
  obscure that.
