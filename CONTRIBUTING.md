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

## Questions?

Open an issue. If you use HOTWORX and something about this app bugs you, we'd love to hear about it.
