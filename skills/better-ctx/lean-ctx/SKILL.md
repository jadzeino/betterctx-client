---
name: better-ctx
description: Context Intelligence Engine with CEP + CCP — 28 MCP tools, 90+ shell patterns, tree-sitter AST for 18 languages, Cognitive Efficiency Protocol (CEP), cross-session memory (CCP), LITM-aware positioning. Compresses LLM context by up to 99%.
metadata: {"openclaw": {"requires": {"bins": ["better-ctx"]}, "install": [{"id": "brew", "kind": "brew", "formula": "better-ctx", "bins": ["better-ctx"], "label": "Install betterCTX (brew tap jadzeino/betterctx-client)"}]}}
---

# betterCTX v2.19.0 — The Intelligence Layer for AI Coding + CEP + CCP + Multi-Agent Sharing

betterCTX is a Rust binary that optimizes LLM context through 28 MCP tools, 90+ shell compression patterns, and tree-sitter AST parsing for 18 languages (TS/JS, Rust, Python, Go, Java, C, C++, Ruby, C#, Kotlin, Swift, PHP, Bash, Dart, Scala, Elixir, Zig). It provides adaptive file reading with per-language entropy thresholds, incremental deltas, intent detection, cross-file deduplication with TF-IDF cosine similarity, task-conditioned relevance scoring, a heuristic attention prediction model, a project intelligence graph, the **Cognitive Efficiency Protocol (CEP)** with output token budgets, the **Context Continuity Protocol (CCP)** for cross-session memory with LITM-aware positioning, and a feedback loop for learning optimal compression parameters.

## When to use better-ctx

Always prefer `better-ctx -c <command>` over running commands directly when:
- The command produces verbose output (build logs, git diffs, dependency trees, test results)
- You are reading files and only need the structure or API surface
- You want to check token savings for the current session

## Shell commands (use instead of raw exec)

```bash
better-ctx -c git status          # Compressed git output
better-ctx -c git diff            # Only meaningful diff lines
better-ctx -c git log --oneline -10
better-ctx -c npm install         # Strips progress bars, noise
better-ctx -c cargo build
better-ctx -c cargo test
better-ctx -c docker ps
better-ctx -c kubectl get pods
better-ctx -c aws ec2 describe-instances
better-ctx -c helm list
better-ctx -c prisma migrate dev
better-ctx -c curl -s <url>       # JSON schema extraction
better-ctx -c ls -la <dir>        # Grouped directory listing
```

Supported: git, npm, pnpm, yarn, bun, deno, cargo, docker, kubectl, helm, gh, pip, ruff, go, eslint, prettier, tsc, aws, psql, mysql, prisma, swift, zig, cmake, ansible, composer, mix, bazel, systemd, terraform, make, maven, dotnet, flutter, poetry, rubocop, playwright, curl, wget, and more.

## File reading (compressed modes)

```bash
better-ctx read <file>                    # Full content with structured header
better-ctx read <file> -m map             # Dependency graph + exports + API (~5-15% tokens)
better-ctx read <file> -m signatures      # Function/class signatures only (~10-20% tokens)
better-ctx read <file> -m aggressive      # Syntax-stripped (~30-50% tokens)
better-ctx read <file> -m entropy         # Shannon entropy filtered (~20-40% tokens)
better-ctx read <file> -m diff            # Only changed lines since last read
```

Use `map` mode when you need to understand what a file does without reading every line.
Use `signatures` mode when you need the API surface of a module (tree-sitter for 18 languages).
Use `full` mode only when you will edit the file.

## AI Tool Integration

```bash
better-ctx init --global          # Install shell aliases
better-ctx init --agent claude    # Claude Code PreToolUse hook
better-ctx init --agent cursor    # Cursor hooks.json
better-ctx init --agent gemini    # Gemini CLI BeforeTool hook
better-ctx init --agent codex     # Codex AGENTS.md
better-ctx init --agent windsurf  # .windsurfrules
better-ctx init --agent cline     # .clinerules
better-ctx init --agent crush     # Crush MCP config
better-ctx init --agent copilot   # VS Code / Copilot .vscode/mcp.json
```

## Multi-Agent & Knowledge (v2.7.0+)

MCP tools:
- `ctx_knowledge(action="remember", category, key, value)` — persistent cross-session project knowledge store
- `ctx_knowledge(action="recall", query)` — search stored facts by text or category
- `ctx_knowledge(action="consolidate")` — extract session findings into permanent knowledge
- `ctx_agent(action="register", agent_type, role)` — multi-agent context sharing with scratchpad messaging
- `ctx_agent(action="post", message, tags)` — share findings/warnings between concurrent agents
- `ctx_agent(action="read")` — read messages from other agents
- `ctx_agent(action="handoff", to_agent, message)` — transfer task to another agent
- `ctx_agent(action="sync")` — multi-agent sync status (active agents, pending messages, shared contexts)
- `ctx_share(action="push", paths, to_agent, message)` — push cached file contexts to another agent
- `ctx_share(action="pull")` — pull shared contexts from other agents
- `ctx_share(action="list")` — list all shared contexts
- `ctx_share(action="clear")` — remove contexts shared by this agent

## Additional Intelligence Tools (v2.19.0)

- `ctx_edit(path, old_string, new_string)` — search-and-replace file editing without native Read/Edit
- `ctx_overview(task)` — task-relevant project map at session start
- `ctx_preload(task)` — proactive context loader, caches task-relevant files
- `ctx_semantic_search(query)` — BM25 code search by meaning across the project
- `ctx_intent` now supports multi-intent detection and complexity classification
- Semantic cache: TF-IDF + cosine similarity for finding similar files across reads

## Session Continuity (CCP)

```bash
better-ctx sessions list          # List all CCP sessions
better-ctx sessions show          # Show latest session state
better-ctx wrapped                # Weekly savings report card
better-ctx wrapped --month        # Monthly savings report card
better-ctx benchmark run          # Real project benchmark (terminal output)
better-ctx benchmark run --json   # Machine-readable JSON output
better-ctx benchmark report       # Shareable Markdown report
```

MCP tools for CCP:
- `ctx_session status` — show current session state (~400 tokens)
- `ctx_session load` — restore previous session (cross-chat memory)
- `ctx_session task "description"` — set current task
- `ctx_session finding "file:line — summary"` — record key finding
- `ctx_session decision "summary"` — record architectural decision
- `ctx_session save` — force persist session to disk
- `ctx_wrapped` — generate savings report card in chat

## Analytics

```bash
better-ctx gain                   # Visual token savings dashboard
better-ctx dashboard              # Web dashboard at localhost:3333
better-ctx session                # Adoption statistics
better-ctx discover               # Find uncompressed commands in shell history
```

## Tips

- The output suffix `[better-ctx: 5029→197 tok, -96%]` shows original vs compressed token count
- For large outputs, better-ctx automatically truncates while preserving relevant context
- JSON responses from curl/wget are reduced to schema outlines
- Build errors are grouped by type with counts
- Test results show only failures with summary counts
- Cached re-reads cost only ~13 tokens
