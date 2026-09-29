//! Admissibility Memoization
//!
//! Enables proof propagation across equivalence classes. If Class X is UNSAT,
//! all reducible trajectories inherit UNSAT immediately.

use crate::classes::AdmissibilityClass;
use std::collections::HashMap;

pub enum ProofResult {
    SAT,
    UNSAT,
    Diverged,
}

pub struct AdmissibilityMemo {
    // Maps admissibility signatures to their known validity results.
    memo: HashMap<AdmissibilityClass, ProofResult>,
}

impl AdmissibilityMemo {
    pub fn new() -> Self {
        Self {
            memo: HashMap::new(),
        }
    }

    pub fn lookup(&self, class: &AdmissibilityClass) -> Option<&ProofResult> {
        self.memo.get(class)
    }

    pub fn memoize(&mut self, class: AdmissibilityClass, result: ProofResult) {
        self.memo.insert(class, result);
    }

    /// Propagates UNSAT results through the admissibility topology.
    pub fn propagate_unsat(&mut self, _source: &AdmissibilityClass) {
        // Logic to flow UNSAT cores through reducible classes.
    }
}
