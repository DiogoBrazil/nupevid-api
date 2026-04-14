use log::error;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::core::contracts::repository::users::UserRepository;
use crate::core::entities::auth::ClaimsToUserToken;
use crate::utils::errors::AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;

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

pub async fn get_user_policies_strict<T: UserRepository + ?Sized>(
    user_repository: &T,
    claims: &ClaimsToUserToken,
) -> Result<Option<JsonValue>, AppError> {
    if claims.profile == Profile::Root {
        return Ok(None);
    }

    let user_id = Uuid::parse_str(&claims.id)
        .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

    match user_repository.get_user_policies_json_by_id(user_id).await {
        Ok(policies) => Ok(Some(policies)),
        Err(RepositoryError::NotFound) => Ok(None),
        Err(e) => {
            error!("[ServiceHelper] Failed to retrieve user policies: {:?}", e);
            Err(AppError::InternalServerError)
        }
    }
}

pub async fn get_user_policies_with_defaults<T: UserRepository + ?Sized>(
    user_repository: &T,
    claims: &ClaimsToUserToken,
) -> Result<JsonValue, AppError> {
    if claims.profile == Profile::Root {
        return Ok(serde_json::json!({}));
    }

    let user_id = Uuid::parse_str(&claims.id)
        .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

    match user_repository.get_user_policies_json_by_id(user_id).await {
        Ok(policies) => return Ok(policies),
        Err(RepositoryError::NotFound) => {}
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

    let defaults = Policy::default_for_profile(&claims.profile, city_id);
    Ok(serde_json::json!(defaults))
}
