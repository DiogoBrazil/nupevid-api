use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Entity not found")]
    NotFound,

    #[error("Unique constraint violation: {constraint:?}")]
    UniqueViolation { constraint: Option<String> },

    #[error("Foreign key constraint violation: {constraint:?}")]
    ForeignKeyViolation { constraint: Option<String> },

    #[error("Database error: {0}")]
    DatabaseError(String),
}
