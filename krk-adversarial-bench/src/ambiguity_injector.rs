//! Ambiguity Injector
//!
//! Generates conflicting admissible effects, equivalent delayed effects,
//! replay storms, and branch-amplifying workloads.
//! All injected events are locally admissible, causally plausible, and policy-valid.

use krk_algebra::Effect;
use rand::Rng;

pub struct AmbiguityInjector;

impl AmbiguityInjector {
    /// Generate an effect with a specific payload but unique provenance.
    /// This creates latent equivalence ambiguity: same intent, different origin.
    pub fn generate_divergent_admissible_effect(&self, payload_seed: &str) -> Effect {
        let mut rng = rand::thread_rng();
        let mut payload_hash = [0u8; 32];
        for (i, b) in payload_seed.as_bytes().iter().enumerate() {
            if i < 32 { payload_hash[i] = *b; }
        }
        let mut provenance = [0u8; 32];
        rng.fill(&mut provenance);
        Effect {
            payload_hash,
            provenance,
            dependencies: vec![],
        }
    }

    /// Generate a batch of effects that are semantically equivalent
    /// but have distinct provenance — simulating a delayed equivalence storm
    pub fn generate_equivalence_storm(&self, payload_seed: &str, count: usize) -> Vec<Effect> {
        (0..count).map(|_| self.generate_divergent_admissible_effect(payload_seed)).collect()
    }

    /// Generate conflicting effects: same domain, incompatible payloads.
    /// These create genuine branch divergence requiring FORK semantics.
    pub fn generate_conflicting_effects(&self, count: usize) -> Vec<Effect> {
        let mut rng = rand::thread_rng();
        (0..count).map(|i| {
            let mut payload_hash = [0u8; 32];
            payload_hash[0] = i as u8;
            rng.fill(&mut payload_hash[1..]);
            let mut provenance = [0u8; 32];
            rng.fill(&mut provenance);
            Effect {
                payload_hash,
                provenance,
                dependencies: vec![],
            }
        }).collect()
    }

    /// Generate replay copies of an existing effect — tests idempotent stability
    pub fn generate_replay_copies(&self, original: &Effect, copies: usize) -> Vec<Effect> {
        (0..copies).map(|_| original.clone()).collect()
    }
}
