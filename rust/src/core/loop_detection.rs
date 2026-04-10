use std::collections::HashMap;
use std::time::{Duration, Instant};

const NORMAL_THRESHOLD: u32 = 3;
const REDUCED_THRESHOLD: u32 = 8;
const BLOCKED_THRESHOLD: u32 = 12;
const WINDOW_SECS: u64 = 300;

#[derive(Debug, Clone)]
pub struct LoopDetector {
    call_history: HashMap<String, Vec<Instant>>,
    duplicate_counts: HashMap<String, u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThrottleLevel {
    Normal,
    Reduced,
    Blocked,
}

#[derive(Debug, Clone)]
pub struct ThrottleResult {
    pub level: ThrottleLevel,
    pub call_count: u32,
    pub message: Option<String>,
}

impl Default for LoopDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LoopDetector {
    pub fn new() -> Self {
        Self {
            call_history: HashMap::new(),
            duplicate_counts: HashMap::new(),
        }
    }

    pub fn record_call(&mut self, tool: &str, args_fingerprint: &str) -> ThrottleResult {
        let key = format!("{tool}:{args_fingerprint}");
        let now = Instant::now();
        let window = Duration::from_secs(WINDOW_SECS);

        let entries = self.call_history.entry(key.clone()).or_default();
        entries.retain(|t| now.duration_since(*t) < window);
        entries.push(now);

        let count = entries.len() as u32;
        *self.duplicate_counts.entry(key).or_default() = count;

        if count > BLOCKED_THRESHOLD {
            ThrottleResult {
                level: ThrottleLevel::Blocked,
                call_count: count,
                message: Some(format!(
                    "⚠ LOOP DETECTED: {tool} called {count}× with same args in {WINDOW_SECS}s. \
                     Call blocked. Use ctx_batch_execute or vary your approach."
                )),
            }
        } else if count > REDUCED_THRESHOLD {
            ThrottleResult {
                level: ThrottleLevel::Reduced,
                call_count: count,
                message: Some(format!(
                    "⚠ Repetitive pattern: {tool} called {count}× with same args. \
                     Results reduced. Consider batching with ctx_batch_execute."
                )),
            }
        } else if count > NORMAL_THRESHOLD {
            ThrottleResult {
                level: ThrottleLevel::Reduced,
                call_count: count,
                message: Some(format!(
                    "Note: {tool} called {count}× with similar args. Consider batching."
                )),
            }
        } else {
            ThrottleResult {
                level: ThrottleLevel::Normal,
                call_count: count,
                message: None,
            }
        }
    }

    pub fn fingerprint(args: &serde_json::Value) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let canonical = canonical_json(args);
        let mut hasher = DefaultHasher::new();
        canonical.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    pub fn stats(&self) -> Vec<(String, u32)> {
        let mut entries: Vec<(String, u32)> = self
            .duplicate_counts
            .iter()
            .filter(|(_, &count)| count > 1)
            .map(|(k, &v)| (k.clone(), v))
            .collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries
    }

    pub fn reset(&mut self) {
        self.call_history.clear();
        self.duplicate_counts.clear();
    }
}

fn canonical_json(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let entries: Vec<String> = keys
                .iter()
                .map(|k| format!("{}:{}", k, canonical_json(&map[*k])))
                .collect();
            format!("{{{}}}", entries.join(","))
        }
        serde_json::Value::Array(arr) => {
            let entries: Vec<String> = arr.iter().map(canonical_json).collect();
            format!("[{}]", entries.join(","))
        }
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_calls_pass_through() {
        let mut detector = LoopDetector::new();
        let r1 = detector.record_call("ctx_read", "abc123");
        assert_eq!(r1.level, ThrottleLevel::Normal);
        assert_eq!(r1.call_count, 1);
        assert!(r1.message.is_none());
    }

    #[test]
    fn repeated_calls_trigger_reduced() {
        let mut detector = LoopDetector::new();
        for _ in 0..NORMAL_THRESHOLD {
            detector.record_call("ctx_read", "same_fp");
        }
        let result = detector.record_call("ctx_read", "same_fp");
        assert_eq!(result.level, ThrottleLevel::Reduced);
        assert!(result.message.is_some());
    }

    #[test]
    fn excessive_calls_get_blocked() {
        let mut detector = LoopDetector::new();
        for _ in 0..BLOCKED_THRESHOLD {
            detector.record_call("ctx_shell", "same_fp");
        }
        let result = detector.record_call("ctx_shell", "same_fp");
        assert_eq!(result.level, ThrottleLevel::Blocked);
        assert!(result.message.unwrap().contains("LOOP DETECTED"));
    }

    #[test]
    fn different_args_tracked_separately() {
        let mut detector = LoopDetector::new();
        for _ in 0..10 {
            detector.record_call("ctx_read", "fp_a");
        }
        let result = detector.record_call("ctx_read", "fp_b");
        assert_eq!(result.level, ThrottleLevel::Normal);
        assert_eq!(result.call_count, 1);
    }

    #[test]
    fn fingerprint_deterministic() {
        let args = serde_json::json!({"path": "test.rs", "mode": "full"});
        let fp1 = LoopDetector::fingerprint(&args);
        let fp2 = LoopDetector::fingerprint(&args);
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn fingerprint_order_independent() {
        let a = serde_json::json!({"mode": "full", "path": "test.rs"});
        let b = serde_json::json!({"path": "test.rs", "mode": "full"});
        assert_eq!(LoopDetector::fingerprint(&a), LoopDetector::fingerprint(&b));
    }

    #[test]
    fn stats_shows_duplicates() {
        let mut detector = LoopDetector::new();
        for _ in 0..5 {
            detector.record_call("ctx_read", "fp_a");
        }
        detector.record_call("ctx_shell", "fp_b");
        let stats = detector.stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].1, 5);
    }

    #[test]
    fn reset_clears_state() {
        let mut detector = LoopDetector::new();
        for _ in 0..5 {
            detector.record_call("ctx_read", "fp_a");
        }
        detector.reset();
        let result = detector.record_call("ctx_read", "fp_a");
        assert_eq!(result.call_count, 1);
    }
}
