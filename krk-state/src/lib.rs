//! Merkle CRDT DAG (Hot Layer Storage)
//!
//! Replaces mutable replicated state with admissible immutable state evolution.

use blake3::Hash;
use krk_tee::AttestationDoc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateNode {
    /// Blake3 Merkle Root of this node
    pub node_hash: Hash,

    /// Merkle DAG Parents
    pub parents: Vec<Hash>,

    pub epoch: u64,
    pub hlc: u128,

    /// Execution constraint inputs
    pub transition_hash: Hash,
    pub state_hash: Hash,

    /// Z3 SAT proof binding
    pub proof_hash: Hash,

    /// Absolute governance binding
    pub governance_hash: Hash,

    /// Hardware-rooted attestations approving this DAG segment
    pub attestations: Vec<AttestationDoc>,

    /// CRDT Delta (OR-Set, LWW-Register, Map Lattice)
    pub payload: Vec<u8>,
}

pub struct StateStore {
    // Under the hood: rocksdb::DB, column families for DAG segments, proofs, etc.
}

impl StateStore {
    pub fn new() -> Self { Self {} }

    pub fn get_root_hash(&self) -> String {
        "00000000000000000000000000000000".to_string()
    }
}
