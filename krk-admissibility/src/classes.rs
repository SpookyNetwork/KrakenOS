//! Admissibility Equivalence Classes
//!
//! Defines structural instability regions to enable equivalence-class reuse
//! of reduction proofs and UNSAT witnesses.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmissibilityClass {
    pub temporal_signature: [u8; 32],
    pub causal_signature: [u8; 32],
    pub pressure_signature: [u8; 32],
    pub governance_signature: [u8; 32],
    pub physics_signature: [u8; 32],
}

impl AdmissibilityClass {
    pub fn new(temporal: [u8; 32], causal: [u8; 32], pressure: [u8; 32], governance: [u8; 32], physics: [u8; 32]) -> Self {
        Self {
            temporal_signature: temporal,
            causal_signature: causal,
            pressure_signature: pressure,
            governance_signature: governance,
            physics_signature: physics,
        }
    }
}
