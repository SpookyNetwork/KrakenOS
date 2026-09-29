//! KRK External Truth Semantics (ETS)
//!
//! Defines the epistemic semantics of external systems.
//! Replaces binary success/failure with confidence-weighted distributions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EpistemicConfidence {
    /// Absolute certainty (reserved for internal proofs)
    Absolute,
    /// High probability (verified by quorum)
    High,
    /// Nominal confidence (standard operation)
    Nominal,
    /// Uncertain (partial failure or jitter)
    Uncertain,
    /// No confidence (silent failure or timeout)
    Zero,
}

/// Physical Signal Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMetadata {
    pub jitter: f64,
    pub drift: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExternalTruth<T> {
    Confirmed(T, EpistemicConfidence, SignalMetadata),
    Partial(T, EpistemicConfidence, SignalMetadata),
    Unknown(EpistemicConfidence, SignalMetadata),
    Conflict(T, T, EpistemicConfidence, SignalMetadata),
    TimedOut,
}

/// Specialized Z3 Truth States
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Z3Truth {
    Sat,
    Unsat,
    Unknown,
    TimeoutUncertain,
}

/// Specialized Financial (Stripe) Truth States
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinancialTruth {
    Settled,
    Pending,
    Escrowed,
    Reverted,
    Inconsistent,
}
