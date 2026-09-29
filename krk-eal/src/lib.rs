//! KRK Epistemic Admission Layer (EAL)
//!
//! The final pre-commit gate. Blocks state mutation until global
//! epistemic compatibility is proven BEFORE commit.
//! Enforces the "Epistemically Synchronous Distributed Commit Model".

use krk_strl::{GlobalTruthGraph, TruthEvent, ReconciliationResult, TruthReconciliationEngine};
use serde::{Deserialize, Serialize};

pub enum AdmissionDecision {
    Admit,
    Block { reason: String },
}

pub struct EpistemicAdmissionLayer {
    pub engine: TruthReconciliationEngine,
}

impl EpistemicAdmissionLayer {
    pub fn new() -> Self {
        Self {
            engine: TruthReconciliationEngine::new(),
        }
    }

    /// Authoritatively evaluates if a truth event is admissible BEFORE state mutation.
    pub fn evaluate_admissibility(
        &self,
        event: &TruthEvent,
        graph: &GlobalTruthGraph
    ) -> AdmissionDecision {
        // Step 1: Speculative inclusion in the graph
        let mut speculative_graph = graph.clone();
        speculative_graph.push(event.clone());

        // Step 2: Global Epistemic Constraint Solving
        match self.engine.reconcile(&speculative_graph) {
            ReconciliationResult::Coherent => AdmissionDecision::Admit,
            ReconciliationResult::Contradiction(reason) => AdmissionDecision::Block {
                reason: format!("Epistemic Inconsistency detected: {}", reason)
            },
            ReconciliationResult::DriftDetected { delta } => {
                if delta > 0.05 {
                    AdmissionDecision::Block { reason: "Temporal drift too high for admission".to_string() }
                } else {
                    AdmissionDecision::Admit
                }
            }
        }
    }
}
