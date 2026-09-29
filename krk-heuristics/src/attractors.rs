//! Instability Attractors
//!
//! Identifies regions likely to produce admissibility fractures
//! such as causal density, temporal ambiguity, and solver entropy.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstabilityAttractor {
    pub causal_density: f64,
    pub temporal_ambiguity: f64,
    pub solver_entropy: f64,
    pub pressure_coupling: f64,
    pub trust_instability: f64,
}

impl InstabilityAttractor {
    /// Estimates the instability gradient for a given trajectory.
    pub fn estimate_gradient(&self) -> f64 {
        // unstable attractors: near-quorum partitions, clock skew + latency spike, etc.
        (self.causal_density * self.temporal_ambiguity * self.solver_entropy).sqrt()
    }
}
