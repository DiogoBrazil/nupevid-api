use crate::core::application_error::ApplicationError as AppError;
use crate::core::value_objects::cpf::{CpfValidationError, validate_cpf_masked};

pub fn validate_cpf(cpf: &str, error_context: &str) -> Result<String, AppError> {
    match validate_cpf_masked(cpf) {
        Ok(normalized) => Ok(normalized),
        Err(CpfValidationError::InvalidLength) => Err(AppError::BadRequest(format!(
            "{}: cpf must be 14 characters in the format 000.000.000-00",
            error_context
        ))),
        Err(CpfValidationError::InvalidFormat) => Err(AppError::BadRequest(format!(
            "{}: cpf must match the format 000.000.000-00",
            error_context
        ))),
        Err(CpfValidationError::InvalidDigits) => Err(AppError::BadRequest(format!(
            "{}: cpf has invalid check digits",
            error_context
        ))),
    }
}
