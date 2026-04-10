//! Downstream task evaluation framework for search quality.
//!
//! Measures how well the search pipeline supports actual coding tasks:
//! - Retrieval precision/recall against known-relevant chunks
//! - Mean Reciprocal Rank (MRR) for expected top results
//! - Normalized Discounted Cumulative Gain (nDCG)
//!
//! Designed to compare BM25-only vs hybrid search and track quality over time.

use std::collections::HashSet;

/// A single evaluation query with expected relevant results.
#[derive(Debug, Clone)]
pub struct EvalQuery {
    pub query: String,
    pub relevant_files: Vec<String>,
    pub expected_top: Option<String>,
}

/// Result of evaluating a search system against a query set.
#[derive(Debug, Clone)]
pub struct EvalReport {
    pub query_count: usize,
    pub precision_at_5: f64,
    pub precision_at_10: f64,
    pub recall_at_10: f64,
    pub mrr: f64,
    pub ndcg_at_10: f64,
    pub per_query: Vec<QueryScore>,
}

#[derive(Debug, Clone)]
pub struct QueryScore {
    pub query: String,
    pub precision_at_k: f64,
    pub recall: f64,
    pub reciprocal_rank: f64,
    pub ndcg: f64,
}

/// Retrieved result for evaluation (file path + score).
#[derive(Debug, Clone)]
pub struct RetrievedItem {
    pub file_path: String,
    pub score: f64,
}

/// Evaluate search results against a set of queries with known relevance.
pub fn evaluate(
    queries: &[EvalQuery],
    retrieve_fn: &dyn Fn(&str) -> Vec<RetrievedItem>,
) -> EvalReport {
    let mut per_query = Vec::with_capacity(queries.len());
    let mut sum_p5 = 0.0;
    let mut sum_p10 = 0.0;
    let mut sum_r10 = 0.0;
    let mut sum_mrr = 0.0;
    let mut sum_ndcg = 0.0;

    for q in queries {
        let results = retrieve_fn(&q.query);
        let relevant: HashSet<&str> = q.relevant_files.iter().map(|s| s.as_str()).collect();

        let p5 = precision_at_k(&results, &relevant, 5);
        let p10 = precision_at_k(&results, &relevant, 10);
        let r10 = recall_at_k(&results, &relevant, 10);
        let rr = reciprocal_rank(&results, &relevant);
        let ndcg = ndcg_at_k(&results, &relevant, 10);

        sum_p5 += p5;
        sum_p10 += p10;
        sum_r10 += r10;
        sum_mrr += rr;
        sum_ndcg += ndcg;

        per_query.push(QueryScore {
            query: q.query.clone(),
            precision_at_k: p10,
            recall: r10,
            reciprocal_rank: rr,
            ndcg,
        });
    }

    let n = queries.len().max(1) as f64;
    EvalReport {
        query_count: queries.len(),
        precision_at_5: sum_p5 / n,
        precision_at_10: sum_p10 / n,
        recall_at_10: sum_r10 / n,
        mrr: sum_mrr / n,
        ndcg_at_10: sum_ndcg / n,
        per_query,
    }
}

fn precision_at_k(results: &[RetrievedItem], relevant: &HashSet<&str>, k: usize) -> f64 {
    let top_k: Vec<&RetrievedItem> = results.iter().take(k).collect();
    if top_k.is_empty() {
        return 0.0;
    }
    let hits = top_k
        .iter()
        .filter(|r| relevant.contains(r.file_path.as_str()))
        .count();
    hits as f64 / top_k.len() as f64
}

fn recall_at_k(results: &[RetrievedItem], relevant: &HashSet<&str>, k: usize) -> f64 {
    if relevant.is_empty() {
        return 0.0;
    }
    let hits = results
        .iter()
        .take(k)
        .filter(|r| relevant.contains(r.file_path.as_str()))
        .count();
    hits as f64 / relevant.len() as f64
}

fn reciprocal_rank(results: &[RetrievedItem], relevant: &HashSet<&str>) -> f64 {
    for (i, r) in results.iter().enumerate() {
        if relevant.contains(r.file_path.as_str()) {
            return 1.0 / (i + 1) as f64;
        }
    }
    0.0
}

fn ndcg_at_k(results: &[RetrievedItem], relevant: &HashSet<&str>, k: usize) -> f64 {
    let dcg = results
        .iter()
        .take(k)
        .enumerate()
        .map(|(i, r)| {
            let gain = if relevant.contains(r.file_path.as_str()) {
                1.0
            } else {
                0.0
            };
            gain / (2.0f64 + i as f64).log2()
        })
        .sum::<f64>();

    let ideal_count = relevant.len().min(k);
    let ideal_dcg: f64 = (0..ideal_count)
        .map(|i| 1.0 / (2.0f64 + i as f64).log2())
        .sum();

    if ideal_dcg == 0.0 {
        return 0.0;
    }
    dcg / ideal_dcg
}

