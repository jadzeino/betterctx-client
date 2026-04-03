use tree_sitter::{Language, Node, Parser, Query, QueryCursor, StreamingIterator};

use super::signatures::Signature;

const QUERY_RUST: &str = r#"
(function_item name: (identifier) @name) @def
(struct_item name: (type_identifier) @name) @def
(enum_item name: (type_identifier) @name) @def
(trait_item name: (type_identifier) @name) @def
(impl_item type: (type_identifier) @name) @def
(type_item name: (type_identifier) @name) @def
(const_item name: (identifier) @name) @def
"#;

const QUERY_TYPESCRIPT: &str = r#"
(function_declaration name: (identifier) @name) @def
(class_declaration name: (type_identifier) @name) @def
(abstract_class_declaration name: (type_identifier) @name) @def
(interface_declaration name: (type_identifier) @name) @def
(type_alias_declaration name: (type_identifier) @name) @def
(method_definition name: (property_identifier) @name) @def
(variable_declarator name: (identifier) @name value: (arrow_function)) @def
"#;

const QUERY_JAVASCRIPT: &str = r#"
(function_declaration name: (identifier) @name) @def
(class_declaration name: (identifier) @name) @def
(method_definition name: (property_identifier) @name) @def
(variable_declarator name: (identifier) @name value: (arrow_function)) @def
"#;

const QUERY_PYTHON: &str = r#"
(function_definition name: (identifier) @name) @def
(class_definition name: (identifier) @name) @def
"#;

const QUERY_GO: &str = r#"
(function_declaration name: (identifier) @name) @def
(method_declaration name: (field_identifier) @name) @def
(type_spec name: (type_identifier) @name) @def
"#;

const QUERY_JAVA: &str = r#"
(method_declaration name: (identifier) @name) @def
(class_declaration name: (identifier) @name) @def
(interface_declaration name: (identifier) @name) @def
(enum_declaration name: (identifier) @name) @def
(constructor_declaration name: (identifier) @name) @def
"#;

const QUERY_C: &str = r#"
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @name)) @def
(struct_specifier name: (type_identifier) @name) @def
(enum_specifier name: (type_identifier) @name) @def
(type_definition declarator: (type_identifier) @name) @def
"#;

const QUERY_CPP: &str = r#"
(function_definition
  declarator: (function_declarator
    declarator: (_) @name)) @def
(struct_specifier name: (type_identifier) @name) @def
(class_specifier name: (type_identifier) @name) @def
(enum_specifier name: (type_identifier) @name) @def
(namespace_definition name: (identifier) @name) @def
"#;

const QUERY_RUBY: &str = r#"
(method name: (identifier) @name) @def
(singleton_method name: (identifier) @name) @def
(class name: (_) @name) @def
(module name: (_) @name) @def
"#;

const QUERY_CSHARP: &str = r#"
(method_declaration name: (identifier) @name) @def
(class_declaration name: (identifier) @name) @def
(interface_declaration name: (identifier) @name) @def
(struct_declaration name: (identifier) @name) @def
(enum_declaration name: (identifier) @name) @def
(record_declaration name: (identifier) @name) @def
(namespace_declaration name: (identifier) @name) @def
"#;

/// Queries [tree-sitter-kotlin-ng](https://crates.io/crates/tree-sitter-kotlin-ng). Interfaces use `class_declaration` with an `interface` keyword (no separate `interface_declaration` node).
const QUERY_KOTLIN: &str = r#"
(function_declaration name: (identifier) @name) @def
(class_declaration name: (identifier) @name) @def
(object_declaration name: (identifier) @name) @def
"#;

/// Swift grammar uses `class_declaration` for class, struct, enum, actor, and extension (via `declaration_kind`).
const QUERY_SWIFT: &str = r#"
(function_declaration name: (simple_identifier) @name) @def
(class_declaration name: (type_identifier) @name) @def
(protocol_declaration name: (type_identifier) @name) @def
(protocol_function_declaration name: (simple_identifier) @name) @def
"#;

const QUERY_PHP: &str = r#"
(function_definition name: (name) @name) @def
(class_declaration name: (name) @name) @def
(interface_declaration name: (name) @name) @def
(trait_declaration name: (name) @name) @def
(method_declaration name: (name) @name) @def
"#;

const QUERY_BASH: &str = r#"
(function_definition name: (word) @name) @def
"#;

const QUERY_DART: &str = r#"
(class_declaration name: (identifier) @name) @def
(enum_declaration name: (identifier) @name) @def
(mixin_declaration (identifier) @name) @def
(type_alias (type_identifier) @name) @def
"#;

const QUERY_SCALA: &str = r#"
(class_definition name: (identifier) @name) @def
(object_definition name: (identifier) @name) @def
(trait_definition name: (identifier) @name) @def
(enum_definition name: (identifier) @name) @def
(function_definition name: (identifier) @name) @def
(type_definition name: (type_identifier) @name) @def
"#;

