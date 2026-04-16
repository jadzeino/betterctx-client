# better-ctx for VS Code

**Context Runtime for AI Agents** — Reduces LLM token consumption by up to 99%. 42 MCP tools, 10 read modes, 90+ compression patterns, cross-session memory.

## Features

- **Status Bar** — Live token savings counter, updates automatically
- **Command Palette** — `better-ctx: Setup`, `Doctor`, `Show Token Savings`, `Open Dashboard`, `Show Context Heatmap`
- **Auto MCP Config** — Detects better-ctx and offers to configure MCP for GitHub Copilot
- **Output Channel** — All better-ctx command output in a dedicated panel

## Requirements

- [better-ctx](https://betterctx.com) binary installed (`cargo install better-ctx` or `brew install better-ctx`)

## Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `better-ctx.binaryPath` | `""` | Path to better-ctx binary (auto-detected if empty) |
| `better-ctx.autoSetup` | `true` | Automatically offer MCP configuration on activation |
| `better-ctx.statusBar` | `true` | Show token savings in the status bar |
| `better-ctx.refreshInterval` | `30` | Status bar refresh interval in seconds |

## Commands

| Command | Description |
|---------|-------------|
| `better-ctx: Setup` | Auto-configure shell hooks + editor integration |
| `better-ctx: Doctor` | Run diagnostics (PATH, config, MCP) |
| `better-ctx: Show Token Savings` | Display token savings dashboard |
| `better-ctx: Open Dashboard` | Open the web dashboard |
| `better-ctx: Show Context Heatmap` | Display project context heatmap |
| `better-ctx: Configure MCP for Copilot` | Create/update `.github/copilot/mcp.json` |

## How It Works

1. Install the extension
2. If better-ctx binary is detected, it auto-activates
3. Status bar shows your cumulative token savings
4. Use the command palette for setup, diagnostics, and analytics
5. MCP integration is auto-configured for GitHub Copilot

## Links

- [Website](https://betterctx.com)
- [Documentation](https://betterctx.com/docs/getting-started)
- [GitHub](https://github.com/jadzeino/betterctx-client)
