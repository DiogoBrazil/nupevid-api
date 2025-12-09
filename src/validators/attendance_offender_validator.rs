use crate::utils::errors::AppError;

pub struct AttendanceOffenderValidator;

impl AttendanceOffenderValidator {
    pub fn validate_fields() -> Result<(), AppError> {
        // TODO: Future validations can be added here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_fields() {
        let result = AttendanceOffenderValidator::validate_fields();
        assert!(result.is_ok());
    }
}
