use crate::core::application_error::ApplicationError as AppError;
use crate::validators::common::validate_required_fields;

pub struct OffenderValidator;

impl OffenderValidator {
    pub fn validate_required_fields(full_name: &str, error_context: &str) -> Result<(), AppError> {
        validate_required_fields(&[("full_name", full_name.trim().is_empty())], error_context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_required_fields_success() {
        let result = OffenderValidator::validate_required_fields("João dos Santos", "test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_required_fields_empty_name() {
        let result = OffenderValidator::validate_required_fields("", "test");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("full_name cannot be empty")
        );
    }

    #[test]
    fn test_validate_required_fields_whitespace_name() {
        let result = OffenderValidator::validate_required_fields("   ", "test");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("full_name cannot be empty")
        );
    }
}
