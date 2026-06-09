# hotworx-api

Unofficial Rust client for the HOTWORX member API.

`hotworx-api` is a small, typed wrapper around the HOTWORX HTTP backend that
the official Android app uses. It's designed to be the foundation for any
Rust tool that wants to read or write your HOTWORX data — desktop apps,
home dashboards, automation scripts, MCP servers, whatever.

## Disclaimer

This crate is **not** affiliated with, endorsed by, or connected to HOTWORX
in any way. It exists because the official app is the only sanctioned client
for HOTWORX members and that's not always enough.

The crate sends headers identifying itself as the HOTWORX Android app —
that's required for the API to respond. If HOTWORX would prefer a different
arrangement we'll happily accommodate.

## Usage

```toml
[dependencies]
hotworx-api = { path = "../crates/hotworx-api" }
# or, once published:
# hotworx-api = "0.1"
```

```rust
use hotworx_api::HotworxClient;

#[tokio::main]
async fn main() -> hotworx_api::Result<()> {
    // Pick a stable per-install identifier (a UUID is fine) and reuse it.
    let device_id = "11111111-2222-3333-4444-555555555555";

    let client = HotworxClient::new(device_id);
    let login = client
        .login_with_password("me@example.com", "my-plaintext-password")
        .await?;

    let token = login.token.expect("first-factor login succeeded");
    let dashboard = HotworxClient::new(device_id)
        .with_token(&token)
        .get_dashboard(None)
        .await?;

    if let Some(sessions) = dashboard.todays_pending_sessions {
        println!("{} sessions booked today", sessions.len());
    }
    Ok(())
}
```

## What's covered

| Endpoint family | Methods |
|---|---|
| Auth | `login_with_password`, `verify_otp` |
| Dashboard | `get_dashboard` |
| Booking | `get_locations`, `get_session_types`, `show_slots`, `book_session`, `delete_session` |
| Profile | `view_profile`, `update_profile` |
| Goals | `view_goals`, `update_goals` |
| Weight | `get_weight`, `set_weight` |
| Stats | `get_summary`, `get_thirty_day_summary`, `get_ninety_day_summary`, `get_calorie_stats` |
| Activity | `get_activity_history` |

The client is stateless beyond the device ID and bearer token — token storage
and encryption are intentionally out of scope. See the OpenSauna app in this
repo for a reference implementation that handles AES-256-GCM token-at-rest
encryption on top of `hotworx-api`.

## Errors

Most calls fail with [`HotworxError::AuthExpired`] when the token is no
longer accepted (HTTP 401/403, or no token set on a call that needs one) —
that's the signal to clear stored credentials and ask the user to log in
again. Other variants cover transport failures, non-auth HTTP errors, and
unexpected response shapes.

## Stability

`0.x` — the public API will move with HOTWORX's wire format. We track the
HOTWORX Android app version (currently **6.6.3**) in `headers::APP_VERSION`;
expect a minor bump every time HOTWORX rolls out an app release that
changes the protocol.

## License

MIT.
