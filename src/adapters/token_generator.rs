use jsonwebtoken::{encode, errors::Error as JwtError, EncodingKey, Header};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::entities::auth::ClaimsToUserToken;

pub use crate::core::contracts::adapters::token_generator::TokenGeneratorPort;

#[derive(Clone)]
pub struct JwtTokenGenerator;

impl Default for JwtTokenGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl JwtTokenGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl TokenGeneratorPort for JwtTokenGenerator {
    fn generate_token(&self,
        id: String,
        rank: String,
        registration: String,
        full_name: String,
        profile: String,
        email: String,
        city_id: Option<String>,
        secret: &str
    ) -> Result<String, JwtError> {
        let expiration: usize = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize + 24 * 3600;

        let claims = ClaimsToUserToken {
            id,
            exp: expiration,
            rank,
            registration,
            full_name,
            profile,
            email,
            city_id,
        };

        encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
    }
}
