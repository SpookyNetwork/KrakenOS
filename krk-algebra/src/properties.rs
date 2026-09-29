use crate::{Effect, SubstrateState, CausalID};
use proptest::prelude::*;

// Strategy to generate arbitrary 32-byte arrays
prop_compose! {
    fn arb_bytes32()(bytes in any::<[u8; 32]>()) -> [u8; 32] {
        bytes
    }
}

// Strategy to generate an arbitrary Effect
prop_compose! {
    fn arb_effect()(
        payload in arb_bytes32(),
        provenance in arb_bytes32(),
        // Just testing with 0 or 1 dependencies for simplicity of property testing
        deps in proptest::collection::vec(arb_bytes32(), 0..2)
    ) -> Effect {
        Effect {
            payload_hash: payload,
            provenance,
            dependencies: deps.into_iter().map(CausalID).collect(),
        }
    }
}

proptest! {
    /// 1. Idempotence: Σ ⊕ e ⊕ e = Σ ⊕ e
    #[test]
    fn test_idempotence(e in arb_effect()) {
        let mut state1 = SubstrateState::new();
        state1.reduce(&e);

        let mut state2 = state1.clone();
        state2.reduce(&e); // Reduce again

        assert!(state1.is_equivalent(&state2));
    }

    /// 2. Commutativity: Σ ⊕ a ⊕ b = Σ ⊕ b ⊕ a
    #[test]
    fn test_commutativity(e1 in arb_effect(), e2 in arb_effect()) {
        let mut state_ab = SubstrateState::new();
        state_ab.reduce(&e1);
        state_ab.reduce(&e2);

        let mut state_ba = SubstrateState::new();
        state_ba.reduce(&e2);
        state_ba.reduce(&e1);

        assert!(state_ab.is_equivalent(&state_ba));
    }

    /// 3. Associativity (with sets of effects)
    /// (Σ ⊕ A) ⊕ B = Σ ⊕ (A ∪ B)
    #[test]
    fn test_associativity(
        set1 in proptest::collection::vec(arb_effect(), 0..5),
        set2 in proptest::collection::vec(arb_effect(), 0..5)
    ) {
        let mut state_left = SubstrateState::new();
        for e in &set1 { state_left.reduce(e); }
        for e in &set2 { state_left.reduce(e); }

        let mut state_right = SubstrateState::new();
        let mut combined = set1.clone();
        combined.extend(set2);
        for e in combined {
            state_right.reduce(&e);
        }

        assert!(state_left.is_equivalent(&state_right));
    }

    /// 4. Provenance Independence (Equivalence discovery)
    /// If payload and deps match, but provenance differs, they still collapse.
    #[test]
    fn test_provenance_independence(
        payload in arb_bytes32(),
        prov1 in arb_bytes32(),
        prov2 in arb_bytes32()
    ) {
        let e1 = Effect {
            payload_hash: payload,
            provenance: prov1,
            dependencies: vec![],
        };
        let e2 = Effect {
            payload_hash: payload,
            provenance: prov2,
            dependencies: vec![],
        };

        // Same causal ID despite different provenance
        assert_eq!(e1.causal_id(), e2.causal_id());

        let mut state = SubstrateState::new();
        state.reduce(&e1);
        state.reduce(&e2);

        // State should only contain ONE effect, because the second collapsed into the first.
        assert_eq!(state.projected_effects.len(), 1);
    }
}
