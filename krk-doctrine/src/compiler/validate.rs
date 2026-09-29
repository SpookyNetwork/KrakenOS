use crate::ast::policy::DoctrinePolicy;
use crate::error::DoctrineError;

pub fn validate_policy(policy: &DoctrinePolicy) -> Result<(), DoctrineError> {
    if policy.system.is_empty() {
        return Err(DoctrineError::ValidationError("System name cannot be empty".to_string()));
    }
    Ok(())
}
