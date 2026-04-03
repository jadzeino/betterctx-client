use crate::core::knowledge::ProjectKnowledge;
use crate::core::session::SessionState;

#[allow(clippy::too_many_arguments)]
pub fn handle(
    project_root: &str,
    action: &str,
    category: Option<&str>,
    key: Option<&str>,
    value: Option<&str>,
    query: Option<&str>,
    session_id: &str,
    pattern_type: Option<&str>,
    examples: Option<Vec<String>>,
    confidence: Option<f32>,
) -> String {
    match action {
        "remember" => {
            let cat = match category {
                Some(c) => c,
                None => return "Error: category is required for remember".to_string(),
            };
            let k = match key {
                Some(k) => k,
                None => return "Error: key is required for remember".to_string(),
            };
            let v = match value {
                Some(v) => v,
                None => return "Error: value is required for remember".to_string(),
            };
            let conf = confidence.unwrap_or(0.8);
            let mut knowledge = ProjectKnowledge::load_or_create(project_root);
            knowledge.remember(cat, k, v, session_id, conf);
            match knowledge.save() {
                Ok(()) => format!("Remembered [{cat}] {k}: {v} (confidence: {:.0}%)", conf * 100.0),
                Err(e) => format!("Remembered but save failed: {e}"),
            }
        }

        "recall" => {
            let knowledge = match ProjectKnowledge::load(project_root) {
                Some(k) => k,
                None => return "No knowledge stored for this project yet.".to_string(),
            };

            if let Some(cat) = category {
                let facts = knowledge.recall_by_category(cat);
                if facts.is_empty() {
                    return format!("No facts in category '{cat}'.");
                }
                return format_facts(&facts, Some(cat));
            }

            if let Some(q) = query {
                let facts = knowledge.recall(q);
                if facts.is_empty() {
                    return format!("No facts matching '{q}'.");
                }
                return format_facts(&facts, None);
            }

            "Error: provide query or category for recall".to_string()
        }

        "pattern" => {
            let pt = match pattern_type {
                Some(p) => p,
                None => return "Error: pattern_type is required".to_string(),
            };
            let desc = match value {
                Some(v) => v,
                None => return "Error: value (description) is required for pattern".to_string(),
            };
            let exs = examples.unwrap_or_default();
            let mut knowledge = ProjectKnowledge::load_or_create(project_root);
            knowledge.add_pattern(pt, desc, exs, session_id);
            match knowledge.save() {
                Ok(()) => format!("Pattern [{pt}] added: {desc}"),
                Err(e) => format!("Pattern added but save failed: {e}"),
            }
        }

        "status" => {
            let knowledge = match ProjectKnowledge::load(project_root) {
                Some(k) => k,
                None => return "No knowledge stored for this project yet. Use ctx_knowledge(action=\"remember\") to start.".to_string(),
            };

            let mut out = format!(
                "Project Knowledge: {} facts, {} patterns, {} history entries\n",
                knowledge.facts.len(),
                knowledge.patterns.len(),
                knowledge.history.len()
            );
            out.push_str(&format!("Last updated: {}\n", knowledge.updated_at.format("%Y-%m-%d %H:%M UTC")));
            out.push_str(&knowledge.format_summary());
            out
        }

        "remove" => {
            let cat = match category {
                Some(c) => c,
                None => return "Error: category is required for remove".to_string(),
            };
            let k = match key {
                Some(k) => k,
                None => return "Error: key is required for remove".to_string(),
            };
            let mut knowledge = ProjectKnowledge::load_or_create(project_root);
            if knowledge.remove_fact(cat, k) {
                match knowledge.save() {
                    Ok(()) => format!("Removed [{cat}] {k}"),
                    Err(e) => format!("Removed but save failed: {e}"),
                }
            } else {
                format!("No fact found: [{cat}] {k}")
            }
        }

        "export" => {
            let knowledge = match ProjectKnowledge::load(project_root) {
                Some(k) => k,
                None => return "No knowledge to export.".to_string(),
            };
            match serde_json::to_string_pretty(&knowledge) {
                Ok(json) => json,
                Err(e) => format!("Export failed: {e}"),
            }
        }

        "consolidate" => {
            let session = match SessionState::load_latest() {
                Some(s) => s,
                None => return "No active session to consolidate.".to_string(),
            };

            let mut knowledge = ProjectKnowledge::load_or_create(project_root);
            let mut consolidated = 0u32;

            for finding in &session.findings {
                let key_text = if let Some(ref file) = finding.file {
                    if let Some(line) = finding.line {
                        format!("{file}:{line}")
                    } else {
                        file.clone()
                    }
                } else {
                    format!("finding-{consolidated}")
                };

                knowledge.remember(
                    "finding",
                    &key_text,
                    &finding.summary,
                    &session.id,
                    0.7,
                );
                consolidated += 1;
            }

            for decision in &session.decisions {
                let key_text = decision
                    .summary
                    .chars()
                    .take(50)
                    .collect::<String>()
                    .replace(' ', "-")
                    .to_lowercase();

                knowledge.remember(
                    "decision",
                    &key_text,
                    &decision.summary,
                    &session.id,
                    0.85,
                );
                consolidated += 1;
            }

            let task_desc = session
                .task
                .as_ref()
                .map(|t| t.description.clone())
                .unwrap_or_else(|| "(no task)".into());

            let summary = format!(
                "Session {}: {} — {} findings, {} decisions consolidated",
                session.id,
                task_desc,
                session.findings.len(),
                session.decisions.len()
            );
            knowledge.consolidate(&summary, vec![session.id.clone()]);

            match knowledge.save() {
                Ok(()) => format!(
                    "Consolidated {consolidated} items from session {} into project knowledge.\n\
                     Facts: {}, Patterns: {}, History: {}",
                    session.id,
                    knowledge.facts.len(),
                    knowledge.patterns.len(),
                    knowledge.history.len()
                ),
                Err(e) => format!("Consolidation done but save failed: {e}"),
            }
        }

        _ => format!(
            "Unknown action: {action}. Use: remember, recall, pattern, status, remove, export, consolidate"
        ),
    }
}

fn format_facts(
    facts: &[&crate::core::knowledge::KnowledgeFact],
    category: Option<&str>,
) -> String {
    let mut out = String::new();
    if let Some(cat) = category {
        out.push_str(&format!("Facts [{cat}] ({}):\n", facts.len()));
    } else {
        out.push_str(&format!("Matching facts ({}):\n", facts.len()));
    }
    for f in facts {
        out.push_str(&format!(
            "  [{}/{}]: {} (confidence: {:.0}%, confirmed: {})\n",
            f.category,
            f.key,
            f.value,
            f.confidence * 100.0,
            f.last_confirmed.format("%Y-%m-%d")
        ));
    }
    out
}
