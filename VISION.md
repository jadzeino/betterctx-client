# betterCTX Vision

## The Cognitive Filter

In 2026, we've moved past "just sending a prompt." High performance with LLMs isn't about bigger context windows — it's about **Information Density**.

betterCTX is the **Intelligence Buffer** between human and machine. Not a proxy. Not a wrapper. A **Cognitive Filter** that ensures every token reaching the LLM carries maximum signal. Every byte of noise stripped away is a byte of reasoning gained.

> The winners won't be those who can afford 1M token contexts.
> They'll be those who achieve the same result with 10K.

## The Four Dimensions

Building a high-performance LLM layer requires mastering four dimensions:

### 1. Compression Layer (Input Efficiency)

Sending 100% of the data dilutes the model's attention mechanism. Boilerplate, redundant re-reads, verbose CLI output — it all competes with the actual signal.

betterCTX solves this with:
- **AST-Based Signatures** — Send the skeleton (classes, methods, types), not the flesh.
- **Delta-Loading** — Only transmit what changed since the last turn.
- **Session Caching** — Re-reads cost 13 tokens instead of thousands.
- **Token Shorthand (TDD)** — Replace verbose grammar with logical symbols to free up thinking tokens.
- **Entropy Filtering** — Shannon entropy analysis removes lines that carry no unique information.
- **90+ CLI Patterns** — Pattern-matched compression for every common dev tool.

### 2. Semantic Router (Model Selection)

Not every query needs a $15/1M token model. The future is tiered:
- **Intent Detection** — Is this a "What" (retrieval) or a "How" (reasoning) question?
- **Tiered Routing** — Simple lookups go to fast models. Complex architectural shifts go to the "Big Brain."
- **Mode Selection** — `map` mode for understanding, `signatures` for API surface, `full` for editing, `entropy` for noisy files.

betterCTX already implements this at the file level: 6 read modes + optional line ranges let the model — or the user — choose the right fidelity for each task.

### 3. Context Manager (Memory Architecture)

Performance drops when the context window is cluttered with irrelevant history.

betterCTX manages this through:
- **Session Cache with Auto-TTL** — Files tracked, diffs computed, stale entries purged.
- **Context Checkpoints** — `ctx_compress` creates ultra-compact state summaries when conversations grow long.
- **Subagent Isolation** — `fresh` parameter and `ctx_cache clear` prevent stale cache hits when new agents spawn.
- **Sliding Window** — The model sees the latest state, not the full history of every read.

### 4. Quality Guardrail (Output Verification)

Performance isn't just speed; it's accuracy. When the model receives focused, high-entropy input:
- **Reasoning improves** — Less noise means more attention on logic nodes.
- **Deterministic anchoring** — Compressed outputs preserve exact paths, variable names, and error locations.
- **Self-correction becomes cheaper** — With lower token budgets consumed by input, more budget remains for iterative refinement.

## The Hidden Metric: Entropy

The goal is to maximize **Information Entropy per token**.

If you send `function`, you've used 1 token to convey almost zero unique information — the AI already knows it's a function. If you send `λ`, you've used 1 token to convey a specific logical operation while saving space for the *actual* logic.

betterCTX is a **Lossless Minifier for Human Thought**.

## Brute Force vs. Cognitive Filter

| Dimension | Brute Force (Standard) | Cognitive Filter (betterCTX) |
|---|---|---|
| **Data Sent** | Full files, raw logs, all history | AST signatures, diffs, state summaries |
| **Latency** | High (large input ingestion) | Low (minimal token processing) |
| **Reasoning** | Distracted by boilerplate | Focused on logic nodes |
| **Cost** | Linear (expensive) | Logarithmic (high ROI) |
| **Context Lifespan** | Burns through window fast | Extends effective session length |

## Where We're Going

betterCTX v2.1 delivers Dimension 1 (Compression) and Dimension 3 (Context Management) with:
- 90+ CLI patterns, 18 tree-sitter languages, 28 MCP tools
- Session cache with TTL, subagent isolation, delta reads
- **Context Continuity Protocol (CCP)** — cross-session memory that persists across chat sessions and context compactions (~400 tokens vs ~50K cold start)
- **LITM-Aware Positioning** — places critical information at attention-optimal positions (begin α=0.9, end γ=0.85) based on Liu et al., 2023
- Persistent stats with USD tracking, visual dashboards, and shareable "Wrapped" reports
- **Project Benchmark Engine** — real token measurements with tiktoken, latency tracking, AST-based preservation scoring, session simulation
- Works with every MCP editor: Cursor, Copilot, Claude Code, Windsurf, Codex, Crush, Antigravity, OpenCode, OpenClaw

Future directions:
- **Semantic Routing** — Automatic mode selection based on query intent
- **Multi-Agent Memory** — Shared context graphs across parallel agents
- **Output Verification** — Post-processing layer for accuracy guarantees
- **Adaptive Compression** — ML-driven entropy thresholds per language and project

The end state: an AI that sees only what matters, remembers what's relevant, and reasons at maximum capacity.

**Tokens are the new gold. Spend them wisely.**
