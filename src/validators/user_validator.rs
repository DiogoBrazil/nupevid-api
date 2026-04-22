use crate::core::application_error::ApplicationError as AppError;
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::registrations::{
    REGISTRATION_MAX_LENGTH, REGISTRATION_PREFIX, is_valid_registration,
};
use crate::validators::common::{is_valid_email, validate_required_fields};
use uuid::Uuid;

pub struct UserValidator;

impl UserValidator {
    pub fn validate_fields(
        registration: &str,
        full_name: &str,
        email: &str,
        password: Option<&str>,
        error_context: &str,
    ) -> Result<(), AppError> {
        let mut fields_to_validate = vec![
            ("registration", registration.is_empty()),
            ("full_name", full_name.is_empty()),
            ("email", email.is_empty()),
        ];

        if let Some(p) = password {
            fields_to_validate.push(("password", p.is_empty()));
        }

        validate_required_fields(&fields_to_validate, error_context)?;

        if !is_valid_registration(registration) {
            return Err(AppError::BadRequest(format!(
                "{}invalid registration '{}'. Registration must start with '{}' and have at most {} characters",
                error_context, registration, REGISTRATION_PREFIX, REGISTRATION_MAX_LENGTH
            )));
        }

        if !is_valid_email(email) {
            return Err(AppError::BadRequest(format!(
                "{}'{}' is not a valid email",
                error_context, email
            )));
        }

        Ok(())
    }

    pub fn validate_city_requirement(
        profile: &Profile,
        city_id: &Option<Uuid>,
        error_context: &str,
    ) -> Result<(), AppError> {
        if *profile != Profile::Root && city_id.is_none() {
            return Err(AppError::BadRequest(format!(
                "{}: city_id is required for profile '{}'",
                error_context, profile
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_validate_fields_success() {
        let result = UserValidator::validate_fields(
            "100012345",
            "João Silva",
            "joao@example.com",
            Some("senha123"),
            "test",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_fields_invalid_registration() {
        let result = UserValidator::validate_fields(
            "999912345",
            "João Silva",
            "joao@example.com",
            Some("senha123"),
            "test",
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid registration")
        );
    }

    #[test]
    fn test_validate_fields_invalid_email() {
        let result = UserValidator::validate_fields(
            "100012345",
            "João Silva",
            "invalid-email",
            Some("senha123"),
            "test",
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not a valid email")
        );
    }

    #[test]
    fn test_validate_city_requirement_root_without_city() {
        let result = UserValidator::validate_city_requirement(&Profile::Root, &None, "test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_city_requirement_city_admin_without_city() {
        let result = UserValidator::validate_city_requirement(&Profile::CityAdmin, &None, "test");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("city_id is required")
        );
    }

    #[test]
    fn test_validate_city_requirement_city_admin_with_city() {
        let city_id = Uuid::new_v4();
        let result =
            UserValidator::validate_city_requirement(&Profile::CityAdmin, &Some(city_id), "test");
        assert!(result.is_ok());
    }
}
