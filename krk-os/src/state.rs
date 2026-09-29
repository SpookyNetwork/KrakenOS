//! KRK-OS State Layer
//!
//! The physical substrate: Event DAG + equivalence classes + RFC buffer + horizons.
//! This is the "world" that the controller acts upon.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

// ─── Event ───

pub type EventId = u64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    /// Payload-only identity hash (provenance-free)
    pub payload_hash: [u8; 32],
    /// Causal dependencies
    pub dependencies: Vec<EventId>,
    /// Timestamp (logical or wall-clock)
    pub timestamp: u64,
}

// ─── Event DAG ───

pub struct EventDAG {
    pub nodes: HashMap<EventId, Event>,
    pub forward_edges: HashMap<EventId, HashSet<EventId>>,
    next_id: EventId,
}

impl EventDAG {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            forward_edges: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn insert(&mut self, mut event: Event) -> EventId {
        let id = self.next_id;
        self.next_id += 1;
        event.id = id;

        for dep in &event.dependencies {
            self.forward_edges.entry(*dep).or_default().insert(id);
        }

        self.nodes.insert(id, event);
        id
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

// ─── Equivalence Classes (Quotient State) ───

/// The quotient structure E / ~_{payload}
pub struct EquivalenceIndex {
    /// Maps payload_hash → set of event IDs with that payload
    pub classes: HashMap<[u8; 32], HashSet<EventId>>,
}

impl EquivalenceIndex {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
        }
    }

    /// Rebuild the equivalence index from the full DAG
    pub fn rebuild(&mut self, dag: &EventDAG) {
        self.classes.clear();
        for (id, event) in &dag.nodes {
            self.classes
                .entry(event.payload_hash)
                .or_default()
                .insert(*id);
        }
    }

    /// Number of equivalence classes
    pub fn class_count(&self) -> usize {
        self.classes.len()
    }

    /// Number of events that have duplicates (divergences)
    pub fn divergence_count(&self) -> usize {
        self.classes.values().filter(|s| s.len() > 1).count()
    }
}

// ─── RFC Buffer (Ambiguity Store) ───

/// Holds events whose equivalence class membership is not yet resolved.
pub struct RFCBuffer {
    pub unresolved: Vec<Event>,
    pub max_capacity: usize,
}

impl RFCBuffer {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            unresolved: Vec::new(),
            max_capacity,
        }
    }

    pub fn push(&mut self, event: Event) -> bool {
        if self.unresolved.len() >= self.max_capacity {
            return false; // Bounded retention: reject if full
        }
        self.unresolved.push(event);
        true
    }

    pub fn drain(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.unresolved)
    }

    pub fn len(&self) -> usize {
        self.unresolved.len()
    }

    pub fn is_empty(&self) -> bool {
        self.unresolved.is_empty()
    }
}

// ─── Horizon (Partial Observability) ───

/// Each node's local view of the DAG.
pub struct Horizon {
    pub visible: HashSet<EventId>,
}

impl Horizon {
    pub fn new() -> Self {
        Self {
            visible: HashSet::new(),
        }
    }

    pub fn observe(&mut self, id: EventId) {
        self.visible.insert(id);
    }

    pub fn size(&self) -> usize {
        self.visible.len()
    }
}

// ─── Unified State ───

/// The complete state object X = (E, ~, H_i, RFC, DAG)
pub struct SubstrateState {
    pub dag: EventDAG,
    pub equivalence: EquivalenceIndex,
    pub rfc: RFCBuffer,
    pub horizon: Horizon,
}

impl SubstrateState {
    pub fn new(rfc_capacity: usize) -> Self {
        Self {
            dag: EventDAG::new(),
            equivalence: EquivalenceIndex::new(),
            rfc: RFCBuffer::new(rfc_capacity),
            horizon: Horizon::new(),
        }
    }

    /// Ingest an event: add to DAG, update horizon, rebuild equivalence
    pub fn ingest(&mut self, event: Event) {
        let id = self.dag.insert(event);
        self.horizon.observe(id);
        self.equivalence.rebuild(&self.dag);
    }

    /// Ingest a batch of events
    pub fn ingest_batch(&mut self, events: Vec<Event>) {
        for event in events {
            let id = self.dag.insert(event);
            self.horizon.observe(id);
        }
        self.equivalence.rebuild(&self.dag);
    }

    /// Attempt to resolve RFC buffer by collapsing into equivalence classes
    pub fn resolve_rfc(&mut self) {
        let pending = self.rfc.drain();
        for event in pending {
            let id = self.dag.insert(event);
            self.horizon.observe(id);
        }
        self.equivalence.rebuild(&self.dag);
    }
}
