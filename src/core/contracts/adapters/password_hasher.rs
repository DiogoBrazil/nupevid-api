pub trait PasswordHasherPort: Send + Sync {
    fn hash_password(&self, password: &str) -> Result<String, argon2::password_hash::Error>;
    fn verify_password(
        &self,
        hash: &str,
        password: &str,
    ) -> Result<bool, argon2::password_hash::Error>;
}
