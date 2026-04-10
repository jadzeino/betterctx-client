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
        "remember" => handle_remember(project_root, category, key, value, session_id, confidence),
        "recall" => handle_recall(project_root, category, query),
        "pattern" => handle_pattern(project_root, pattern_type, value, examples, session_id),
        "status" => handle_status(project_root),
        "remove" => handle_remove(project_root, category, key),
        "export" => handle_export(project_root),
        "consolidate" => handle_consolidate(project_root),
        "timeline" => handle_timeline(project_root, category),
        "rooms" => handle_rooms(project_root),
        "search" => handle_search(query),
        "wakeup" => handle_wakeup(project_root),
        _ => format!(
            "Unknown action: {action}. Use: remember, recall, pattern, status, remove, export, consolidate, timeline, rooms, search, wakeup"
        ),
    }
}

fn handle_remember(
    project_root: &str,
    category: Option<&str>,
    key: Option<&str>,
    value: Option<&str>,
    session_id: &str,
    confidence: Option<f32>,
) -> String {
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
    let contradiction = knowledge.remember(cat, k, v, session_id, conf);

    let mut result = format!(
        "Remembered [{cat}] {k}: {v} (confidence: {:.0}%)",
        conf * 100.0
    );

    if let Some(c) = contradiction {
        result.push_str(&format!("\n⚠ CONTRADICTION DETECTED: {}", c.resolution));
    }

    match knowledge.save() {
        Ok(()) => result,
        Err(e) => format!("{result}\n(save failed: {e})"),
    }
}

