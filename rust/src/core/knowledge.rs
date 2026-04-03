use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const MAX_FACTS: usize = 200;
const MAX_PATTERNS: usize = 50;
const MAX_HISTORY: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectKnowledge {
    pub project_root: String,
    pub project_hash: String,
    pub facts: Vec<KnowledgeFact>,
    pub patterns: Vec<ProjectPattern>,
    pub history: Vec<ConsolidatedInsight>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeFact {
    pub category: String,
    pub key: String,
    pub value: String,
    pub source_session: String,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
    pub last_confirmed: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPattern {
    pub pattern_type: String,
    pub description: String,
    pub examples: Vec<String>,
    pub source_session: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedInsight {
    pub summary: String,
    pub from_sessions: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

impl ProjectKnowledge {
    pub fn new(project_root: &str) -> Self {
        Self {
            project_root: project_root.to_string(),
            project_hash: hash_project_root(project_root),
            facts: Vec::new(),
            patterns: Vec::new(),
            history: Vec::new(),
            updated_at: Utc::now(),
        }
    }

    pub fn remember(
        &mut self,
        category: &str,
        key: &str,
        value: &str,
        session_id: &str,
        confidence: f32,
    ) {
        if let Some(existing) = self
            .facts
            .iter_mut()
            .find(|f| f.category == category && f.key == key)
        {
            existing.value = value.to_string();
            existing.confidence = confidence;
            existing.last_confirmed = Utc::now();
            existing.source_session = session_id.to_string();
        } else {
            let now = Utc::now();
            self.facts.push(KnowledgeFact {
                category: category.to_string(),
                key: key.to_string(),
                value: value.to_string(),
                source_session: session_id.to_string(),
                confidence,
                created_at: now,
                last_confirmed: now,
            });
            if self.facts.len() > MAX_FACTS {
                self.facts
                    .sort_by(|a, b| b.last_confirmed.cmp(&a.last_confirmed));
                self.facts.truncate(MAX_FACTS);
            }
        }
        self.updated_at = Utc::now();
    }

    pub fn recall(&self, query: &str) -> Vec<&KnowledgeFact> {
        let q = query.to_lowercase();
        let terms: Vec<&str> = q.split_whitespace().collect();

        let mut results: Vec<(&KnowledgeFact, f32)> = self
            .facts
            .iter()
            .filter_map(|f| {
                let searchable = format!(
                    "{} {} {} {}",
                    f.category.to_lowercase(),
                    f.key.to_lowercase(),
                    f.value.to_lowercase(),
                    f.source_session
                );
                let match_count = terms.iter().filter(|t| searchable.contains(**t)).count();
                if match_count > 0 {
                    let relevance = (match_count as f32 / terms.len() as f32) * f.confidence;
                    Some((f, relevance))
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().map(|(f, _)| f).collect()
    }

    pub fn recall_by_category(&self, category: &str) -> Vec<&KnowledgeFact> {
        self.facts
            .iter()
            .filter(|f| f.category == category)
            .collect()
    }

    pub fn add_pattern(
        &mut self,
        pattern_type: &str,
        description: &str,
        examples: Vec<String>,
        session_id: &str,
    ) {
        if let Some(existing) = self
            .patterns
            .iter_mut()
            .find(|p| p.pattern_type == pattern_type && p.description == description)
        {
            for ex in &examples {
                if !existing.examples.contains(ex) {
                    existing.examples.push(ex.clone());
                }
            }
            return;
        }

        self.patterns.push(ProjectPattern {
            pattern_type: pattern_type.to_string(),
            description: description.to_string(),
            examples,
            source_session: session_id.to_string(),
            created_at: Utc::now(),
        });

        if self.patterns.len() > MAX_PATTERNS {
            self.patterns.truncate(MAX_PATTERNS);
        }
        self.updated_at = Utc::now();
    }

    pub fn consolidate(&mut self, summary: &str, session_ids: Vec<String>) {
        self.history.push(ConsolidatedInsight {
            summary: summary.to_string(),
            from_sessions: session_ids,
            timestamp: Utc::now(),
        });

        if self.history.len() > MAX_HISTORY {
            self.history.drain(0..self.history.len() - MAX_HISTORY);
        }
        self.updated_at = Utc::now();
    }

    pub fn remove_fact(&mut self, category: &str, key: &str) -> bool {
        let before = self.facts.len();
        self.facts
            .retain(|f| !(f.category == category && f.key == key));
        let removed = self.facts.len() < before;
        if removed {
            self.updated_at = Utc::now();
        }
        removed
    }

    pub fn format_summary(&self) -> String {
        let mut out = String::new();

        if !self.facts.is_empty() {
            out.push_str("PROJECT KNOWLEDGE:\n");
            let mut categories: Vec<&str> =
                self.facts.iter().map(|f| f.category.as_str()).collect();
            categories.sort();
            categories.dedup();

            for cat in categories {
                out.push_str(&format!("  [{cat}]\n"));
                for f in self.facts.iter().filter(|f| f.category == cat) {
                    out.push_str(&format!(
                        "    {}: {} (confidence: {:.0}%)\n",
                        f.key,
                        f.value,
                        f.confidence * 100.0
                    ));
                }
            }
        }

        if !self.patterns.is_empty() {
            out.push_str("PROJECT PATTERNS:\n");
            for p in &self.patterns {
                out.push_str(&format!("  [{}] {}\n", p.pattern_type, p.description));
            }
        }

        out
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = knowledge_dir(&self.project_hash)?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

        let path = dir.join("knowledge.json");
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())
    }

    pub fn load(project_root: &str) -> Option<Self> {
        let hash = hash_project_root(project_root);
        let dir = knowledge_dir(&hash).ok()?;
        let path = dir.join("knowledge.json");

        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn load_or_create(project_root: &str) -> Self {
        Self::load(project_root).unwrap_or_else(|| Self::new(project_root))
    }
}

fn knowledge_dir(project_hash: &str) -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Cannot determine home directory")?;
    Ok(home
        .join(".better-ctx")
        .join("knowledge")
        .join(project_hash))
}

fn hash_project_root(root: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    root.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remember_and_recall() {
        let mut k = ProjectKnowledge::new("/tmp/test-project");
        k.remember("architecture", "auth", "JWT RS256", "session-1", 0.9);
        k.remember("api", "rate-limit", "100/min", "session-1", 0.8);

        let results = k.recall("auth");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "JWT RS256");

        let results = k.recall("api rate");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "rate-limit");
    }

    #[test]
    fn upsert_existing_fact() {
        let mut k = ProjectKnowledge::new("/tmp/test");
        k.remember("arch", "db", "PostgreSQL", "s1", 0.7);
        k.remember("arch", "db", "PostgreSQL 16 with pgvector", "s2", 0.95);

        assert_eq!(k.facts.len(), 1);
        assert_eq!(k.facts[0].value, "PostgreSQL 16 with pgvector");
        assert_eq!(k.facts[0].confidence, 0.95);
    }

    #[test]
    fn remove_fact() {
        let mut k = ProjectKnowledge::new("/tmp/test");
        k.remember("arch", "db", "PostgreSQL", "s1", 0.9);
        assert!(k.remove_fact("arch", "db"));
        assert!(k.facts.is_empty());
        assert!(!k.remove_fact("arch", "db"));
    }

    #[test]
    fn consolidate_history() {
        let mut k = ProjectKnowledge::new("/tmp/test");
        k.consolidate(
            "Migrated from REST to GraphQL",
            vec!["s1".into(), "s2".into()],
        );
        assert_eq!(k.history.len(), 1);
        assert_eq!(k.history[0].from_sessions.len(), 2);
    }

    #[test]
    fn format_summary_output() {
        let mut k = ProjectKnowledge::new("/tmp/test");
        k.remember("architecture", "auth", "JWT RS256", "s1", 0.9);
        k.add_pattern(
            "naming",
            "snake_case for functions",
            vec!["get_user()".into()],
            "s1",
        );
        let summary = k.format_summary();
        assert!(summary.contains("PROJECT KNOWLEDGE:"));
        assert!(summary.contains("auth: JWT RS256"));
        assert!(summary.contains("PROJECT PATTERNS:"));
    }
}
