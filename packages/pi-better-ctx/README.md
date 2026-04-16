# pi-better-ctx

[Pi Coding Agent](https://github.com/badlogic/pi-mono) extension with **first-class MCP support** — routes all tool output through [better-ctx](https://betterctx.com) for **60–90% token savings** and exposes **25+ MCP tools** natively in Pi.

## What it does

### Built-in Tool Overrides (CLI)

Overrides Pi's built-in tools to route them through `better-ctx`:

| Tool | Compression |
|------|------------|
| `bash` | All shell commands compressed via better-ctx's 90+ patterns |
| `read` | Smart mode selection (full/map/signatures) based on file type and size |
| `grep` | Results grouped and compressed via ripgrep + better-ctx |
| `find` | File listings compressed and .gitignore-aware |
| `ls` | Directory output compressed |

### MCP Tools (Embedded Bridge)

Additionally, pi-better-ctx spawns better-ctx as an MCP server and registers all advanced tools directly in Pi:

| Tool | Purpose |
|------|---------|
| `ctx_session` | Session state management and persistence |
| `ctx_knowledge` | Project knowledge graph with temporal validity |
| `ctx_semantic_search` | Find code by meaning, not exact text |
| `ctx_overview` | Codebase overview and architecture analysis |
| `ctx_compress` | Manual compression control |
| `ctx_metrics` | Token savings dashboard |
| `ctx_agent` | Multi-agent coordination and handoffs |
| `ctx_graph` | Dependency graph analysis |
| `ctx_discover` | Smart code discovery |
| `ctx_context` | Context window management |
| `ctx_preload` | Predictive file preloading |
| `ctx_delta` | Changed-lines-only reads |
| `ctx_edit` | Read-modify-write in one call |
| `ctx_dedup` | Duplicate context elimination |
| `ctx_fill` | Template completion |
| `ctx_intent` | Intent-based task routing |
| `ctx_response` | Response optimization |
| `ctx_wrapped` | Wrapped command execution |
| `ctx_benchmark` | Compression benchmarking |
| `ctx_analyze` | Code analysis |
| `ctx_cache` | Cache management |
| `ctx_execute` | Direct command execution |

These MCP tools are the same ones available in Cursor, Claude Code, VS Code, and all other supported editors — now first-class in Pi.

## Install

```bash
# 1. Install better-ctx (if not already installed)
cargo install better-ctx
# or: brew tap jadzeino/betterctx-client && brew install better-ctx

# 2. Install the Pi package
pi install npm:pi-better-ctx

# 3. Restart Pi
```

Or use the automated setup:

```bash
better-ctx init --agent pi
```

## How it works

### CLI overrides (bash, read, grep, find, ls)

These tools invoke the `better-ctx` binary via CLI with `BETTER_CTX_COMPRESS=1`. The output is parsed for compression stats and displayed with a token savings footer.

### MCP bridge (all other tools)

On startup, pi-better-ctx spawns the `better-ctx` binary as an MCP server (JSON-RPC over stdio). It discovers available tools via `list_tools`, filters out those already covered by CLI overrides, and registers the rest as native Pi tools.

If `better-ctx` is already configured as an MCP server via [pi-mcp-adapter](https://github.com/nicobailon/pi-mcp-adapter) in `~/.pi/agent/mcp.json`, the embedded bridge is skipped to avoid duplicate tools.

### Automatic reconnection

If the MCP server process crashes, the bridge automatically reconnects (up to 3 attempts with exponential backoff). If reconnection fails, CLI-based tools continue working normally — only the advanced MCP tools become unavailable.

## pi-mcp-adapter compatibility

If you prefer using [pi-mcp-adapter](https://github.com/nicobailon/pi-mcp-adapter) to manage your MCP servers, better-ctx integrates automatically:

```bash
# Option A: better-ctx writes the config for you
better-ctx init --agent pi

# Option B: Manual configuration in ~/.pi/agent/mcp.json
```

```json
{
  "mcpServers": {
    "better-ctx": {
      "command": "/path/to/better-ctx",
      "lifecycle": "lazy",
      "directTools": true
    }
  }
}
```

When pi-mcp-adapter manages the better-ctx MCP server, pi-better-ctx detects this and only registers its CLI-based tool overrides, leaving MCP tool management to the adapter.

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
| Code files (55+ extensions) | < 8 KB | `full` |
| Code files | 8–96 KB | `map` (deps + API signatures) |
| Code files | > 96 KB | `signatures` (AST extraction) |
| Other files | < 48 KB | `full` |
| Other files | > 48 KB | `map` |

## Slash Command

Use `/better-ctx` in Pi to check:
- Which binary is being used
- MCP bridge status (embedded vs. adapter, connected/disconnected)
- Number and names of registered MCP tools

## Disabling specific tools

To disable specific MCP tools, configure `disabled_tools` in `~/.better-ctx/config.toml`:

```toml
disabled_tools = ["ctx_graph", "ctx_benchmark"]
```

Or via environment variable:

```bash
BETTER_CTX_DISABLED_TOOLS=ctx_graph,ctx_benchmark pi
```

## Links

- [better-ctx](https://betterctx.com) — The Cognitive Filter for AI Engineering
- [GitHub](https://github.com/jadzeino/betterctx-client)
- [Discord](https://discord.gg/betterctx)
