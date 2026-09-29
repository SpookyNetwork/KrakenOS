//! Arbitration — Conflict resolution between competing control signals
//!
//! When multiple agents or PSC channels propose conflicting actions,
//! the arbitrator selects the safest or highest-authority action.

use crate::{ControlSignal, SignalSource};

/// Arbitrates between two competing control signals.
/// Safety-first: PSC signals always override agent signals.
/// Among equal sources, the more conservative signal wins.
pub fn arbitrate(a: &ControlSignal, b: &ControlSignal) -> ControlSignal {
    // PSC always wins over agents.
    if a.source == SignalSource::PSC && b.source != SignalSource::PSC {
        return a.clone();
    }
    if b.source == SignalSource::PSC && a.source != SignalSource::PSC {
        return b.clone();
    }
    // Among equal sources, choose more conservative (smaller magnitude).
    let mag_a = a.u_gossip.abs() + a.u_collapse.abs();
    let mag_b = b.u_gossip.abs() + b.u_collapse.abs();
    if mag_a <= mag_b { a.clone() } else { b.clone() }
}
