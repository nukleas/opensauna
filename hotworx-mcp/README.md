# hotworx-mcp

A [Model Context Protocol](https://modelcontextprotocol.io) server that
exposes HOTWORX as tools for Claude Code (and any other MCP client).

Built on top of [`hotworx-api`](../crates/hotworx-api) — the same crate
the OpenSauna desktop app uses. If you want to drive HOTWORX from a chat
agent, this is the front door.

## What it does

Once installed and authenticated, the assistant can:

- Read your dashboard (today's sessions, summary stats).
- List studios, available session types, and time slots.
- Book and cancel sessions.
- Pull profile and lifetime calorie stats.
- Page through activity history.

Every action goes through the same HOTWORX endpoints the official Android
app uses; nothing privileged or undocumented.

## Install

From the repo root:

```bash
cargo install --path hotworx-mcp
```

That puts a `hotworx-mcp` binary on your PATH. The server speaks MCP over
stdio; it isn't meant to be run directly — your MCP client invokes it.

### Wiring it up to Claude Code

```bash
claude mcp add hotworx hotworx-mcp
```

…or, in `~/.config/claude/mcp.json`:

```json
{
  "mcpServers": {
    "hotworx": {
      "command": "hotworx-mcp"
    }
  }
}
```

Restart your Claude Code session and the `hotworx_*` tools will be
available.

## First run

The first MCP call generates a per-install device ID and writes it (along
with your auth token, once you log in) to:

```
~/.hotworx-mcp/config.json
```

To authenticate:

```
hotworx_login(email="me@example.com", password="…")
# If two-factor is enabled:
hotworx_verify_otp(otp="123456")
```

After that the token persists across sessions until you `hotworx_logout`
or HOTWORX expires it server-side.

## Tools

| Tool | Description |
|---|---|
| `hotworx_login` | Sign in with email + password |
| `hotworx_verify_otp` | Submit the OTP code if 2FA is enabled |
| `hotworx_logout` | Clear stored credentials |
| `hotworx_dashboard` | Today's pending + completed sessions, summary stats |
| `hotworx_get_locations` | All bookable studios |
| `hotworx_get_session_types` | Workouts available at a location/date |
| `hotworx_get_available_slots` | Time slots for a session type |
| `hotworx_book_session` | Reserve a slot |
| `hotworx_cancel_session` | Cancel a previous booking |
| `hotworx_get_profile` | Member profile (name, email, height, weight) |
| `hotworx_get_activity_history` | Paginated past sessions |
| `hotworx_get_calorie_stats` | Lifetime calorie burn summary |

## Disclaimer

Unaffiliated, unofficial, not endorsed by HOTWORX. Use at your own risk.
Your credentials never leave your machine — the server hashes the
password before sending it (per the HOTWORX login protocol) and stores
only the bearer token returned by HOTWORX.

## License

MIT.
