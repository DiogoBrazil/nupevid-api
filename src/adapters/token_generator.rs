use jsonwebtoken::{EncodingKey, Header, encode};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::errors::DomainError;

pub use crate::core::contracts::adapters::token_generator::{TokenClaimsInput, TokenGeneratorPort};

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
    fn generate_token(
        &self,
        claims: TokenClaimsInput<'_>,
        secret: &str,
    ) -> Result<String, DomainError> {
        let expiration: usize = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize
            + 24 * 3600;

        let claims = ClaimsToUserToken {
            id: claims.id.to_string(),
            exp: expiration,
            rank: claims.rank.clone(),
            registration: claims.registration.to_string(),
            full_name: claims.full_name.to_string(),
            profile: claims.profile.clone(),
            email: claims.email.to_string(),
            city_id: claims.city_id.map(|s| s.to_string()),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| DomainError::AdapterError(e.to_string()))
    }
}
