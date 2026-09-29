//! Admissibility Reduction Index
//!
//! Provides indexed lookup for admissibility class membership,
//! enabling O(1) classification of events against the quotient structure.

use std::collections::HashMap;
use crate::classes::AdmissibilityClass;

/// Index mapping payload hashes to their admissibility class.
pub struct ReductionIndex {
    index: HashMap<[u8; 32], AdmissibilityClass>,
}

impl ReductionIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    pub fn insert(&mut self, hash: [u8; 32], class: AdmissibilityClass) {
        self.index.insert(hash, class);
    }

    pub fn lookup(&self, hash: &[u8; 32]) -> Option<&AdmissibilityClass> {
        self.index.get(hash)
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
}
