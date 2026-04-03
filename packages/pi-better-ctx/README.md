# pi-better-ctx

[Pi Coding Agent](https://github.com/badlogic/pi-mono) extension that routes all tool output through [better-ctx](https://betterctx.com) for **60–90% token savings**.

## What it does

Overrides Pi's built-in tools to route them through `better-ctx`:

| Tool | Compression |
|------|------------|
| `bash` | All shell commands compressed via better-ctx's 90+ patterns |
| `read` | Smart mode selection (full/map/signatures) based on file type and size |
| `grep` | Results grouped and compressed via ripgrep + better-ctx |
| `find` | File listings compressed and .gitignore-aware |
| `ls` | Directory output compressed |

## Install

```bash
# 1. Install better-ctx (if not already installed)
cargo install better-ctx
# or: brew tap jadzeino/betterctx-client && brew install better-ctx

# 2. Install the Pi package
pi install npm:pi-better-ctx
```

## Binary Resolution

The extension locates the `better-ctx` binary in this order:

1. `BETTER_CTX_BIN` environment variable
2. `~/.cargo/bin/better-ctx`
3. `~/.local/bin/better-ctx` (Linux) or `%APPDATA%\Local\better-ctx\better-ctx.exe` (Windows)
4. `/usr/local/bin/better-ctx` (macOS/Linux)
5. `better-ctx` on PATH

## Smart Read Modes

The `read` tool automatically selects the optimal better-ctx mode:

| File Type | Size | Mode |
|-----------|------|------|
| `.md`, `.json`, `.toml`, `.yaml`, etc. | Any | `full` |
| Code files (55+ extensions) | < 24 KB | `full` |
| Code files | 24–160 KB | `map` (deps + API signatures) |
| Code files | > 160 KB | `signatures` (AST extraction) |
| Other files | < 48 KB | `full` |
| Other files | > 48 KB | `map` |

Code extensions include: `.rs`, `.ts`, `.tsx`, `.js`, `.jsx`, `.py`, `.go`, `.java`, `.c`, `.cpp`, `.cs`, `.rb`, `.php`, `.swift`, `.kt`, `.vue`, `.svelte`, `.astro`, `.html`, `.css`, `.scss`, `.lua`, `.zig`, `.dart`, `.scala`, `.sql`, `.graphql`, `.proto`, `.tf`, `.sh`, `.bash`, `.zsh`, `.fish`, `.ps1`, and more.

**Partial reads** (with `offset`/`limit`) are also routed through better-ctx using `lines:N-M` mode for compression.

## Slash Command

Use `/better-ctx` in Pi to check which binary is being used.

## Links

- [better-ctx](https://betterctx.com) — The Cognitive Filter for AI Engineering
- [GitHub](https://github.com/jadzeino/betterctx-client)
- [Discord](https://discord.gg/betterctx)
