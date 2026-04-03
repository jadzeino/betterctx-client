use std::collections::HashMap;
use std::path::Path;

use super::entropy::kolmogorov_proxy;

#[derive(Debug, Clone)]
pub struct CompressionThresholds {
    pub bpe_entropy: f64,
    pub jaccard: f64,
    pub auto_delta: f64,
}

impl Default for CompressionThresholds {
    fn default() -> Self {
        Self {
            bpe_entropy: 1.0,
            jaccard: 0.7,
            auto_delta: 0.6,
        }
    }
}

static LANGUAGE_THRESHOLDS: &[(&str, CompressionThresholds)] = &[
    // Python: English-like syntax, significant whitespace → higher entropy baseline
    (
        "py",
        CompressionThresholds {
            bpe_entropy: 1.2,
            jaccard: 0.65,
            auto_delta: 0.55,
        },
    ),
    // Rust: Repetitive keywords (fn, pub, impl, let, mut) → lower threshold catches more
    (
        "rs",
        CompressionThresholds {
            bpe_entropy: 0.85,
            jaccard: 0.72,
            auto_delta: 0.6,
        },
    ),
    // TypeScript/JavaScript: Type annotations are predictable
    (
        "ts",
        CompressionThresholds {
            bpe_entropy: 0.95,
            jaccard: 0.68,
            auto_delta: 0.58,
        },
    ),
    (
        "tsx",
        CompressionThresholds {
            bpe_entropy: 0.95,
            jaccard: 0.68,
            auto_delta: 0.58,
        },
    ),
    (
        "js",
        CompressionThresholds {
            bpe_entropy: 1.0,
            jaccard: 0.68,
            auto_delta: 0.58,
        },
    ),
    (
        "jsx",
        CompressionThresholds {
            bpe_entropy: 1.0,
            jaccard: 0.68,
            auto_delta: 0.58,
        },
    ),
    // Go: Verbose but highly structured → aggressive threshold
    (
        "go",
        CompressionThresholds {
            bpe_entropy: 0.9,
            jaccard: 0.72,
            auto_delta: 0.55,
        },
    ),
    // Java/Kotlin: Very verbose, lots of boilerplate
    (
        "java",
        CompressionThresholds {
            bpe_entropy: 0.8,
            jaccard: 0.65,
            auto_delta: 0.5,
        },
    ),
    (
        "kt",
        CompressionThresholds {
            bpe_entropy: 0.85,
            jaccard: 0.68,
            auto_delta: 0.55,
        },
    ),
    // C/C++: Headers are highly repetitive
    (
        "c",
        CompressionThresholds {
            bpe_entropy: 0.9,
            jaccard: 0.7,
            auto_delta: 0.6,
        },
    ),
    (
        "h",
        CompressionThresholds {
            bpe_entropy: 0.75,
            jaccard: 0.65,
            auto_delta: 0.5,
        },
    ),
    (
        "cpp",
        CompressionThresholds {
            bpe_entropy: 0.9,
            jaccard: 0.7,
            auto_delta: 0.6,
        },
    ),
    (
        "hpp",
        CompressionThresholds {
            bpe_entropy: 0.75,
            jaccard: 0.65,
            auto_delta: 0.5,
        },
    ),
    // Ruby: English-like, high entropy
    (
        "rb",
        CompressionThresholds {
            bpe_entropy: 1.15,
            jaccard: 0.65,
            auto_delta: 0.55,
        },
    ),
    // Config/data files: highly repetitive
    (
        "json",
        CompressionThresholds {
            bpe_entropy: 0.6,
            jaccard: 0.6,
            auto_delta: 0.4,
        },
    ),
    (
        "yaml",
        CompressionThresholds {
            bpe_entropy: 0.7,
            jaccard: 0.62,
            auto_delta: 0.45,
        },
    ),
    (
        "yml",
        CompressionThresholds {
            bpe_entropy: 0.7,
            jaccard: 0.62,
            auto_delta: 0.45,
        },
    ),
    (
        "toml",
        CompressionThresholds {
            bpe_entropy: 0.7,
            jaccard: 0.62,
            auto_delta: 0.45,
        },
    ),
    (
        "xml",
        CompressionThresholds {
            bpe_entropy: 0.6,
            jaccard: 0.6,
            auto_delta: 0.4,
        },
    ),
    // Markdown/docs: natural language, high entropy
    (
        "md",
        CompressionThresholds {
            bpe_entropy: 1.3,
            jaccard: 0.6,
            auto_delta: 0.55,
        },
    ),
    // CSS: very repetitive selectors/properties
    (
        "css",
        CompressionThresholds {
            bpe_entropy: 0.7,
            jaccard: 0.6,
            auto_delta: 0.45,
        },
    ),
    (
        "scss",
        CompressionThresholds {
            bpe_entropy: 0.75,
            jaccard: 0.62,
            auto_delta: 0.48,
        },
    ),
    // SQL: repetitive keywords
    (
        "sql",
        CompressionThresholds {
            bpe_entropy: 0.8,
            jaccard: 0.65,
            auto_delta: 0.5,
        },
    ),
    // Shell scripts
    (
        "sh",
        CompressionThresholds {
            bpe_entropy: 1.0,
            jaccard: 0.68,
            auto_delta: 0.55,
        },
    ),
    (
        "bash",
        CompressionThresholds {
            bpe_entropy: 1.0,
            jaccard: 0.68,
            auto_delta: 0.55,
        },
    ),
    // Swift/C#
    (
        "swift",
        CompressionThresholds {
            bpe_entropy: 0.9,
            jaccard: 0.68,
            auto_delta: 0.55,
        },
    ),
    (
        "cs",
        CompressionThresholds {
            bpe_entropy: 0.85,
            jaccard: 0.65,
            auto_delta: 0.52,
        },
    ),
    // PHP
    (
        "php",
        CompressionThresholds {
            bpe_entropy: 0.95,
            jaccard: 0.68,
            auto_delta: 0.55,
        },
    ),
];

