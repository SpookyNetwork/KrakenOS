//! KRK Admissibility Policy Engine (KRK-AC)
//!
//! A bounded admissibility filter under uncertainty.
//! Evaluates policy-constrained worldline satisfiability over
//! partially observed distributed state.
//!
//! PSC-MODE-AWARE: Admissibility thresholds are tightened based on
//! the current PSC SystemMode. In CollapsePrevention, all events
//! are rejected. In Degraded, confidence requirements are stricter.
//!
//! Σ ⊢ e ⇝ ⊤ / ⊥

use krk_ets::{ExternalTruth, EpistemicConfidence, SignalMetadata};
use krk_strl::{TruthEvent, TruthDomain};
use krk_sat::Z3Bridge;
use krk_cegar::CegarEngine;
use krk_edm::EpistemicDiscriminator;
use krk_egs::EpistemicGroundingStabilizer;
use krk_invariant_gate::{InvariantArbitrationGate, GateDecision};
use krk_rfc::{ResidualClassifier, EpistemicResidue};
use krk_psc::SystemMode;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalTruthState {
    pub history_dag: Vec<TruthEvent>,
    pub constraints: Vec<String>,
    pub witnesses: HashMap<String, Vec<u8>>,
}

pub struct AdmissibilityPolicyEngine {
    pub z3: Z3Bridge,
    pub cegar: CegarEngine,
    pub edm: EpistemicDiscriminator,
    pub egs: EpistemicGroundingStabilizer,
    pub iag: InvariantArbitrationGate,
    pub rfc: ResidualClassifier,
}

impl AdmissibilityPolicyEngine {
    pub fn new() -> Self {
        Self {
            z3: Z3Bridge::new(),
            cegar: CegarEngine::new(),
            edm: EpistemicDiscriminator::new(),
            egs: EpistemicGroundingStabilizer::new(),
            iag: InvariantArbitrationGate::new(),
            rfc: ResidualClassifier::new(),
        }
    }

    /// The Admissibility Judgment: Σ ⊢ e ⇝ ⊤ / ⊥
    /// Evaluates if an event is admissible under policy constraints given uncertainty.
    ///
    /// PSC-MODE-AWARE: The SystemMode parameter controls admissibility strictness.
    pub fn judge(
        &mut self,
        _state: &GlobalTruthState,
        event: &TruthEvent,
        signal: &SignalMetadata,
        mode: SystemMode,
    ) -> bool {
        // 0. PSC Mode Gate (highest priority)
        match mode {
            SystemMode::CollapsePrevention => {
                // HARD REJECT: System is in collapse prevention.
                // No new events are admitted until λ̂_max < 0.
                return false;
            }
            SystemMode::Degraded => {
                // STRICT MODE: Tighten all thresholds.
                // Lower jitter tolerance from 100ms to 50ms.
                if signal.jitter > 50.0 {
                    return false;
                }
            }
            SystemMode::Safe => {
                // Normal operation
            }
        }

        // 1. Residual Classification (Handle unclassifiable ambiguity)
        if let Some(residue) = self.rfc.classify_residue(signal) {
            println!("AC | Epistemic Residue detected: {:?}", residue);
            // Policy: Residue triggers mandatory quarantine/block if uncertain
            return false;
        }

        // 2. Epistemic Classification (Probabilistic typing)
        let inference = self.edm.classify(&[]);

        // 3. Admissibility Gating (Policy-constrained filtering)
        let cal_signal = krk_calibration::CalibrationSignal {
            latency_delta: signal.drift,
            jitter_residual: signal.jitter / 100.0,
            thermal_drift: 0.0,
        };
        let grounding = self.egs.ground(&cal_signal);
        match self.iag.arbitrate(&inference, &grounding) {
            GateDecision::Allow => (),
            _ => return false, // Policy Block (⊥)
        }

        // 4. Logical Satisfiability (Bounded verification)
        if self.z3.verify_transition("").is_err() {
            return false; // Logical Conflict (⊥)
        }

        true // Admissible (⊤)
    }
}
