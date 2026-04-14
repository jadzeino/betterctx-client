# better-ctx-bin

Pre-built binary distribution of [better-ctx](https://github.com/jadzeino/betterctx-client) — the Cognitive Filter for AI Engineering.

No Rust toolchain required. The correct binary for your platform is downloaded automatically during `npm install`.

## Install

```bash
npm install -g better-ctx-bin
```

After installing, run the one-command setup:

```bash
better-ctx setup
```

This auto-detects your shell and editors, installs shell aliases, creates MCP configs, and verifies everything.

## Supported Platforms

| Platform | Architecture |
|----------|-------------|
| Linux | x86_64, aarch64 |
| macOS | x86_64, Apple Silicon |
| Windows | x86_64 |

## Alternative Install Methods

```bash
# Universal installer (no Rust needed)
curl -fsSL https://betterctx.com/install.sh | sh

# Homebrew (macOS/Linux)
brew tap jadzeino/betterctx-client && brew install better-ctx

# Cargo (requires Rust)
cargo install better-ctx
```

## Links

- [Documentation](https://betterctx.com/docs/getting-started)
- [GitHub](https://github.com/jadzeino/betterctx-client)
- [crates.io](https://crates.io/crates/better-ctx)
- [Discord](https://discord.gg/pTHkG9Hew9)
