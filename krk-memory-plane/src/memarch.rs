//! Memarch — Raw Append-Only Capture Layer
//!
//! Rules:
//! - Append-only: events are NEVER deleted or modified.
//! - Content-addressed: each record is keyed by blake3 hash of its content.
//! - Full entropy retention: nothing is lost.

use crate::*;
use chrono::Utc;

/// The Memarch raw event store.
pub struct Memarch {
    records: Vec<MemoryRecord>,
}

impl Memarch {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    /// Append a raw event. Returns the content hash.
    pub fn append(&mut self, event: MemoryEvent) -> [u8; 32] {
        let serialized = format!("{:?}", event);
        let content_hash: [u8; 32] = blake3::hash(serialized.as_bytes()).into();
        let id = content_hash; // Content-addressed identity

        let record = MemoryRecord {
            id,
            timestamp: Utc::now(),
            event,
            content_hash,
            layer: MemoryLayer::Raw,
        };

        self.records.push(record);
        content_hash
    }

    /// Total number of raw events stored.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Check if store is empty.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Get all records (read-only).
    pub fn records(&self) -> &[MemoryRecord] {
        &self.records
    }

    /// Temporal slice by time range.
    pub fn slice(&self, range: &TimeRange) -> Vec<&MemoryRecord> {
        self.records.iter()
            .filter(|r| r.timestamp >= range.start && r.timestamp <= range.end)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memarch_append_and_retrieve() {
        let mut m = Memarch::new();
        let hash = m.append(MemoryEvent::Raw("hello world".to_string()));
        assert_eq!(m.len(), 1);
        assert_ne!(hash, [0u8; 32]);
    }

    #[test]
    fn test_memarch_content_addressing() {
        let mut m = Memarch::new();
        let h1 = m.append(MemoryEvent::Raw("same content".to_string()));
        let h2 = m.append(MemoryEvent::Raw("same content".to_string()));
        // Same content → same hash (content-addressed)
        assert_eq!(h1, h2);
        // But both are stored (append-only, never deduplicate)
        assert_eq!(m.len(), 2);
    }
}
