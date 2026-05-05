# Security policy

## Reporting a vulnerability

If you think you've found a security issue in BookWorx — anything that could
leak credentials, allow account takeover, or compromise a member's HOTWORX
session — please **do not open a public GitHub issue**.

Instead, email the maintainer privately:

> Nader Heidari — open a GitHub Issue marked `[SECURITY]` *with no details*
> asking for a private contact, or use the GitHub security advisory system
> for this repository to file a draft.

You can expect:

- An acknowledgement within 7 days.
- A best-effort fix or mitigation timeline once the issue is reproduced.
- Public disclosure coordinated with you after a fix ships.

Please include in your report:

- A clear description of the issue and its impact.
- Steps to reproduce, ideally a minimal proof-of-concept.
- The version (commit hash or release tag) you're testing against.
- Any environment specifics (OS, Tauri version) that matter.

## Scope

In scope:

- The desktop app (`src/`, `src-tauri/`) — token storage, IPC surface, UI.
- The `hotworx-api` crate — auth handling, HTTP request construction.
- The `hotworx-mcp` server — token storage, MCP tool surface.

Out of scope:

- Vulnerabilities in HOTWORX's own API or infrastructure. Report those
  directly to HOTWORX.
- Issues that require an attacker to already have full access to the
  victim's machine (e.g. read `~/.hotworx-mcp/config.json` directly).
  That's the threat model; the AES-256-GCM token-at-rest encryption is
  defence in depth, not a guarantee against local attackers.
- Attacks against the official HOTWORX mobile app or web portal.

## Threat model summary

- We assume the user's device is trusted while they're logged in.
- Bearer tokens are encrypted at rest with a key derived from a device
  identifier; this raises the bar against casual disk inspection.
- Passwords are SHA-256 hashed before leaving the device — same as the
  official app's protocol — and never stored in any form.
- The native backend is the only thing that talks to HOTWORX. The WASM
  frontend cannot make HOTWORX requests directly.
