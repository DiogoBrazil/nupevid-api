use crate::utils::errors::AppError;
use crate::validators::common::validate_required_fields;

pub struct ProtectiveMeasureValidator;

impl ProtectiveMeasureValidator {
    pub fn validate_required_fields(
        process_number: &str,
        judicial_authority: &str,
        violence_types: &[crate::core::entities::protective_measures::ViolenceType],
        error_context: &str,
    ) -> Result<(), AppError> {
        validate_required_fields(
            &[
                ("process_number", process_number.is_empty()),
                ("judicial_authority", judicial_authority.is_empty()),
                ("violence_types", violence_types.is_empty()),
            ],
            error_context,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_required_fields_success() {
        let result = ProtectiveMeasureValidator::validate_required_fields(
            "2025.001.000001-0",
            "1ª Vara Criminal de Porto Velho",
            &[crate::core::entities::protective_measures::ViolenceType::Physical],
            "test",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_required_fields_empty_process_number() {
        let result = ProtectiveMeasureValidator::validate_required_fields(
            "",
            "1ª Vara Criminal de Porto Velho",
            &[crate::core::entities::protective_measures::ViolenceType::Physical],
            "test",
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("process_number cannot be empty")
        );
    }

    #[test]
    fn test_validate_required_fields_empty_judicial_authority() {
        let result = ProtectiveMeasureValidator::validate_required_fields(
            "2025.001.000001-0",
            "",
            &[crate::core::entities::protective_measures::ViolenceType::Physical],
            "test",
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("judicial_authority cannot be empty")
        );
    }

    #[test]
    fn test_validate_required_fields_both_empty() {
        let result = ProtectiveMeasureValidator::validate_required_fields(
            "",
            "",
            &[crate::core::entities::protective_measures::ViolenceType::Physical],
            "test",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_required_fields_empty_violence_types() {
        let result = ProtectiveMeasureValidator::validate_required_fields(
            "2025.001.000001-0",
            "1ª Vara Criminal de Porto Velho",
            &[],
            "test",
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("violence_types cannot be empty")
        );
    }
}
