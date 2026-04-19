use log::{error, info, warn};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_helpers::PolicyMap;
use crate::core::entities::auth::UserClaims;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;

pub fn check_policy(
    claims: &UserClaims,
    policy: &Policy,
    city_id: Uuid,
    user_policies: &PolicyMap,
) -> Result<(), AppError> {
    info!(
        "[Authorization] Checking policy '{}' for city '{}' by user '{}'",
        policy, city_id, claims.id
    );

    if claims.profile == Profile::Root {
        info!("[Authorization] ROOT user - implicit access granted");
        return Ok(());
    }

    if let Some(city_ids) = user_policies.get(policy)
        && city_ids.contains(&city_id)
    {
        info!(
            "[Authorization] Policy '{}' found for city '{}'",
            policy, city_id
        );
        return Ok(());
    }

    warn!(
        "[Authorization] Policy '{}' not found for city '{}' for user '{}'",
        policy, city_id, claims.id
    );
    Err(AppError::Forbidden(format!(
        "You don't have permission to perform '{}' for this city",
        policy
    )))
}

pub fn get_allowed_cities_for_policy(
    claims: &UserClaims,
    policy: &Policy,
    user_policies: &PolicyMap,
) -> Option<Vec<Uuid>> {
    info!(
        "[Authorization] Getting allowed cities for policy '{}' by user '{}'",
        policy, claims.id
    );

    if claims.profile == Profile::Root {
        info!("[Authorization] ROOT user - access to all cities");
        return None;
    }

    let allowed_cities = user_policies.get(policy).cloned().unwrap_or_default();

    info!(
        "[Authorization] Found {} allowed cities for policy '{}'",
        allowed_cities.len(),
        policy
    );
    Some(allowed_cities)
}

pub fn validate_user_creation_permission(
    creator_profile: &Profile,
    target_profile: &Profile,
) -> Result<(), AppError> {
    info!(
        "[Authorization] Validating user creation permission: '{}' creating '{}'",
        creator_profile, target_profile
    );

    match creator_profile {
        Profile::Root => {
            info!("[Authorization] ROOT user - allowed to create any profile");
            Ok(())
        }
        Profile::CityUser => {
            error!("[Authorization] CITY_USER cannot create users");
            Err(AppError::Forbidden(
                "CITY_USER profile is not allowed to create users".to_string(),
            ))
        }
        Profile::CityAdmin => match target_profile {
            Profile::Root => {
                error!("[Authorization] CITY_ADMIN cannot create ROOT users");
                Err(AppError::Forbidden(
                    "CITY_ADMIN is not allowed to create ROOT users".to_string(),
                ))
            }
            Profile::CityAdmin => {
                error!("[Authorization] CITY_ADMIN cannot create other CITY_ADMIN users");
                Err(AppError::Forbidden(
                    "CITY_ADMIN is not allowed to create other CITY_ADMIN users".to_string(),
                ))
            }
            Profile::CityUser => {
                info!("[Authorization] CITY_ADMIN creating CITY_USER - allowed");
                Ok(())
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value_objects::ranks::Rank;
    use std::collections::HashMap;

    fn create_test_claims(profile: Profile, city_id: Option<&str>) -> UserClaims {
        UserClaims {
            id: "test-user-id".to_string(),
            exp: 9999999999,
            iss: "nupevid-api".to_string(),
            aud: "nupevid-api".to_string(),
            rank: Rank::CapPm,
            registration: "100012345".to_string(),
            full_name: "Test User".to_string(),
            profile,
            email: "test@test.com".to_string(),
            city_id: city_id.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_root_has_implicit_access() {
        let claims = create_test_claims(Profile::Root, None);
        let policies = HashMap::new();
        let city_id = Uuid::new_v4();

        let result = check_policy(&claims, &Policy::ReadVictims, city_id, &policies);
        assert!(result.is_ok());
    }

    #[test]
    fn test_city_admin_with_policy() {
        let city_id = Uuid::new_v4();
        let claims = create_test_claims(Profile::CityAdmin, Some(&city_id.to_string()));
        let mut policies = HashMap::new();
        policies.insert(Policy::ReadVictims, vec![city_id]);
        policies.insert(Policy::CreateVictims, vec![city_id]);

        let result = check_policy(&claims, &Policy::ReadVictims, city_id, &policies);
        assert!(result.is_ok());
    }

    #[test]
    fn test_city_admin_without_policy() {
        let city_id = Uuid::new_v4();
        let other_city_id = Uuid::new_v4();
        let claims = create_test_claims(Profile::CityAdmin, Some(&city_id.to_string()));
        let mut policies = HashMap::new();
        policies.insert(Policy::ReadVictims, vec![city_id]);

        let result = check_policy(&claims, &Policy::ReadVictims, other_city_id, &policies);
        assert!(result.is_err());

        let result = check_policy(&claims, &Policy::DeleteVictims, city_id, &policies);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_allowed_cities_for_policy() {
        let city_id1 = Uuid::new_v4();
        let city_id2 = Uuid::new_v4();
        let claims = create_test_claims(Profile::CityAdmin, Some(&city_id1.to_string()));
        let mut policies = HashMap::new();
        policies.insert(Policy::ReadVictims, vec![city_id1, city_id2]);

        let allowed = get_allowed_cities_for_policy(&claims, &Policy::ReadVictims, &policies);
        assert!(allowed.is_some());
        let cities = allowed.unwrap();
        assert_eq!(cities.len(), 2);
        assert!(cities.contains(&city_id1));
        assert!(cities.contains(&city_id2));
    }

    #[test]
    fn test_root_get_allowed_cities_returns_none() {
        let claims = create_test_claims(Profile::Root, None);
        let policies = HashMap::new();

        let allowed = get_allowed_cities_for_policy(&claims, &Policy::ReadVictims, &policies);
        assert!(allowed.is_none());
    }
}
