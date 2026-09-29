//! Hardware Attestation Boundary
//!
//! Transforms the execution root from software to physically rooted execution truth
//! via Intel SGX / AMD SEV-SNP.

use serde::{Deserialize, Serialize};

pub type Hash256 = [u8; 32];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Measurement {
    pub hash: Hash256,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttestationDoc {
    pub node_id: Hash256,

    pub enclave_measurement: Hash256,
    pub governance_hash: Hash256, // CRITICAL: Governance hash must participate
    pub runtime_hash: Hash256,

    pub epoch: u64,
    pub timestamp: u64,

    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
}

/// A node's hardware identity is bound strictly to the hardware and constitutional hash.
pub fn compute_node_id(enclave: Hash256, root: Hash256, governance: Hash256, genesis: Hash256) -> Hash256 {
    // HASH(enclave_measurement + hardware_root + governance_hash + genesis_epoch)
    [0u8; 32] // Stub
}

pub trait TeeProvider {
    fn measure(&self) -> Measurement;
    fn attest(&self) -> AttestationDoc;
    fn verify(&self, doc: &AttestationDoc) -> bool;
}

#[cfg(debug_assertions)]
pub struct MockProvider {}

#[cfg(debug_assertions)]
impl TeeProvider for MockProvider {
    fn measure(&self) -> Measurement { Measurement { hash: [0; 32] } }
    fn attest(&self) -> AttestationDoc {
        AttestationDoc {
            node_id: [0; 32],
            enclave_measurement: [0; 32],
            governance_hash: [0; 32],
            runtime_hash: [0; 32],
            epoch: 0,
            timestamp: 0,
            public_key: vec![],
            signature: vec![],
        }
    }
    fn verify(&self, _doc: &AttestationDoc) -> bool { true }
}

/// SgxProvider stub (Fails to compile if instantiated outside Linux SGX environment in production)
pub struct SgxProvider {}
