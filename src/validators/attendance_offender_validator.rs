use crate::utils::errors::AppError;

pub struct AttendanceOffenderValidator;

impl AttendanceOffenderValidator {
    pub fn validate_fields() -> Result<(), AppError> {
        // TODO: Future validations can be added here
        Ok(())
    }
}
