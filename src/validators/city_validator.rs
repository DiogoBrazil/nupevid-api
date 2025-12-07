use crate::utils::errors::AppError;
use crate::validators::common::*;

pub struct CityValidator;

impl CityValidator {
    pub fn validate_fields(
        name: &str,
        state: &str,
        battalion: &str,
        error_context: &str
    ) -> Result<(), AppError> {
        validate_required_fields(&[
            ("name", name.is_empty()),
            ("state", state.is_empty()),
            ("battalion", battalion.is_empty()),
        ], error_context)?;

        if !is_valid_city_name(name) {
            return Err(AppError::BadRequest(
                format!("{}: invalid city name '{}'. Valid cities: {:?}", error_context, name, VALID_CITIES)
            ));
        }

        if !is_valid_state(state) {
            return Err(AppError::BadRequest(
                format!("{}: invalid state '{}'. Valid states: {:?}", error_context, state, VALID_STATES)
            ));
        }

        if !is_valid_battalion(battalion) {
            return Err(AppError::BadRequest(
                format!("{}: invalid battalion '{}'. Valid battalions: {:?}", error_context, battalion, VALID_BATTALIONS)
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_fields_success() {
        let result = CityValidator::validate_fields(
            "PORTO VELHO",
            "RO",
            "1ºBPM",
            "test"
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_fields_case_insensitive_city() {
        let result = CityValidator::validate_fields(
            "porto velho", // lowercase
            "RO",
            "1ºBPM",
            "test"
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_fields_case_insensitive_state() {
        let result = CityValidator::validate_fields(
            "PORTO VELHO",
            "ro", // lowercase
            "1ºBPM",
            "test"
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_fields_empty_name() {
        let result = CityValidator::validate_fields(
            "",
            "RO",
            "1ºBPM",
            "test"
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name cannot be empty"));
    }

    #[test]
    fn test_validate_fields_invalid_city_name() {
        let result = CityValidator::validate_fields(
            "INVALID CITY",
            "RO",
            "1ºBPM",
            "test"
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid city name"));
    }

    #[test]
    fn test_validate_fields_invalid_state() {
        let result = CityValidator::validate_fields(
            "PORTO VELHO",
            "SP",
            "1ºBPM",
            "test"
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid state"));
    }

    #[test]
    fn test_validate_fields_invalid_battalion() {
        let result = CityValidator::validate_fields(
            "PORTO VELHO",
            "RO",
            "INVALID",
            "test"
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid battalion"));
    }
}
