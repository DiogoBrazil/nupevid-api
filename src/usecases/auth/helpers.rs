use chrono::{Duration, Utc};
use rand::distributions::Alphanumeric;
use rand::{Rng, rngs::OsRng};
use uuid::Uuid;

use crate::config::config_env::Config;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::adapters::password_hasher::PasswordHasherPort;
use crate::core::contracts::adapters::token_generator::{TokenClaimsInput, TokenGeneratorPort};
use crate::core::entities::auth::{ClientMetadata, NewRefreshToken};
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;

/// Length of the random secret embedded in a refresh token (cryptographically secure).
const REFRESH_SECRET_LEN: usize = 48;

/// Generates a cryptographically secure, URL-safe alphanumeric secret using the OS RNG.
pub fn generate_refresh_secret() -> String {
    OsRng
        .sample_iter(&Alphanumeric)
        .take(REFRESH_SECRET_LEN)
        .map(char::from)
        .collect()
}

/// Parses a refresh token in the `{token_id}.{secret}` format.
pub fn parse_refresh_token(token: &str) -> Result<(Uuid, String), AppError> {
    let (id_part, secret) = token
        .split_once('.')
        .ok_or_else(|| AppError::Unauthorized("Invalid refresh token".to_string()))?;

    if secret.is_empty() {
        return Err(AppError::Unauthorized("Invalid refresh token".to_string()));
    }

    let id = Uuid::parse_str(id_part)
        .map_err(|_| AppError::Unauthorized("Invalid refresh token".to_string()))?;

    Ok((id, secret.to_string()))
}

/// Fields required to build the access token claims.
pub struct AccessTokenSubject<'a> {
    pub id: Uuid,
    pub rank: &'a Rank,
    pub registration: &'a str,
    pub full_name: &'a str,
    pub profile: &'a Profile,
    pub email: &'a str,
    pub city_id: Option<Uuid>,
}

/// Issues a short-lived access token (JWT). Returns the token and its lifetime in seconds.
pub fn issue_access_token(
    token_generator: &dyn TokenGeneratorPort,
    config: &Config,
    subject: AccessTokenSubject<'_>,
) -> Result<(String, i64), AppError> {
    let id = subject.id.to_string();
    let city_id = subject.city_id.map(|id| id.to_string());

    let token = token_generator
        .generate_token(
            TokenClaimsInput {
                id: &id,
                rank: subject.rank,
                registration: subject.registration,
                full_name: subject.full_name,
                profile: subject.profile,
                email: subject.email,
                city_id: city_id.as_deref(),
                issuer: &config.jwt_issuer,
                audience: &config.jwt_audience,
                expires_in_seconds: config.access_token_ttl_seconds,
            },
            &config.jwt_secret,
        )
        .map_err(|_| AppError::InternalServerError)?;

    Ok((token, config.access_token_ttl_seconds))
}

/// Builds a new refresh token record (hashing the secret) and the plaintext `{id}.{secret}`
/// value to return to the client. The plaintext is never persisted.
pub fn build_new_refresh_token(
    hasher: &dyn PasswordHasherPort,
    user_id: Uuid,
    ttl_seconds: i64,
    metadata: &ClientMetadata,
) -> Result<(NewRefreshToken, String), AppError> {
    let token_id = Uuid::new_v4();
    let secret = generate_refresh_secret();
    let token_hash = hasher
        .hash_password(&secret)
        .map_err(|_| AppError::InternalServerError)?;

    let new_token = NewRefreshToken {
        id: token_id,
        user_id,
        token_hash,
        expires_at: Utc::now() + Duration::seconds(ttl_seconds),
        user_agent: metadata.user_agent.clone(),
        ip_address: metadata.ip_address.clone(),
    };

    let plaintext = format!("{}.{}", token_id, secret);

    Ok((new_token, plaintext))
}
