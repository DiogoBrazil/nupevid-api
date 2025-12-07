use crate::utils::errors::AppError;
use crate::validators::common::validate_required_fields;

pub struct VictimValidator;

impl VictimValidator {
    pub fn validate_required_fields(
        full_name: &str,
        error_context: &str
    ) -> Result<(), AppError> {
        validate_required_fields(&[
            ("full_name", full_name.trim().is_empty()),
        ], error_context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_required_fields_success() {
        let result = VictimValidator::validate_required_fields(
            "Maria da Silva",
            "test"
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_required_fields_empty_name() {
        let result = VictimValidator::validate_required_fields(
            "",
            "test"
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("full_name cannot be empty"));
    }

    #[test]
    fn test_validate_required_fields_whitespace_name() {
        let result = VictimValidator::validate_required_fields(
            "   ",
            "test"
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("full_name cannot be empty"));
    }
}
