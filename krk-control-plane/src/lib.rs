//! KRK Control Plane — Decision + Policy Runtime Layer
//!
//! The arbitration brain between PSC control signals and OS execution.
//! This layer does NOT compute physics or state — it decides:
//! - What actions are allowed
//! - What agents are authorized
//! - What control signals are valid
//! - How to resolve conflicting PSC outputs
//!
//! Core invariant: PSC regulates, control-plane authorizes, kernel executes.
//! No control signal reaches the kernel without passing through policy evaluation.

pub mod policy;
pub mod router;
pub mod scheduler;
pub mod authority;
pub mod arbitration;
pub mod constraints;

use krk_psc::SystemMode;
use serde::{Deserialize, Serialize};

// ─── CORE DOMAIN TYPES ───

/// A control signal proposed by the PSC or an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlSignal {
    pub u_gossip: f64,
    pub u_collapse: f64,
    pub source: SignalSource,
}

/// Where the signal originated — critical for trust evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalSource {
    PSC,
    Agent(String),
    Operator,
    System,
}

/// An action request from an agent in the swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    pub agent_id: String,
    pub action_type: String,
    pub payload_hash: [u8; 32],
    pub entropy_estimate: f64,
}

/// The PSC signal snapshot at decision time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PSCSignal {
    pub lambda_hat: f64,
    pub mode: SystemMode,
    pub adversarial_detected: bool,
    pub bias_correction: f64,
}

/// Full context for a policy evaluation decision.
#[derive(Debug, Clone)]
pub struct ControlContext {
    pub state_hash: [u8; 32],
    pub psc_signal: PSCSignal,
    pub agent_request: Option<AgentRequest>,
    pub entropy_level: f64,
    pub telemetry_age_ms: f64,
}

/// Authority levels for escalation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorityLevel {
    /// Autonomous — no human required.
    Autonomous,
    /// Supervised — human notified, auto-proceed after timeout.
    Supervised,
    /// Gated — human approval required before execution.
    Gated,
    /// Constitutional — requires governance key signature.
    Constitutional,
}

/// The output of policy evaluation.
#[derive(Debug, Clone)]
pub enum PolicyDecision {
    /// Signal is safe and authorized — execute as-is.
    Allow(ControlSignal),
    /// Signal is modified (clamped, attenuated) before execution.
    Modify(ControlSignal),
    /// Signal is rejected with reason.
    Reject(String),
    /// Signal requires escalation to a higher authority.
    Escalate(AuthorityLevel),
}

// ─── CORE TRAITS ───

/// Policy evaluator trait — the fundamental decision interface.
pub trait ControlPolicy {
    fn evaluate(&self, ctx: &ControlContext) -> PolicyDecision;
}

/// The central dispatcher routing decisions through the policy stack.
pub trait ControlRouter {
    fn route(&self, ctx: ControlContext) -> PolicyDecision;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_context_construction() {
        let ctx = ControlContext {
            state_hash: [0u8; 32],
            psc_signal: PSCSignal {
                lambda_hat: -0.1,
                mode: SystemMode::Safe,
                adversarial_detected: false,
                bias_correction: 0.0,
            },
            agent_request: None,
            entropy_level: 0.05,
            telemetry_age_ms: 10.0,
        };
        assert!(ctx.entropy_level < 0.1);
    }
}
