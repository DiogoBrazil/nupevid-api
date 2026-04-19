use std::collections::HashMap;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_helpers::{PolicyMap, get_user_policies_strict};
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::User;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;

pub struct UserReadScope {
    pub allowed_cities: Option<Vec<Uuid>>,
    pub exclude_root: bool,
}

pub async fn build_user_read_scope(
    user_repository: &dyn UserRepository,
    claims: &UserClaims,
) -> Result<UserReadScope, AppError> {
    if claims.profile == Profile::Root {
        return Ok(UserReadScope {
            allowed_cities: None,
            exclude_root: false,
        });
    }

    if claims.profile == Profile::CityAdmin {
        let allowed = get_user_policies_strict(user_repository, claims)
            .await?
            .and_then(|policies| policies.get(&Policy::ReadUsers).cloned())
            .unwrap_or_default();

        return Ok(UserReadScope {
            allowed_cities: Some(allowed),
            exclude_root: true,
        });
    }

    Ok(UserReadScope {
        allowed_cities: None,
        exclude_root: true,
    })
}

pub fn filter_users_by_scope(mut users: Vec<User>, scope: &UserReadScope) -> Vec<User> {
    if scope.exclude_root {
        users.retain(|user| user.profile != Profile::Root);
    }

    if let Some(allowed_cities) = scope.allowed_cities.as_ref() {
        users.retain(|user| {
            user.city_id
                .map(|city_id| allowed_cities.contains(&city_id))
                .unwrap_or(false)
        });
    }

    users
}

pub async fn get_existing_user(
    user_repository: &dyn UserRepository,
    id: Uuid,
) -> Result<User, AppError> {
    match user_repository.get_user_by_id(id).await {
        Ok(user) => Ok(user),
        Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
            "User with id '{}' not found",
            id
        ))),
        Err(_) => Err(AppError::InternalServerError),
    }
}

pub async fn get_claims_policies_or_empty(
    user_repository: &dyn UserRepository,
    claims: &UserClaims,
) -> Result<PolicyMap, AppError> {
    Ok(get_user_policies_strict(user_repository, claims)
        .await?
        .unwrap_or_else(HashMap::new))
}

pub fn generate_temporary_password() -> String {
    use rand::Rng;
    use rand::distributions::Alphanumeric;
    use rand::rngs::OsRng;

    let suffix: String = OsRng
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    format!("prov{}", suffix)
}
