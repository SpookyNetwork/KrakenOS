//! Branch Pressure Monitor
//!
//! Tracks active worldlines, unresolved equivalence classes, RFC occupancy,
//! and branch persistence duration. Determines if ambiguity is decaying
//! or accumulating faster than collapse.

/// Time-series snapshot of branch pressure state
#[derive(Debug, Clone, serde::Serialize)]
pub struct PressureSnapshot {
    pub time_ms: f64,
    pub active_worldlines: usize,
    pub committed_effects: usize,
    pub rfc_occupancy: usize,
    pub branch_density: f64,
    pub rfc_growth_rate: f64,
}

#[derive(Debug)]
pub struct BranchPressureMonitor {
    pub active_worldlines: usize,
    pub committed_effects: usize,
    pub rfc_occupancy: usize,
    pub peak_worldlines: usize,
    pub peak_rfc: usize,
    pub history: Vec<PressureSnapshot>,
}

impl BranchPressureMonitor {
    pub fn new() -> Self {
        Self {
            active_worldlines: 0,
            committed_effects: 0,
            rfc_occupancy: 0,
            peak_worldlines: 0,
            peak_rfc: 0,
            history: vec![],
        }
    }

    /// Branch density: D_B = active_worldlines / committed_effects
    pub fn density(&self) -> f64 {
        if self.committed_effects == 0 { return 0.0; }
        self.active_worldlines as f64 / self.committed_effects as f64
    }

    /// Record a divergence event (new worldline created)
    pub fn record_divergence(&mut self, count: usize) {
        self.active_worldlines += count;
        self.rfc_occupancy += count;
        self.peak_worldlines = self.peak_worldlines.max(self.active_worldlines);
        self.peak_rfc = self.peak_rfc.max(self.rfc_occupancy);
    }

    /// Record a collapse event (worldlines merged)
    pub fn record_collapse(&mut self, collapsed: usize) {
        self.active_worldlines = self.active_worldlines.saturating_sub(collapsed);
        self.rfc_occupancy = self.rfc_occupancy.saturating_sub(collapsed);
        self.committed_effects += collapsed;
    }

    /// Take a time-series snapshot
    pub fn snapshot(&mut self, time_ms: f64) {
        let prev_rfc = self.history.last().map(|s| s.rfc_occupancy as f64).unwrap_or(0.0);
        let prev_time = self.history.last().map(|s| s.time_ms).unwrap_or(0.0);
        let dt = time_ms - prev_time;
        let rfc_rate = if dt > 0.0 { (self.rfc_occupancy as f64 - prev_rfc) / dt } else { 0.0 };

        self.history.push(PressureSnapshot {
            time_ms,
            active_worldlines: self.active_worldlines,
            committed_effects: self.committed_effects,
            rfc_occupancy: self.rfc_occupancy,
            branch_density: self.density(),
            rfc_growth_rate: rfc_rate,
        });
    }
}
