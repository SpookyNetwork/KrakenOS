use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofLineage {
    pub chain: Vec<[u8; 32]>,
}
