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
    let public_routes = ["/api/v1/auth/login", "/api/swagger"];
    public_routes.iter().any(|route| path.starts_with(route))
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
