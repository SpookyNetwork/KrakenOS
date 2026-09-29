use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofEvent {
    pub hash: [u8; 32],
    pub parents: Vec<[u8; 32]>,
}

pub struct ProofGraph {
    pub events: Vec<ProofEvent>,
}
