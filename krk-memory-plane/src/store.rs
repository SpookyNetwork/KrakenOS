//! Unified Memory Store — Bridges Memarch and Hermes
//!
//! Implements the MemoryStore trait by writing to both layers
//! and compressing from Memarch → Hermes.

use crate::*;
use crate::memarch::Memarch;
use crate::hermes::Hermes;
use chrono::Utc;

/// The unified dual-layer memory store.
pub struct DualMemoryStore {
    pub memarch: Memarch,
    pub hermes: Hermes,
    compression_counter: usize,
    /// Number of raw events before auto-compression.
    pub auto_compress_threshold: usize,
}

impl DualMemoryStore {
    pub fn new() -> Self {
        Self {
            memarch: Memarch::new(),
            hermes: Hermes::new(),
            compression_counter: 0,
            auto_compress_threshold: 100,
        }
    }
}

impl MemoryStore for DualMemoryStore {
    fn write(&mut self, event: MemoryEvent) {
        // Always write to Memarch (append-only, never lost).
        self.memarch.append(event.clone());
        self.compression_counter += 1;

        // Auto-compress if threshold hit.
        if self.compression_counter >= self.auto_compress_threshold {
            let _ = self.compress();
        }
    }

    fn retrieve(&self, query: &MemoryQuery) -> Vec<MemoryRecord> {
        let layer = query.layer_filter.unwrap_or(MemoryLayer::Canonical);
        match layer {
            MemoryLayer::Raw => {
                self.memarch.records()
                    .iter()
                    .rev()
                    .take(query.limit)
                    .cloned()
                    .collect()
            }
            MemoryLayer::Canonical => {
                // Return Hermes canonical facts as MemoryRecords.
                self.hermes.snapshot()
                    .into_iter()
                    .take(query.limit)
                    .map(|fact| MemoryRecord {
                        id: blake3::hash(fact.key.as_bytes()).into(),
                        timestamp: fact.last_updated,
                        event: MemoryEvent::Summary(format!("{}: {}", fact.key, fact.value)),
                        content_hash: blake3::hash(fact.value.as_bytes()).into(),
                        layer: MemoryLayer::Canonical,
                    })
                    .collect()
            }
        }
    }

    fn compress(&mut self) -> MemorySnapshot {
        // Compress recent Memarch events into Hermes canonical facts.
        let raw_count = self.memarch.len();

        for record in self.memarch.records() {
            match &record.event {
                MemoryEvent::Raw(text) => {
                    self.hermes.upsert(
                        format!("raw_{}", hex::encode(&record.content_hash[..4])),
                        text.clone(),
                    );
                }
                MemoryEvent::Summary(text) => {
                    self.hermes.upsert("last_summary".to_string(), text.clone());
                }
                MemoryEvent::ControlSignal { lambda_hat, mode } => {
                    self.hermes.upsert("psc_lambda".to_string(), format!("{}", lambda_hat));
                    self.hermes.upsert("psc_mode".to_string(), mode.clone());
                }
                MemoryEvent::AgentAction { agent_id, action } => {
                    self.hermes.upsert(
                        format!("agent_{}_last", agent_id),
                        action.clone(),
                    );
                }
            }
        }

        self.compression_counter = 0;

        let snap_content = format!("{:?}", self.hermes.snapshot());
        MemorySnapshot {
            snapshot_hash: blake3::hash(snap_content.as_bytes()).into(),
            timestamp: Utc::now(),
            canonical_facts: self.hermes.snapshot(),
            total_raw_events: raw_count,
        }
    }
}

// hex encoding helper for content hash keys
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
