use actix_web::{HttpRequest, HttpMessage};
use log::error;
use uuid::Uuid;
use serde_json::Value as JsonValue;

use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::contracts::repository::users::UserRepository;
use crate::utils::errors::AppError;
use crate::validators::common::{PROFILE_ROOT, generate_default_policies};

pub fn extract_claims(req: &HttpRequest) -> Result<ClaimsToUserToken, AppError> {
    req.extensions()
        .get::<ClaimsToUserToken>()
        .cloned()
        .ok_or_else(|| {
            error!("[ServiceHelper] No claims found in request");
            AppError::Unauthorized("Unauthorized".to_string())
        })
}

pub fn extract_city_id_from_claims(claims: &ClaimsToUserToken) -> Result<Uuid, AppError> {
    claims
        .city_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| {
            error!("[ServiceHelper] User has no city_id in claims");
            AppError::Forbidden("User must be associated with a city".to_string())
        })
}

pub async fn get_user_policies_strict<T: UserRepository>(
    user_repository: &T,
    claims: &ClaimsToUserToken,
) -> Result<Option<JsonValue>, AppError> {
    if claims.profile == PROFILE_ROOT {
        return Ok(None);
    }

    let user_id = Uuid::parse_str(&claims.id)
        .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

    match user_repository.get_user_policies_json_by_id(user_id).await {
        Ok(policies) => Ok(Some(policies)),
        Err(sqlx::Error::RowNotFound) => Ok(None),
        Err(e) => {
            error!("[ServiceHelper] Failed to retrieve user policies: {:?}", e);
            Err(AppError::InternalServerError)
        }
    }
}

pub async fn get_user_policies_with_defaults<T: UserRepository>(
    user_repository: &T,
    claims: &ClaimsToUserToken,
) -> Result<JsonValue, AppError> {
    if claims.profile == PROFILE_ROOT {
        return Ok(serde_json::json!({}));
    }

    let user_id = Uuid::parse_str(&claims.id)
        .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

    match user_repository.get_user_policies_json_by_id(user_id).await {
        Ok(policies) => return Ok(policies),
        Err(sqlx::Error::RowNotFound) => {
        }
        Err(e) => {
            error!("[ServiceHelper] Failed to retrieve user policies: {:?}", e);
            return Err(AppError::InternalServerError);
        }
    }

    let city_id = if let Some(city_id_str) = &claims.city_id {
        Uuid::parse_str(city_id_str).ok()
    } else {
        None
    };

    let defaults = generate_default_policies(&claims.profile, city_id);
    Ok(serde_json::json!(defaults))
}
