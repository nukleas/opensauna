## What this changes

<!-- One or two sentences. The "why," not just the "what." -->

## Why

<!-- Optional, but useful for non-trivial changes. Link to an issue if there is one. -->

## How to verify

<!-- Steps a reviewer (or future you) can follow to test the behaviour. -->

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] For UI changes: ran `cargo tauri dev` and exercised the affected screens
- [ ] No secrets, tokens, or credentials in the diff

## Screenshots

<!-- For UI changes — paste before/after if it helps. -->
