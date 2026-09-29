//! Collapse Profiler
//!
//! Measures reconciliation CPU cost, DAG traversal depth,
//! quotient collapse complexity, and merge amplification.

/// Individual collapse event measurement
#[derive(Debug, Clone, serde::Serialize)]
pub struct CollapseEvent {
    pub time_ms: f64,
    pub traversal_depth: usize,
    pub duration_ms: f64,
    pub effects_collapsed: usize,
}

#[derive(Debug)]
pub struct CollapseProfiler {
    pub total_reconciliation_time_ms: f64,
    pub max_traversal_depth: usize,
    pub total_collapses: usize,
    pub total_effects_collapsed: usize,
    pub collapse_events: Vec<CollapseEvent>,
    pub peak_collapse_spike_ms: f64,
}

impl CollapseProfiler {
    pub fn new() -> Self {
        Self {
            total_reconciliation_time_ms: 0.0,
            max_traversal_depth: 0,
            total_collapses: 0,
            total_effects_collapsed: 0,
            collapse_events: vec![],
            peak_collapse_spike_ms: 0.0,
        }
    }

    /// Record a collapse operation
    pub fn profile_collapse(&mut self, depth: usize, duration_ms: f64, effects: usize, time_ms: f64) {
        self.max_traversal_depth = self.max_traversal_depth.max(depth);
        self.total_reconciliation_time_ms += duration_ms;
        self.total_collapses += 1;
        self.total_effects_collapsed += effects;
        self.peak_collapse_spike_ms = self.peak_collapse_spike_ms.max(duration_ms);
        self.collapse_events.push(CollapseEvent {
            time_ms,
            traversal_depth: depth,
            duration_ms,
            effects_collapsed: effects,
        });
    }

    /// Collapse amplification factor: total traversal cost / total effects
    pub fn amplification_factor(&self) -> f64 {
        if self.total_effects_collapsed == 0 { return 0.0; }
        self.total_reconciliation_time_ms / self.total_effects_collapsed as f64
    }

    /// Average collapse cost per event
    pub fn avg_collapse_cost(&self) -> f64 {
        if self.total_collapses == 0 { return 0.0; }
        self.total_reconciliation_time_ms / self.total_collapses as f64
    }
}
