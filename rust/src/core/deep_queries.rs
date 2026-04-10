//! Tree-sitter deep queries for extracting imports, call sites, and type definitions.
//!
//! Replaces regex-based extraction in `deps.rs` with precise AST parsing.
//! Supports: TypeScript/JavaScript, Python, Rust, Go, Java.

#[cfg(feature = "tree-sitter")]
use tree_sitter::{Language, Node, Parser};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImportInfo {
    pub source: String,
    pub names: Vec<String>,
    pub kind: ImportKind,
    pub line: usize,
    pub is_type_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImportKind {
    Named,
    Default,
    Star,
    SideEffect,
    Dynamic,
    Reexport,
}

#[derive(Debug, Clone)]
pub struct CallSite {
    pub callee: String,
    pub line: usize,
    pub col: usize,
    pub receiver: Option<String>,
    pub is_method: bool,
}

#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: String,
    pub kind: TypeDefKind,
    pub line: usize,
    pub end_line: usize,
    pub is_exported: bool,
    pub generics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDefKind {
    Class,
    Interface,
    TypeAlias,
    Enum,
    Struct,
    Trait,
    Protocol,
    Record,
    Annotation,
    Union,
}

#[derive(Debug, Clone)]
pub struct DeepAnalysis {
    pub imports: Vec<ImportInfo>,
    pub calls: Vec<CallSite>,
    pub types: Vec<TypeDef>,
    pub exports: Vec<String>,
}

impl DeepAnalysis {
    pub fn empty() -> Self {
        Self {
            imports: Vec::new(),
            calls: Vec::new(),
            types: Vec::new(),
            exports: Vec::new(),
        }
    }
}

pub fn analyze(content: &str, ext: &str) -> DeepAnalysis {
    #[cfg(feature = "tree-sitter")]
    {
        if let Some(result) = analyze_with_tree_sitter(content, ext) {
            return result;
        }
    }

    let _ = (content, ext);
    DeepAnalysis::empty()
}

#[cfg(feature = "tree-sitter")]
fn analyze_with_tree_sitter(content: &str, ext: &str) -> Option<DeepAnalysis> {
    let language = get_language(ext)?;
    let mut parser = Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(content.as_bytes(), None)?;
    let root = tree.root_node();

    let imports = extract_imports(root, content, ext);
    let calls = extract_calls(root, content, ext);
    let types = extract_types(root, content, ext);
    let exports = extract_exports(root, content, ext);

    Some(DeepAnalysis {
        imports,
        calls,
        types,
        exports,
    })
}

#[cfg(feature = "tree-sitter")]
fn get_language(ext: &str) -> Option<Language> {
    match ext {
        "rs" => Some(tree_sitter_rust::LANGUAGE.into()),
        "ts" | "tsx" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "js" | "jsx" => Some(tree_sitter_javascript::LANGUAGE.into()),
        "py" => Some(tree_sitter_python::LANGUAGE.into()),
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "java" => Some(tree_sitter_java::LANGUAGE.into()),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Imports
// ---------------------------------------------------------------------------

#[cfg(feature = "tree-sitter")]
fn extract_imports(root: Node, src: &str, ext: &str) -> Vec<ImportInfo> {
    match ext {
        "ts" | "tsx" | "js" | "jsx" => extract_imports_ts(root, src),
        "rs" => extract_imports_rust(root, src),
        "py" => extract_imports_python(root, src),
        "go" => extract_imports_go(root, src),
        "java" => extract_imports_java(root, src),
        _ => Vec::new(),
    }
}

#[cfg(feature = "tree-sitter")]
fn extract_imports_ts(root: Node, src: &str) -> Vec<ImportInfo> {
    let mut imports = Vec::new();
    let mut cursor = root.walk();

    for node in root.children(&mut cursor) {
        match node.kind() {
            "import_statement" => {
                if let Some(info) = parse_ts_import(node, src) {
                    imports.push(info);
                }
            }
            "export_statement" => {
                if let Some(source) = find_child_by_kind(node, "string") {
                    let source_text = unquote(node_text(source, src));
                    let names = collect_named_imports(node, src);
                    imports.push(ImportInfo {
                        source: source_text,
                        names,
                        kind: ImportKind::Reexport,
                        line: node.start_position().row + 1,
                        is_type_only: false,
                    });
                }
            }
            _ => {}
        }
    }

    walk_for_dynamic_imports(root, src, &mut imports);

    imports
}

#[cfg(feature = "tree-sitter")]
fn parse_ts_import(node: Node, src: &str) -> Option<ImportInfo> {
    let source_node =
        find_child_by_kind(node, "string").or_else(|| find_descendant_by_kind(node, "string"))?;
    let source = unquote(node_text(source_node, src));

    let is_type_only = node_text(node, src).starts_with("import type");

    let clause = find_child_by_kind(node, "import_clause");
    let (kind, names) = match clause {
        Some(c) => classify_ts_import_clause(c, src),
        None => (ImportKind::SideEffect, Vec::new()),
    };

    Some(ImportInfo {
        source,
        names,
        kind,
        line: node.start_position().row + 1,
        is_type_only,
    })
}

#[cfg(feature = "tree-sitter")]
fn classify_ts_import_clause(clause: Node, src: &str) -> (ImportKind, Vec<String>) {
    let mut names = Vec::new();
    let mut has_default = false;
    let mut has_star = false;

    let mut cursor = clause.walk();
    for child in clause.children(&mut cursor) {
        match child.kind() {
            "identifier" => {
                has_default = true;
                names.push(node_text(child, src).to_string());
            }
            "namespace_import" => {
                has_star = true;
                if let Some(id) = find_child_by_kind(child, "identifier") {
                    names.push(format!("* as {}", node_text(id, src)));
                }
            }
            "named_imports" => {
                let mut inner = child.walk();
                for spec in child.children(&mut inner) {
                    if spec.kind() == "import_specifier" {
                        let name = find_child_by_kind(spec, "identifier")
                            .map(|n| node_text(n, src).to_string());
                        if let Some(n) = name {
                            names.push(n);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let kind = if has_star {
        ImportKind::Star
    } else if has_default && names.len() == 1 {
        ImportKind::Default
    } else {
        ImportKind::Named
    };

    (kind, names)
}

#[cfg(feature = "tree-sitter")]
fn walk_for_dynamic_imports(node: Node, src: &str, imports: &mut Vec<ImportInfo>) {
    if node.kind() == "call_expression" {
        let callee = find_child_by_kind(node, "import");
        if callee.is_some() {
            if let Some(args) = find_child_by_kind(node, "arguments") {
                if let Some(first_arg) = find_child_by_kind(args, "string") {
                    imports.push(ImportInfo {
                        source: unquote(node_text(first_arg, src)),
                        names: Vec::new(),
                        kind: ImportKind::Dynamic,
                        line: node.start_position().row + 1,
                        is_type_only: false,
                    });
                }
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_for_dynamic_imports(child, src, imports);
    }
}

#[cfg(feature = "tree-sitter")]
fn extract_imports_rust(root: Node, src: &str) -> Vec<ImportInfo> {
    let mut imports = Vec::new();
    let mut cursor = root.walk();

    for node in root.children(&mut cursor) {
        if node.kind() == "mod_item" {
            let text = node_text(node, src);
            if !text.contains('{') {
                if let Some(name_node) = find_child_by_kind(node, "identifier") {
                    let mod_name = node_text(name_node, src).to_string();
                    imports.push(ImportInfo {
                        source: mod_name.clone(),
                        names: vec![mod_name],
                        kind: ImportKind::Named,
                        line: node.start_position().row + 1,
                        is_type_only: false,
                    });
                }
            }
        } else if node.kind() == "use_declaration" {
            let is_pub = node_text(node, src).trim_start().starts_with("pub");
            let kind = if is_pub {
                ImportKind::Reexport
            } else {
                ImportKind::Named
            };

            if let Some(arg) = find_child_by_kind(node, "use_as_clause")
                .or_else(|| find_child_by_kind(node, "scoped_identifier"))
                .or_else(|| find_child_by_kind(node, "scoped_use_list"))
                .or_else(|| find_child_by_kind(node, "use_wildcard"))
                .or_else(|| find_child_by_kind(node, "identifier"))
            {
                let full_path = node_text(arg, src).to_string();

                let (source, names) = if full_path.contains('{') {
                    let parts: Vec<&str> = full_path.splitn(2, "::").collect();
                    let base = parts[0].to_string();
                    let items: Vec<String> = full_path
                        .split('{')
                        .nth(1)
                        .unwrap_or("")
                        .trim_end_matches('}')
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    (base, items)
                } else if full_path.ends_with("::*") {
                    (
                        full_path.trim_end_matches("::*").to_string(),
                        vec!["*".to_string()],
                    )
                } else {
                    let name = full_path.rsplit("::").next().unwrap_or(&full_path);
                    (full_path.clone(), vec![name.to_string()])
                };

                let is_std = source.starts_with("std")
                    || source.starts_with("core")
                    || source.starts_with("alloc");
                if !is_std {
                    imports.push(ImportInfo {
                        source,
                        names,
                        kind: if full_path.contains('*') {
                            ImportKind::Star
                        } else {
                            kind.clone()
                        },
                        line: node.start_position().row + 1,
                        is_type_only: false,
                    });
                }
            }
        }
    }

    imports
}

#[cfg(feature = "tree-sitter")]
fn extract_imports_python(root: Node, src: &str) -> Vec<ImportInfo> {
    let mut imports = Vec::new();
    let mut cursor = root.walk();

    for node in root.children(&mut cursor) {
        match node.kind() {
            "import_statement" => {
                let mut inner = node.walk();
                for child in node.children(&mut inner) {
                    if child.kind() == "dotted_name" || child.kind() == "aliased_import" {
                        let text = node_text(child, src);
                        let module = if child.kind() == "aliased_import" {
                            find_child_by_kind(child, "dotted_name")
                                .map(|n| node_text(n, src).to_string())
                                .unwrap_or_else(|| text.to_string())
                        } else {
                            text.to_string()
                        };
                        imports.push(ImportInfo {
                            source: module,
                            names: Vec::new(),
                            kind: ImportKind::Named,
                            line: node.start_position().row + 1,
                            is_type_only: false,
                        });
                    }
                }
            }
            "import_from_statement" => {
                let module = find_child_by_kind(node, "dotted_name")
                    .or_else(|| find_child_by_kind(node, "relative_import"))
                    .map(|n| node_text(n, src).to_string())
                    .unwrap_or_default();

                let mut names = Vec::new();
                let mut is_star = false;

                let mut inner = node.walk();
                for child in node.children(&mut inner) {
                    if child.kind() == "wildcard_import" {
                        is_star = true;
                    } else if child.kind() == "import_prefix" {
                        // relative import dots handled via module already
                    } else if child.kind() == "dotted_name"
                        && child.start_position() != node.start_position()
                    {
                        names.push(node_text(child, src).to_string());
                    } else if child.kind() == "aliased_import" {
                        if let Some(n) = find_child_by_kind(child, "dotted_name")
                            .or_else(|| find_child_by_kind(child, "identifier"))
                        {
                            names.push(node_text(n, src).to_string());
                        }
                    }
                }

                imports.push(ImportInfo {
                    source: module,
                    names,
                    kind: if is_star {
                        ImportKind::Star
                    } else {
                        ImportKind::Named
                    },
                    line: node.start_position().row + 1,
                    is_type_only: false,
                });
            }
            _ => {}
        }
    }

    imports
}

#[cfg(feature = "tree-sitter")]
fn extract_imports_go(root: Node, src: &str) -> Vec<ImportInfo> {
    let mut imports = Vec::new();
    let mut cursor = root.walk();

    for node in root.children(&mut cursor) {
        if node.kind() == "import_declaration" {
            let mut inner = node.walk();
            for child in node.children(&mut inner) {
                match child.kind() {
                    "import_spec" => {
                        if let Some(path_node) =
                            find_child_by_kind(child, "interpreted_string_literal")
                        {
                            let source = unquote(node_text(path_node, src));
                            let alias = find_child_by_kind(child, "package_identifier")
                                .or_else(|| find_child_by_kind(child, "dot"))
                                .or_else(|| find_child_by_kind(child, "blank_identifier"));
                            let kind = match alias.map(|a| node_text(a, src)) {
                                Some(".") => ImportKind::Star,
                                Some("_") => ImportKind::SideEffect,
                                _ => ImportKind::Named,
                            };
                            imports.push(ImportInfo {
                                source,
                                names: Vec::new(),
                                kind,
                                line: child.start_position().row + 1,
                                is_type_only: false,
                            });
                        }
                    }
                    "import_spec_list" => {
                        let mut spec_cursor = child.walk();
                        for spec in child.children(&mut spec_cursor) {
                            if spec.kind() == "import_spec" {
                                if let Some(path_node) =
                                    find_child_by_kind(spec, "interpreted_string_literal")
                                {
                                    let source = unquote(node_text(path_node, src));
                                    let alias = find_child_by_kind(spec, "package_identifier")
                                        .or_else(|| find_child_by_kind(spec, "dot"))
                                        .or_else(|| find_child_by_kind(spec, "blank_identifier"));
                                    let kind = match alias.map(|a| node_text(a, src)) {
                                        Some(".") => ImportKind::Star,
                                        Some("_") => ImportKind::SideEffect,
                                        _ => ImportKind::Named,
                                    };
                                    imports.push(ImportInfo {
                                        source,
                                        names: Vec::new(),
                                        kind,
                                        line: spec.start_position().row + 1,
                                        is_type_only: false,
                                    });
                                }
                            }
                        }
                    }
                    "interpreted_string_literal" => {
                        let source = unquote(node_text(child, src));
                        imports.push(ImportInfo {
                            source,
                            names: Vec::new(),
                            kind: ImportKind::Named,
                            line: child.start_position().row + 1,
                            is_type_only: false,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    imports
}

#[cfg(feature = "tree-sitter")]
fn extract_imports_java(root: Node, src: &str) -> Vec<ImportInfo> {
    let mut imports = Vec::new();
    let mut cursor = root.walk();

    for node in root.children(&mut cursor) {
        if node.kind() == "import_declaration" {
            let text = node_text(node, src).to_string();
            let _is_static = text.contains("static ");

            let path_node = find_child_by_kind(node, "scoped_identifier")
                .or_else(|| find_child_by_kind(node, "identifier"));
            if let Some(p) = path_node {
                let full_path = node_text(p, src).to_string();

                let is_wildcard = find_child_by_kind(node, "asterisk").is_some();
                let kind = if is_wildcard {
                    ImportKind::Star
                } else {
                    ImportKind::Named
                };

                let name = full_path
                    .rsplit('.')
                    .next()
                    .unwrap_or(&full_path)
                    .to_string();
                imports.push(ImportInfo {
                    source: full_path,
                    names: vec![name],
                    kind,
                    line: node.start_position().row + 1,
                    is_type_only: false,
                });
            }
        }
    }

    imports
}

// ---------------------------------------------------------------------------
// Call Sites
// ---------------------------------------------------------------------------

#[cfg(feature = "tree-sitter")]
fn extract_calls(root: Node, src: &str, ext: &str) -> Vec<CallSite> {
    let mut calls = Vec::new();
    walk_calls(root, src, ext, &mut calls);
    calls
}

#[cfg(feature = "tree-sitter")]
fn walk_calls(node: Node, src: &str, ext: &str, calls: &mut Vec<CallSite>) {
    if node.kind() == "call_expression" || node.kind() == "method_invocation" {
        if let Some(call) = parse_call(node, src, ext) {
            calls.push(call);
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_calls(child, src, ext, calls);
    }
}

#[cfg(feature = "tree-sitter")]
fn parse_call(node: Node, src: &str, ext: &str) -> Option<CallSite> {
    match ext {
        "ts" | "tsx" | "js" | "jsx" => parse_call_ts(node, src),
        "rs" => parse_call_rust(node, src),
        "py" => parse_call_python(node, src),
        "go" => parse_call_go(node, src),
        "java" => parse_call_java(node, src),
        _ => None,
    }
}

#[cfg(feature = "tree-sitter")]
fn parse_call_ts(node: Node, src: &str) -> Option<CallSite> {
    let func = find_child_by_kind(node, "member_expression")
        .or_else(|| find_child_by_kind(node, "identifier"))
        .or_else(|| find_child_by_kind(node, "subscript_expression"))?;

    if func.kind() == "member_expression" {
        let obj =
            find_child_by_kind(func, "identifier").or_else(|| find_child_by_kind(func, "this"))?;
        let prop = find_child_by_kind(func, "property_identifier")?;
        Some(CallSite {
            callee: node_text(prop, src).to_string(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            receiver: Some(node_text(obj, src).to_string()),
            is_method: true,
        })
    } else {
        Some(CallSite {
            callee: node_text(func, src).to_string(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            receiver: None,
            is_method: false,
        })
    }
}

#[cfg(feature = "tree-sitter")]
fn parse_call_rust(node: Node, src: &str) -> Option<CallSite> {
    let func = node.child(0)?;
    match func.kind() {
        "field_expression" => {
            let field = find_child_by_kind(func, "field_identifier")?;
            let receiver = func.child(0).map(|r| node_text(r, src).to_string());
            Some(CallSite {
                callee: node_text(field, src).to_string(),
                line: node.start_position().row + 1,
                col: node.start_position().column,
                receiver,
                is_method: true,
            })
        }
        "scoped_identifier" | "identifier" => Some(CallSite {
            callee: node_text(func, src).to_string(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            receiver: None,
            is_method: false,
        }),
        _ => None,
    }
}

#[cfg(feature = "tree-sitter")]
fn parse_call_python(node: Node, src: &str) -> Option<CallSite> {
    let func = node.child(0)?;
    match func.kind() {
        "attribute" => {
            let attr = find_child_by_kind(func, "identifier");
            let obj = func.child(0).map(|r| node_text(r, src).to_string());
            let name = attr
                .map(|a| node_text(a, src).to_string())
                .or_else(|| {
                    let text = node_text(func, src);
                    text.rsplit('.').next().map(|s| s.to_string())
                })
                .unwrap_or_default();
            Some(CallSite {
                callee: name,
                line: node.start_position().row + 1,
                col: node.start_position().column,
                receiver: obj,
                is_method: true,
            })
        }
        "identifier" => Some(CallSite {
            callee: node_text(func, src).to_string(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            receiver: None,
            is_method: false,
        }),
        _ => None,
    }
}

#[cfg(feature = "tree-sitter")]
fn parse_call_go(node: Node, src: &str) -> Option<CallSite> {
    let func = node.child(0)?;
    match func.kind() {
        "selector_expression" => {
            let field = find_child_by_kind(func, "field_identifier")?;
            let obj = func.child(0).map(|r| node_text(r, src).to_string());
            Some(CallSite {
                callee: node_text(field, src).to_string(),
                line: node.start_position().row + 1,
                col: node.start_position().column,
                receiver: obj,
                is_method: true,
            })
        }
        "identifier" => Some(CallSite {
            callee: node_text(func, src).to_string(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            receiver: None,
            is_method: false,
        }),
        _ => None,
    }
}

#[cfg(feature = "tree-sitter")]
fn parse_call_java(node: Node, src: &str) -> Option<CallSite> {
    if node.kind() == "method_invocation" {
        let name = find_child_by_kind(node, "identifier")?;
        let obj = find_child_by_kind(node, "field_access")
            .or_else(|| {
                let first = node.child(0)?;
                if first.kind() == "identifier" && first.id() != name.id() {
                    Some(first)
                } else {
                    None
                }
            })
            .map(|o| node_text(o, src).to_string());
        return Some(CallSite {
            callee: node_text(name, src).to_string(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            receiver: obj,
            is_method: true,
        });
    }

    let func = node.child(0)?;
    Some(CallSite {
        callee: node_text(func, src).to_string(),
        line: node.start_position().row + 1,
        col: node.start_position().column,
        receiver: None,
        is_method: false,
    })
}

// ---------------------------------------------------------------------------
// Type Definitions
// ---------------------------------------------------------------------------

#[cfg(feature = "tree-sitter")]
fn extract_types(root: Node, src: &str, ext: &str) -> Vec<TypeDef> {
    let mut types = Vec::new();
    walk_types(root, src, ext, &mut types, false);
    types
}

#[cfg(feature = "tree-sitter")]
fn walk_types(node: Node, src: &str, ext: &str, types: &mut Vec<TypeDef>, parent_exported: bool) {
    let exported = parent_exported || is_exported_node(node, src, ext);

    if let Some(td) = match_type_def(node, src, ext, exported) {
        types.push(td);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_types(child, src, ext, types, exported);
    }
}

#[cfg(feature = "tree-sitter")]
fn match_type_def(node: Node, src: &str, ext: &str, parent_exported: bool) -> Option<TypeDef> {
    let (name, kind) = match ext {
        "ts" | "tsx" | "js" | "jsx" => match_type_def_ts(node, src)?,
        "rs" => match_type_def_rust(node, src)?,
        "py" => match_type_def_python(node, src)?,
        "go" => match_type_def_go(node, src)?,
        "java" => match_type_def_java(node, src)?,
        _ => return None,
    };

    let is_exported = parent_exported || is_exported_node(node, src, ext);
    let generics = extract_generics(node, src);

    Some(TypeDef {
        name,
        kind,
        line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
        is_exported,
        generics,
    })
}

#[cfg(feature = "tree-sitter")]
fn match_type_def_ts(node: Node, src: &str) -> Option<(String, TypeDefKind)> {
    match node.kind() {
        "class_declaration" | "abstract_class_declaration" => {
            let name = find_child_by_kind(node, "type_identifier")
                .or_else(|| find_child_by_kind(node, "identifier"))?;
            Some((node_text(name, src).to_string(), TypeDefKind::Class))
        }
        "interface_declaration" => {
            let name = find_child_by_kind(node, "type_identifier")?;
            Some((node_text(name, src).to_string(), TypeDefKind::Interface))
        }
        "type_alias_declaration" => {
            let name = find_child_by_kind(node, "type_identifier")?;
            let text = node_text(node, src);
            let kind = if text.contains(" | ") {
                TypeDefKind::Union
            } else {
                TypeDefKind::TypeAlias
            };
            Some((node_text(name, src).to_string(), kind))
        }
        "enum_declaration" => {
            let name = find_child_by_kind(node, "identifier")?;
            Some((node_text(name, src).to_string(), TypeDefKind::Enum))
        }
        _ => None,
    }
}

#[cfg(feature = "tree-sitter")]
fn match_type_def_rust(node: Node, src: &str) -> Option<(String, TypeDefKind)> {
    match node.kind() {
        "struct_item" => {
            let name = find_child_by_kind(node, "type_identifier")?;
            Some((node_text(name, src).to_string(), TypeDefKind::Struct))
        }
        "enum_item" => {
            let name = find_child_by_kind(node, "type_identifier")?;
            Some((node_text(name, src).to_string(), TypeDefKind::Enum))
        }
        "trait_item" => {
            let name = find_child_by_kind(node, "type_identifier")?;
            Some((node_text(name, src).to_string(), TypeDefKind::Trait))
        }
        "type_item" => {
            let name = find_child_by_kind(node, "type_identifier")?;
            Some((node_text(name, src).to_string(), TypeDefKind::TypeAlias))
        }
        _ => None,
    }
}

#[cfg(feature = "tree-sitter")]
fn match_type_def_python(node: Node, src: &str) -> Option<(String, TypeDefKind)> {
    if node.kind() == "class_definition" {
        let name = find_child_by_kind(node, "identifier")?;
        let text = node_text(node, src);
        let kind = if text.contains("Protocol") {
            TypeDefKind::Protocol
        } else if text.contains("TypedDict") || text.contains("@dataclass") {
            TypeDefKind::Struct
        } else if text.contains("Enum") {
            TypeDefKind::Enum
        } else {
            TypeDefKind::Class
        };
        Some((node_text(name, src).to_string(), kind))
    } else {
        None
    }
}

#[cfg(feature = "tree-sitter")]
fn match_type_def_go(node: Node, src: &str) -> Option<(String, TypeDefKind)> {
    if node.kind() == "type_spec" {
        let name = find_child_by_kind(node, "type_identifier")?;
        let count = node.child_count();
        let type_body = node.child((count.saturating_sub(1)) as u32)?;
        let kind = match type_body.kind() {
            "struct_type" => TypeDefKind::Struct,
            "interface_type" => TypeDefKind::Interface,
            _ => TypeDefKind::TypeAlias,
        };
        Some((node_text(name, src).to_string(), kind))
    } else {
        None
    }
}

#[cfg(feature = "tree-sitter")]
fn match_type_def_java(node: Node, src: &str) -> Option<(String, TypeDefKind)> {
    match node.kind() {
        "class_declaration" => {
            let name = find_child_by_kind(node, "identifier")?;
            Some((node_text(name, src).to_string(), TypeDefKind::Class))
        }
        "interface_declaration" => {
            let name = find_child_by_kind(node, "identifier")?;
            Some((node_text(name, src).to_string(), TypeDefKind::Interface))
        }
        "enum_declaration" => {
            let name = find_child_by_kind(node, "identifier")?;
            Some((node_text(name, src).to_string(), TypeDefKind::Enum))
        }
        "record_declaration" => {
            let name = find_child_by_kind(node, "identifier")?;
            Some((node_text(name, src).to_string(), TypeDefKind::Record))
        }
        "annotation_type_declaration" => {
            let name = find_child_by_kind(node, "identifier")?;
            Some((node_text(name, src).to_string(), TypeDefKind::Annotation))
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

#[cfg(feature = "tree-sitter")]
fn extract_exports(root: Node, src: &str, ext: &str) -> Vec<String> {
    let mut exports = Vec::new();
    walk_exports(root, src, ext, &mut exports);
    exports
}

#[cfg(feature = "tree-sitter")]
fn walk_exports(node: Node, src: &str, ext: &str, exports: &mut Vec<String>) {
    if is_exported_node(node, src, ext) {
        if let Some(name) = get_declaration_name(node, src) {
            exports.push(name);
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_exports(child, src, ext, exports);
    }
}

#[cfg(feature = "tree-sitter")]
fn is_exported_node(node: Node, src: &str, ext: &str) -> bool {
    match ext {
        "ts" | "tsx" | "js" | "jsx" => {
            node.kind() == "export_statement"
                || node
                    .parent()
                    .is_some_and(|p| p.kind() == "export_statement")
        }
        "rs" => node_text(node, src).trim_start().starts_with("pub "),
        "go" => {
            if let Some(name) = get_declaration_name(node, src) {
                name.starts_with(char::is_uppercase)
            } else {
                false
            }
        }
        "java" => node_text(node, src).trim_start().starts_with("public "),
        "py" => {
            if let Some(name) = get_declaration_name(node, src) {
                !name.starts_with('_')
            } else {
                false
            }
        }
        _ => false,
    }
}

#[cfg(feature = "tree-sitter")]
fn get_declaration_name(node: Node, src: &str) -> Option<String> {
    for kind in &[
        "identifier",
        "type_identifier",
        "property_identifier",
        "field_identifier",
    ] {
        if let Some(name_node) = find_child_by_kind(node, kind) {
            return Some(node_text(name_node, src).to_string());
        }
    }
    None
}

#[cfg(feature = "tree-sitter")]
fn extract_generics(node: Node, src: &str) -> Vec<String> {
    let tp = find_child_by_kind(node, "type_parameters")
        .or_else(|| find_child_by_kind(node, "type_parameter_list"));
    match tp {
        Some(params) => {
            let mut result = Vec::new();
            let mut cursor = params.walk();
            for child in params.children(&mut cursor) {
                if child.kind() == "type_parameter"
                    || child.kind() == "type_identifier"
                    || child.kind() == "identifier"
                {
                    result.push(node_text(child, src).to_string());
                }
            }
            result
        }
        None => Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[cfg(feature = "tree-sitter")]
fn node_text<'a>(node: Node, src: &'a str) -> &'a str {
    &src[node.byte_range()]
}

#[cfg(feature = "tree-sitter")]
fn find_child_by_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    let result = node.children(&mut cursor).find(|c| c.kind() == kind);
    result
}

#[cfg(feature = "tree-sitter")]
fn find_descendant_by_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    if node.kind() == kind {
        return Some(node);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_descendant_by_kind(child, kind) {
            return Some(found);
        }
    }
    None
}

#[cfg(feature = "tree-sitter")]
fn collect_named_imports(node: Node, src: &str) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(named) = find_descendant_by_kind(node, "named_imports") {
        let mut cursor = named.walk();
        for child in named.children(&mut cursor) {
            if child.kind() == "import_specifier" || child.kind() == "export_specifier" {
                if let Some(id) = find_child_by_kind(child, "identifier") {
                    names.push(node_text(id, src).to_string());
                }
            }
        }
    }
    names
}

fn unquote(s: &str) -> String {
    s.trim_matches(|c| c == '\'' || c == '"' || c == '`')
        .to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[cfg(feature = "tree-sitter")]
mod tests {
    use super::*;

    #[test]
    fn ts_named_import() {
        let src = r#"import { useState, useEffect } from 'react';"#;
        let analysis = analyze(src, "ts");
        assert_eq!(analysis.imports.len(), 1);
        assert_eq!(analysis.imports[0].source, "react");
        assert_eq!(analysis.imports[0].names, vec!["useState", "useEffect"]);
    }

    #[test]
    fn ts_default_import() {
        let src = r#"import React from 'react';"#;
        let analysis = analyze(src, "ts");
        assert_eq!(analysis.imports.len(), 1);
        assert_eq!(analysis.imports[0].kind, ImportKind::Default);
        assert_eq!(analysis.imports[0].names, vec!["React"]);
    }

    #[test]
    fn ts_star_import() {
        let src = r#"import * as path from 'path';"#;
        let analysis = analyze(src, "ts");
        assert_eq!(analysis.imports.len(), 1);
        assert_eq!(analysis.imports[0].kind, ImportKind::Star);
    }

    #[test]
    fn ts_side_effect_import() {
        let src = r#"import './styles.css';"#;
        let analysis = analyze(src, "ts");
        assert_eq!(analysis.imports.len(), 1);
        assert_eq!(analysis.imports[0].kind, ImportKind::SideEffect);
        assert_eq!(analysis.imports[0].source, "./styles.css");
    }

    #[test]
    fn ts_type_only_import() {
        let src = r#"import type { User } from './types';"#;
        let analysis = analyze(src, "ts");
        assert_eq!(analysis.imports.len(), 1);
        assert!(analysis.imports[0].is_type_only);
    }

    #[test]
    fn ts_reexport() {
        let src = r#"export { foo, bar } from './utils';"#;
        let analysis = analyze(src, "ts");
        assert_eq!(analysis.imports.len(), 1);
        assert_eq!(analysis.imports[0].kind, ImportKind::Reexport);
    }

    #[test]
    fn ts_call_sites() {
        let src = r#"
const x = foo(1);
const y = obj.method(2);
"#;
        let analysis = analyze(src, "ts");
        assert!(analysis.calls.len() >= 2);
        let fns: Vec<&str> = analysis.calls.iter().map(|c| c.callee.as_str()).collect();
        assert!(fns.contains(&"foo"));
        assert!(fns.contains(&"method"));
    }

    #[test]
    fn ts_interface() {
        let src = r#"
export interface User {
    name: string;
    age: number;
}
"#;
        let analysis = analyze(src, "ts");
        assert_eq!(analysis.types.len(), 1);
        assert_eq!(analysis.types[0].name, "User");
        assert_eq!(analysis.types[0].kind, TypeDefKind::Interface);
    }

    #[test]
    fn ts_type_alias_union() {
        let src = r#"type Result = Success | Error;"#;
        let analysis = analyze(src, "ts");
        assert_eq!(analysis.types.len(), 1);
        assert_eq!(analysis.types[0].kind, TypeDefKind::Union);
    }

    #[test]
    fn rust_use_statements() {
        let src = r#"
use crate::core::session;
use anyhow::Result;
use std::collections::HashMap;
"#;
        let analysis = analyze(src, "rs");
        assert_eq!(analysis.imports.len(), 2);
        let sources: Vec<&str> = analysis.imports.iter().map(|i| i.source.as_str()).collect();
        assert!(sources.contains(&"crate::core::session"));
        assert!(sources.contains(&"anyhow::Result"));
    }

    #[test]
    fn rust_pub_use_reexport() {
        let src = r#"pub use crate::tools::ctx_read;"#;
        let analysis = analyze(src, "rs");
        assert_eq!(analysis.imports.len(), 1);
        assert_eq!(analysis.imports[0].kind, ImportKind::Reexport);
    }

    #[test]
    fn rust_struct_and_trait() {
        let src = r#"
pub struct Config {
    pub name: String,
}

pub trait Service {
    fn run(&self);
}
"#;
        let analysis = analyze(src, "rs");
        assert_eq!(analysis.types.len(), 2);
        let names: Vec<&str> = analysis.types.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"Config"));
        assert!(names.contains(&"Service"));
    }

    #[test]
    fn rust_call_sites() {
        let src = r#"
fn main() {
    let x = calculate(42);
    let y = self.process();
    Vec::new();
}
"#;
        let analysis = analyze(src, "rs");
        assert!(analysis.calls.len() >= 2);
        let fns: Vec<&str> = analysis.calls.iter().map(|c| c.callee.as_str()).collect();
        assert!(fns.contains(&"calculate"));
    }

    #[test]
    fn python_imports() {
        let src = r#"
import os
from pathlib import Path
from . import utils
from ..models import User, Role
"#;
        let analysis = analyze(src, "py");
        assert!(analysis.imports.len() >= 3);
    }

    #[test]
    fn python_class_protocol() {
        let src = r#"
class MyProtocol(Protocol):
    def method(self) -> None: ...

class User:
    name: str
"#;
        let analysis = analyze(src, "py");
        assert_eq!(analysis.types.len(), 2);
        assert_eq!(analysis.types[0].kind, TypeDefKind::Protocol);
        assert_eq!(analysis.types[1].kind, TypeDefKind::Class);
    }

    #[test]
    fn go_imports() {
        let src = r#"
package main

import (
    "fmt"
    "net/http"
    _ "github.com/lib/pq"
)
"#;
        let analysis = analyze(src, "go");
        assert!(analysis.imports.len() >= 3);
        let side_effect = analysis.imports.iter().find(|i| i.source.contains("pq"));
        assert!(side_effect.is_some());
        assert_eq!(side_effect.unwrap().kind, ImportKind::SideEffect);
    }

    #[test]
    fn go_struct_and_interface() {
        let src = r#"
package main

type Server struct {
    Port int
}

type Handler interface {
    Handle(r *Request)
}
"#;
        let analysis = analyze(src, "go");
        assert_eq!(analysis.types.len(), 2);
        let kinds: Vec<&TypeDefKind> = analysis.types.iter().map(|t| &t.kind).collect();
        assert!(kinds.contains(&&TypeDefKind::Struct));
        assert!(kinds.contains(&&TypeDefKind::Interface));
    }

    #[test]
    fn java_imports() {
        let src = r#"
import java.util.List;
import java.util.Map;
import static org.junit.Assert.*;
"#;
        let analysis = analyze(src, "java");
        assert!(analysis.imports.len() >= 2);
    }

    #[test]
    fn java_class_and_interface() {
        let src = r#"
public class UserService {
    public void save(User u) {}
}

public interface Repository<T> {
    T findById(int id);
}

public enum Status { ACTIVE, INACTIVE }

public record Point(int x, int y) {}
"#;
        let analysis = analyze(src, "java");
        assert!(analysis.types.len() >= 3);
        let kinds: Vec<&TypeDefKind> = analysis.types.iter().map(|t| &t.kind).collect();
        assert!(kinds.contains(&&TypeDefKind::Class));
        assert!(kinds.contains(&&TypeDefKind::Interface));
        assert!(kinds.contains(&&TypeDefKind::Enum));
    }

    #[test]
    fn ts_generics_extracted() {
        let src = r#"interface Result<T, E> { ok: T; err: E; }"#;
        let analysis = analyze(src, "ts");
        assert_eq!(analysis.types.len(), 1);
        assert!(!analysis.types[0].generics.is_empty());
    }

    #[test]
    fn mixed_analysis_ts() {
        let src = r#"
import { Request, Response } from 'express';
import type { User } from './models';

export interface Handler {
    handle(req: Request): Response;
}

export class Router {
    register(path: string, handler: Handler) {
        this.handlers.set(path, handler);
    }
}

const app = express();
app.listen(3000);
"#;
        let analysis = analyze(src, "ts");
        assert!(analysis.imports.len() >= 2, "Should find imports");
        assert!(!analysis.types.is_empty(), "Should find types");
        assert!(!analysis.calls.is_empty(), "Should find calls");
    }

    #[test]
    fn empty_file() {
        let analysis = analyze("", "ts");
        assert!(analysis.imports.is_empty());
        assert!(analysis.calls.is_empty());
        assert!(analysis.types.is_empty());
    }

    #[test]
    fn unsupported_extension() {
        let analysis = analyze("some content", "txt");
        assert!(analysis.imports.is_empty());
    }
}