const QUERY_ELIXIR: &str = r#"
(call
  target: (identifier) @_keyword
  (arguments (alias) @name)
  (#any-of? @_keyword "defmodule" "defprotocol")) @def

(call
  target: (identifier) @_keyword
  (arguments
    [
      (identifier) @name
      (call target: (identifier) @name)
      (binary_operator left: (call target: (identifier) @name) operator: "when")
    ])
  (#any-of? @_keyword "def" "defp" "defmacro" "defmacrop")) @def
"#;

const QUERY_ZIG: &str = r#"
(function_declaration name: (identifier) @name) @def
"#;

pub fn extract_signatures_ts(content: &str, file_ext: &str) -> Option<Vec<Signature>> {
    if matches!(file_ext, "svelte" | "vue") {
        return extract_sfc_signatures(content);
    }

    let language = get_language(file_ext)?;
    let query_src = get_query(file_ext)?;

    thread_local! {
        static PARSER: std::cell::RefCell<Parser> = std::cell::RefCell::new(Parser::new());
    }

    let tree = PARSER.with(|p| {
        let mut parser = p.borrow_mut();
        let _ = parser.set_language(&language);
        parser.parse(content, None)
    })?;
    let query = Query::new(&language, query_src).ok()?;

    let def_idx = find_capture_index(&query, "def")?;
    let name_idx = find_capture_index(&query, "name")?;

    let source = content.as_bytes();
    let mut sigs = Vec::new();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source);

    while let Some(m) = matches.next() {
        let mut def_node: Option<Node> = None;
        let mut name_text = String::new();

        for cap in m.captures.iter() {
            if cap.index == def_idx {
                def_node = Some(cap.node);
            } else if cap.index == name_idx {
                if let Ok(text) = cap.node.utf8_text(source) {
                    name_text = text.to_string();
                }
            }
        }

        if let Some(node) = def_node {
            if !name_text.is_empty() {
                if let Some(sig) = node_to_signature(&node, &name_text, file_ext, source) {
                    sigs.push(sig);
                }
            }
        }
    }

    Some(sigs)
}

fn get_language(ext: &str) -> Option<Language> {
    Some(match ext {
        "rs" => tree_sitter_rust::LANGUAGE.into(),
        "ts" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        "tsx" => tree_sitter_typescript::LANGUAGE_TSX.into(),
        "js" | "jsx" => tree_sitter_javascript::LANGUAGE.into(),
        "py" => tree_sitter_python::LANGUAGE.into(),
        "go" => tree_sitter_go::LANGUAGE.into(),
        "java" => tree_sitter_java::LANGUAGE.into(),
        "c" | "h" => tree_sitter_c::LANGUAGE.into(),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => tree_sitter_cpp::LANGUAGE.into(),
        "rb" => tree_sitter_ruby::LANGUAGE.into(),
        "cs" => tree_sitter_c_sharp::LANGUAGE.into(),
        "kt" | "kts" => tree_sitter_kotlin_ng::LANGUAGE.into(),
        "swift" => tree_sitter_swift::LANGUAGE.into(),
        "php" => tree_sitter_php::LANGUAGE_PHP.into(),
        "sh" | "bash" => tree_sitter_bash::LANGUAGE.into(),
        "dart" => tree_sitter_dart::LANGUAGE.into(),
        "scala" | "sc" => tree_sitter_scala::LANGUAGE.into(),
        "ex" | "exs" => tree_sitter_elixir::LANGUAGE.into(),
        "zig" => tree_sitter_zig::LANGUAGE.into(),
        _ => return None,
    })
}

fn get_query(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "rs" => QUERY_RUST,
        "ts" | "tsx" => QUERY_TYPESCRIPT,
        "js" | "jsx" => QUERY_JAVASCRIPT,
        "py" => QUERY_PYTHON,
        "go" => QUERY_GO,
        "java" => QUERY_JAVA,
        "c" | "h" => QUERY_C,
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => QUERY_CPP,
        "rb" => QUERY_RUBY,
        "cs" => QUERY_CSHARP,
        "kt" | "kts" => QUERY_KOTLIN,
        "swift" => QUERY_SWIFT,
        "php" => QUERY_PHP,
        "sh" | "bash" => QUERY_BASH,
        "dart" => QUERY_DART,
        "scala" | "sc" => QUERY_SCALA,
        "ex" | "exs" => QUERY_ELIXIR,
        "zig" => QUERY_ZIG,
        _ => return None,
    })
}

fn find_capture_index(query: &Query, name: &str) -> Option<u32> {
    query
        .capture_names()
        .iter()
        .position(|n| *n == name)
        .map(|i| i as u32)
}

fn node_to_signature(node: &Node, name: &str, ext: &str, source: &[u8]) -> Option<Signature> {
    let kind_str = node.kind();
    let start_col = node.start_position().column;

    match kind_str {
        "function_item" => rust_function(node, name, source),
        "struct_item" => rust_struct_like(node, name, "struct"),
        "enum_item" => rust_struct_like(node, name, "enum"),
        "trait_item" => rust_struct_like(node, name, "trait"),
        "impl_item" => rust_impl(node, name, source),
        "type_item" => rust_struct_like(node, name, "type"),
        "const_item" => rust_const(node, name, source),

        "function_declaration" => match ext {
            "kt" | "kts" => kotlin_function(node, name, source),
            "swift" => swift_function(node, name, source),
            "zig" => zig_function(node, name, source),
            _ => ts_or_go_function(node, name, ext, source),
        },
        "protocol_function_declaration" => swift_protocol_function(node, name, source),
        "function_definition" => match ext {
            "sh" | "bash" => simple_def(name, "fn"),
            "scala" | "sc" => scala_function(node, name, source),
            _ => py_or_c_function(node, name, ext, start_col, source),
        },
        "method_definition" => ts_method(node, name, source),
        "method_declaration" => go_or_java_method(node, name, ext, source),
        "variable_declarator" => ts_arrow_function(node, name, source),

        "class_declaration" | "abstract_class_declaration" | "class_specifier" => {
            if ext == "swift" {
                swift_class_declaration(node, name, source)
            } else {
                class_like(node, name, "class", ext, source)
            }
        }
        "object_declaration" => class_like(node, name, "class", ext, source),
        "record_declaration" => class_like(node, name, "class", ext, source),
        "protocol_declaration" => class_like(node, name, "interface", ext, source),
        "trait_declaration" => class_like(node, name, "trait", ext, source),
        "namespace_declaration" => simple_def(name, "class"),
        "struct_declaration" => class_like(node, name, "struct", ext, source),
        "class_definition" => Some(Signature {
            kind: "class",
            name: name.to_string(),
            params: String::new(),
            return_type: String::new(),
            is_async: false,
            is_exported: !name.starts_with('_'),
            indent: 0,
        }),
        "class" | "module" => Some(Signature {
            kind: "class",
            name: name.to_string(),
            params: String::new(),
            return_type: String::new(),
            is_async: false,
            is_exported: true,
            indent: 0,
        }),

        "struct_specifier" => simple_def(name, "struct"),
        "interface_declaration" => class_like(node, name, "interface", ext, source),
        "type_alias_declaration" => Some(Signature {
            kind: "type",
            name: name.to_string(),
            params: String::new(),
            return_type: String::new(),
            is_async: false,
            is_exported: is_in_export(node),
            indent: 0,
        }),
        "type_definition" => simple_def(name, "type"),
        "namespace_definition" => simple_def(name, "class"),

        "enum_declaration" | "enum_specifier" | "enum_definition" => {
            let exported = match ext {
                "java" => has_modifier(node, "public", source),
                "cs" => csharp_has_modifier_text(node, "public", source),
                _ => true,
            };
            Some(Signature {
                kind: "enum",
                name: name.to_string(),
                params: String::new(),
                return_type: String::new(),
                is_async: false,
                is_exported: exported,
                indent: 0,
            })
        }

        "type_spec" => go_type_spec(node, name, source),

        "method" | "singleton_method" => ruby_method(node, name, source),
        "constructor_declaration" => java_constructor(node, name, source),

        // Dart
        "mixin_declaration" => simple_def(name, "class"),
        "type_alias" => simple_def(name, "type"),

        // Scala
        "object_definition" => simple_def(name, "class"),
        "trait_definition" => simple_def(name, "trait"),

        // Elixir — defmodule/def/defp are all `call` nodes
        "call" if ext == "ex" || ext == "exs" => elixir_call(node, name, source),

        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Rust handlers
// ---------------------------------------------------------------------------

fn rust_function(node: &Node, name: &str, source: &[u8]) -> Option<Signature> {
    let params = field_text(node, "parameters", source);
    let ret = field_text(node, "return_type", source);
    let exported = has_named_child(node, "visibility_modifier");
    let is_async = has_keyword_child(node, "async");
    let start_col = node.start_position().column;
    let is_method = start_col > 0;

    Some(Signature {
        kind: if is_method { "method" } else { "fn" },
        name: name.to_string(),
        params: super::signatures::compact_params(&strip_parens(&params)),
        return_type: clean_return_type(&ret),
        is_async,
        is_exported: exported,
        indent: if is_method { 2 } else { 0 },
    })
}

fn rust_struct_like(node: &Node, name: &str, kind: &'static str) -> Option<Signature> {
    Some(Signature {
        kind,
        name: name.to_string(),
        params: String::new(),
        return_type: String::new(),
        is_async: false,
        is_exported: has_named_child(node, "visibility_modifier"),
        indent: 0,
    })
}

fn rust_impl(node: &Node, name: &str, source: &[u8]) -> Option<Signature> {
    let trait_name = node
        .child_by_field_name("trait")
        .and_then(|n| n.utf8_text(source).ok())
        .map(|s| s.to_string());
    let full_name = match trait_name {
        Some(t) => format!("{t} for {name}"),
        None => name.to_string(),
    };
    Some(Signature {
        kind: "class",
        name: full_name,
        params: String::new(),
        return_type: String::new(),
        is_async: false,
        is_exported: false,
        indent: 0,
    })
}

fn rust_const(node: &Node, name: &str, source: &[u8]) -> Option<Signature> {
    let ret = field_text(node, "type", source);
    Some(Signature {
        kind: "const",
        name: name.to_string(),
        params: String::new(),
        return_type: clean_return_type(&ret),
        is_async: false,
        is_exported: has_named_child(node, "visibility_modifier"),
        indent: 0,
    })
}

// ---------------------------------------------------------------------------
// TypeScript / JavaScript handlers
// ---------------------------------------------------------------------------

fn ts_or_go_function(node: &Node, name: &str, ext: &str, source: &[u8]) -> Option<Signature> {
    let params = field_text(node, "parameters", source);
    let (ret, exported, is_async) = match ext {
        "go" => (
            field_text(node, "result", source),
            name.starts_with(|c: char| c.is_uppercase()),
            false,
        ),
        _ => (
            field_text(node, "return_type", source),
            is_in_export(node),
            has_keyword_child(node, "async"),
        ),
    };

    Some(Signature {
        kind: "fn",
        name: name.to_string(),
        params: super::signatures::compact_params(&strip_parens(&params)),
        return_type: clean_return_type(&ret),
        is_async,
        is_exported: exported,
        indent: 0,
    })
}

fn ts_method(node: &Node, name: &str, source: &[u8]) -> Option<Signature> {
    let params = field_text(node, "parameters", source);
    let ret = field_text(node, "return_type", source);

    Some(Signature {
        kind: "method",
        name: name.to_string(),
        params: super::signatures::compact_params(&strip_parens(&params)),
        return_type: clean_return_type(&ret),
        is_async: has_keyword_child(node, "async"),
        is_exported: false,
        indent: 2,
    })
}

fn ts_arrow_function(node: &Node, name: &str, source: &[u8]) -> Option<Signature> {
    let arrow = node.child_by_field_name("value")?;
    let params = field_text(&arrow, "parameters", source);
    let ret = field_text(&arrow, "return_type", source);
    let exported = node
        .parent()
        .and_then(|p| p.parent())
        .map(|gp| gp.kind() == "export_statement")
        .unwrap_or(false);

    Some(Signature {
        kind: "fn",
        name: name.to_string(),
        params: super::signatures::compact_params(&strip_parens(&params)),
        return_type: clean_return_type(&ret),
        is_async: has_keyword_child(&arrow, "async"),
        is_exported: exported,
        indent: 0,
    })
}

// ---------------------------------------------------------------------------
// Python / C / C++ handlers
// ---------------------------------------------------------------------------

fn py_or_c_function(
    node: &Node,
    name: &str,
    ext: &str,
    start_col: usize,
    source: &[u8],
) -> Option<Signature> {
    match ext {
        "py" | "php" => {
            let params = field_text(node, "parameters", source);
            let ret = field_text(node, "return_type", source);
            let is_method = start_col > 0;
            Some(Signature {
                kind: if is_method { "method" } else { "fn" },
                name: name.to_string(),
                params: super::signatures::compact_params(&strip_parens(&params)),
                return_type: clean_return_type(&ret),
                is_async: has_keyword_child(node, "async"),
                is_exported: !name.starts_with('_'),
                indent: if is_method { 2 } else { 0 },
            })
        }
        _ => {
            let ret = field_text(node, "type", source);
            let params = node
                .child_by_field_name("declarator")
                .and_then(|d| d.child_by_field_name("parameters"))
                .and_then(|p| p.utf8_text(source).ok())
                .unwrap_or("")
                .to_string();
            Some(Signature {
                kind: "fn",
                name: name.to_string(),
                params: super::signatures::compact_params(&strip_parens(&params)),
                return_type: ret.trim().to_string(),
                is_async: false,
                is_exported: true,
                indent: 0,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Go / Java handlers
// ---------------------------------------------------------------------------

fn go_or_java_method(node: &Node, name: &str, ext: &str, source: &[u8]) -> Option<Signature> {
    match ext {
        "go" => {
            let params = field_text(node, "parameters", source);
            let ret = field_text(node, "result", source);
            Some(Signature {
                kind: "method",
                name: name.to_string(),
                params: super::signatures::compact_params(&strip_parens(&params)),
                return_type: clean_return_type(&ret),
                is_async: false,
                is_exported: name.starts_with(|c: char| c.is_uppercase()),
                indent: 2,
            })
        }
        "java" => {
            let params = field_text(node, "parameters", source);
            let ret = field_text(node, "type", source);
            let is_method = node.start_position().column > 0;
            Some(Signature {
                kind: if is_method { "method" } else { "fn" },
                name: name.to_string(),
                params: super::signatures::compact_params(&strip_parens(&params)),
                return_type: ret.trim().to_string(),
                is_async: false,
                is_exported: has_modifier(node, "public", source),
                indent: if is_method { 2 } else { 0 },
            })
        }
        "cs" => {
            let params = field_text(node, "parameters", source);
            let ret = field_text(node, "returns", source);
            let is_method = node.start_position().column > 0;
            Some(Signature {
                kind: if is_method { "method" } else { "fn" },
                name: name.to_string(),
                params: super::signatures::compact_params(&strip_parens(&params)),
                return_type: ret.trim().to_string(),
                is_async: false,
                is_exported: csharp_has_modifier_text(node, "public", source),
                indent: if is_method { 2 } else { 0 },
            })
        }
        "php" => {
            let params = field_text(node, "parameters", source);
            let ret = field_text(node, "return_type", source);
            let is_method = node.start_position().column > 0;
            Some(Signature {
                kind: if is_method { "method" } else { "fn" },
                name: name.to_string(),
                params: super::signatures::compact_params(&strip_parens(&params)),
                return_type: clean_return_type(&ret),
                is_async: false,
                is_exported: true,
                indent: if is_method { 2 } else { 0 },
            })
        }
        _ => None,
    }
}

fn go_type_spec(node: &Node, name: &str, _source: &[u8]) -> Option<Signature> {
    let type_kind = node
        .child_by_field_name("type")
        .map(|n| n.kind().to_string())
        .unwrap_or_default();
    let kind = match type_kind.as_str() {
        "struct_type" => "struct",
        "interface_type" => "interface",
        _ => "type",
    };
    Some(Signature {
        kind,
        name: name.to_string(),
        params: String::new(),
        return_type: String::new(),
        is_async: false,
        is_exported: name.starts_with(|c: char| c.is_uppercase()),
        indent: 0,
    })
}

fn java_constructor(node: &Node, name: &str, source: &[u8]) -> Option<Signature> {
    let params = field_text(node, "parameters", source);
    Some(Signature {
        kind: "fn",
        name: name.to_string(),
        params: super::signatures::compact_params(&strip_parens(&params)),
        return_type: String::new(),
        is_async: false,
        is_exported: has_modifier(node, "public", source),
        indent: 2,
    })
}

// ---------------------------------------------------------------------------
// Ruby handler
// ---------------------------------------------------------------------------

fn ruby_method(node: &Node, name: &str, source: &[u8]) -> Option<Signature> {
    let params = field_text(node, "parameters", source);
    Some(Signature {
        kind: "method",
        name: name.to_string(),
        params: super::signatures::compact_params(&strip_parens(&params)),
        return_type: String::new(),
        is_async: false,
        is_exported: true,
        indent: 2,
    })
}

// ---------------------------------------------------------------------------
// Kotlin (tree-sitter-kotlin-ng) / Swift / C# helpers
// ---------------------------------------------------------------------------

fn kotlin_function(node: &Node, name: &str, source: &[u8]) -> Option<Signature> {
    let mut params = String::new();
    let mut ret = String::new();
    let mut seen_params = false;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "function_value_parameters" {
            params = child.utf8_text(source).unwrap_or("").to_string();
            seen_params = true;
        } else if seen_params && child.kind() == "type" {
            ret = child.utf8_text(source).unwrap_or("").to_string();
            ret = ret.trim_start_matches(':').trim().to_string();
            break;
        }
    }
    let is_method = node.start_position().column > 0;
    let exported = kotlin_modifiers_text(node, source)
        .map(|t| !t.contains("private"))
        .unwrap_or(true);
    Some(Signature {
        kind: if is_method { "method" } else { "fn" },
        name: name.to_string(),
        params: super::signatures::compact_params(&strip_parens(&params)),
        return_type: clean_return_type(&ret),
        is_async: false,
        is_exported: exported,
        indent: if is_method { 2 } else { 0 },
    })
}

fn kotlin_modifiers_text<'a>(node: &Node, source: &'a [u8]) -> Option<&'a str> {
    let mut cursor = node.walk();
    for c in node.children(&mut cursor) {
        if c.kind() == "modifiers" {
            return c.utf8_text(source).ok();
        }
    }
    None
}

fn kotlin_declaration_exported(node: &Node, source: &[u8]) -> bool {
    kotlin_modifiers_text(node, source)
        .map(|t| !t.contains("private"))
        .unwrap_or(true)
}

fn swift_function(node: &Node, name: &str, source: &[u8]) -> Option<Signature> {
    let ret = field_text(node, "return_type", source);
    let params = swift_parameters_before_body(node, source);
    let is_async = has_named_child(node, "async");
    let is_method = node.start_position().column > 0;
    Some(Signature {
        kind: if is_method { "method" } else { "fn" },
        name: name.to_string(),
        params: super::signatures::compact_params(&strip_parens(&params)),
        return_type: clean_return_type(&ret),
        is_async,
        is_exported: true,
        indent: if is_method { 2 } else { 0 },
    })
}

fn swift_protocol_function(node: &Node, name: &str, source: &[u8]) -> Option<Signature> {
    let ret = field_text(node, "return_type", source);
    Some(Signature {
        kind: "fn",
        name: name.to_string(),
        params: String::new(),
        return_type: clean_return_type(&ret),
        is_async: has_named_child(node, "async"),
        is_exported: true,
        indent: 2,
    })
}

fn swift_class_declaration(node: &Node, name: &str, source: &[u8]) -> Option<Signature> {
    let kind = node
        .child_by_field_name("declaration_kind")
        .and_then(|n| n.utf8_text(source).ok())
        .unwrap_or("class");
    let kind_static: &'static str = match kind {
        "struct" => "struct",
        "enum" => "enum",
        _ => "class",
    };
    Some(Signature {
        kind: kind_static,
        name: name.to_string(),
        params: String::new(),
        return_type: String::new(),
        is_async: false,
        is_exported: true,
        indent: 0,
    })
}

fn swift_parameters_before_body(node: &Node, source: &[u8]) -> String {
    let end_byte = node
        .child_by_field_name("body")
        .map(|b| b.start_byte())
        .unwrap_or(usize::MAX);
    let mut parts: Vec<String> = Vec::new();
    fn walk(n: &Node, end_byte: usize, source: &[u8], parts: &mut Vec<String>) {
        if n.start_byte() >= end_byte {
            return;
        }
        if n.kind() == "parameter" {
            if let Ok(t) = n.utf8_text(source) {
                parts.push(t.to_string());
            }
        }
        let mut c = n.walk();
        for child in n.children(&mut c) {
            if child.start_byte() < end_byte {
                walk(&child, end_byte, source, parts);
            }
        }
    }
    walk(node, end_byte, source, &mut parts);
    if parts.is_empty() {
        String::new()
    } else {
        format!("({})", parts.join(", "))
    }
}

fn csharp_has_modifier_text(node: &Node, needle: &str, source: &[u8]) -> bool {
    let mut cursor = node.walk();
    for c in node.children(&mut cursor) {
        if c.kind() == "modifier" {
            if let Ok(t) = c.utf8_text(source) {
                if t.contains(needle) {
                    return true;
                }
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Scala handlers
// ---------------------------------------------------------------------------

fn scala_function(node: &Node, name: &str, source: &[u8]) -> Option<Signature> {
    let params = field_text(node, "parameters", source);
    let ret = field_text(node, "return_type", source);
    let is_method = node.start_position().column > 0;
    Some(Signature {
        kind: if is_method { "method" } else { "fn" },
        name: name.to_string(),
        params: super::signatures::compact_params(&strip_parens(&params)),
        return_type: clean_return_type(&ret),
        is_async: false,
        is_exported: !name.starts_with('_'),
        indent: if is_method { 2 } else { 0 },
    })
}

// ---------------------------------------------------------------------------
// Elixir handlers
// ---------------------------------------------------------------------------

fn elixir_call(node: &Node, name: &str, source: &[u8]) -> Option<Signature> {
    let target = node
        .child_by_field_name("target")
        .and_then(|n| n.utf8_text(source).ok())
        .unwrap_or("");
    match target {
        "defmodule" | "defprotocol" => Some(Signature {
            kind: "class",
            name: name.to_string(),
            params: String::new(),
            return_type: String::new(),
            is_async: false,
            is_exported: true,
            indent: 0,
        }),
        "def" | "defmacro" | "defdelegate" | "defguard" => Some(Signature {
            kind: "fn",
            name: name.to_string(),
            params: String::new(),
            return_type: String::new(),
            is_async: false,
            is_exported: true,
            indent: 2,
        }),
        "defp" | "defmacrop" | "defguardp" => Some(Signature {
            kind: "fn",
            name: name.to_string(),
            params: String::new(),
            return_type: String::new(),
            is_async: false,
            is_exported: false,
            indent: 2,
        }),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Zig handlers
// ---------------------------------------------------------------------------

fn zig_function(node: &Node, name: &str, source: &[u8]) -> Option<Signature> {
    let params = field_text(node, "parameters", source);
    let ret = field_text(node, "return_type", source);
    let exported = has_keyword_child(node, "pub");
    Some(Signature {
        kind: "fn",
        name: name.to_string(),
        params: super::signatures::compact_params(&strip_parens(&params)),
        return_type: clean_return_type(&ret),
        is_async: false,
        is_exported: exported,
        indent: 0,
    })
}

// ---------------------------------------------------------------------------
// SFC (Single File Component) support for Svelte/Vue
// ---------------------------------------------------------------------------

fn extract_sfc_signatures(content: &str) -> Option<Vec<Signature>> {
    let script_content = extract_script_block(content)?;
    let is_ts = content.contains("lang=\"ts\"") || content.contains("lang=\"typescript\"");
    let ext = if is_ts { "ts" } else { "js" };
    extract_signatures_ts(&script_content, ext)
}

fn extract_script_block(content: &str) -> Option<String> {
    let lower = content.to_lowercase();
    let start_tag_pos = lower.find("<script")?;
    let tag_end = content[start_tag_pos..].find('>')? + start_tag_pos + 1;
    let end_tag = "</script>";
    let end_pos = lower[tag_end..].find(end_tag)? + tag_end;
    let script = &content[tag_end..end_pos];
    if script.trim().is_empty() {
        return None;
    }
    Some(script.to_string())
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn class_like(
    node: &Node,
    name: &str,
    kind: &'static str,
    ext: &str,
    source: &[u8],
) -> Option<Signature> {
    let exported = match ext {
        "ts" | "tsx" | "js" | "jsx" => is_in_export(node),
        "java" => has_modifier(node, "public", source),
        "cs" => csharp_has_modifier_text(node, "public", source),
        "kt" | "kts" => kotlin_declaration_exported(node, source),
        _ => true,
    };
    Some(Signature {
        kind,
        name: name.to_string(),
        params: String::new(),
        return_type: String::new(),
        is_async: false,
        is_exported: exported,
        indent: 0,
    })
}

fn simple_def(name: &str, kind: &'static str) -> Option<Signature> {
    Some(Signature {
        kind,
        name: name.to_string(),
        params: String::new(),
        return_type: String::new(),
        is_async: false,
        is_exported: true,
        indent: 0,
    })
}

fn field_text(node: &Node, field: &str, source: &[u8]) -> String {
    node.child_by_field_name(field)
        .and_then(|n| n.utf8_text(source).ok())
        .unwrap_or("")
        .to_string()
}

fn strip_parens(s: &str) -> String {
    let s = s.trim();
    let s = s.strip_prefix('(').unwrap_or(s);
    let s = s.strip_suffix(')').unwrap_or(s);
    s.to_string()
}

fn clean_return_type(ret: &str) -> String {
    let ret = ret.trim();
    if ret.is_empty() {
        return String::new();
    }
    let ret = ret.strip_prefix("->").unwrap_or(ret).trim();
    let ret = ret.strip_prefix(':').unwrap_or(ret).trim();
    ret.to_string()
}

fn has_named_child(node: &Node, kind: &str) -> bool {
    let mut cursor = node.walk();
    let result = node.children(&mut cursor).any(|c| c.kind() == kind);
    result
}

fn has_keyword_child(node: &Node, keyword: &str) -> bool {
    let mut cursor = node.walk();
    let result = node
        .children(&mut cursor)
        .any(|c| !c.is_named() && c.kind() == keyword);
    result
}

fn is_in_export(node: &Node) -> bool {
    node.parent()
        .map(|p| p.kind() == "export_statement")
        .unwrap_or(false)
}

fn has_modifier(node: &Node, modifier: &str, source: &[u8]) -> bool {
    node.child_by_field_name("modifiers")
        .and_then(|m| m.utf8_text(source).ok())
        .map(|t| t.contains(modifier))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_signatures() {
        let src = r#"
pub struct Config {
    name: String,
}

pub enum Status {
    Active,
    Inactive,
}

pub trait Handler {
    fn handle(&self);
}

impl Handler for Config {
    fn handle(&self) {
        println!("handling");
    }
}

pub async fn process(input: &str) -> Result<String, Error> {
    Ok(input.to_string())
}

fn helper(x: i32) -> bool {
    x > 0
}
"#;
        let sigs = extract_signatures_ts(src, "rs").unwrap();
        assert!(sigs.len() >= 5, "expected >=5 sigs, got {}", sigs.len());

        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Config"));
        assert!(names.contains(&"Status"));
        assert!(names.contains(&"Handler"));
        assert!(names.contains(&"process"));
        assert!(names.contains(&"helper"));
    }

    #[test]
    fn test_typescript_signatures() {
        let src = r#"
export function greet(name: string): string {
    return `Hello ${name}`;
}

export class UserService {
    async findUser(id: number): Promise<User> {
        return db.find(id);
    }
}

export interface Config {
    host: string;
    port: number;
}

export type UserId = string;

const handler = async (req: Request): Promise<Response> => {
    return new Response();
};
"#;
        let sigs = extract_signatures_ts(src, "ts").unwrap();
        assert!(sigs.len() >= 5, "expected >=5 sigs, got {}", sigs.len());

        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"greet"));
        assert!(names.contains(&"UserService"));
        assert!(names.contains(&"Config"));
        assert!(names.contains(&"UserId"));
        assert!(names.contains(&"handler"));
    }

    #[test]
    fn test_python_signatures() {
        let src = r#"
class AuthService:
    def __init__(self, db):
        self.db = db

    async def authenticate(self, email: str, password: str) -> bool:
        user = await self.db.find(email)
        return check(user, password)

def create_app() -> Flask:
    return Flask(__name__)

def _internal_helper(x):
    return x * 2
"#;
        let sigs = extract_signatures_ts(src, "py").unwrap();
        assert!(sigs.len() >= 4, "expected >=4 sigs, got {}", sigs.len());

        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"AuthService"));
        assert!(names.contains(&"authenticate"));
        assert!(names.contains(&"create_app"));

        let auth = sigs.iter().find(|s| s.name == "authenticate").unwrap();
        assert!(auth.is_async);
        assert_eq!(auth.kind, "method");

        let helper = sigs.iter().find(|s| s.name == "_internal_helper").unwrap();
        assert!(!helper.is_exported);
    }

    #[test]
    fn test_go_signatures() {
        let src = r#"
package main

type Config struct {
    Host string
    Port int
}

type Handler interface {
    Handle() error
}

func NewConfig(host string, port int) *Config {
    return &Config{Host: host, Port: port}
}

func (c *Config) Validate() error {
    return nil
}

func helper() {
}
"#;
        let sigs = extract_signatures_ts(src, "go").unwrap();
        assert!(sigs.len() >= 4, "expected >=4 sigs, got {}", sigs.len());

        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Config"));
        assert!(names.contains(&"Handler"));
        assert!(names.contains(&"NewConfig"));
        assert!(names.contains(&"Validate"));

        let nc = sigs.iter().find(|s| s.name == "NewConfig").unwrap();
        assert!(nc.is_exported);

        let h = sigs.iter().find(|s| s.name == "helper").unwrap();
        assert!(!h.is_exported);
    }

    #[test]
    fn test_java_signatures() {
        let src = r#"
public class UserController {
    public UserController(UserService service) {
        this.service = service;
    }

    public User getUser(int id) {
        return service.findById(id);
    }

    private void validate(User user) {
        // validation logic
    }
}

public interface Repository {
    User findById(int id);
}

public enum Role {
    ADMIN,
    USER
}
"#;
        let sigs = extract_signatures_ts(src, "java").unwrap();
        assert!(sigs.len() >= 4, "expected >=4 sigs, got {}", sigs.len());

        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"UserController"));
        assert!(names.contains(&"getUser"));
        assert!(names.contains(&"Repository"));
        assert!(names.contains(&"Role"));
    }

    #[test]
    fn test_c_signatures() {
        let src = r#"
typedef unsigned int uint;

struct Config {
    char* host;
    int port;
};

enum Status {
    ACTIVE,
    INACTIVE
};

int process(const char* input, int len) {
    return 0;
}

void cleanup(struct Config* cfg) {
    free(cfg);
}
"#;
        let sigs = extract_signatures_ts(src, "c").unwrap();
        assert!(sigs.len() >= 3, "expected >=3 sigs, got {}", sigs.len());

        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"process"));
        assert!(names.contains(&"cleanup"));
    }

    #[test]
    fn test_ruby_signatures() {
        let src = r#"
module Authentication
  class UserService
    def initialize(db)
      @db = db
    end

    def authenticate(email, password)
      user = @db.find(email)
      user&.check(password)
    end

    def self.create(config)
      new(config[:db])
    end
  end
end
"#;
        let sigs = extract_signatures_ts(src, "rb").unwrap();
        assert!(sigs.len() >= 3, "expected >=3 sigs, got {}", sigs.len());

        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"UserService"));
        assert!(names.contains(&"authenticate"));
    }

    #[test]
    fn test_multiline_rust_signature() {
        let src = r#"
pub fn complex_function<T: Display + Debug>(
    first_arg: &str,
    second_arg: Vec<T>,
    third_arg: Option<HashMap<String, Vec<u8>>>,
) -> Result<(), Box<dyn Error>> {
    Ok(())
}
"#;
        let sigs = extract_signatures_ts(src, "rs").unwrap();
        assert!(!sigs.is_empty(), "should parse multiline function");
        assert_eq!(sigs[0].name, "complex_function");
        assert!(sigs[0].is_exported);
    }

    #[test]
    fn test_arrow_function_ts() {
        let src = r#"
export const fetchData = async (url: string): Promise<Response> => {
    return fetch(url);
};

const internal = (x: number) => x * 2;
"#;
        let sigs = extract_signatures_ts(src, "ts").unwrap();
        assert!(sigs.len() >= 2, "expected >=2 sigs, got {}", sigs.len());

        let fetch = sigs.iter().find(|s| s.name == "fetchData").unwrap();
        assert!(fetch.is_async);
        assert!(fetch.is_exported);
        assert_eq!(fetch.kind, "fn");

        let internal = sigs.iter().find(|s| s.name == "internal").unwrap();
        assert!(!internal.is_exported);
    }

    #[test]
    fn test_csharp_signatures() {
        let src = r#"
namespace Demo;
public record Person(string Name);
public interface IRepo { void Save(); }
public struct Point { public int X; }
public enum Role { Admin, User }
public class Service {
    public string Hello(string name) => name;
}
"#;
        let sigs = extract_signatures_ts(src, "cs").unwrap();
        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Person"), "got {:?}", names);
        assert!(names.contains(&"IRepo"));
        assert!(names.contains(&"Point"));
        assert!(names.contains(&"Role"));
        assert!(names.contains(&"Service"));
        assert!(names.contains(&"Hello"));
    }

    #[test]
    fn test_kotlin_signatures() {
        let src = r#"
class UserService {
    fun greet(name: String): String = "Hi $name"
}
object Factory {
    fun build(): UserService = UserService()
}
interface Handler {
    fun handle()
}
"#;
        let sigs = extract_signatures_ts(src, "kt").unwrap();
        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"UserService"), "got {:?}", names);
        assert!(names.contains(&"Factory"));
        assert!(names.contains(&"Handler"));
        assert!(names.contains(&"greet"));
        assert!(names.contains(&"build"));
        assert!(names.contains(&"handle"));
    }

    #[test]
    fn test_swift_signatures() {
        let src = r#"
class Box {
    func size() -> Int { 0 }
}
struct Point {
    var x: Int
}
enum Kind { case a, b }
protocol Drawable {
    func draw()
}
func topLevel() {}
"#;
        let sigs = extract_signatures_ts(src, "swift").unwrap();
        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Box"), "got {:?}", names);
        assert!(names.contains(&"Point"));
        assert!(names.contains(&"Kind"));
        assert!(names.contains(&"Drawable"));
        assert!(names.contains(&"size"));
        assert!(names.contains(&"draw"));
        assert!(names.contains(&"topLevel"));
    }

    #[test]
    fn test_php_signatures() {
        let src = r#"<?php
function helper(int $x): int { return $x; }
class User {
    public function name(): string { return ''; }
}
interface IAuth { public function check(): bool; }
trait Loggable { function log(): void {} }
"#;
        let sigs = extract_signatures_ts(src, "php").unwrap();
        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"helper"), "got {:?}", names);
        assert!(names.contains(&"User"));
        assert!(names.contains(&"name"));
        assert!(names.contains(&"IAuth"));
        assert!(names.contains(&"check"));
        assert!(names.contains(&"Loggable"));
        assert!(names.contains(&"log"));
    }

    #[test]
    fn test_unsupported_extension_returns_none() {
        let sigs = extract_signatures_ts("some content", "xyz");
        assert!(sigs.is_none());
    }

    #[test]
    fn test_bash_signatures() {
        let src = r#"
greet() {
    echo "Hello $1"
}

function cleanup {
    rm -rf /tmp/build
}

function deploy() {
    echo "deploying"
}
"#;
        let sigs = extract_signatures_ts(src, "sh").unwrap();
        assert!(sigs.len() >= 2, "expected >=2 sigs, got {}", sigs.len());
        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"greet"), "got {:?}", names);
        assert!(names.contains(&"cleanup"), "got {:?}", names);
    }

    #[test]
    fn test_dart_signatures() {
        let src = r#"
class UserService {
  Future<User> getUser(int id) async {
    return db.find(id);
  }
}

enum Status { active, inactive }

mixin Logging {
  void log(String msg) => print(msg);
}

typedef JsonMap = Map<String, dynamic>;
"#;
        let sigs = extract_signatures_ts(src, "dart").unwrap();
        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"UserService"), "got {:?}", names);
        assert!(names.contains(&"Status"), "got {:?}", names);
        assert!(names.contains(&"Logging"), "got {:?}", names);
    }

    #[test]
    fn test_scala_signatures() {
        let src = r#"
package example

trait Handler {
  def handle(): Unit
}

class UserService(db: Database) {
  def findUser(id: Int): Option[User] = db.find(id)
  private def validate(user: User): Boolean = true
}

object Factory {
  def create(): UserService = new UserService(db)
}

enum Color:
  case Red, Green, Blue

type UserId = String
"#;
        let sigs = extract_signatures_ts(src, "scala").unwrap();
        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Handler"), "got {:?}", names);
        assert!(names.contains(&"UserService"), "got {:?}", names);
        assert!(names.contains(&"Factory"), "got {:?}", names);
        assert!(names.contains(&"findUser"), "got {:?}", names);
    }

    #[test]
    fn test_elixir_signatures() {
        let src = r#"
defmodule MyApp.UserService do
  def get_user(id) do
    Repo.get(User, id)
  end

  defp validate(user) do
    user.valid?
  end

  defmacro trace(expr) do
    quote do: IO.inspect(unquote(expr))
  end
end

defprotocol Printable do
  def print(data)
end
"#;
        let sigs = extract_signatures_ts(src, "ex").unwrap();
        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"MyApp.UserService") || names.contains(&"UserService"),
            "got {:?}",
            names
        );
    }

    #[test]
    fn test_svelte_signatures() {
        let src = r#"
<script lang="ts">
export function greet(name: string): string {
    return `Hello ${name}`;
}

export class Counter {
    count = 0;
    increment() { this.count++; }
}
</script>

<div>{greeting}</div>
"#;
        let sigs = extract_signatures_ts(src, "svelte").unwrap();
        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"greet"), "got {:?}", names);
        assert!(names.contains(&"Counter"), "got {:?}", names);
    }

    #[test]
    fn test_vue_signatures() {
        let src = r#"
<template>
  <div>{{ msg }}</div>
</template>

<script>
export default {
  name: 'MyComponent'
}

export function helper(x) {
    return x * 2;
}

export class DataService {
    fetch() { return []; }
}
</script>
"#;
        let sigs = extract_signatures_ts(src, "vue").unwrap();
        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"helper"), "got {:?}", names);
        assert!(names.contains(&"DataService"), "got {:?}", names);
    }

    #[test]
    fn test_zig_signatures() {
        let src = r#"
const std = @import("std");

pub fn init(allocator: std.mem.Allocator) !*Self {
    return allocator.create(Self);
}

fn helper(x: u32) u32 {
    return x * 2;
}

pub fn main() !void {
    std.debug.print("hello\n", .{});
}
"#;
        let sigs = extract_signatures_ts(src, "zig").unwrap();
        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"init"), "got {:?}", names);
        assert!(names.contains(&"helper"), "got {:?}", names);
        assert!(names.contains(&"main"), "got {:?}", names);

        let init_sig = sigs.iter().find(|s| s.name == "init").unwrap();
        assert!(init_sig.is_exported);
        let helper_sig = sigs.iter().find(|s| s.name == "helper").unwrap();
        assert!(!helper_sig.is_exported);
    }
}
