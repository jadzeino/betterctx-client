//! Graph-driven context loading — automatically includes related files
//! based on Property Graph proximity and token budgeting.
//!
//! Used by `ctx_read` in `graph` mode to load a file plus its most
//! relevant dependencies, staying within a token budget.

use std::collections::HashSet;
use std::path::Path;

use super::property_graph::CodeGraph;
use super::tokens::count_tokens;

#[derive(Debug)]
pub struct GraphContext {
    pub primary_file: String,
    pub related_files: Vec<RelatedFile>,
    pub total_tokens: usize,
    pub budget_remaining: usize,
}

#[derive(Debug)]
pub struct RelatedFile {
    pub path: String,
    pub relationship: Relationship,
    pub token_count: usize,
}

#[derive(Debug, Clone)]
pub enum Relationship {
    DirectDependency,
    DirectDependent,
    TransitiveDependency,
    TypeProvider,
}

impl Relationship {
    pub fn label(&self) -> &'static str {
        match self {
            Relationship::DirectDependency => "imports",
            Relationship::DirectDependent => "imported-by",
            Relationship::TransitiveDependency => "transitive-dep",
            Relationship::TypeProvider => "type-provider",
        }
    }

    fn priority(&self) -> usize {
        match self {
            Relationship::DirectDependency => 0,
            Relationship::TypeProvider => 1,
            Relationship::DirectDependent => 2,
            Relationship::TransitiveDependency => 3,
        }
    }
}

const DEFAULT_TOKEN_BUDGET: usize = 8000;

pub fn build_graph_context(
    file_path: &str,
    project_root: &str,
    token_budget: Option<usize>,
) -> Option<GraphContext> {
    let graph = CodeGraph::open(Path::new(project_root)).ok()?;
    let node_count = graph.node_count().ok()?;
    if node_count == 0 {
        return None;
    }

    let budget = token_budget.unwrap_or(DEFAULT_TOKEN_BUDGET);

    let rel_path = file_path
        .strip_prefix(project_root)
        .unwrap_or(file_path)
        .trim_start_matches('/');

    let primary_content = std::fs::read_to_string(file_path).ok()?;
    let primary_tokens = count_tokens(&primary_content);

    let remaining = budget.saturating_sub(primary_tokens);
    if remaining < 200 {
        return Some(GraphContext {
            primary_file: rel_path.to_string(),
            related_files: Vec::new(),
            total_tokens: primary_tokens,
            budget_remaining: 0,
        });
    }

    let mut candidates = collect_candidates(&graph, rel_path);
    candidates.sort_by_key(|c| c.relationship.priority());

    let mut related: Vec<RelatedFile> = Vec::new();
    let mut tokens_used = primary_tokens;
    let mut seen: HashSet<String> = HashSet::new();
    seen.insert(rel_path.to_string());

    for candidate in candidates {
        if seen.contains(&candidate.path) {
            continue;
        }

        let abs_path = format!("{project_root}/{}", candidate.path);
        if let Ok(content) = std::fs::read_to_string(&abs_path) {
            let tokens = count_tokens(&content);
            if tokens_used + tokens > budget {
                continue;
            }
            tokens_used += tokens;
            seen.insert(candidate.path.clone());
            related.push(RelatedFile {
                path: candidate.path,
                relationship: candidate.relationship,
                token_count: tokens,
            });
        }
    }

    Some(GraphContext {
        primary_file: rel_path.to_string(),
        related_files: related,
        total_tokens: tokens_used,
        budget_remaining: budget.saturating_sub(tokens_used),
    })
}

struct Candidate {
    path: String,
    relationship: Relationship,
}

fn collect_candidates(graph: &CodeGraph, file_path: &str) -> Vec<Candidate> {
    let mut candidates: Vec<Candidate> = Vec::new();

    if let Ok(deps) = graph.dependencies(file_path) {
        for dep in deps {
            candidates.push(Candidate {
                path: dep,
                relationship: Relationship::DirectDependency,
            });
        }
    }

    if let Ok(dependents) = graph.dependents(file_path) {
        for dep in dependents {
            candidates.push(Candidate {
                path: dep,
                relationship: Relationship::DirectDependent,
            });
        }
    }

    if let Ok(impact) = graph.impact_analysis(file_path, 2) {
        for affected in impact.affected_files {
            let already = candidates.iter().any(|c| c.path == affected);
            if !already {
                candidates.push(Candidate {
                    path: affected,
                    relationship: Relationship::TransitiveDependency,
                });
            }
        }
    }

    candidates
}

pub fn format_graph_context(ctx: &GraphContext) -> String {
    if ctx.related_files.is_empty() {
        return String::new();
    }

    let mut result = format!(
        "\n--- GRAPH CONTEXT ({} related files, {} tok) ---\n",
        ctx.related_files.len(),
        ctx.total_tokens
    );

    for rf in &ctx.related_files {
        result.push_str(&format!(
            "  {} [{}] ({} tok)\n",
            rf.path,
            rf.relationship.label(),
            rf.token_count
        ));
    }

    result.push_str("--- END GRAPH CONTEXT ---");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relationship_priorities() {
        assert!(
            Relationship::DirectDependency.priority() < Relationship::DirectDependent.priority()
        );
        assert!(
            Relationship::DirectDependent.priority()
                < Relationship::TransitiveDependency.priority()
        );
    }

    #[test]
    fn relationship_labels() {
        assert_eq!(Relationship::DirectDependency.label(), "imports");
        assert_eq!(Relationship::DirectDependent.label(), "imported-by");
        assert_eq!(Relationship::TransitiveDependency.label(), "transitive-dep");
        assert_eq!(Relationship::TypeProvider.label(), "type-provider");
    }

    #[test]
    fn format_empty_context() {
        let ctx = GraphContext {
            primary_file: "main.rs".to_string(),
            related_files: vec![],
            total_tokens: 100,
            budget_remaining: 7900,
        };
        assert!(format_graph_context(&ctx).is_empty());
    }

    #[test]
    fn format_with_related() {
        let ctx = GraphContext {
            primary_file: "main.rs".to_string(),
            related_files: vec![
                RelatedFile {
                    path: "lib.rs".to_string(),
                    relationship: Relationship::DirectDependency,
                    token_count: 500,
                },
                RelatedFile {
                    path: "utils.rs".to_string(),
                    relationship: Relationship::DirectDependent,
                    token_count: 300,
                },
            ],
            total_tokens: 900,
            budget_remaining: 7100,
        };
        let output = format_graph_context(&ctx);
        assert!(output.contains("2 related files"));
        assert!(output.contains("lib.rs [imports]"));
        assert!(output.contains("utils.rs [imported-by]"));
    }

    #[test]
    fn nonexistent_root_returns_none() {
        let result = build_graph_context("/nonexistent/file.rs", "/nonexistent", None);
        assert!(result.is_none());
    }
}
