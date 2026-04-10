use std::io::Read;

pub fn handle_rewrite() {
    let binary = resolve_binary();
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        return;
    }

    let tool = extract_json_field(&input, "tool_name");
    if !matches!(tool.as_deref(), Some("Bash" | "bash")) {
        return;
    }

    let cmd = match extract_json_field(&input, "command") {
        Some(c) => c,
        None => return,
    };

    if cmd.starts_with("better-ctx ") || cmd.starts_with(&format!("{binary} ")) {
        return;
    }

    let should_rewrite = REWRITABLE_PREFIXES
        .iter()
        .any(|prefix| cmd.starts_with(prefix) || cmd == prefix.trim_end_matches(' '));

    if should_rewrite {
        let shell_escaped = cmd.replace('\\', "\\\\").replace('"', "\\\"");
        let shell_cmd = format!("{binary} -c \"{shell_escaped}\"");
        let json_escaped = shell_cmd.replace('\\', "\\\\").replace('"', "\\\"");
        print!(
            "{{\"hookSpecificOutput\":{{\"hookEventName\":\"PreToolUse\",\"permissionDecision\":\"allow\",\"updatedInput\":{{\"command\":\"{json_escaped}\"}}}}}}"
        );
    }
}

pub fn handle_redirect() {
    // Allow all native tools (Read, Grep, ListFiles) to pass through.
    // Blocking them breaks Edit (which requires native Read) and causes
    // unnecessary friction. The MCP instructions already guide the AI
    // to prefer ctx_read/ctx_search/ctx_tree.
}

const REWRITABLE_PREFIXES: &[&str] = &[
    "git ", "gh ", "cargo ", "npm ", "pnpm ", "yarn ", "docker ", "kubectl ", "pip ", "pip3 ",
    "ruff ", "go ", "curl ", "grep ", "rg ", "find ", "cat ", "head ", "tail ", "ls ", "ls",
    "aws ", "helm ", "eslint", "prettier", "tsc", "pytest", "mypy",
];

fn resolve_binary() -> String {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "better-ctx".to_string())
}

fn extract_json_field(input: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{}\":\"", field);
    let start = input.find(&pattern)? + pattern.len();
    let rest = &input[start..];
    let bytes = rest.as_bytes();
    let mut end = 0;
    while end < bytes.len() {
        if bytes[end] == b'\\' && end + 1 < bytes.len() {
            end += 2;
            continue;
        }
        if bytes[end] == b'"' {
            break;
        }
        end += 1;
    }
    if end >= bytes.len() {
        return None;
    }
    let raw = &rest[..end];
    Some(raw.replace("\\\"", "\"").replace("\\\\", "\\"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_rewrite(cmd: &str) -> String {
        let shell_escaped = cmd.replace('\\', "\\\\").replace('"', "\\\"");
        let shell_cmd = format!("better-ctx -c \"{shell_escaped}\"");
        shell_cmd.replace('\\', "\\\\").replace('"', "\\\"")
    }

    #[test]
    fn rewrite_simple_command() {
        let json = build_rewrite("git status");
        assert_eq!(json, "better-ctx -c \\\"git status\\\"");
    }

    #[test]
    fn rewrite_pipe_command() {
        let json = build_rewrite("git log --oneline | grep fix");
        assert_eq!(json, "better-ctx -c \\\"git log --oneline | grep fix\\\"");
    }

    #[test]
    fn rewrite_embedded_quotes() {
        let json = build_rewrite("curl -H \"Auth\" https://api.com");
        assert_eq!(
            json,
            "better-ctx -c \\\"curl -H \\\\\\\"Auth\\\\\\\" https://api.com\\\""
        );
        assert!(
            json.contains("\\\\\\\"Auth\\\\\\\""),
            "embedded quotes must be double-escaped: {json}"
        );
    }

    #[test]
    fn extract_field_works() {
        let input = r#"{"tool_name":"Bash","command":"git status"}"#;
        assert_eq!(
            extract_json_field(input, "tool_name"),
            Some("Bash".to_string())
        );
        assert_eq!(
            extract_json_field(input, "command"),
            Some("git status".to_string())
        );
    }

    #[test]
    fn extract_field_handles_escaped_quotes() {
        let input = r#"{"tool_name":"Bash","command":"grep -r \"TODO\" src/"}"#;
        assert_eq!(
            extract_json_field(input, "command"),
            Some(r#"grep -r "TODO" src/"#.to_string())
        );
    }

    #[test]
    fn extract_field_handles_escaped_backslash() {
        let input = r#"{"tool_name":"Bash","command":"echo \\\"hello\\\""}"#;
        assert_eq!(
            extract_json_field(input, "command"),
            Some(r#"echo \"hello\""#.to_string())
        );
    }

    #[test]
    fn extract_field_handles_complex_curl() {
        let input = r#"{"tool_name":"Bash","command":"curl -H \"Authorization: Bearer token\" https://api.com"}"#;
        assert_eq!(
            extract_json_field(input, "command"),
            Some(r#"curl -H "Authorization: Bearer token" https://api.com"#.to_string())
        );
    }

    #[test]
    fn rewrite_grep_with_quoted_pattern() {
        let json = build_rewrite("grep -r \"TODO\" src/");
        assert!(
            json.contains("\\\\\\\"TODO\\\\\\\""),
            "grep pattern quotes must be double-escaped: {json}"
        );
    }
}
