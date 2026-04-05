use std::sync::Arc;

use rmcp::model::*;
use serde_json::{json, Map, Value};

pub fn tool_def(name: &'static str, description: &'static str, schema_value: Value) -> Tool {
    let schema: Map<String, Value> = match schema_value {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    Tool::new(name, description, Arc::new(schema))
}

pub fn granular_tool_defs() -> Vec<Tool> {
    vec![
        tool_def(
            "ctx_read",
            "Read file (cached, compressed). Re-reads ~13 tok. Auto-selects optimal mode. \
Modes: full|map|signatures|diff|aggressive|entropy|task|reference|lines:N-M. fresh=true re-reads.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute file path to read" },
                    "mode": {
                        "type": "string",
                        "description": "Compression mode (default: full). Use 'map' for context-only files. For line ranges: 'lines:N-M' (e.g. 'lines:400-500')."
                    },
                    "start_line": {
                        "type": "integer",
                        "description": "Read from this line number to end of file. Bypasses cache stub — always returns actual content."
                    },
                    "fresh": {
                        "type": "boolean",
                        "description": "Bypass cache and force a full re-read. Use when running as a subagent that may not have the parent's context."
                    }
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "ctx_multi_read",
            "Batch read files in one call. Same modes as ctx_read.",
            json!({
                "type": "object",
                "properties": {
                    "paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Absolute file paths to read, in order"
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["full", "signatures", "map", "diff", "aggressive", "entropy"],
                        "description": "Compression mode (default: full)"
                    }
                },
                "required": ["paths"]
            }),
        ),
        tool_def(
            "ctx_tree",
            "Directory listing with file counts.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Directory path (default: .)" },
                    "depth": { "type": "integer", "description": "Max depth (default: 3)" },
                    "show_hidden": { "type": "boolean", "description": "Show hidden files" }
                }
            }),
        ),
        tool_def(
            "ctx_shell",
            "Run shell command (compressed output, 90+ patterns). Use raw=true to skip compression and get full output.",
            json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "Shell command to execute" },
                    "raw": { "type": "boolean", "description": "Skip compression, return full uncompressed output. Use for small outputs or when full detail is critical." }
                },
                "required": ["command"]
            }),
        ),
        tool_def(
            "ctx_search",
            "Regex code search (.gitignore aware, compact results).",
            json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Regex pattern" },
                    "path": { "type": "string", "description": "Directory to search" },
                    "ext": { "type": "string", "description": "File extension filter" },
                    "max_results": { "type": "integer", "description": "Max results (default: 20)" },
                    "ignore_gitignore": { "type": "boolean", "description": "Set true to scan ALL files including .gitignore'd paths (default: false)" }
                },
                "required": ["pattern"]
            }),
        ),
        tool_def(
            "ctx_compress",
            "Context checkpoint for long conversations.",
            json!({
                "type": "object",
                "properties": {
                    "include_signatures": { "type": "boolean", "description": "Include signatures (default: true)" }
                }
            }),
        ),
        tool_def(
            "ctx_benchmark",
            "Benchmark compression modes for a file or project.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path (action=file) or project directory (action=project)" },
                    "action": { "type": "string", "description": "file (default) or project", "default": "file" },
                    "format": { "type": "string", "description": "Output format for project benchmark: terminal, markdown, json", "default": "terminal" }
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "ctx_metrics",
            "Session token stats, cache rates, per-tool savings.",
            json!({
                "type": "object",
                "properties": {}
            }),
        ),
        tool_def(
            "ctx_analyze",
            "Entropy analysis — recommends optimal compression mode for a file.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path to analyze" }
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "ctx_cache",
            "Cache ops: status|clear|invalidate.",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["status", "clear", "invalidate"],
                        "description": "Cache operation to perform"
                    },
                    "path": {
                        "type": "string",
                        "description": "File path (required for 'invalidate' action)"
                    }
                },
                "required": ["action"]
            }),
        ),
        tool_def(
            "ctx_discover",
            "Find missed compression opportunities in shell history.",
            json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Max number of command types to show (default: 15)"
                    }
                }
            }),
        ),
        tool_def(
            "ctx_smart_read",
            "Auto-select optimal read mode for a file.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute file path to read" }
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "ctx_delta",
            "Incremental diff — sends only changed lines since last read.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute file path" }
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "ctx_edit",
            "Edit a file via search-and-replace. Works without native Read/Edit tools. Use this when the IDE's Edit tool requires Read but Read is unavailable.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute file path" },
                    "old_string": { "type": "string", "description": "Exact text to find and replace (must be unique unless replace_all=true)" },
                    "new_string": { "type": "string", "description": "Replacement text" },
                    "replace_all": { "type": "boolean", "description": "Replace all occurrences (default: false)", "default": false },
                    "create": { "type": "boolean", "description": "Create a new file with new_string as content (ignores old_string)", "default": false }
                },
                "required": ["path", "new_string"]
            }),
        ),
        tool_def(
            "ctx_dedup",
            "Cross-file dedup: analyze or apply shared block references.",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "description": "analyze (default) or apply (register shared blocks for auto-dedup in ctx_read)",
                        "default": "analyze"
                    }
                }
            }),
        ),
        tool_def(
            "ctx_fill",
            "Budget-aware context fill — auto-selects compression per file within token limit.",
            json!({
                "type": "object",
                "properties": {
                    "paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "File paths to consider"
                    },
                    "budget": {
                        "type": "integer",
                        "description": "Maximum token budget to fill"
                    }
                },
                "required": ["paths", "budget"]
            }),
        ),
        tool_def(
            "ctx_intent",
            "Intent detection — auto-reads relevant files based on task description.",
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Natural language description of the task" },
                    "project_root": { "type": "string", "description": "Project root directory (default: .)" }
                },
                "required": ["query"]
            }),
        ),
        tool_def(
            "ctx_response",
            "Compress LLM response text (remove filler, apply TDD).",
            json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Response text to compress" }
                },
                "required": ["text"]
            }),
        ),
        tool_def(
            "ctx_context",
            "Session context overview — cached files, seen files, session state.",
            json!({
                "type": "object",
                "properties": {}
            }),
        ),
        tool_def(
            "ctx_graph",
            "Code dependency graph. Actions: build (index project), related (find files connected to path), \
symbol (lookup definition/usages as file::name), impact (blast radius of changes to path), status (index stats).",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["build", "related", "symbol", "impact", "status"],
                        "description": "Graph operation: build, related, symbol, impact, status"
                    },
                    "path": {
                        "type": "string",
                        "description": "File path (related/impact) or file::symbol_name (symbol)"
                    },
                    "project_root": {
                        "type": "string",
                        "description": "Project root directory (default: .)"
                    }
                },
                "required": ["action"]
            }),
        ),
        tool_def(
            "ctx_session",
            "Cross-session memory (CCP). Actions: load (restore previous session ~400 tok), \
save, status, task (set current task), finding (record discovery), decision (record choice), \
reset, list (show sessions), cleanup.",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["status", "load", "save", "task", "finding", "decision", "reset", "list", "cleanup"],
                        "description": "Session operation to perform"
                    },
                    "value": {
                        "type": "string",
                        "description": "Value for task/finding/decision actions"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Session ID for load action (default: latest)"
                    }
                },
                "required": ["action"]
            }),
        ),
        tool_def(
            "ctx_knowledge",
            "Persistent project knowledge (survives sessions). Actions: remember (store fact with category+key+value), \
recall (search by query), pattern (record naming/structure pattern), consolidate (extract session findings into knowledge), \
status (list all), remove, export.",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["remember", "recall", "pattern", "consolidate", "status", "remove", "export"],
                        "description": "Knowledge operation to perform"
                    },
                    "category": {
                        "type": "string",
                        "description": "Fact category (architecture, api, testing, deployment, conventions, dependencies)"
                    },
                    "key": {
                        "type": "string",
                        "description": "Fact key/identifier (e.g. 'auth-method', 'db-engine', 'test-framework')"
                    },
                    "value": {
                        "type": "string",
                        "description": "Fact value or pattern description"
                    },
                    "query": {
                        "type": "string",
                        "description": "Search query for recall action (matches against category, key, and value)"
                    },
                    "pattern_type": {
                        "type": "string",
                        "description": "Pattern type for pattern action (naming, structure, testing, error-handling)"
                    },
                    "examples": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Examples for pattern action"
                    },
                    "confidence": {
                        "type": "number",
                        "description": "Confidence score 0.0-1.0 for remember action (default: 0.8)"
                    }
                },
                "required": ["action"]
            }),
        ),
        tool_def(
            "ctx_agent",
            "Multi-agent coordination (shared message bus). Actions: register (join with agent_type+role), \
post (broadcast or direct message with category), read (poll messages), status (update state: active|idle|finished), \
list, info.",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["register", "list", "post", "read", "status", "info"],
                        "description": "Agent operation to perform"
                    },
                    "agent_type": {
                        "type": "string",
                        "description": "Agent type for register (cursor, claude, codex, gemini, subagent)"
                    },
                    "role": {
                        "type": "string",
                        "description": "Agent role (dev, review, test, plan)"
                    },
                    "message": {
                        "type": "string",
                        "description": "Message text for post action, or status detail for status action"
                    },
                    "category": {
                        "type": "string",
                        "description": "Message category for post (finding, warning, request, status)"
                    },
                    "to_agent": {
                        "type": "string",
                        "description": "Target agent ID for direct message (omit for broadcast)"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["active", "idle", "finished"],
                        "description": "New status for status action"
                    }
                },
                "required": ["action"]
            }),
        ),
        tool_def(
            "ctx_overview",
            "Task-relevant project map — use at session start.",
            json!({
                "type": "object",
                "properties": {
                    "task": {
                        "type": "string",
                        "description": "Task description for relevance scoring (e.g. 'fix auth bug in login flow')"
                    },
                    "path": {
                        "type": "string",
                        "description": "Project root directory (default: .)"
                    }
                }
            }),
        ),
        tool_def(
            "ctx_preload",
            "Proactive context loader — caches task-relevant files, returns L-curve-optimized summary (~50-100 tokens vs ~5000 for individual reads).",
            json!({
                "type": "object",
                "properties": {
                    "task": {
                        "type": "string",
                        "description": "Task description (e.g. 'fix auth bug in validate_token')"
                    },
                    "path": {
                        "type": "string",
                        "description": "Project root (default: .)"
                    }
                },
                "required": ["task"]
            }),
        ),
        tool_def(
            "ctx_wrapped",
            "Savings report card. Periods: week|month|all.",
            json!({
                "type": "object",
                "properties": {
                    "period": {
                        "type": "string",
                        "enum": ["week", "month", "all"],
                        "description": "Report period (default: week)"
                    }
                }
            }),
        ),
        tool_def(
            "ctx_semantic_search",
            "BM25 code search by meaning. action=reindex to rebuild.",
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Natural language search query" },
                    "path": { "type": "string", "description": "Project root to search (default: .)" },
                    "top_k": { "type": "integer", "description": "Number of results (default: 10)" },
                    "action": { "type": "string", "description": "reindex to rebuild index" }
                },
                "required": ["query"]
            }),
        ),
    ]
}

