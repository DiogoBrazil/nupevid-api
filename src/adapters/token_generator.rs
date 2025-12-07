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
        id: &str,
        rank: &str,
        registration: &str,
        full_name: &str,
        profile: &str,
        email: &str,
        city_id: Option<&str>,
        secret: &str
    ) -> Result<String, JwtError> {
        let expiration: usize = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize + 24 * 3600;

        let claims = ClaimsToUserToken {
            id: id.to_string(),
            exp: expiration,
            rank: rank.to_string(),
            registration: registration.to_string(),
            full_name: full_name.to_string(),
            profile: profile.to_string(),
            email: email.to_string(),
            city_id: city_id.map(|s| s.to_string()),
        };

        encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
    }
}
