//! Byzantine Admissibility Result
//!
//! Formalizes hostile-world admissibility semantics.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ByzantineResult {
    /// Logically and physically valid under all constraints.
    Admissible,
    /// Logically impossible state.
    Unsat,
    /// Admissible but conflicting/unresolved trajectory.
    Divergent,
    /// Malicious admissibility attempt detected.
    Byzantine,
    /// Contaminated proof lineage detected.
    Poisoned,
    /// Conflicting causal history detected (locally SAT, globally impossible).
    TemporalFork,
}
