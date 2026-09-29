use crate::ast::policy::DoctrinePolicy;
use crate::error::DoctrineError;

pub fn parse_yaml(input: &str) -> Result<DoctrinePolicy, DoctrineError> {
    serde_yaml::from_str(input)
        .map_err(|e| DoctrineError::ParseError(e.to_string()))
}
