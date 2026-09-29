//! KRK Effect Algebra
//!
//! Formalizes the substrate as a Commutative Monoid over a partially
//! ordered event log. State is a fold over idempotent effects.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[cfg(test)]
pub mod properties;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CausalID(pub [u8; 32]);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub payload_hash: [u8; 32],
    pub provenance: [u8; 32],
    pub dependencies: Vec<CausalID>,
}

impl Effect {
    /// The equivalence relation (~) over effects.
    ///
    /// CRITICAL DESIGN DECISION (discovered via Wind Tunnel Run 1):
    /// Equivalence is defined over PAYLOAD + DEPENDENCIES only.
    /// Provenance is explicitly EXCLUDED from the identity hash.
    ///
    /// This enables delayed equivalence discovery:
    ///   Two effects with identical payloads but different origins
    ///   will produce the SAME CausalID and collapse via idempotent projection.
    ///
    /// Provenance is retained on the Effect struct for forensic reconstruction
    /// but does not participate in the equivalence relation.
    pub fn causal_id(&self) -> CausalID {
        let mut hasher = Sha256::new();
        hasher.update(self.payload_hash);
        // NOTE: provenance is intentionally EXCLUDED from identity.
        // This is the architectural pivot that enables ambiguity metabolism.
        for dep in &self.dependencies {
            hasher.update(dep.0);
        }
        CausalID(hasher.finalize().into())
    }
}

/// The Commutative Monoid State Structure (Σ)
#[derive(Debug, Clone, Default)]
pub struct SubstrateState {
    pub projected_effects: std::collections::HashSet<CausalID>,
}

impl SubstrateState {
    pub fn new() -> Self {
        Self::default()
    }

    /// The Commutative Monoid Operation: Σ ⊕ e
    /// This is the "fold" that ensures convergence over equivalent event sets.
    pub fn reduce(&mut self, effect: &Effect) {
        let id = effect.causal_id();

        // Idempotence: Σ ⊕ e ⊕ e = Σ ⊕ e
        if !self.projected_effects.contains(&id) {
            self.projected_effects.insert(id);
        }
        // Idempotent: duplicate effects are silently absorbed.
    }

    /// Global Convergence Check
    /// If two states have the same projected_effects, they are semantically identical.
    pub fn is_equivalent(&self, other: &Self) -> bool {
        self.projected_effects == other.projected_effects
    }
}
