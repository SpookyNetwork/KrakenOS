//! Control Router — Central Dispatch
//!
//! Routes incoming control contexts through a prioritized policy stack.
//! First decisive policy wins. If all policies pass, the signal is allowed.

use crate::*;

/// A stack-based router that evaluates policies in priority order.
pub struct PolicyStackRouter {
    policies: Vec<Box<dyn ControlPolicy>>,
}

impl PolicyStackRouter {
    pub fn new() -> Self {
        Self { policies: Vec::new() }
    }

    /// Add a policy to the evaluation stack (evaluated in insertion order).
    pub fn add_policy(&mut self, policy: Box<dyn ControlPolicy>) {
        self.policies.push(policy);
    }
}

impl ControlRouter for PolicyStackRouter {
    fn route(&self, ctx: ControlContext) -> PolicyDecision {
        for policy in &self.policies {
            let decision = policy.evaluate(&ctx);
            match &decision {
                PolicyDecision::Allow(_) => continue, // Pass — try next policy
                _ => return decision,                 // Reject/Modify/Escalate — stop
            }
        }
        // All policies passed — allow with system source.
        PolicyDecision::Allow(ControlSignal {
            u_gossip: 0.0,
            u_collapse: 0.0,
            source: SignalSource::System,
        })
    }
}
