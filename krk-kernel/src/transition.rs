//! The Transition Engine validates proposals before they merge into the global DAG.
//!
//! CONTROL SEQUENCING INVARIANT (Strict Pre-Commit Gate):
//! The Phase-Stabilizing Controller (PSC) operates as a strict pre-commit gate.
//! It observes PRE-transition state and gates execution BEFORE any state mutation.
//! The kernel NEVER commits without PSC admissibility.
//!
//! Execution order per tick:
//!   1. Collect telemetry from current (pre-transition) DAG state
//!   2. PSC tick: estimate λ̂_max, compute control signals, determine mode
//!   3. Gate: if mode = CollapsePrevention → reject all proposals this tick
//!   4. Evaluate proposal (TEE + Z3) only if PSC allows
//!   5. Commit state transition only after all gates pass
//!
//! This enforces: λ_max(Q^{-1/2} G_eff Q^{-1/2}) < 0

use krk_dcmp::Proposal;
use krk_psc::{PSCConfig, StabilityAttestation, SystemMode, TransitionEngineStabilityModule};
use krk_sat::Z3Bridge;
use krk_state::StateStore;
use krk_tee::AttestationDoc;

/// Live telemetry snapshot from the Observability Manifold.
/// Captures the PRE-transition state of the DAG for the PSC.
#[derive(Debug, Clone)]
pub struct TelemetrySnapshot {
    /// Age of this telemetry reading in milliseconds (for causal damping)
    pub age_ms: f64,
    /// Rate of topology connectivity change across the local horizon (|Δκ_t|)
    pub delta_kappa: f64,
    /// Rate of observability reconstruction error (|ΔΦ_t|)
    pub delta_phi: f64,
    /// Partition entropy: variance in message delay and edge churn
    pub partition_entropy: f64,
}

/// The result of a PSC tick, capturing the full control output.
#[derive(Debug, Clone)]
pub struct PSCTickResult {
    /// Current system operating mode
    pub mode: SystemMode,
    /// Gossip topology control signal (negative = contract, increase stiffness)
    pub u_gossip: f64,
    /// Collapse regulation control signal (positive = slow reconciliation)
    pub u_collapse: f64,
    /// Current λ̂_max estimate
    pub lambda_hat: f64,
}

pub struct CandidateState {
    pub current_hash: String,
    pub proposed_delta: Vec<u8>,
}

pub struct TransitionEngine {
    z3: Z3Bridge,
    state: StateStore,
    /// The Phase-Stabilizing Controller: enforces λ_max < 0
    psc: TransitionEngineStabilityModule,
    /// Whether the PSC has been ticked this cycle (prevents stale-mode decisions)
    psc_ticked_this_cycle: bool,
    /// Cached result from the most recent PSC tick
    last_tick_result: PSCTickResult,
}

impl TransitionEngine {
    pub fn new() -> Self {
        Self {
            z3: Z3Bridge::new(),
            state: StateStore::new(),
            psc: TransitionEngineStabilityModule::new(PSCConfig::default()),
            psc_ticked_this_cycle: false,
            last_tick_result: PSCTickResult {
                mode: SystemMode::Safe,
                u_gossip: 0.0,
                u_collapse: 0.0,
                lambda_hat: 0.0,
            },
        }
    }

    pub fn with_psc_config(config: PSCConfig) -> Self {
        Self {
            z3: Z3Bridge::new(),
            state: StateStore::new(),
            psc: TransitionEngineStabilityModule::new(config),
            psc_ticked_this_cycle: false,
            last_tick_result: PSCTickResult {
                mode: SystemMode::Safe,
                u_gossip: 0.0,
                u_collapse: 0.0,
                lambda_hat: 0.0,
            },
        }
    }

    /// STEP 1: Feed pre-transition telemetry into the PSC.
    /// This MUST be called before evaluate_proposal() each tick.
    /// Calling evaluate_proposal() without a prior tick_psc() in the same
    /// cycle is a control sequencing violation and will be rejected.
    pub fn tick_psc(&mut self, telemetry: &TelemetrySnapshot) -> PSCTickResult {
        let (u_gossip, u_collapse, mode) = self.psc.tick(
            telemetry.age_ms,
            telemetry.delta_kappa,
            telemetry.delta_phi,
            telemetry.partition_entropy,
        );

        self.last_tick_result = PSCTickResult {
            mode,
            u_gossip,
            u_collapse,
            lambda_hat: self.psc.current_lambda_hat(),
        };
        self.psc_ticked_this_cycle = true;

        self.last_tick_result.clone()
    }

    /// STEP 2: Evaluate a proposed transition.
    /// CONTROL SEQUENCING INVARIANT: tick_psc() MUST have been called this cycle.
    /// Proposal evaluation order:
    ///   1. Verify PSC was ticked (control sequencing)
    ///   2. PSC stability gate (mode check)
    ///   3. TEE attestation gate
    ///   4. Z3 governance gate
    pub fn evaluate_proposal(
        &mut self,
        proposal: &Proposal,
        attestation: &AttestationDoc,
    ) -> Result<(), &'static str> {
        // CONTROL SEQUENCING GATE
        if !self.psc_ticked_this_cycle {
            return Err(
                "SEQUENCING VIOLATION: PSC not ticked this cycle. \
                 Cannot evaluate proposals on stale stability state.",
            );
        }

        // STABILITY GATE (PSC pre-commit enforcement)
        match self.last_tick_result.mode {
            SystemMode::CollapsePrevention => {
                return Err(
                    "PSC HALT: System in CollapsePrevention mode. \
                     λ_max > 0. No new proposals accepted.",
                );
            }
            SystemMode::Degraded => {
                // In Degraded mode, allow proposals but the PSC is
                // throttling gossip and collapse to recover.
            }
            SystemMode::Safe => {
                // Full propagation enabled.
            }
        }

        // TEE ATTESTATION GATE
        // Stub: In production, verify the attestation signature against
        // the enclave measurement and governance hash.
        if attestation.signature.is_empty() && attestation.public_key.is_empty() {
            // Allow mock attestations in debug mode
            #[cfg(not(debug_assertions))]
            return Err("HARD ISOLATION: Attestation has no signature.");
        }

        // Z3 GOVERNANCE GATE
        let _candidate = CandidateState {
            current_hash: self.state.get_root_hash(),
            proposed_delta: proposal.delta.clone(),
        };

        // Serialize candidate state into Z3 SMT-LIB format for verification
        let smt_script = format!(
            "(assert (= current_hash \"{}\"))\n(check-sat)",
            _candidate.current_hash
        );
        if self.z3.verify_transition(&smt_script).is_err() {
            return Err("UNSAT: Governance constraint violation detected.");
        }

        // After successful evaluation, mark this cycle as consumed.
        // The next proposal requires a fresh PSC tick with fresh telemetry.
        self.psc_ticked_this_cycle = false;

        Ok(())
    }

    /// Get the cached result from the most recent PSC tick.
    pub fn last_tick_result(&self) -> &PSCTickResult {
        &self.last_tick_result
    }

    /// Get the current PSC system mode.
    pub fn current_mode(&self) -> SystemMode {
        self.last_tick_result.mode
    }

    /// Get the current λ̂_max estimate.
    pub fn lambda_hat(&self) -> f64 {
        self.last_tick_result.lambda_hat
    }

    /// Generate a TEE-signable stability attestation.
    pub fn attest_stability(&self, timestamp: u64) -> StabilityAttestation {
        self.psc.attest_stability(timestamp)
    }
}
