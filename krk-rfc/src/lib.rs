//! KRK Residual Failure Classifier (RFC)
//!
//! Handles "Unclassifiable Epistemic Residue": Ambiguous states that cannot
//! yet be classified but must still be acted upon.

use serde::{Deserialize, Serialize};
use krk_ets::SignalMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EpistemicResidue {
    /// Partial packet corruption or malformed frame
    SignalResidue(String),
    /// Quorum view inconsistency without consensus violation
    QuorumDivergence { detected_views: usize },
    /// Clock skew detected without causal violation
    TemporalResidue { skew_ms: i64 },
    /// Ambiguous state requiring speculative quarantine
    Ambiguous(String),
}

pub struct ResidualClassifier {}

impl ResidualClassifier {
    pub fn new() -> Self {
        Self {}
    }

    /// Classifies ambiguous signals into actionable residue types.
    pub fn classify_residue(&self, signal: &SignalMetadata) -> Option<EpistemicResidue> {
        if signal.jitter > 100.0 {
            return Some(EpistemicResidue::SignalResidue("High Jitter Residue".to_string()));
        }

        if signal.drift.abs() > 0.05 {
             return Some(EpistemicResidue::TemporalResidue { skew_ms: (signal.drift * 1000.0) as i64 });
        }

        None
    }
}
