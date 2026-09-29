//! KRK Safety Kernel (Pure Projection Operator)
//!
//! A deterministic projection function over already-validated events.
//! Apply(Σ, e): Σ' iff KRK-AC(Σ, e) = ⊤

use krk_ac::{AdmissibilityPolicyEngine, GlobalTruthState};
use krk_strl::TruthEvent;
use krk_ets::SignalMetadata;
use krk_psc::SystemMode;
use krk_quotient::QuotientLattice;

pub struct SafetyKernel {
    pub state: GlobalTruthState,
    pub quotient: QuotientLattice,
    pub policy_engine: AdmissibilityPolicyEngine,
    pub current_mode: SystemMode,
}

impl SafetyKernel {
    pub fn new() -> Self {
        Self {
            state: GlobalTruthState {
                history_dag: Vec::new(),
                constraints: Vec::new(),
                witnesses: std::collections::HashMap::new(),
            },
            quotient: QuotientLattice::new(),
            policy_engine: AdmissibilityPolicyEngine::new(),
            current_mode: SystemMode::Safe,
        }
    }

    /// Update the PSC mode (called by the control loop).
    pub fn set_mode(&mut self, mode: SystemMode) {
        self.current_mode = mode;
    }

    /// The Algebraic Projection Operation: Σ' = Σ ⊕ e
    /// Enforces Idempotence and Commutativity where applicable.
    /// PSC-mode-aware: passes current system mode to admissibility engine.
    pub fn step(&mut self, event: TruthEvent, signal: SignalMetadata) {
        // 1. ADMISSIBILITY (Policy Filter, PSC-mode-aware)
        if self.policy_engine.judge(&self.state, &event, &signal, self.current_mode) {
            // 2. RECONCILIATION (Idempotent Projection)
            self.apply(event);
        } else {
            // 3. QUARANTINE
            println!("KERNEL | BLOCK: Event failed policy judgment or is epistemically ambiguous.");
        }
    }

    fn apply(&mut self, event: TruthEvent) {
        // Identity check via payload_hash (equivalence relation)
        let effect_id = event.payload_hash;

        if self.state.history_dag.iter().any(|e| e.payload_hash == effect_id) {
            println!("KERNEL | IDEMPOTENT: Effect already projected. Skipping mutation.");
            return;
        }

        // Project New Effect
        self.state.history_dag.push(event);
        println!("KERNEL | COMMIT: New unique effect projected into Σ.");
    }
}
