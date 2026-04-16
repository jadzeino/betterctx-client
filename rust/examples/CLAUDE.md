# better-ctx — Token Optimization

better-ctx is configured as an MCP server. Use better-ctx MCP tools instead of built-in tools:

| Built-in | Use instead | Why |
|----------|-------------|-----|
| Read / cat / head | `ctx_read` | Session caching, 6 compression modes, re-reads cost ~13 tokens |
| Bash (shell commands) | `ctx_shell` | Pattern-based compression for git, npm, cargo, docker, tsc |
| Grep / rg | `ctx_search` | Compact context, token-efficient results |
| ls / find | `ctx_tree` | Compact directory maps with file counts |

For shell commands that don't have MCP equivalents, prefix with `better-ctx -c`:

```bash
better-ctx -c git status    # compressed output
better-ctx -c cargo test    # compressed output
better-ctx -c npm install   # compressed output
```

## ctx_read Modes

- `full` — cached read (use for files you will edit)
- `map` — deps + API signatures (use for context-only files)
- `signatures` — API surface only
- `diff` — changed lines only (after edits)
- `aggressive` — syntax stripped
- `entropy` — Shannon + Jaccard filtering

Use native Edit/StrReplace when available. If Edit requires Read and Read is unavailable,
use `ctx_edit(path, old_string, new_string)` — it reads, replaces, and writes in one MCP call.
NEVER loop trying to make Edit work. If it fails, switch to ctx_edit immediately.
Write, Delete have no better-ctx equivalent — use them normally.