fn language_map() -> HashMap<&'static str, &'static CompressionThresholds> {
    LANGUAGE_THRESHOLDS
        .iter()
        .map(|(ext, t)| (*ext, t))
        .collect()
}

pub fn thresholds_for_path(path: &str) -> CompressionThresholds {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let map = language_map();
    if let Some(t) = map.get(ext) {
        return (*t).clone();
    }

    CompressionThresholds::default()
}

pub fn adaptive_thresholds(path: &str, content: &str) -> CompressionThresholds {
    let mut base = thresholds_for_path(path);

    // Apply learned thresholds from feedback loop if available
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let feedback = super::feedback::FeedbackStore::load();
    if let Some(learned_entropy) = feedback.get_learned_entropy(ext) {
        base.bpe_entropy = base.bpe_entropy * 0.6 + learned_entropy * 0.4;
    }
    if let Some(learned_jaccard) = feedback.get_learned_jaccard(ext) {
        base.jaccard = base.jaccard * 0.6 + learned_jaccard * 0.4;
    }

    if content.len() > 500 {
        let k = kolmogorov_proxy(content);
        let k_adjustment = (k - 0.45) * 0.5;
        base.bpe_entropy = (base.bpe_entropy + k_adjustment).clamp(0.4, 2.0);
        base.jaccard = (base.jaccard - k_adjustment * 0.3).clamp(0.5, 0.85);
    }

    base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_has_lower_threshold_than_python() {
        let rs = thresholds_for_path("src/main.rs");
        let py = thresholds_for_path("src/main.py");
        assert!(rs.bpe_entropy < py.bpe_entropy);
    }

    #[test]
    fn json_has_lowest_threshold() {
        let json = thresholds_for_path("config.json");
        let rs = thresholds_for_path("main.rs");
        assert!(json.bpe_entropy < rs.bpe_entropy);
    }

    #[test]
    fn unknown_ext_uses_default() {
        let t = thresholds_for_path("file.xyz");
        assert!((t.bpe_entropy - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn adaptive_adjusts_for_compressibility() {
        let repetitive = "use std::io;\n".repeat(200);
        let diverse: String = (0..200)
            .map(|i| format!("let var_{i} = compute_{i}(arg_{i});\n"))
            .collect();

        let t_rep = adaptive_thresholds("main.rs", &repetitive);
        let t_div = adaptive_thresholds("main.rs", &diverse);
        assert!(
            t_rep.bpe_entropy < t_div.bpe_entropy,
            "repetitive content should get lower threshold: {} vs {}",
            t_rep.bpe_entropy,
            t_div.bpe_entropy
        );
    }
}
