use thiserror::Error;

#[derive(Error, Debug)]
pub enum DoctrineError {
    #[error("parse error: {0}")]
    ParseError(String),

    #[error("validation error: {0}")]
    ValidationError(String),
}
