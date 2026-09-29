pub mod lower;
pub mod validate;

pub use lower::lower_policy;
pub use validate::validate_policy;

use crate::ast::policy::DoctrinePolicy;
use crate::ir::graph::PolicyGraph;
use crate::error::DoctrineError;

pub fn compile_doctrine(policy: DoctrinePolicy) -> Result<PolicyGraph, DoctrineError> {
    validate_policy(&policy)?;
    Ok(lower_policy(policy))
}
