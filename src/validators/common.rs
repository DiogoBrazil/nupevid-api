use crate::core::application_error::ApplicationError as AppError;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref EMAIL_VALIDATION_REGEX: Regex =
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
}

pub fn is_valid_email(email: &str) -> bool {
    EMAIL_VALIDATION_REGEX.is_match(email)
}

pub fn is_public_route(path: &str) -> bool {
    const PUBLIC_EXACT: &[&str] = &[
        "/api/v1/auth/login",
        "/api/v1/auth/refresh",
        "/api/v1/auth/logout",
        "/api/swagger",
    ];
    const PUBLIC_PREFIX: &[&str] = &["/api/swagger/", "/logstreamer"];

    PUBLIC_EXACT.contains(&path) || PUBLIC_PREFIX.iter().any(|prefix| path.starts_with(prefix))
}

pub fn validate_required_fields(
    validations: &[(&str, bool)],
    error_prefix: &str,
) -> Result<(), AppError> {
    for (field_name, is_empty) in validations {
        if *is_empty {
            return Err(AppError::BadRequest(format!(
                "{}: {} cannot be empty",
                error_prefix, field_name
            )));
        }
    }
    Ok(())
}

pub fn validate_person_name(full_name: &str, error_context: &str) -> Result<(), AppError> {
    validate_required_fields(&[("full_name", full_name.trim().is_empty())], error_context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_person_name_success_victim() {
        let result = validate_person_name("Maria da Silva", "test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_person_name_success_offender() {
        let result = validate_person_name("João dos Santos", "test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_person_name_empty() {
        let result = validate_person_name("", "test");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("full_name cannot be empty")
        );
    }

    #[test]
    fn test_validate_person_name_whitespace() {
        let result = validate_person_name("   ", "test");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("full_name cannot be empty")
        );
    }

    #[test]
    fn is_public_route_returns_true_for_login_exact() {
        assert!(is_public_route("/api/v1/auth/login"));
    }

    #[test]
    fn is_public_route_returns_true_for_swagger_exact() {
        assert!(is_public_route("/api/swagger"));
    }

    #[test]
    fn is_public_route_returns_true_for_refresh_exact() {
        assert!(is_public_route("/api/v1/auth/refresh"));
    }

    #[test]
    fn is_public_route_returns_true_for_logout_exact() {
        assert!(is_public_route("/api/v1/auth/logout"));
    }

    #[test]
    fn is_public_route_returns_true_for_swagger_asset() {
        assert!(is_public_route("/api/swagger/index.html"));
    }

    #[test]
    fn is_public_route_returns_false_for_login_suffix_attack() {
        assert!(!is_public_route("/api/v1/auth/login-malicious"));
    }

    #[test]
    fn is_public_route_returns_false_for_login_trailing_slash() {
        assert!(!is_public_route("/api/v1/auth/login/"));
    }

    #[test]
    fn is_public_route_returns_false_for_login_plural() {
        assert!(!is_public_route("/api/v1/auth/logins"));
    }

    #[test]
    fn is_public_route_returns_false_for_swagger_suffix_attack() {
        assert!(!is_public_route("/api/swaggerfoo"));
    }

    #[test]
    fn is_public_route_returns_false_for_users_list() {
        assert!(!is_public_route("/api/v1/users"));
    }

    #[test]
    fn is_public_route_returns_false_for_empty_path() {
        assert!(!is_public_route(""));
    }

    #[test]
    fn is_public_route_returns_false_for_root() {
        assert!(!is_public_route("/"));
    }
}
