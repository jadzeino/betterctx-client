use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

static IMPORT_RE: OnceLock<Regex> = OnceLock::new();
static REQUIRE_RE: OnceLock<Regex> = OnceLock::new();
static RUST_USE_RE: OnceLock<Regex> = OnceLock::new();
static PY_IMPORT_RE: OnceLock<Regex> = OnceLock::new();
static GO_IMPORT_RE: OnceLock<Regex> = OnceLock::new();

fn import_re() -> &'static Regex {
    IMPORT_RE.get_or_init(|| {
        Regex::new(r#"import\s+(?:\{[^}]*\}\s+from\s+|.*from\s+)['"]([^'"]+)['"]"#).unwrap()
    })
}
fn require_re() -> &'static Regex {
    REQUIRE_RE.get_or_init(|| Regex::new(r#"require\(['"]([^'"]+)['"]\)"#).unwrap())
}
fn rust_use_re() -> &'static Regex {
    RUST_USE_RE.get_or_init(|| Regex::new(r"^use\s+([\w:]+)").unwrap())
}
fn py_import_re() -> &'static Regex {
    PY_IMPORT_RE.get_or_init(|| Regex::new(r"^(?:from\s+(\S+)\s+import|import\s+(\S+))").unwrap())
}
fn go_import_re() -> &'static Regex {
    GO_IMPORT_RE.get_or_init(|| Regex::new(r#""([^"]+)""#).unwrap())
}

#[derive(Debug, Clone)]
pub struct DepInfo {
    pub imports: Vec<String>,
    pub exports: Vec<String>,
}

pub fn extract_deps(content: &str, ext: &str) -> DepInfo {
    match ext {
        "ts" | "tsx" | "js" | "jsx" | "svelte" | "vue" => extract_ts_deps(content),
        "rs" => extract_rust_deps(content),
        "py" => extract_python_deps(content),
        "go" => extract_go_deps(content),
        _ => DepInfo {
            imports: Vec::new(),
            exports: Vec::new(),
        },
    }
}

fn extract_ts_deps(content: &str) -> DepInfo {
    let mut imports = HashSet::new();
    let mut exports = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(caps) = import_re().captures(trimmed) {
            let path = &caps[1];
            if path.starts_with('.') || path.starts_with('/') {
                imports.insert(clean_import_path(path));
            }
        }
        if let Some(caps) = require_re().captures(trimmed) {
            let path = &caps[1];
            if path.starts_with('.') || path.starts_with('/') {
                imports.insert(clean_import_path(path));
            }
        }

        if trimmed.starts_with("export ") {
            if let Some(name) = extract_export_name(trimmed) {
                exports.push(name);
            }
        }
    }

    DepInfo {
        imports: imports.into_iter().collect(),
        exports,
    }
}

fn extract_rust_deps(content: &str) -> DepInfo {
    let mut imports = HashSet::new();
    let mut exports = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(caps) = rust_use_re().captures(trimmed) {
            let path = &caps[1];
            if !path.starts_with("std::") && !path.starts_with("core::") {
                imports.insert(path.to_string());
            }
        }

        if trimmed.starts_with("pub fn ") || trimmed.starts_with("pub async fn ") {
            if let Some(name) = trimmed
                .split('(')
                .next()
                .and_then(|s| s.split_whitespace().last())
            {
                exports.push(name.to_string());
            }
        } else if trimmed.starts_with("pub struct ")
            || trimmed.starts_with("pub enum ")
            || trimmed.starts_with("pub trait ")
        {
            if let Some(name) = trimmed.split_whitespace().nth(2) {
                let clean = name.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
                exports.push(clean.to_string());
            }
        }
    }

    DepInfo {
        imports: imports.into_iter().collect(),
        exports,
    }
}

fn extract_python_deps(content: &str) -> DepInfo {
    let mut imports = HashSet::new();
    let mut exports = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(caps) = py_import_re().captures(trimmed) {
            if let Some(m) = caps.get(1).or(caps.get(2)) {
                let module = m.as_str();
                if !module.starts_with("os")
                    && !module.starts_with("sys")
                    && !module.starts_with("json")
                {
                    imports.insert(module.to_string());
                }
            }
        }

        if trimmed.starts_with("def ") && !trimmed.contains("_") {
            if let Some(name) = trimmed
                .strip_prefix("def ")
                .and_then(|s| s.split('(').next())
            {
                exports.push(name.to_string());
            }
        } else if trimmed.starts_with("class ") {
            if let Some(name) = trimmed
                .strip_prefix("class ")
                .and_then(|s| s.split(['(', ':']).next())
            {
                exports.push(name.to_string());
            }
        }
    }

    DepInfo {
        imports: imports.into_iter().collect(),
        exports,
    }
}

fn extract_go_deps(content: &str) -> DepInfo {
    let mut imports = HashSet::new();
    let mut exports = Vec::new();

    let mut in_import_block = false;
    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("import (") {
            in_import_block = true;
            continue;
        }
        if in_import_block {
            if trimmed == ")" {
                in_import_block = false;
                continue;
            }
            if let Some(caps) = go_import_re().captures(trimmed) {
                imports.insert(caps[1].to_string());
            }
        }

        if trimmed.starts_with("func ") {
            let name_part = trimmed.strip_prefix("func ").unwrap_or("");
            if let Some(name) = name_part.split('(').next() {
                let name = name.trim();
                if !name.is_empty() && name.starts_with(char::is_uppercase) {
                    exports.push(name.to_string());
                }
            }
        }
    }

    DepInfo {
        imports: imports.into_iter().collect(),
        exports,
    }
}

fn clean_import_path(path: &str) -> String {
    path.trim_start_matches("./")
        .trim_end_matches(".js")
        .trim_end_matches(".ts")
        .trim_end_matches(".tsx")
        .trim_end_matches(".jsx")
        .to_string()
}

fn extract_export_name(line: &str) -> Option<String> {
    let without_export = line.strip_prefix("export ")?;
    let without_default = without_export
        .strip_prefix("default ")
        .unwrap_or(without_export);

    for keyword in &[
        "function ",
        "async function ",
        "class ",
        "const ",
        "let ",
        "type ",
        "interface ",
        "enum ",
    ] {
        if let Some(rest) = without_default.strip_prefix(keyword) {
            let name = rest
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .next()?;
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    None
}
