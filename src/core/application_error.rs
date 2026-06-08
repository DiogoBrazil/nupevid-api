use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Internal Server Error")]
    InternalServerError,

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("UnprocessableEntity: {message}")]
    UnprocessableEntity {
        message: String,
        field: Option<String>,
    },

    #[error("Database Error: {0}")]
    DatabaseError(String),

    #[error("Invalid Method Error: {0}")]
    InvalidMethodError(String),
}