pub fn unified_tool_defs() -> Vec<Tool> {
    vec![
        tool_def(
            "ctx_read",
            "Read file (cached, compressed). Modes: full|map|signatures|diff|aggressive|entropy|task|reference|lines:N-M. fresh=true re-reads.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path" },
                    "mode": { "type": "string" },
                    "start_line": { "type": "integer" },
                    "fresh": { "type": "boolean" }
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "ctx_shell",
            "Run shell command (compressed output). raw=true skips compression.",
            json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "Shell command" },
                    "raw": { "type": "boolean", "description": "Skip compression for full output" }
                },
                "required": ["command"]
            }),
        ),
        tool_def(
            "ctx_search",
            "Regex code search (.gitignore aware).",
            json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Regex pattern" },
                    "path": { "type": "string" },
                    "ext": { "type": "string" },
                    "max_results": { "type": "integer" },
                    "ignore_gitignore": { "type": "boolean" }
                },
                "required": ["pattern"]
            }),
        ),
        tool_def(
            "ctx_tree",
            "Directory listing with file counts.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "depth": { "type": "integer" },
                    "show_hidden": { "type": "boolean" }
                }
            }),
        ),
        tool_def(
            "ctx",
            "Meta-tool: set tool= to sub-tool name. Sub-tools: compress (checkpoint), metrics (stats), \
analyze (entropy), cache (status|clear|invalidate), discover (missed patterns), smart_read (auto-mode), \
delta (incremental diff), dedup (cross-file), fill (budget-aware batch read), intent (auto-read by task), \
response (compress LLM text), context (session state), graph (build|related|symbol|impact|status), \
session (load|save|task|finding|decision|status|reset|list|cleanup), \
knowledge (remember|recall|pattern|consolidate|status|remove|export), \
agent (register|post|read|status|list|info), overview (project map), \
wrapped (savings report), benchmark (file|project), multi_read (batch), semantic_search (BM25).",
            json!({
                "type": "object",
                "properties": {
                    "tool": {
                        "type": "string",
                        "description": "compress|metrics|analyze|cache|discover|smart_read|delta|dedup|fill|intent|response|context|graph|session|knowledge|agent|overview|wrapped|benchmark|multi_read|semantic_search"
                    },
                    "action": { "type": "string" },
                    "path": { "type": "string" },
                    "paths": { "type": "array", "items": { "type": "string" } },
                    "query": { "type": "string" },
                    "value": { "type": "string" },
                    "category": { "type": "string" },
                    "key": { "type": "string" },
                    "budget": { "type": "integer" },
                    "task": { "type": "string" },
                    "mode": { "type": "string" },
                    "text": { "type": "string" },
                    "message": { "type": "string" },
                    "session_id": { "type": "string" },
                    "period": { "type": "string" },
                    "format": { "type": "string" },
                    "agent_type": { "type": "string" },
                    "role": { "type": "string" },
                    "status": { "type": "string" },
                    "pattern_type": { "type": "string" },
                    "examples": { "type": "array", "items": { "type": "string" } },
                    "confidence": { "type": "number" },
                    "project_root": { "type": "string" },
                    "include_signatures": { "type": "boolean" },
                    "limit": { "type": "integer" },
                    "to_agent": { "type": "string" },
                    "show_hidden": { "type": "boolean" }
                },
                "required": ["tool"]
            }),
        ),
    ]
}