impl EvalReport {
    pub fn to_compact_string(&self) -> String {
        format!(
            "P@5={:.3} P@10={:.3} R@10={:.3} MRR={:.3} nDCG@10={:.3} (n={})",
            self.precision_at_5,
            self.precision_at_10,
            self.recall_at_10,
            self.mrr,
            self.ndcg_at_10,
            self.query_count,
        )
    }

    pub fn passed_threshold(&self, min_mrr: f64, min_ndcg: f64) -> bool {
        self.mrr >= min_mrr && self.ndcg_at_10 >= min_ndcg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items(files: &[&str]) -> Vec<RetrievedItem> {
        files
            .iter()
            .enumerate()
            .map(|(i, f)| RetrievedItem {
                file_path: f.to_string(),
                score: 10.0 - i as f64,
            })
            .collect()
    }

    #[test]
    fn precision_at_k_perfect() {
        let relevant: HashSet<&str> = ["a.rs", "b.rs"].into_iter().collect();
        let results = items(&["a.rs", "b.rs", "c.rs"]);
        assert!((precision_at_k(&results, &relevant, 2) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn precision_at_k_half() {
        let relevant: HashSet<&str> = ["a.rs"].into_iter().collect();
        let results = items(&["a.rs", "b.rs"]);
        assert!((precision_at_k(&results, &relevant, 2) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn precision_at_k_none() {
        let relevant: HashSet<&str> = ["x.rs"].into_iter().collect();
        let results = items(&["a.rs", "b.rs"]);
        assert!((precision_at_k(&results, &relevant, 2) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn recall_at_k_full() {
        let relevant: HashSet<&str> = ["a.rs"].into_iter().collect();
        let results = items(&["a.rs", "b.rs", "c.rs"]);
        assert!((recall_at_k(&results, &relevant, 3) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn recall_at_k_partial() {
        let relevant: HashSet<&str> = ["a.rs", "d.rs"].into_iter().collect();
        let results = items(&["a.rs", "b.rs", "c.rs"]);
        assert!((recall_at_k(&results, &relevant, 3) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn mrr_first_position() {
        let relevant: HashSet<&str> = ["a.rs"].into_iter().collect();
        let results = items(&["a.rs", "b.rs"]);
        assert!((reciprocal_rank(&results, &relevant) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn mrr_second_position() {
        let relevant: HashSet<&str> = ["b.rs"].into_iter().collect();
        let results = items(&["a.rs", "b.rs"]);
        assert!((reciprocal_rank(&results, &relevant) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn mrr_not_found() {
        let relevant: HashSet<&str> = ["x.rs"].into_iter().collect();
        let results = items(&["a.rs", "b.rs"]);
        assert!((reciprocal_rank(&results, &relevant) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn ndcg_perfect() {
        let relevant: HashSet<&str> = ["a.rs", "b.rs"].into_iter().collect();
        let results = items(&["a.rs", "b.rs", "c.rs"]);
        let score = ndcg_at_k(&results, &relevant, 3);
        assert!(
            (score - 1.0).abs() < 1e-6,
            "perfect ranking should give nDCG=1.0, got {score}"
        );
    }

    #[test]
    fn ndcg_imperfect() {
        let relevant: HashSet<&str> = ["b.rs"].into_iter().collect();
        let results = items(&["a.rs", "b.rs", "c.rs"]);
        let score = ndcg_at_k(&results, &relevant, 3);
        assert!(score > 0.0 && score < 1.0, "imperfect ranking: {score}");
    }

    #[test]
    fn evaluate_pipeline() {
        let queries = vec![
            EvalQuery {
                query: "authentication".to_string(),
                relevant_files: vec!["auth.rs".to_string()],
                expected_top: Some("auth.rs".to_string()),
            },
            EvalQuery {
                query: "database connection".to_string(),
                relevant_files: vec!["db.rs".to_string(), "pool.rs".to_string()],
                expected_top: Some("db.rs".to_string()),
            },
        ];

        let report = evaluate(&queries, &|q| {
            if q.contains("auth") {
                items(&["auth.rs", "user.rs", "session.rs"])
            } else {
                items(&["db.rs", "pool.rs", "config.rs"])
            }
        });

        assert_eq!(report.query_count, 2);
        assert!(report.mrr > 0.5, "MRR should be high: {}", report.mrr);
        assert!(
            report.ndcg_at_10 > 0.5,
            "nDCG should be high: {}",
            report.ndcg_at_10
        );
    }

    #[test]
    fn report_threshold() {
        let queries = vec![EvalQuery {
            query: "test".to_string(),
            relevant_files: vec!["test.rs".to_string()],
            expected_top: None,
        }];
        let report = evaluate(&queries, &|_| items(&["test.rs", "other.rs"]));
        assert!(report.passed_threshold(0.5, 0.5));
        assert!(!report.passed_threshold(2.0, 2.0));
    }
}
