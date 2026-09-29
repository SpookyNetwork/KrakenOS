//! Prioritized Adversarial Search
//!
//! Ranks exploration targets by their risk score to guide the soak engine.

use crate::attractors::InstabilityAttractor;

pub struct PrioritizedSearch {
    pub instability_attractor: InstabilityAttractor,
}

impl PrioritizedSearch {
    /// SAFETY RULE: Heuristics may prioritize exploration but NEVER alter admissibility truth.
    /// heuristics ↛ admissibility
    pub fn calculate_risk_score(&self, physics_pressure: f64) -> f64 {
        self.instability_attractor.estimate_gradient() * physics_pressure
    }
}
