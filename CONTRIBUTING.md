# Contributing to better-ctx

Thank you for your interest in better-ctx! We welcome contributions of all kinds.

## Getting Started

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- Git

### Setup

```bash
git clone https://github.com/jadzeino/betterctx-client.git
cd better-ctx/rust
cargo build
cargo test
```

### Running Tests

```bash
cargo test              # all tests
cargo test patterns     # pattern tests only
cargo test --release    # release mode (catches optimization bugs)
cargo clippy            # lints — must pass with zero warnings
```

## Project Structure

```
rust/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── server.rs            # MCP server (tool registration + dispatch)
│   ├── cli.rs               # Shell hook integration
│   ├── core/
│   │   ├── patterns/        # 90+ shell command compression patterns
│   │   ├── cache.rs         # Session cache with file refs
│   │   ├── compressor.rs    # AST-aware file compression
│   │   ├── entropy.rs       # Shannon entropy + Jaccard analysis
│   │   ├── graph_index.rs   # Persistent project dependency graph
│   │   ├── session.rs       # Cross-session state (CCP)
│   │   ├── knowledge.rs     # Permanent project knowledge store
│   │   ├── vector_index.rs  # BM25 semantic code search
│   │   └── ...
│   ├── tools/               # MCP tool handlers
│   └── dashboard/           # Local web dashboard
└── tests/                   # Integration tests
```

## How to Contribute

### Adding a Shell Compression Pattern

This is the easiest way to contribute. Each pattern compresses a specific CLI command's output.

1. Create `rust/src/core/patterns/<tool>.rs`
2. Implement `pub fn compress(command: &str, output: &str) -> Option<String>`
3. Register in `rust/src/core/patterns/mod.rs`:
   - Add `pub mod <tool>;`
   - Add routing in `try_specific_pattern()`
4. Add tests (see `ruff.rs` or `mypy.rs` for examples)

### Adding tree-sitter Language Support

1. Add the grammar crate to `Cargo.toml` under `[dependencies]`
2. Add the language to `signatures_ts.rs` (`get_language` + `get_query`)
3. Add test cases

### Bug Fixes

1. Open an issue describing the bug
2. Fork and create a branch: `fix/<description>`
3. Write a failing test, then fix it
4. Submit a PR

### Feature Requests

Open an issue with the `enhancement` label. Describe:
- What problem it solves
- How it should work
- Why it belongs in the core (vs. a plugin)

## Code Style

- **Zero clippy warnings** — run `cargo clippy` before submitting
- **No mock data** — tests use real patterns, not fake values
- **Compact output** — compression results should be token-efficient
- **Edge cases matter** — handle empty input, missing files, malformed output

## Commit Messages

Format: `<type>: <description>`

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`

Examples:
- `feat: add mypy compression pattern`
- `fix: handle empty cargo test output`
- `refactor: extract BM25 scoring into separate function`

## Pull Request Process

1. Ensure all tests pass: `cargo test`
2. Ensure zero clippy warnings: `cargo clippy`
3. Update relevant documentation
4. PRs are reviewed within 48 hours

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
