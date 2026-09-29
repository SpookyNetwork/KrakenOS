//! Memory Index — Content-addressed lookup
//!
//! Provides fast key-based indexing over memory records.

use std::collections::HashMap;

/// A content-addressed index mapping hashes to record positions.
pub struct MemoryIndex {
    hash_to_position: HashMap<[u8; 32], Vec<usize>>,
}

impl MemoryIndex {
    pub fn new() -> Self {
        Self { hash_to_position: HashMap::new() }
    }

    /// Register a record at a given position.
    pub fn insert(&mut self, content_hash: [u8; 32], position: usize) {
        self.hash_to_position.entry(content_hash).or_default().push(position);
    }

    /// Lookup positions by content hash.
    pub fn lookup(&self, content_hash: &[u8; 32]) -> Option<&Vec<usize>> {
        self.hash_to_position.get(content_hash)
    }

    /// Total indexed entries.
    pub fn len(&self) -> usize {
        self.hash_to_position.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.hash_to_position.is_empty()
    }
}
