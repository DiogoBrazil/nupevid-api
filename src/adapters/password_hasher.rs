use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

pub use crate::core::contracts::adapters::password_hasher::PasswordHasherPort;
use crate::core::errors::DomainError;

#[derive(Clone)]
pub struct Argon2PasswordHasher;

impl Default for Argon2PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Argon2PasswordHasher {
    pub fn new() -> Self {
        Self
    }
}

impl PasswordHasherPort for Argon2PasswordHasher {
    fn hash_password(&self, password: &str) -> Result<String, DomainError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Ok(argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| DomainError::AdapterError(e.to_string()))?
            .to_string())
    }

    fn verify_password(&self, hash: &str, password: &str) -> Result<bool, DomainError> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| DomainError::AdapterError(e.to_string()))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
