//! Control Plane Policy Engine
//!
//! Implements the concrete policy stack that gates every control signal.
//! Policies are evaluated in priority order; first decisive policy wins.

use crate::*;

/// The default safety policy — prevents blind PSC drift execution.
pub struct SafetyPolicy {
    /// Maximum entropy at which autonomous execution is permitted.
    pub max_autonomous_entropy: f64,
    /// Maximum tolerable telemetry age before escalation.
    pub max_telemetry_age_ms: f64,
}

impl Default for SafetyPolicy {
    fn default() -> Self {
        Self {
            max_autonomous_entropy: 0.5,
            max_telemetry_age_ms: 200.0,
        }
    }
}

impl ControlPolicy for SafetyPolicy {
    fn evaluate(&self, ctx: &ControlContext) -> PolicyDecision {
        // RULE 1: Adversarial regime → escalate to supervised authority.
        if ctx.psc_signal.adversarial_detected {
            return PolicyDecision::Escalate(AuthorityLevel::Supervised);
        }

        // RULE 2: Stale telemetry → reject signal (flying blind).
        if ctx.telemetry_age_ms > self.max_telemetry_age_ms {
            return PolicyDecision::Reject(
                format!("Telemetry age {}ms exceeds {}ms limit",
                    ctx.telemetry_age_ms, self.max_telemetry_age_ms)
            );
        }

        // RULE 3: High entropy → clamp control signals to conservative range.
        if ctx.entropy_level > self.max_autonomous_entropy {
            return PolicyDecision::Modify(ControlSignal {
                u_gossip: 0.0,    // Freeze gossip
                u_collapse: 0.0,  // Freeze collapse
                source: SignalSource::System,
            });
        }

        // RULE 4: CollapsePrevention mode → only system-sourced signals allowed.
        if ctx.psc_signal.mode == SystemMode::CollapsePrevention {
            if let Some(ref req) = ctx.agent_request {
                return PolicyDecision::Reject(
                    format!("Agent {} blocked: system in CollapsePrevention mode", req.agent_id)
                );
            }
        }

        // DEFAULT: Allow with PSC-computed signal.
        PolicyDecision::Allow(ControlSignal {
            u_gossip: 0.0,
            u_collapse: 0.0,
            source: SignalSource::PSC,
        })
    }
}

/// Agent authorization policy — gates which agents can act in which modes.
pub struct AgentAuthorizationPolicy;

impl ControlPolicy for AgentAuthorizationPolicy {
    fn evaluate(&self, ctx: &ControlContext) -> PolicyDecision {
        if let Some(ref req) = ctx.agent_request {
            // Agents with high entropy estimates are quarantined.
            if req.entropy_estimate > 0.8 {
                return PolicyDecision::Reject(
                    format!("Agent {} entropy {} exceeds safety threshold",
                        req.agent_id, req.entropy_estimate)
                );
            }
        }
        // No agent request or agent is within bounds — pass through.
        PolicyDecision::Allow(ControlSignal {
            u_gossip: 0.0,
            u_collapse: 0.0,
            source: SignalSource::System,
        })
    }
}

/// Real-time Doctrine enforcement policy checking Compiled Intent Constraints.
pub struct DoctrineEnforcementPolicy {
    pub doctrine_handle: krk_doctrine::runtime::RuntimePolicyHandle,
}