fn handle_recall(project_root: &str, category: Option<&str>, query: Option<&str>) -> String {
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

fn handle_pattern(
    project_root: &str,
    pattern_type: Option<&str>,
    value: Option<&str>,
    examples: Option<Vec<String>>,
    session_id: &str,
) -> String {
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

fn handle_status(project_root: &str) -> String {
    let knowledge = match ProjectKnowledge::load(project_root) {
        Some(k) => k,
        None => {
            return "No knowledge stored for this project yet. Use ctx_knowledge(action=\"remember\") to start.".to_string();
        }
    };

    let current_facts = knowledge.facts.iter().filter(|f| f.is_current()).count();
    let archived_facts = knowledge.facts.len() - current_facts;

    let mut out = format!(
        "Project Knowledge: {} active facts ({} archived), {} patterns, {} history entries\n",
        current_facts,
        archived_facts,
        knowledge.patterns.len(),
        knowledge.history.len()
    );
    out.push_str(&format!(
        "Last updated: {}\n",
        knowledge.updated_at.format("%Y-%m-%d %H:%M UTC")
    ));

    let rooms = knowledge.list_rooms();
    if !rooms.is_empty() {
        out.push_str("Rooms: ");
        let room_strs: Vec<String> = rooms.iter().map(|(c, n)| format!("{c}({n})")).collect();
        out.push_str(&room_strs.join(", "));
        out.push('\n');
    }

    out.push_str(&knowledge.format_summary());
    out
}

fn handle_remove(project_root: &str, category: Option<&str>, key: Option<&str>) -> String {
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

fn handle_export(project_root: &str) -> String {
    let knowledge = match ProjectKnowledge::load(project_root) {
        Some(k) => k,
        None => return "No knowledge to export.".to_string(),
    };
    match serde_json::to_string_pretty(&knowledge) {
        Ok(json) => json,
        Err(e) => format!("Export failed: {e}"),
    }
}

fn handle_consolidate(project_root: &str) -> String {
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

        knowledge.remember("finding", &key_text, &finding.summary, &session.id, 0.7);
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

        knowledge.remember("decision", &key_text, &decision.summary, &session.id, 0.85);
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

fn handle_timeline(project_root: &str, category: Option<&str>) -> String {
    let knowledge = match ProjectKnowledge::load(project_root) {
        Some(k) => k,
        None => return "No knowledge stored yet.".to_string(),
    };

    let cat = match category {
        Some(c) => c,
        None => return "Error: category is required for timeline".to_string(),
    };

    let facts = knowledge.timeline(cat);
    if facts.is_empty() {
        return format!("No history for category '{cat}'.");
    }

    let mut out = format!("Timeline [{cat}] ({} entries):\n", facts.len());
    for f in &facts {
        let status = if f.is_current() {
            "CURRENT"
        } else {
            "archived"
        };
        let valid_range = match (f.valid_from, f.valid_until) {
            (Some(from), Some(until)) => format!(
                "{} → {}",
                from.format("%Y-%m-%d %H:%M"),
                until.format("%Y-%m-%d %H:%M")
            ),
            (Some(from), None) => format!("{} → now", from.format("%Y-%m-%d %H:%M")),
            _ => "unknown".to_string(),
        };
        out.push_str(&format!(
            "  {} = {} [{status}] ({valid_range}) conf={:.0}% x{}\n",
            f.key,
            f.value,
            f.confidence * 100.0,
            f.confirmation_count
        ));
    }
    out
}

fn handle_rooms(project_root: &str) -> String {
    let knowledge = match ProjectKnowledge::load(project_root) {
        Some(k) => k,
        None => return "No knowledge stored yet.".to_string(),
    };

    let rooms = knowledge.list_rooms();
    if rooms.is_empty() {
        return "No knowledge rooms yet. Use ctx_knowledge(action=\"remember\", category=\"...\") to create rooms.".to_string();
    }

    let mut out = format!(
        "Knowledge Rooms ({} rooms, project: {}):\n",
        rooms.len(),
        short_hash(&knowledge.project_hash)
    );
    for (cat, count) in &rooms {
        out.push_str(&format!("  [{cat}] {count} fact(s)\n"));
    }
    out
}

fn handle_search(query: Option<&str>) -> String {
    let q = match query {
        Some(q) => q,
        None => return "Error: query is required for search".to_string(),
    };

    let sessions_dir = match dirs::home_dir() {
        Some(h) => h.join(".better-ctx").join("sessions"),
        None => return "Cannot determine home directory.".to_string(),
    };

    if !sessions_dir.exists() {
        return "No sessions found.".to_string();
    }

    let knowledge_dir = match dirs::home_dir() {
        Some(h) => h.join(".better-ctx").join("knowledge"),
        None => return "Cannot determine home directory.".to_string(),
    };

    let q_lower = q.to_lowercase();
    let terms: Vec<&str> = q_lower.split_whitespace().collect();
    let mut results = Vec::new();

    if knowledge_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&knowledge_dir) {
            for entry in entries.flatten() {
                let knowledge_file = entry.path().join("knowledge.json");
                if let Ok(content) = std::fs::read_to_string(&knowledge_file) {
                    if let Ok(knowledge) = serde_json::from_str::<ProjectKnowledge>(&content) {
                        for fact in &knowledge.facts {
                            let searchable = format!(
                                "{} {} {}",
                                fact.category.to_lowercase(),
                                fact.key.to_lowercase(),
                                fact.value.to_lowercase()
                            );
                            let match_count =
                                terms.iter().filter(|t| searchable.contains(**t)).count();
                            if match_count > 0 {
                                results.push((
                                    knowledge.project_root.clone(),
                                    fact.category.clone(),
                                    fact.key.clone(),
                                    fact.value.clone(),
                                    fact.confidence,
                                    match_count as f32 / terms.len() as f32,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir(&sessions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if path.file_name().and_then(|n| n.to_str()) == Some("latest.json") {
                continue;
            }
            if let Ok(json) = std::fs::read_to_string(&path) {
                if let Ok(session) = serde_json::from_str::<SessionState>(&json) {
                    for finding in &session.findings {
                        let searchable = finding.summary.to_lowercase();
                        let match_count = terms.iter().filter(|t| searchable.contains(**t)).count();
                        if match_count > 0 {
                            let project = session
                                .project_root
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string());
                            results.push((
                                project,
                                "session-finding".to_string(),
                                session.id.clone(),
                                finding.summary.clone(),
                                0.6,
                                match_count as f32 / terms.len() as f32,
                            ));
                        }
                    }
                    for decision in &session.decisions {
                        let searchable = decision.summary.to_lowercase();
                        let match_count = terms.iter().filter(|t| searchable.contains(**t)).count();
                        if match_count > 0 {
                            let project = session
                                .project_root
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string());
                            results.push((
                                project,
                                "session-decision".to_string(),
                                session.id.clone(),
                                decision.summary.clone(),
                                0.7,
                                match_count as f32 / terms.len() as f32,
                            ));
                        }
                    }
                }
            }
        }
    }

    if results.is_empty() {
        return format!("No results found for '{q}' across all sessions and projects.");
    }

    results.sort_by(|a, b| b.5.partial_cmp(&a.5).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(20);

    let mut out = format!("Cross-session search '{q}' ({} results):\n", results.len());
    for (project, cat, key, value, conf, _relevance) in &results {
        let project_short = short_path(project);
        out.push_str(&format!(
            "  [{cat}/{key}] {value} (project: {project_short}, conf: {:.0}%)\n",
            conf * 100.0
        ));
    }
    out
}

fn handle_wakeup(project_root: &str) -> String {
    let knowledge = match ProjectKnowledge::load(project_root) {
        Some(k) => k,
        None => return "No knowledge for wake-up briefing.".to_string(),
    };
    let aaak = knowledge.format_aaak();
    if aaak.is_empty() {
        return "No knowledge yet. Start using ctx_knowledge(action=\"remember\") to build project memory.".to_string();
    }
    format!("WAKE-UP BRIEFING:\n{aaak}")
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
        let temporal = if !f.is_current() { " [archived]" } else { "" };
        out.push_str(&format!(
            "  [{}/{}]: {} (confidence: {:.0}%, confirmed: {} x{}){temporal}\n",
            f.category,
            f.key,
            f.value,
            f.confidence * 100.0,
            f.last_confirmed.format("%Y-%m-%d"),
            f.confirmation_count
        ));
    }
    out
}

fn short_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() <= 2 {
        return path.to_string();
    }
    parts[parts.len() - 2..].join("/")
}

fn short_hash(hash: &str) -> &str {
    if hash.len() > 8 {
        &hash[..8]
    } else {
        hash
    }
}
