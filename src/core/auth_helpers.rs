use log::error;
use std::collections::HashMap;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::entities::auth::UserClaims;
use crate::core::policy_defaults;
use crate::core::value_objects::policies::PermissionPolicies;
use crate::core::value_objects::profiles::Profile;

pub type PolicyMap = PermissionPolicies;

pub fn extract_city_id_from_claims(claims: &UserClaims) -> Result<Uuid, AppError> {
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
    claims: &UserClaims,
) -> Result<Option<PolicyMap>, AppError> {
    if claims.profile == Profile::Root {
        return Ok(None);
    }

    let user_id = Uuid::parse_str(&claims.id)
        .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

    match user_repository.get_user_policies_by_id(user_id).await {
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
    claims: &UserClaims,
) -> Result<PolicyMap, AppError> {
    if claims.profile == Profile::Root {
        return Ok(HashMap::new());
    }

    let user_id = Uuid::parse_str(&claims.id)
        .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

    match user_repository.get_user_policies_by_id(user_id).await {
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

    Ok(policy_defaults::default_for_profile(
        &claims.profile,
        city_id,
    ))
}
