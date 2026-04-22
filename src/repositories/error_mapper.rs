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

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, error::Error as StdError, fmt, io};

    use sqlx::error::{DatabaseError, ErrorKind};

    use super::*;

    #[derive(Debug)]
    struct TestDatabaseError {
        message: &'static str,
        kind: ErrorKind,
        constraint: Option<&'static str>,
    }

    impl fmt::Display for TestDatabaseError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl StdError for TestDatabaseError {}

    impl DatabaseError for TestDatabaseError {
        fn message(&self) -> &str {
            self.message
        }

        fn code(&self) -> Option<Cow<'_, str>> {
            None
        }

        fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
            self
        }

        fn as_error_mut(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
            self
        }

        fn into_error(self: Box<Self>) -> Box<dyn StdError + Send + Sync + 'static> {
            self
        }

        fn constraint(&self) -> Option<&str> {
            self.constraint
        }

        fn kind(&self) -> ErrorKind {
            match self.kind {
                ErrorKind::UniqueViolation => ErrorKind::UniqueViolation,
                ErrorKind::ForeignKeyViolation => ErrorKind::ForeignKeyViolation,
                ErrorKind::NotNullViolation => ErrorKind::NotNullViolation,
                ErrorKind::CheckViolation => ErrorKind::CheckViolation,
                ErrorKind::Other => ErrorKind::Other,
                _ => ErrorKind::Other,
            }
        }
    }

    fn database_error(kind: ErrorKind, constraint: Option<&'static str>) -> sqlx::Error {
        sqlx::Error::Database(Box::new(TestDatabaseError {
            message: "database failure",
            kind,
            constraint,
        }))
    }

    #[test]
    fn maps_row_not_found_to_not_found() {
        let mapped = map_sqlx_error(sqlx::Error::RowNotFound);

        assert!(matches!(mapped, RepositoryError::NotFound));
    }

    #[test]
    fn maps_unique_violation_to_unique_violation_with_constraint() {
        let mapped = map_sqlx_error(database_error(
            ErrorKind::UniqueViolation,
            Some("users_email_key"),
        ));

        assert!(matches!(
            mapped,
            RepositoryError::UniqueViolation { constraint }
                if constraint.as_deref() == Some("users_email_key")
        ));
    }

    #[test]
    fn maps_foreign_key_violation_to_foreign_key_violation_with_constraint() {
        let mapped = map_sqlx_error(database_error(
            ErrorKind::ForeignKeyViolation,
            Some("victims_city_id_fkey"),
        ));

        assert!(matches!(
            mapped,
            RepositoryError::ForeignKeyViolation { constraint }
                if constraint.as_deref() == Some("victims_city_id_fkey")
        ));
    }

    #[test]
    fn maps_not_null_violation_to_database_error_when_no_specific_variant_exists() {
        let mapped = map_sqlx_error(database_error(ErrorKind::NotNullViolation, None));

        assert!(
            matches!(mapped, RepositoryError::DatabaseError(message) if message.contains("database failure"))
        );
    }

    #[test]
    fn maps_connection_error_to_database_error() {
        let mapped = map_sqlx_error(sqlx::Error::Io(io::Error::new(
            io::ErrorKind::ConnectionRefused,
            "connection refused",
        )));

        assert!(
            matches!(mapped, RepositoryError::DatabaseError(message) if message.contains("connection refused"))
        );
    }
}
