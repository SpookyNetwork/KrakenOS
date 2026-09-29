//! Horizon Tracker
//!
//! Central observability primitive. Continuously measures:
//!   V_H = d|H_i(t)| / dt
//! Tracks visible causal ancestry, equivalence discovery rate,
//! branch collapse rate, and unresolved ambiguity depth.

use krk_algebra::CausalID;
use std::collections::HashSet;

/// Per-node epistemic horizon state
#[derive(Debug, Clone)]
pub struct NodeHorizon {
    pub node_id: String,
    pub visible_effects: HashSet<CausalID>,
    pub snapshot_times: Vec<(f64, usize)>, // (time_ms, horizon_size)
}

impl NodeHorizon {
    pub fn new(node_id: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
            visible_effects: HashSet::new(),
            snapshot_times: vec![],
        }
    }

    pub fn observe(&mut self, id: CausalID, time_ms: f64) {
        self.visible_effects.insert(id);
        self.snapshot_times.push((time_ms, self.visible_effects.len()));
    }

    pub fn size(&self) -> usize {
        self.visible_effects.len()
    }
}

/// Global horizon tracker across all nodes
#[derive(Debug)]
pub struct HorizonTracker {
    pub horizons: Vec<NodeHorizon>,
    pub equivalence_discoveries: usize,
    pub total_collapse_events: usize,
    pub current_horizon_size: usize,
    pub equivalence_discovery_timestamps: Vec<f64>,
}

impl HorizonTracker {
    pub fn new(node_ids: &[&str]) -> Self {
        Self {
            horizons: node_ids.iter().map(|id| NodeHorizon::new(id)).collect(),
            equivalence_discoveries: 0,
            total_collapse_events: 0,
            current_horizon_size: 0,
            equivalence_discovery_timestamps: vec![],
        }
    }

    /// Measure global horizon expansion velocity (events / ms)
    pub fn measure_velocity(&self, dt_ms: f64) -> f64 {
        if dt_ms <= 0.0 { return 0.0; }
        let total: usize = self.horizons.iter().map(|h| h.size()).sum();
        total as f64 / dt_ms
    }

    /// Record an equivalence discovery event
    pub fn record_equivalence_discovery(&mut self, time_ms: f64) {
        self.equivalence_discoveries += 1;
        self.equivalence_discovery_timestamps.push(time_ms);
    }

    /// Measure horizon intersection between two nodes
    pub fn horizon_intersection(&self, node_a: usize, node_b: usize) -> usize {
        if node_a >= self.horizons.len() || node_b >= self.horizons.len() { return 0; }
        self.horizons[node_a].visible_effects
            .intersection(&self.horizons[node_b].visible_effects)
            .count()
    }

    /// Measure horizon asymmetry (difference between largest and smallest)
    pub fn horizon_asymmetry(&self) -> usize {
        let sizes: Vec<usize> = self.horizons.iter().map(|h| h.size()).collect();
        sizes.iter().max().unwrap_or(&0) - sizes.iter().min().unwrap_or(&0)
    }
}
