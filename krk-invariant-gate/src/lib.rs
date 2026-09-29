//! KRK Invariant Arbitration Gate (IAG)
//!
//! Filters reality into admissible state transitions.
//! Acts as the authoritative arbitrator between inference layers (EDM/Experiment)
//! and the execution layer (Safety Kernel).

use krk_edm::InferenceResult;
use krk_egs::GroundedCategory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GateDecision {
    /// Proposal is epistemically stable and safe to execute.
    Allow,
    /// Proposal violates invariants or is epistemically noisy.
    Reject { reason: String },
    /// Proposal is suspicious; isolate node or halt execution.
    Quarantine { risk_score: f64 },
    /// Interpretation mismatch detected; trigger CEGAR loop.
    RefineModel,
}

pub struct InvariantArbitrationGate {}

impl InvariantArbitrationGate {
    pub fn new() -> Self {
        Self {}
    }

    /// Arbitrates between classifications and grounded signals.
    pub fn arbitrate(
        &self,
        inference: &InferenceResult,
        grounding: &GroundedCategory
    ) -> GateDecision {
        // Single Authority Principle: Check cross-layer consistency.
        match (inference.classification.clone(), grounding) {
            (krk_edm::EpistemicClass::Latency { .. }, GroundedCategory::Physical) => {
                if inference.confidence > 0.8 {
                    GateDecision::Allow
                } else {
                    GateDecision::Reject { reason: "Low confidence latency".to_string() }
                }
            }
            (krk_edm::EpistemicClass::Byzantine { .. }, _) => {
                GateDecision::Quarantine { risk_score: inference.confidence }
            }
            _ => GateDecision::RefineModel,
        }
    }
}
