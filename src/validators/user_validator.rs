use crate::utils::errors::AppError;
use crate::validators::common::*;
use uuid::Uuid;

pub struct UserValidator;

impl UserValidator {
    pub fn validate_fields(
        rank: &str,
        registration: &str,
        full_name: &str,
        profile: &str,
        email: &str,
        password: Option<&str>,
        error_context: &str,
    ) -> Result<(), AppError> {
        let mut fields_to_validate = vec![
            ("rank", rank.is_empty()),
            ("registration", registration.is_empty()),
            ("full_name", full_name.is_empty()),
            ("profile", profile.is_empty()),
            ("email", email.is_empty()),
        ];

        if let Some(p) = password {
            fields_to_validate.push(("password", p.is_empty()));
        }

        validate_required_fields(&fields_to_validate, error_context)?;

        if !is_valid_rank(rank) {
            return Err(AppError::BadRequest(format!(
                "{}invalid rank '{}'. Valid ranks are: {:?}",
                error_context, rank, VALID_RANKS
            )));
        }

        if !is_valid_registration(registration) {
            return Err(AppError::BadRequest(format!(
                "{}invalid registration '{}'. Registration must start with '{}' and have at most {} characters",
                error_context, registration, REGISTRATION_PREFIX, REGISTRATION_MAX_LENGTH
            )));
        }

        if !is_valid_profile(profile) {
            return Err(AppError::BadRequest(format!(
                "{}invalid profile '{}'. Valid profiles are: {:?}",
                error_context, profile, VALID_PROFILES
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
        profile: &str,
        city_id: &Option<Uuid>,
        error_context: &str,
    ) -> Result<(), AppError> {
        if profile != PROFILE_ROOT && city_id.is_none() {
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
            "CEL PM",
            "100012345",
            "João Silva",
            "ROOT",
            "joao@example.com",
            Some("senha123"),
            "test",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_fields_empty_rank() {
        let result = UserValidator::validate_fields(
            "",
            "100012345",
            "João Silva",
            "ROOT",
            "joao@example.com",
            Some("senha123"),
            "test",
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("rank cannot be empty")
        );
    }

    #[test]
    fn test_validate_fields_invalid_rank() {
        let result = UserValidator::validate_fields(
            "INVALID_RANK",
            "100012345",
            "João Silva",
            "ROOT",
            "joao@example.com",
            Some("senha123"),
            "test",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid rank"));
    }

    #[test]
    fn test_validate_fields_invalid_registration() {
        let result = UserValidator::validate_fields(
            "CEL PM",
            "999912345", // Should start with 1000
            "João Silva",
            "ROOT",
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
    fn test_validate_fields_invalid_profile() {
        let result = UserValidator::validate_fields(
            "CEL PM",
            "100012345",
            "João Silva",
            "INVALID_PROFILE",
            "joao@example.com",
            Some("senha123"),
            "test",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid profile"));
    }

    #[test]
    fn test_validate_fields_invalid_email() {
        let result = UserValidator::validate_fields(
            "CEL PM",
            "100012345",
            "João Silva",
            "ROOT",
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
        let result = UserValidator::validate_city_requirement(PROFILE_ROOT, &None, "test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_city_requirement_city_admin_without_city() {
        let result = UserValidator::validate_city_requirement(PROFILE_CITY_ADMIN, &None, "test");
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
            UserValidator::validate_city_requirement(PROFILE_CITY_ADMIN, &Some(city_id), "test");
        assert!(result.is_ok());
    }
}
