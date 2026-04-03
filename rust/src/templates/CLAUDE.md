# better-ctx — Context Engineering Layer

MANDATORY: Use better-ctx MCP tools for ALL reads, searches, and shell commands.

| FORBIDDEN | USE INSTEAD | Why |
|-----------|-------------|-----|
| Read / cat / head / tail | `ctx_read(path)` | Session caching, 8 compression modes, re-reads cost ~13 tokens |
| Bash (shell commands) | `ctx_shell(command)` | Pattern-based compression for git, npm, cargo, docker, tsc |
| Grep / rg | `ctx_search(pattern, path)` | Compact context, token-efficient results |
| ls / find | `ctx_tree(path, depth)` | Compact directory maps with file counts |

## ctx_read Modes

- `full` — cached read (use for files you will edit)
- `map` — deps + API signatures (use for context-only files)
- `signatures` — API surface only
- `diff` — changed lines only (after edits)
- `aggressive` — syntax stripped
- `entropy` — Shannon + Jaccard filtering
- `lines:N-M` — specific range

Write, StrReplace, Delete have no better-ctx equivalent — use them normally.

REMINDER: NEVER use native Read, Bash, Grep, or ls. ALWAYS use ctx_read, ctx_shell, ctx_search, ctx_tree.
