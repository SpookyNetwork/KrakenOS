//! Exploration Frontier Mapping
//!
//! Maps the progress through admissibility space, distinguishing between
//! explored, unstable, unresolved, and forbidden regions.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationFrontier {
    pub explored: HashSet<[u8; 32]>,
    pub unstable: HashSet<[u8; 32]>,
    pub unresolved: HashSet<[u8; 32]>,
    pub forbidden: HashSet<[u8; 32]>,
}

impl ExplorationFrontier {
    pub fn new() -> Self {
        Self {
            explored: HashSet::new(),
            unstable: HashSet::new(),
            unresolved: HashSet::new(),
            forbidden: HashSet::new(),
        }
    }

    /// Marks a region as explored.
    pub fn mark_explored(&mut self, region_hash: [u8; 32]) {
        self.explored.insert(region_hash);
    }
}
