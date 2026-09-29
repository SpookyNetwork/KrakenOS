//! Hermes — Canonical Truth Compression Layer
//!
//! Rules:
//! - Overwrites canonical state (lossy compression of raw events).
//! - Configurable pruning cycle (default 7-day TTL).
//! - Strict fact registry: each key has exactly one canonical value.
//! - Agents read ONLY from Hermes for decision-making.

use crate::*;
use chrono::{Duration, Utc};
use std::collections::HashMap;

/// The Hermes canonical fact store.
pub struct Hermes {
    /// Key → CanonicalFact mapping. Each key has exactly one truth.
    facts: HashMap<String, CanonicalFact>,
    /// Pruning TTL in days.
    prune_ttl_days: i64,
}

impl Hermes {
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
            prune_ttl_days: 7,
        }
    }

    /// Set the pruning TTL in days.
    pub fn with_ttl(mut self, days: i64) -> Self {
        self.prune_ttl_days = days;
        self
    }

    /// Upsert a canonical fact. If the key exists, it is OVERWRITTEN.
    pub fn upsert(&mut self, key: String, value: String) {
        let now = Utc::now();
        let entry = self.facts.entry(key).or_insert(CanonicalFact {
            key: String::new(),
            value: String::new(),
            last_updated: now,
            source_count: 0,
        });
        entry.value = value;
        entry.last_updated = now;
        entry.source_count += 1;
    }

    /// Get a canonical fact by key.
    pub fn get(&self, key: &str) -> Option<&CanonicalFact> {
        self.facts.get(key)
    }

    /// Prune facts older than TTL.
    pub fn prune(&mut self) -> usize {
        let cutoff = Utc::now() - Duration::days(self.prune_ttl_days);
        let before = self.facts.len();
        self.facts.retain(|_, f| f.last_updated > cutoff);
        before - self.facts.len()
    }

    /// Export all canonical facts as a snapshot.
    pub fn snapshot(&self) -> Vec<CanonicalFact> {
        self.facts.values().cloned().collect()
    }

    /// Total number of canonical facts.
    pub fn len(&self) -> usize {
        self.facts.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hermes_upsert_overwrites() {
        let mut h = Hermes::new();
        h.upsert("key1".to_string(), "value_a".to_string());
        h.upsert("key1".to_string(), "value_b".to_string());
        assert_eq!(h.len(), 1);
        assert_eq!(h.get("key1").unwrap().value, "value_b");
        assert_eq!(h.get("key1").unwrap().source_count, 2);
    }

    #[test]
    fn test_hermes_snapshot() {
        let mut h = Hermes::new();
        h.upsert("a".to_string(), "1".to_string());
        h.upsert("b".to_string(), "2".to_string());
        let snap = h.snapshot();
        assert_eq!(snap.len(), 2);
    }
}