impl ControlPolicy for DoctrineEnforcementPolicy {
    fn evaluate(&self, ctx: &ControlContext) -> PolicyDecision {
        if let Some(ref req) = ctx.agent_request {
            if req.action_type == "read_memory" && !self.doctrine_handle.allows("memory:read") {
                return PolicyDecision::Reject(format!(
                    "Doctrine Block: agent {} read_memory action blocked by capability rules",
                    req.agent_id
                ));
            }
            if req.action_type == "write_memory" && !self.doctrine_handle.allows("memory:write") {
                return PolicyDecision::Reject(format!(
                    "Doctrine Block: agent {} write_memory action blocked by capability rules",
                    req.agent_id
                ));
            }
            if req.action_type == "external_call" && !self.doctrine_handle.allows("execution:external") {
                return PolicyDecision::Reject(format!(
                    "Doctrine Block: agent {} external call forbidden by policy",
                    req.agent_id
                ));
            }
        }
        PolicyDecision::Allow(ControlSignal {
            u_gossip: 0.0,
            u_collapse: 0.0,
            source: SignalSource::System,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_policy_rejects_stale_telemetry() {
        let policy = SafetyPolicy::default();
        let ctx = ControlContext {
            state_hash: [0u8; 32],
            psc_signal: PSCSignal {
                lambda_hat: -0.1,
                mode: SystemMode::Safe,
                adversarial_detected: false,
                bias_correction: 0.0,
            },
            agent_request: None,
            entropy_level: 0.1,
            telemetry_age_ms: 500.0, // Way too old
        };
        match policy.evaluate(&ctx) {
            PolicyDecision::Reject(reason) => {
                assert!(reason.contains("Telemetry age"));
            }
            _ => panic!("Expected Reject for stale telemetry"),
        }
    }

    #[test]
    fn test_safety_policy_escalates_adversarial() {
        let policy = SafetyPolicy::default();
        let ctx = ControlContext {
            state_hash: [0u8; 32],
            psc_signal: PSCSignal {
                lambda_hat: 0.1,
                mode: SystemMode::CollapsePrevention,
                adversarial_detected: true,
                bias_correction: 0.05,
            },
            agent_request: None,
            entropy_level: 0.1,
            telemetry_age_ms: 10.0,
        };
        match policy.evaluate(&ctx) {
            PolicyDecision::Escalate(AuthorityLevel::Supervised) => (),
            _ => panic!("Expected Escalate for adversarial regime"),
        }
    }

    #[test]
    fn test_safety_policy_allows_normal_operation() {
        let policy = SafetyPolicy::default();
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
        match policy.evaluate(&ctx) {
            PolicyDecision::Allow(_) => (),
            _ => panic!("Expected Allow for normal conditions"),
        }
    }

    #[test]
    fn test_doctrine_compiler_and_enforcement() {
        let raw_yaml = "
system: krk-swarm
policy:
  memory:
    read: allowed
    write: sandboxed
  execution:
    external_calls: forbidden
";
        let parsed = krk_doctrine::parser::yaml::parse_yaml(raw_yaml).unwrap();
        let ir_graph = krk_doctrine::compiler::compile_doctrine(parsed).unwrap();
        let runtime_handle = krk_doctrine::runtime::RuntimePolicyHandle::new(ir_graph);
        let policy = DoctrineEnforcementPolicy {
            doctrine_handle: runtime_handle,
        };

        // Allowed actions should pass
        let allowed_ctx = ControlContext {
            state_hash: [0u8; 32],
            psc_signal: PSCSignal {
                lambda_hat: 0.0,
                mode: SystemMode::Safe,
                adversarial_detected: false,
                bias_correction: 0.0,
            },
            agent_request: Some(AgentRequest {
                agent_id: "agent-1".to_string(),
                action_type: "read_memory".to_string(),
                payload_hash: [0u8; 32],
                entropy_estimate: 0.1,
            }),
            entropy_level: 0.0,
            telemetry_age_ms: 1.0,
        };
        match policy.evaluate(&allowed_ctx) {
            PolicyDecision::Allow(_) => (),
            _ => panic!("Expected Allowed action to succeed"),
        }

        // Forbidden action should be rejected
        let blocked_ctx = ControlContext {
            state_hash: [0u8; 32],
            psc_signal: PSCSignal {
                lambda_hat: 0.0,
                mode: SystemMode::Safe,
                adversarial_detected: false,
                bias_correction: 0.0,
            },
            agent_request: Some(AgentRequest {
                agent_id: "agent-1".to_string(),
                action_type: "external_call".to_string(),
                payload_hash: [0u8; 32],
                entropy_estimate: 0.1,
            }),
            entropy_level: 0.0,
            telemetry_age_ms: 1.0,
        };
        match policy.evaluate(&blocked_ctx) {
            PolicyDecision::Reject(reason) => {
                assert!(reason.contains("forbidden"));
            }
            _ => panic!("Expected Forbidden action to be rejected"),
        }
    }
}
