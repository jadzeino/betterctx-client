use std::collections::HashMap;

/// Cross-file semantic deduplication via TF-IDF codebook.
///
/// Identifies patterns that appear frequently across files (high TF, low IDF)
/// and creates short references for them. This avoids sending the same
/// boilerplate to the LLM multiple times across different file reads.

#[derive(Debug, Clone)]
pub struct CodebookEntry {
    pub id: String,
    pub pattern: String,
    pub frequency: usize,
    pub idf: f64,
}

#[derive(Debug, Default)]
pub struct Codebook {
    entries: Vec<CodebookEntry>,
    pattern_to_id: HashMap<String, String>,
    next_id: usize,
}

impl Codebook {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build codebook from multiple file contents.
    /// Identifies lines that appear in 3+ files and creates short references.
    pub fn build_from_files(&mut self, files: &[(String, String)]) {
        let total_docs = files.len() as f64;
        if total_docs < 2.0 {
            return;
        }

        // Count document frequency for each normalized line
        let mut doc_freq: HashMap<String, usize> = HashMap::new();
        let mut term_freq: HashMap<String, usize> = HashMap::new();

        for (_, content) in files {
            let mut seen_in_doc: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            for line in content.lines() {
                let normalized = normalize_line(line);
                if normalized.len() < 10 {
                    continue;
                }

                *term_freq.entry(normalized.clone()).or_insert(0) += 1;

                if seen_in_doc.insert(normalized.clone()) {
                    *doc_freq.entry(normalized).or_insert(0) += 1;
                }
            }
        }

        // Select patterns with high DF (appear in many files) — these are boilerplate
        let mut candidates: Vec<(String, usize, f64)> = doc_freq
            .into_iter()
            .filter(|(_, df)| *df >= 3) // appears in 3+ files
            .map(|(pattern, df)| {
                let idf = (total_docs / df as f64).ln();
                let tf = *term_freq.get(&pattern).unwrap_or(&0);
                (pattern, tf, idf)
            })
            .collect();

        // Sort by frequency descending (most common boilerplate first)
        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        // Take top 50 patterns to keep codebook compact
        for (pattern, freq, idf) in candidates.into_iter().take(50) {
            let id = format!("§{}", self.next_id);
            self.next_id += 1;
            self.pattern_to_id.insert(pattern.clone(), id.clone());
            self.entries.push(CodebookEntry {
                id,
                pattern,
                frequency: freq,
                idf,
            });
        }
    }

    /// Apply codebook to content: replace known patterns with short references.
    /// Returns (compressed content, references used).
    pub fn compress(&self, content: &str) -> (String, Vec<String>) {
        if self.entries.is_empty() {
            return (content.to_string(), vec![]);
        }

        let mut result = Vec::new();
        let mut refs_used = Vec::new();

        for line in content.lines() {
            let normalized = normalize_line(line);
            if let Some(id) = self.pattern_to_id.get(&normalized) {
                if !refs_used.contains(id) {
                    refs_used.push(id.clone());
                }
                result.push(format!("[{id}]"));
            } else {
                result.push(line.to_string());
            }
        }

        (result.join("\n"), refs_used)
    }

