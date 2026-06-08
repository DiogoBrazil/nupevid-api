use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Adapter error: {0}")]
    AdapterError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
