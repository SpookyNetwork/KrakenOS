use crate::capabilities::CapabilityScope;
use crate::budget::ExecutionBudget;
use krk_doctrine::runtime::RuntimePolicyHandle;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Capability violation: {0}")]
    CapabilityViolation(String),
    #[error("Doctrine blocked action: {0}")]
    DoctrineBlocked(String),
    #[error("Budget exceeded: {0}")]
    BudgetExceeded(String),
    #[error("Sandbox initialization failed: {0}")]
    InitFailed(String),
}

#[derive(Debug, Clone)]
pub struct DoctrineGuard {
    policy: RuntimePolicyHandle,
}

impl DoctrineGuard {
    pub fn new(policy: RuntimePolicyHandle) -> Self {
        Self { policy }
    }

    pub fn validate_action(&self, action: &str) -> Result<(), SandboxError> {
        if self.policy.allows(action) {
            Ok(())
        } else {
            Err(SandboxError::DoctrineBlocked(action.to_string()))
        }
    }
}

/// An immutable, signed envelope carrying execution permissions and budgets.
#[derive(Debug, Clone)]
pub struct ExecutionEnvelope {
    pub task_id: String,
    pub scope: CapabilityScope,
    pub budget: ExecutionBudget,
    pub doctrine: Option<DoctrineGuard>,
}

impl ExecutionEnvelope {
    pub fn new(task_id: String, scope: CapabilityScope, budget: ExecutionBudget) -> Self {
        Self {
            task_id,
            scope,
            budget,
            doctrine: None,
        }
    }

    pub fn with_doctrine(
        task_id: String,
        scope: CapabilityScope,
        budget: ExecutionBudget,
        doctrine: DoctrineGuard,
    ) -> Self {
        Self {
            task_id,
            scope,
            budget,
            doctrine: Some(doctrine),
        }
    }

    pub fn assert_action_allowed(&self, action: &str) -> Result<(), SandboxError> {
        if let Some(doctrine) = &self.doctrine {
            doctrine.validate_action(action)?;
        }
        Ok(())
    }

    /// Agent calls this during execution to assert it's allowed to perform a network call.
    pub fn assert_network_egress(&self, domain: &str) -> Result<(), SandboxError> {
        self.assert_action_allowed("execution:external")?;
        if !self.scope.has_network_egress(domain) {
            return Err(SandboxError::CapabilityViolation(format!(
                "Network egress to {} is denied in current capability scope.",
                domain
            )));
        }
        Ok(())
    }

    /// Agent calls this when network egress must satisfy both doctrine and capability scope.
    pub fn assert_network_egress_with_doctrine(
        &self,
        domain: &str,
        doctrine: &DoctrineGuard,
    ) -> Result<(), SandboxError> {
        doctrine.validate_action("execution:external")?;
        if !self.scope.has_network_egress(domain) {
            return Err(SandboxError::CapabilityViolation(format!(
                "Network egress to {} is denied in current capability scope.",
                domain
            )));
        }
        Ok(())
    }

    /// Agent calls this after LLM operations to record token consumption.
    pub fn consume_tokens(&mut self, amount: usize) -> Result<(), SandboxError> {
        self.budget.consume_tokens(amount)
            .map_err(|e| SandboxError::BudgetExceeded(e.to_string()))
    }
}

pub struct SandboxManager;

impl SandboxManager {
    /// Grants a new execution envelope to an agent requesting to perform a task.
    /// This is typically called by the `krk-control-plane` after policy arbitration.
    pub fn provision_envelope(task_id: &str, max_tokens: usize, max_ttl_ms: u64) -> ExecutionEnvelope {
        // By default, grant strict isolation until dynamically requested otherwise.
        let scope = CapabilityScope::new_strict_isolation();
        let budget = ExecutionBudget::new(max_tokens, max_ttl_ms);
        ExecutionEnvelope::new(task_id.to_string(), scope, budget)
    }

    pub fn provision_envelope_with_doctrine(
        task_id: &str,
        max_tokens: usize,
        max_ttl_ms: u64,
        doctrine: DoctrineGuard,
    ) -> ExecutionEnvelope {
        let scope = CapabilityScope::new_strict_isolation();
        let budget = ExecutionBudget::new(max_tokens, max_ttl_ms);
        ExecutionEnvelope::with_doctrine(task_id.to_string(), scope, budget, doctrine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::{NetworkScope, Capability};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_strict_isolation_envelope() {
        let mut env = SandboxManager::provision_envelope("task_001", 1000, 5000);

        // Network should be denied
        assert!(env.assert_network_egress("api.openai.com").is_err());

        // Token budget
        assert!(env.consume_tokens(500).is_ok());
        assert!(env.consume_tokens(600).is_err()); // Exceeds 1000 max
    }

    #[test]
    fn test_time_budget() {
        let env = SandboxManager::provision_envelope("task_002", 1000, 50); // 50ms TTL
        thread::sleep(Duration::from_millis(60));
        assert!(env.budget.check_limits().is_err());
    }

    #[test]
    fn test_doctrine_blocks_network_even_when_capability_scope_allows_it() {
        let raw_yaml = "
system: krk-swarm
policy:
  execution:
    external_calls: forbidden
";
        let parsed = krk_doctrine::parser::yaml::parse_yaml(raw_yaml).unwrap();
        let graph = krk_doctrine::compiler::compile_doctrine(parsed).unwrap();
        let doctrine = DoctrineGuard::new(krk_doctrine::runtime::RuntimePolicyHandle::new(graph));

        let scope = CapabilityScope {
            capabilities: vec![
                Capability::Network(NetworkScope::AllowAll),
            ],
            max_ttl_ms: 5_000,
        };
        let budget = ExecutionBudget::new(1000, 5_000);
        let env = ExecutionEnvelope::with_doctrine("task_003".to_string(), scope, budget, doctrine.clone());

        assert!(env.assert_action_allowed("execution:external").is_err());
        assert!(env.assert_network_egress("api.openai.com").is_err());
        assert!(env.assert_network_egress_with_doctrine("api.openai.com", &doctrine).is_err());
    }

    #[test]
    fn test_doctrine_permits_network_when_policy_and_capability_scope_agree() {
        let raw_yaml = "
system: krk-swarm
policy:
  execution:
    external_calls: allowed
";
        let parsed = krk_doctrine::parser::yaml::parse_yaml(raw_yaml).unwrap();
        let graph = krk_doctrine::compiler::compile_doctrine(parsed).unwrap();
        let doctrine = DoctrineGuard::new(krk_doctrine::runtime::RuntimePolicyHandle::new(graph));

        let scope = CapabilityScope {
            capabilities: vec![
                Capability::Network(NetworkScope::AllowEgress(vec!["api.openai.com".to_string()])),
            ],
            max_ttl_ms: 5_000,
        };
        let budget = ExecutionBudget::new(1000, 5_000);
        let env = ExecutionEnvelope::with_doctrine("task_004".to_string(), scope, budget, doctrine);

        assert!(env.assert_action_allowed("execution:external").is_ok());
        assert!(env.assert_network_egress("api.openai.com").is_ok());
    }
}
