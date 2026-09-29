//! Retrieval — Query interfaces for downstream consumers.

use crate::*;

/// Simple text-match retrieval over memory records (stub for semantic search).
pub fn text_match(records: &[MemoryRecord], query: &str) -> Vec<MemoryRecord> {
    records.iter()
        .filter(|r| {
            let text = match &r.event {
                MemoryEvent::Raw(t) | MemoryEvent::Summary(t) => t.clone(),
                MemoryEvent::ControlSignal { mode, .. } => mode.clone(),
                MemoryEvent::AgentAction { action, .. } => action.clone(),
            };
            text.contains(query)
        })
        .cloned()
        .collect()
}
