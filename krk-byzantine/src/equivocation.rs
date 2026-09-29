use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivocationWitness {
    pub node_id: [u8; 32],
    pub conflicting_hashes: Vec<[u8; 32]>,
}
