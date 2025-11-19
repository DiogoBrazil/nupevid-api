use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};

pub trait PasswordHasherPort: Send + Sync {
    fn hash_password(&self, password: &str) -> Result<String, argon2::password_hash::Error>;
    fn verify_password(&self, hash: &str, password: &str) -> Result<bool, argon2::password_hash::Error>;
}


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
    fn hash_password(&self, password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Ok(argon2.hash_password(password.as_bytes(), &salt)?.to_string())
    }

    fn verify_password(&self, hash: &str, password: &str) -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(hash)?;
        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
}
