use log::{error, warn};
use std::collections::HashMap;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::entities::auth::UserClaims;
use crate::core::value_objects::policies::PermissionPolicies;
use crate::core::value_objects::profiles::Profile;

/// Masks an email address for safe logging: keeps only the first character of
/// the local part and the domain (e.g. `j***@example.com`).
pub fn mask_email(email: &str) -> String {
    match email.split_once('@') {
        Some((local, domain)) if !local.is_empty() => {
            let first = local.chars().next().unwrap_or('*');
            format!("{}***@{}", first, domain)
        }
        _ => "***".to_string(),
    }
}

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
) -> Result<Option<PermissionPolicies>, AppError> {
    if claims.profile == Profile::Root {
        return Ok(None);
    }

    let user_id = Uuid::parse_str(&claims.id)
        .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

    match user_repository.get_user_policies_by_id(user_id).await {
        Ok(policies) => Ok(Some(policies)),
        Err(RepositoryError::NotFound) => {
            // The token references a user that no longer exists (e.g. deleted
            // after the token was issued): reject instead of granting access.
            warn!(
                "[ServiceHelper] Token for nonexistent/deleted user {} rejected",
                user_id
            );
            Err(AppError::Unauthorized("Invalid credentials".to_string()))
        }
        Err(e) => {
            error!("[ServiceHelper] Failed to retrieve user policies: {:?}", e);
            Err(AppError::InternalServerError)
        }
    }
}

pub async fn get_user_policies<T: UserRepository + ?Sized>(
    user_repository: &T,
    claims: &UserClaims,
) -> Result<PermissionPolicies, AppError> {
    if claims.profile == Profile::Root {
        return Ok(HashMap::new());
    }

    let user_id = Uuid::parse_str(&claims.id)
        .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

    match user_repository.get_user_policies_by_id(user_id).await {
        Ok(policies) => Ok(policies),
        Err(RepositoryError::NotFound) => {
            // Never fall back to default policies here: a deleted user with a
            // still-valid access token must lose all access immediately.
            warn!(
                "[ServiceHelper] Token for nonexistent/deleted user {} rejected",
                user_id
            );
            Err(AppError::Unauthorized("Invalid credentials".to_string()))
        }
        Err(e) => {
            error!("[ServiceHelper] Failed to retrieve user policies: {:?}", e);
            Err(AppError::InternalServerError)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mask_email;

    #[test]
    fn mask_email_keeps_first_char_and_domain() {
        assert_eq!(mask_email("joao@example.com"), "j***@example.com");
    }

    #[test]
    fn mask_email_single_char_local_part() {
        assert_eq!(mask_email("a@example.com"), "a***@example.com");
    }

    #[test]
    fn mask_email_without_at_sign() {
        assert_eq!(mask_email("not-an-email"), "***");
    }

    #[test]
    fn mask_email_empty_local_part() {
        assert_eq!(mask_email("@example.com"), "***");
    }

    #[test]
    fn mask_email_empty_string() {
        assert_eq!(mask_email(""), "***");
    }
}
