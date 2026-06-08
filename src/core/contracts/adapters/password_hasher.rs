use crate::core::errors::DomainError;

pub trait PasswordHasherPort: Send + Sync {
    fn hash_password(&self, password: &str) -> Result<String, DomainError>;
    fn verify_password(&self, hash: &str, password: &str) -> Result<bool, DomainError>;
}