    /// Format the codebook legend for lines that were referenced.
    pub fn format_legend(&self, refs_used: &[String]) -> String {
        if refs_used.is_empty() {
            return String::new();
        }

        let mut lines = vec!["§CODEBOOK:".to_string()];
        for entry in &self.entries {
            if refs_used.contains(&entry.id) {
                let short = if entry.pattern.len() > 60 {
                    format!("{}...", &entry.pattern[..57])
                } else {
                    entry.pattern.clone()
                };
                lines.push(format!("  {}={}", entry.id, short));
            }
        }
        lines.join("\n")
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Cosine similarity between two documents using TF-IDF vectors.
/// Used for embedding-space deduplication approximation.
pub fn tfidf_cosine_similarity(doc_a: &str, doc_b: &str) -> f64 {
    let tf_a = term_frequencies(doc_a);
    let tf_b = term_frequencies(doc_b);

    let all_terms: std::collections::HashSet<&str> =
        tf_a.keys().chain(tf_b.keys()).copied().collect();
    if all_terms.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0;
    let mut mag_a = 0.0;
    let mut mag_b = 0.0;

    for term in &all_terms {
        let a = *tf_a.get(term).unwrap_or(&0.0);
        let b = *tf_b.get(term).unwrap_or(&0.0);
        dot += a * b;
        mag_a += a * a;
        mag_b += b * b;
    }

    let magnitude = (mag_a * mag_b).sqrt();
    if magnitude < f64::EPSILON {
        return 0.0;
    }

    dot / magnitude
}

/// Identify semantically duplicate blocks across files.
/// Returns pairs of (file_a, file_b, similarity) where similarity > threshold.
pub fn find_semantic_duplicates(
    files: &[(String, String)],
    threshold: f64,
) -> Vec<(String, String, f64)> {
    let mut duplicates = Vec::new();

    for i in 0..files.len() {
        for j in (i + 1)..files.len() {
            let sim = tfidf_cosine_similarity(&files[i].1, &files[j].1);
            if sim >= threshold {
                duplicates.push((files[i].0.clone(), files[j].0.clone(), sim));
            }
        }
    }

    duplicates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    duplicates
}

fn term_frequencies(text: &str) -> HashMap<&str, f64> {
    let mut freq: HashMap<&str, f64> = HashMap::new();
    let words: Vec<&str> = text.split_whitespace().collect();
    let total = words.len() as f64;
    if total == 0.0 {
        return freq;
    }

    for word in &words {
        *freq.entry(word).or_insert(0.0) += 1.0;
    }

    for val in freq.values_mut() {
        *val /= total;
    }

    freq
}

fn normalize_line(line: &str) -> String {
    line.split_whitespace().collect::<Vec<&str>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codebook_identifies_common_patterns() {
        let files = vec![
            (
                "a.rs".to_string(),
                "use std::io;\nuse std::collections::HashMap;\nfn main() {}\n".to_string(),
            ),
            (
                "b.rs".to_string(),
                "use std::io;\nuse std::collections::HashMap;\nfn helper() {}\n".to_string(),
            ),
            (
                "c.rs".to_string(),
                "use std::io;\nuse std::collections::HashMap;\nfn other() {}\n".to_string(),
            ),
            (
                "d.rs".to_string(),
                "use std::io;\nfn unique() {}\n".to_string(),
            ),
        ];

        let mut cb = Codebook::new();
        cb.build_from_files(&files);
        assert!(!cb.is_empty(), "should find common patterns");
    }

    #[test]
    fn cosine_identical_is_one() {
        let sim = tfidf_cosine_similarity("hello world foo", "hello world foo");
        assert!((sim - 1.0).abs() < 0.01);
    }

    #[test]
    fn cosine_disjoint_is_zero() {
        let sim = tfidf_cosine_similarity("alpha beta gamma", "delta epsilon zeta");
        assert!(sim < 0.01);
    }

    #[test]
    fn cosine_partial_overlap() {
        let sim = tfidf_cosine_similarity("hello world foo bar", "hello world baz qux");
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn find_duplicates_detects_similar_files() {
        let files = vec![
            (
                "a.rs".to_string(),
                "fn main() { let x = 1; let y = 2; println!(x + y); }".to_string(),
            ),
            (
                "b.rs".to_string(),
                "fn main() { let x = 1; let y = 2; println!(x + y); }".to_string(),
            ),
            (
                "c.rs".to_string(),
                "completely different content here with no overlap at all".to_string(),
            ),
        ];

        let dups = find_semantic_duplicates(&files, 0.8);
        assert_eq!(dups.len(), 1);
        assert!(dups[0].2 > 0.99);
    }
}
