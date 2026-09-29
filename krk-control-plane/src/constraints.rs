//! Constraints — Hard invariants for control signal bounds
//!
//! Enforces physical and policy constraints on control signals
//! before they reach the kernel.

use crate::ControlSignal;

/// Clamps a control signal to safe operational bounds.
pub fn enforce_bounds(signal: &mut ControlSignal) {
    signal.u_gossip = signal.u_gossip.clamp(-1.0, 1.0);
    signal.u_collapse = signal.u_collapse.clamp(-1.0, 1.0);
}

/// Validates that a signal is within operational constraints.
pub fn is_valid(signal: &ControlSignal) -> bool {
    signal.u_gossip.is_finite()
        && signal.u_collapse.is_finite()
        && signal.u_gossip.abs() <= 1.0
        && signal.u_collapse.abs() <= 1.0
}
