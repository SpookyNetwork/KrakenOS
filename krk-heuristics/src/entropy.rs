//! Entropy-Guided Exploration
//!
//! Guides the runtime drift toward high-uncertainty admissibility regions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyField {
    pub causal_entropy: f64,
    pub temporal_entropy: f64,
    pub proof_entropy: f64,
    pub topology_entropy: f64,
}

impl EntropyField {
    /// Returns a combined uncertainty score to bias exploration.
    pub fn combined_uncertainty(&self) -> f64 {
        self.causal_entropy + self.temporal_entropy + self.proof_entropy + self.topology_entropy
    }
}
