#[cfg(test)]
mod tests {
    use crate::utils::validations::{is_valid_email, validate_required_fields};
    use crate::utils::errors::AppError;

    #[test]
    fn test_is_valid_email_valid() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("test.user@domain.co.uk"));
        assert!(is_valid_email("user+tag@domain.com"));
        assert!(is_valid_email("user_name@test-domain.org"));
        assert!(is_valid_email("123@numbers.com"));
    }

    #[test]
    fn test_is_valid_email_invalid() {
        assert!(!is_valid_email("plaintext"));
        assert!(!is_valid_email("@domain.com"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email("user@domain"));
        assert!(!is_valid_email("user domain@test.com"));
        assert!(!is_valid_email(""));
        assert!(!is_valid_email("user@@domain.com"));
    }

    #[test]
    fn test_validate_required_fields_success() {
        let fields = vec![
            ("name", false),
            ("email", false),
            ("password", false),
        ];

        let result = validate_required_fields(&fields, "Test: ");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_required_fields_empty_field() {
        let fields = vec![
            ("name", false),
            ("email", true), // Empty
            ("password", false),
        ];

        let result = validate_required_fields(&fields, "Test");
        assert!(result.is_err());
        
        match result {
            Err(AppError::BadRequest(msg)) => {
                assert!(msg.contains("Test"));
                assert!(msg.contains("email"));
                assert!(msg.contains("cannot be empty"));
            }
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[test]
    fn test_validate_required_fields_multiple_empty() {
        let fields = vec![
            ("name", true), // Empty - will fail first
            ("email", true), // Empty
            ("password", false),
        ];

        let result = validate_required_fields(&fields, "Error");
        assert!(result.is_err());
        
        match result {
            Err(AppError::BadRequest(msg)) => {
                assert!(msg.contains("Error"));
                // Only first empty field is reported
                assert!(msg.contains("name"));
                assert!(msg.contains("cannot be empty"));
            }
            _ => panic!("Expected BadRequest error"),
        }
    }
}
