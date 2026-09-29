//! KRK Memory Plane — Hermes + Memarch Hybrid Truth Persistence
//!
//! The dual-channel memory substrate immune to token collapse:
//!
//! 1. **Memarch** (Raw Capture): Append-only, content-addressed, never deletes.
//!    Acts as the full entropy capture layer.
//!
//! 2. **Hermes** (Truth Compression): Overwrites canonical state,
//!    prunes stale facts on a configurable cycle, maintains strict fact registry.
//!
//! Core invariant: Agents NEVER read raw Memarch for decisions.
//! They read filtered summaries or Hermes canonical snapshots.

pub mod memarch;
pub mod hermes;
pub mod store;
pub mod index;
pub mod compression;
pub mod retrieval;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ─── CORE DOMAIN TYPES ───

/// A memory event — the unit of information entering the memory plane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryEvent {
    /// Raw unstructured observation.
    Raw(String),
    /// Compressed summary.
    Summary(String),
    /// PSC control signal snapshot.
    ControlSignal { lambda_hat: f64, mode: String },
    /// Agent action record.
    AgentAction { agent_id: String, action: String },
}

/// A stored memory record with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: [u8; 32],
    pub timestamp: DateTime<Utc>,
    pub event: MemoryEvent,
    pub content_hash: [u8; 32],
    /// Which layer owns this record.
    pub layer: MemoryLayer,
}

/// Which memory subsystem owns this record.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MemoryLayer {
    /// Raw append-only capture (Memarch).
    Raw,
    /// Canonical compressed truth (Hermes).
    Canonical,
}

/// A compressed snapshot of canonical state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub snapshot_hash: [u8; 32],
    pub timestamp: DateTime<Utc>,
    pub canonical_facts: Vec<CanonicalFact>,
    pub total_raw_events: usize,
}

/// A single canonical fact in the Hermes registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalFact {
    pub key: String,
    pub value: String,
    pub last_updated: DateTime<Utc>,
    pub source_count: usize,
}

/// A query for memory retrieval.
#[derive(Debug, Clone)]
pub struct MemoryQuery {
    pub text: Option<String>,
    pub time_range: Option<TimeRange>,
    pub layer_filter: Option<MemoryLayer>,
    pub limit: usize,
}

/// Time range for temporal slicing.
#[derive(Debug, Clone)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

// ─── CORE TRAITS ───

/// The fundamental memory store interface.
pub trait MemoryStore {
    /// Write a new event into the memory plane.
    fn write(&mut self, event: MemoryEvent);

    /// Retrieve records matching a query.
    fn retrieve(&self, query: &MemoryQuery) -> Vec<MemoryRecord>;

    /// Compress raw events into a canonical snapshot.
    fn compress(&mut self) -> MemorySnapshot;
}

/// Retrieval interface for downstream consumers (agents, control-plane).
pub trait MemoryRetrieval {
    /// Semantic similarity search over memory records.
    fn semantic_search(&self, query: &str, limit: usize) -> Vec<MemoryRecord>;

    /// Temporal slice over a time range.
    fn temporal_slice(&self, range: &TimeRange) -> Vec<MemoryRecord>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_event_serialization() {
        let event = MemoryEvent::ControlSignal {
            lambda_hat: -0.12,
            mode: "Safe".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("lambda_hat"));
    }

    #[test]
    fn test_memory_query_construction() {
        let query = MemoryQuery {
            text: Some("test".to_string()),
            time_range: None,
            layer_filter: Some(MemoryLayer::Canonical),
            limit: 10,
        };
        assert_eq!(query.limit, 10);
    }
}
