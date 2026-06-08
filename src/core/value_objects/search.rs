use crate::core::errors::DomainError;
use crate::core::value_objects::cpf::{CpfValidationError, validate_cpf_masked};

pub enum SearchCriteria {
    ByName(String),
    ByCpf(String),
}

impl SearchCriteria {
    pub fn parse(name: Option<String>, cpf: Option<String>) -> Result<Self, DomainError> {
        match (name, cpf) {
            (Some(_), Some(_)) => Err(DomainError::ValidationError(
                "provide either 'name' or 'cpf', not both".to_string(),
            )),
            (None, None) => Err(DomainError::ValidationError(
                "query parameter 'name' or 'cpf' is required".to_string(),
            )),
            (Some(name), None) => {
                let trimmed = name.trim();
                if trimmed.is_empty() {
                    return Err(DomainError::ValidationError(
                        "query parameter 'name' cannot be empty".to_string(),
                    ));
                }
                Ok(SearchCriteria::ByName(trimmed.to_string()))
            }
            (None, Some(cpf)) => {
                let trimmed = cpf.trim();
                if trimmed.is_empty() {
                    return Err(DomainError::ValidationError(
                        "query parameter 'cpf' cannot be empty".to_string(),
                    ));
                }
                let normalized = validate_cpf_masked(trimmed).map_err(|e| match e {
                    CpfValidationError::InvalidLength => DomainError::ValidationError(
                        "cpf must be 14 characters in the format 000.000.000-00".to_string(),
                    ),
                    CpfValidationError::InvalidFormat => DomainError::ValidationError(
                        "cpf must match the format 000.000.000-00".to_string(),
                    ),
                    CpfValidationError::InvalidDigits => {
                        DomainError::ValidationError("cpf has invalid check digits".to_string())
                    }
                })?;
                Ok(SearchCriteria::ByCpf(normalized))
            }
        }
    }
}
