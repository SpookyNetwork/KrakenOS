use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalForkWitness {
    pub fork_point: u64,
    pub branch_hashes: Vec<[u8; 32]>,
}
