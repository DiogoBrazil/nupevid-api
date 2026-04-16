use crate::core::application_error::ApplicationError;
use crate::core::value_objects::registrations::{
    REGISTRATION_MAX_LENGTH, REGISTRATION_PREFIX, is_valid_registration,
};

pub enum UserSearchCriteria {
    ByName(String),
    ByRegistration(String),
}

impl UserSearchCriteria {
    pub fn parse(
        name: Option<String>,
        registration: Option<String>,
    ) -> Result<Self, ApplicationError> {
        match (name, registration) {
            (Some(_), Some(_)) => Err(ApplicationError::BadRequest(
                "provide either 'name' or 'registration', not both".to_string(),
            )),
            (None, None) => Err(ApplicationError::BadRequest(
                "query parameter 'name' or 'registration' is required".to_string(),
            )),
            (Some(name), None) => {
                let trimmed = name.trim();
                if trimmed.is_empty() {
                    return Err(ApplicationError::BadRequest(
                        "query parameter 'name' cannot be empty".to_string(),
                    ));
                }
                Ok(Self::ByName(trimmed.to_string()))
            }
            (None, Some(registration)) => {
                let trimmed = registration.trim();
                if trimmed.is_empty() {
                    return Err(ApplicationError::BadRequest(
                        "query parameter 'registration' cannot be empty".to_string(),
                    ));
                }
                if !is_valid_registration(trimmed) {
                    return Err(ApplicationError::BadRequest(format!(
                        "invalid registration '{}'. Registration must start with '{}' and have at most {} characters",
                        trimmed, REGISTRATION_PREFIX, REGISTRATION_MAX_LENGTH
                    )));
                }
                Ok(Self::ByRegistration(trimmed.to_string()))
            }
        }
    }
}
