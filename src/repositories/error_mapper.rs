use crate::core::contracts::repository::error::RepositoryError;

pub fn map_sqlx_error(err: sqlx::Error) -> RepositoryError {
    match err {
        sqlx::Error::RowNotFound => RepositoryError::NotFound,
        sqlx::Error::Database(db_err) => {
            if db_err.is_unique_violation() {
                RepositoryError::UniqueViolation {
                    constraint: db_err.constraint().map(|s| s.to_string()),
                }
            } else if db_err.is_foreign_key_violation() {
                RepositoryError::ForeignKeyViolation {
                    constraint: db_err.constraint().map(|s| s.to_string()),
                }
            } else {
                RepositoryError::DatabaseError(db_err.to_string())
            }
        }
        other => RepositoryError::DatabaseError(other.to_string()),
    }
}
