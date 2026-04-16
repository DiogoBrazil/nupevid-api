use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Entity not found")]
    NotFound,

    #[error("Unique constraint violation: {constraint:?}")]
    UniqueViolation { constraint: Option<String> },

    #[error("Foreign key constraint violation: {constraint:?}")]
    ForeignKeyViolation { constraint: Option<String> },

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("Referenced entity not found: {0}")]
    ReferencedEntityNotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}
