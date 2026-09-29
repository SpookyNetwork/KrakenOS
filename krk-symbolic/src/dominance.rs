//! Symbolic Dominance Pruning
//!
//! Formally bounds adversarial search space by discarding trajectories
//! that are strictly worse than already-solved counterexamples.

pub struct Trajectory {
    pub latency: u64,
    pub memory_usage: u8, // 0-100
    pub clock_skew: u64,
}

pub trait DominanceRelation {
    /// Returns true if `a` dominates `b` (is "worse" or more constrained).
    /// If B SAT-fails, and B dominates A, A cannot produce a more dangerous contradiction.
    fn dominates(a: &Trajectory, b: &Trajectory) -> bool;
}

pub struct PressureDominance;

impl DominanceRelation for PressureDominance {
    fn dominates(a: &Trajectory, b: &Trajectory) -> bool {
        a.latency >= b.latency &&
        a.memory_usage >= b.memory_usage &&
        a.clock_skew >= b.clock_skew
    }
}
