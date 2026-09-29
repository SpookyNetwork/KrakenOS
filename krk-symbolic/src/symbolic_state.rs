//! Symbolic Pressure Classes
//!
//! Collapses millions of concrete states into symbolic equivalence classes
//! to prevent adversarial combinatorial infinity.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SymbolicPressure {
    Safe,
    Elevated,
    Critical,
    Exhausted,
}

impl SymbolicPressure {
    pub fn from_concrete(latency: u64, threshold: u64) -> Self {
        if latency >= threshold * 2 {
            SymbolicPressure::Exhausted
        } else if latency >= threshold {
            SymbolicPressure::Critical
        } else if latency >= threshold / 2 {
            SymbolicPressure::Elevated
        } else {
            SymbolicPressure::Safe
        }
    }
}
