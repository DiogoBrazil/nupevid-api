#[cfg(test)]
mod tests {
    use crate::adapters::password_hasher::{Argon2PasswordHasher, PasswordHasherPort};

    #[test]
    fn test_hash_password_generates_different_hashes() {
        let hasher = Argon2PasswordHasher::new();
        let password = "my_secret_password";

        let hash1 = hasher.hash_password(password).expect("Failed to hash password");
        let hash2 = hasher.hash_password(password).expect("Failed to hash password");

        // Due to salt, same password should generate different hashes
        assert_ne!(hash1, hash2);
        assert!(!hash1.is_empty());
        assert!(!hash2.is_empty());
    }

    #[test]
    fn test_verify_password_correct() {
        let hasher = Argon2PasswordHasher::new();
        let password = "correct_password";

        let hash = hasher.hash_password(password).expect("Failed to hash password");
        let result = hasher.verify_password(&hash, password).expect("Failed to verify password");

        assert!(result, "Password verification should succeed with correct password");
    }

    #[test]
    fn test_verify_password_incorrect() {
        let hasher = Argon2PasswordHasher::new();
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let hash = hasher.hash_password(password).expect("Failed to hash password");
        let result = hasher.verify_password(&hash, wrong_password).expect("Failed to verify password");

        assert!(!result, "Password verification should fail with incorrect password");
    }

    #[test]
    fn test_verify_password_empty_password() {
        let hasher = Argon2PasswordHasher::new();
        let password = "";

        let hash = hasher.hash_password(password).expect("Failed to hash empty password");
        let result = hasher.verify_password(&hash, password).expect("Failed to verify empty password");

        assert!(result, "Empty password should verify correctly");
    }

    #[test]
    fn test_verify_password_special_characters() {
        let hasher = Argon2PasswordHasher::new();
        let password = "P@ssw0rd!#$%^&*()_+=[]{}|;:',.<>?/`~";

        let hash = hasher.hash_password(password).expect("Failed to hash password with special chars");
        let result = hasher.verify_password(&hash, password).expect("Failed to verify password");

        assert!(result, "Password with special characters should verify correctly");
    }

    #[test]
    fn test_hash_unicode_password() {
        let hasher = Argon2PasswordHasher::new();
        let password = "Señor123 日本語 🔐";

        let hash = hasher.hash_password(password).expect("Failed to hash unicode password");
        let result = hasher.verify_password(&hash, password).expect("Failed to verify unicode password");

        assert!(result, "Unicode password should hash and verify correctly");
    }
}