pub fn list_all_tool_defs() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        ("ctx_read", "Read file (cached, compressed). Re-reads ~13 tok. Auto-selects optimal mode. \
Modes: full|map|signatures|diff|aggressive|entropy|task|reference|lines:N-M. fresh=true re-reads.", json!({"type": "object", "properties": {"path": {"type": "string"}, "mode": {"type": "string"}, "start_line": {"type": "integer"}, "fresh": {"type": "boolean"}}, "required": ["path"]})),
        ("ctx_multi_read", "Batch read files in one call. Same modes as ctx_read.", json!({"type": "object", "properties": {"paths": {"type": "array", "items": {"type": "string"}}, "mode": {"type": "string"}}, "required": ["paths"]})),
        ("ctx_tree", "Directory listing with file counts.", json!({"type": "object", "properties": {"path": {"type": "string"}, "depth": {"type": "integer"}, "show_hidden": {"type": "boolean"}}})),
        ("ctx_shell", "Run shell command (compressed output, 90+ patterns).", json!({"type": "object", "properties": {"command": {"type": "string"}}, "required": ["command"]})),
        ("ctx_search", "Regex code search (.gitignore aware, compact results).", json!({"type": "object", "properties": {"pattern": {"type": "string"}, "path": {"type": "string"}, "ext": {"type": "string"}, "max_results": {"type": "integer"}}, "required": ["pattern"]})),
        ("ctx_compress", "Context checkpoint for long conversations.", json!({"type": "object", "properties": {"include_signatures": {"type": "boolean"}}})),
        ("ctx_benchmark", "Benchmark compression modes for a file or project.", json!({"type": "object", "properties": {"path": {"type": "string"}, "action": {"type": "string"}, "format": {"type": "string"}}, "required": ["path"]})),
        ("ctx_metrics", "Session token stats, cache rates, per-tool savings.", json!({"type": "object", "properties": {}})),
        ("ctx_analyze", "Entropy analysis — recommends optimal compression mode for a file.", json!({"type": "object", "properties": {"path": {"type": "string"}}, "required": ["path"]})),
        ("ctx_cache", "Cache ops: status|clear|invalidate.", json!({"type": "object", "properties": {"action": {"type": "string"}, "path": {"type": "string"}}, "required": ["action"]})),
        ("ctx_discover", "Find missed compression opportunities in shell history.", json!({"type": "object", "properties": {"limit": {"type": "integer"}}})),
        ("ctx_smart_read", "Auto-select optimal read mode for a file.", json!({"type": "object", "properties": {"path": {"type": "string"}}, "required": ["path"]})),
        ("ctx_delta", "Incremental diff — sends only changed lines since last read.", json!({"type": "object", "properties": {"path": {"type": "string"}}, "required": ["path"]})),
        ("ctx_edit", "Edit a file via search-and-replace. Works without native Read/Edit tools. Use when Edit requires Read but Read is unavailable.", json!({"type": "object", "properties": {"path": {"type": "string"}, "old_string": {"type": "string"}, "new_string": {"type": "string"}, "replace_all": {"type": "boolean"}, "create": {"type": "boolean"}}, "required": ["path", "new_string"]})),
        ("ctx_dedup", "Cross-file dedup: analyze or apply shared block references.", json!({"type": "object", "properties": {"action": {"type": "string"}}})),
        ("ctx_fill", "Budget-aware context fill — auto-selects compression per file within token limit.", json!({"type": "object", "properties": {"paths": {"type": "array", "items": {"type": "string"}}, "budget": {"type": "integer"}}, "required": ["paths", "budget"]})),
        ("ctx_intent", "Intent detection — auto-reads relevant files based on task description.", json!({"type": "object", "properties": {"query": {"type": "string"}, "project_root": {"type": "string"}}, "required": ["query"]})),
        ("ctx_response", "Compress LLM response text (remove filler, apply TDD).", json!({"type": "object", "properties": {"text": {"type": "string"}}, "required": ["text"]})),
        ("ctx_context", "Session context overview — cached files, seen files, session state.", json!({"type": "object", "properties": {}})),
        ("ctx_graph", "Code dependency graph. Actions: build (index project), related (find files connected to path), \
symbol (lookup definition/usages as file::name), impact (blast radius of changes to path), status (index stats).", json!({"type": "object", "properties": {"action": {"type": "string"}, "path": {"type": "string"}, "project_root": {"type": "string"}}, "required": ["action"]})),
        ("ctx_session", "Cross-session memory (CCP). Actions: load (restore previous session ~400 tok), \
save, status, task (set current task), finding (record discovery), decision (record choice), \
reset, list (show sessions), cleanup.", json!({"type": "object", "properties": {"action": {"type": "string"}, "value": {"type": "string"}, "session_id": {"type": "string"}}, "required": ["action"]})),
        ("ctx_knowledge", "Persistent project knowledge (survives sessions). Actions: remember (store fact with category+key+value), \
recall (search by query), pattern (record naming/structure pattern), consolidate (extract session findings into knowledge), \
status (list all), remove, export.", json!({"type": "object", "properties": {"action": {"type": "string"}, "category": {"type": "string"}, "key": {"type": "string"}, "value": {"type": "string"}, "query": {"type": "string"}}, "required": ["action"]})),
        ("ctx_agent", "Multi-agent coordination (shared message bus). Actions: register (join with agent_type+role), \
post (broadcast or direct message with category), read (poll messages), status (update state: active|idle|finished), \
list, info.", json!({"type": "object", "properties": {"action": {"type": "string"}, "agent_type": {"type": "string"}, "role": {"type": "string"}, "message": {"type": "string"}}, "required": ["action"]})),
        ("ctx_overview", "Task-relevant project map — use at session start.", json!({"type": "object", "properties": {"task": {"type": "string"}, "path": {"type": "string"}}})),
        ("ctx_preload", "Proactive context loader — reads and caches task-relevant files, returns compact L-curve-optimized summary with critical lines, imports, and signatures. Costs ~50-100 tokens instead of ~5000 for individual reads.", json!({"type": "object", "properties": {"task": {"type": "string", "description": "Task description (e.g. 'fix auth bug in validate_token')"}, "path": {"type": "string", "description": "Project root (default: .)"}}, "required": ["task"]})),
        ("ctx_wrapped", "Savings report card. Periods: week|month|all.", json!({"type": "object", "properties": {"period": {"type": "string"}}})),
        ("ctx_semantic_search", "BM25 code search by meaning. action=reindex to rebuild.", json!({"type": "object", "properties": {"query": {"type": "string"}, "path": {"type": "string"}, "top_k": {"type": "integer"}, "action": {"type": "string"}}, "required": ["query"]})),
    ]
}
