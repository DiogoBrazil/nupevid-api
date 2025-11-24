use log::{info, warn};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::core::entities::auth::ClaimsToUserToken;
use crate::utils::errors::AppError;
use crate::utils::validations::PROFILE_ROOT;

/// Check if a user has a specific policy for a given city
///
/// # Arguments
/// * `claims` - The user's JWT claims
/// * `policy_name` - The policy to check (e.g., "read_victims", "create_attendances")
/// * `city_id` - The city ID to check permission for
/// * `user_policies` - The user's permission_policies as JSON (from database)
///
/// # Returns
/// * `Ok(())` if the user has the policy
/// * `Err(AppError::Forbidden)` if the user does not have the policy
pub fn check_policy(
    claims: &ClaimsToUserToken,
    policy_name: &str,
    city_id: Uuid,
    user_policies: &JsonValue,
) -> Result<(), AppError> {
    info!("[Authorization] Checking policy '{}' for city '{}' by user '{}'",
        policy_name, city_id, claims.id);

    // ROOT has implicit total access
    if claims.profile == PROFILE_ROOT {
        info!("[Authorization] ROOT user - implicit access granted");
        return Ok(());
    }

    // Check if the policy exists and contains the city_id
    if let Some(city_ids) = user_policies.get(policy_name) {
        if let Some(city_array) = city_ids.as_array() {
            for cid in city_array {
                if let Some(cid_str) = cid.as_str() {
                    if let Ok(parsed_cid) = Uuid::parse_str(cid_str) {
                        if parsed_cid == city_id {
                            info!("[Authorization] Policy '{}' found for city '{}'", policy_name, city_id);
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    warn!("[Authorization] Policy '{}' not found for city '{}' for user '{}'",
        policy_name, city_id, claims.id);
    Err(AppError::Forbidden(
        format!("You don't have permission to perform '{}' for this city", policy_name)
    ))
}

/// Check if a user has a specific policy for any city
/// Returns the list of city IDs where the user has the policy
///
/// # Arguments
/// * `claims` - The user's JWT claims
/// * `policy_name` - The policy to check
/// * `user_policies` - The user's permission_policies as JSON
///
/// # Returns
/// * `Some(Vec<Uuid>)` - List of city IDs where the user has the policy
/// * `None` - If the user is ROOT (has access to all cities)
pub fn get_allowed_cities_for_policy(
    claims: &ClaimsToUserToken,
    policy_name: &str,
    user_policies: &JsonValue,
) -> Option<Vec<Uuid>> {
    info!("[Authorization] Getting allowed cities for policy '{}' by user '{}'",
        policy_name, claims.id);

    // ROOT has implicit total access - return None to indicate "all cities"
    if claims.profile == PROFILE_ROOT {
        info!("[Authorization] ROOT user - access to all cities");
        return None;
    }

    let mut allowed_cities = Vec::new();

    if let Some(city_ids) = user_policies.get(policy_name) {
        if let Some(city_array) = city_ids.as_array() {
            for cid in city_array {
                if let Some(cid_str) = cid.as_str() {
                    if let Ok(parsed_cid) = Uuid::parse_str(cid_str) {
                        allowed_cities.push(parsed_cid);
                    }
                }
            }
        }
    }

    info!("[Authorization] Found {} allowed cities for policy '{}'",
        allowed_cities.len(), policy_name);
    Some(allowed_cities)
}

/// Check if a user has a specific policy (without checking for a specific city)
/// This is useful for checking if a user can perform an action at all
///
/// # Arguments
/// * `claims` - The user's JWT claims
/// * `policy_name` - The policy to check
/// * `user_policies` - The user's permission_policies as JSON
///
/// # Returns
/// * `true` if the user has the policy for at least one city, or is ROOT
/// * `false` otherwise
pub fn has_policy(
    claims: &ClaimsToUserToken,
    policy_name: &str,
    user_policies: &JsonValue,
) -> bool {
    // ROOT has implicit total access
    if claims.profile == PROFILE_ROOT {
        return true;
    }

    if let Some(city_ids) = user_policies.get(policy_name) {
        if let Some(city_array) = city_ids.as_array() {
            return !city_array.is_empty();
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_claims(profile: &str, city_id: Option<&str>) -> ClaimsToUserToken {
        ClaimsToUserToken {
            id: "test-user-id".to_string(),
            exp: 9999999999,
            rank: "CAP PM".to_string(),
            registration: "100012345".to_string(),
            full_name: "Test User".to_string(),
            profile: profile.to_string(),
            email: "test@test.com".to_string(),
            city_id: city_id.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_root_has_implicit_access() {
        let claims = create_test_claims("ROOT", None);
        let policies = json!({});
        let city_id = Uuid::new_v4();

        let result = check_policy(&claims, "read_victims", city_id, &policies);
        assert!(result.is_ok());
    }

    #[test]
    fn test_city_admin_with_policy() {
        let city_id = Uuid::new_v4();
        let claims = create_test_claims("CITY_ADMIN", Some(&city_id.to_string()));
        let policies = json!({
            "read_victims": [city_id.to_string()],
            "create_victims": [city_id.to_string()]
        });

        let result = check_policy(&claims, "read_victims", city_id, &policies);
        assert!(result.is_ok());
    }

    #[test]
    fn test_city_admin_without_policy() {
        let city_id = Uuid::new_v4();
        let other_city_id = Uuid::new_v4();
        let claims = create_test_claims("CITY_ADMIN", Some(&city_id.to_string()));
        let policies = json!({
            "read_victims": [city_id.to_string()]
        });

        // Should fail for a different city
        let result = check_policy(&claims, "read_victims", other_city_id, &policies);
        assert!(result.is_err());

        // Should fail for a policy they don't have
        let result = check_policy(&claims, "delete_victims", city_id, &policies);
        assert!(result.is_err());
    }

    #[test]
    fn test_has_policy() {
        let city_id = Uuid::new_v4();
        let claims = create_test_claims("CITY_USER", Some(&city_id.to_string()));
        let policies = json!({
            "read_victims": [city_id.to_string()]
        });

        assert!(has_policy(&claims, "read_victims", &policies));
        assert!(!has_policy(&claims, "create_victims", &policies));
    }

    #[test]
    fn test_get_allowed_cities_for_policy() {
        let city_id1 = Uuid::new_v4();
        let city_id2 = Uuid::new_v4();
        let claims = create_test_claims("CITY_ADMIN", Some(&city_id1.to_string()));
        let policies = json!({
            "read_victims": [city_id1.to_string(), city_id2.to_string()]
        });

        let allowed = get_allowed_cities_for_policy(&claims, "read_victims", &policies);
        assert!(allowed.is_some());
        let cities = allowed.unwrap();
        assert_eq!(cities.len(), 2);
        assert!(cities.contains(&city_id1));
        assert!(cities.contains(&city_id2));
    }

    #[test]
    fn test_root_get_allowed_cities_returns_none() {
        let claims = create_test_claims("ROOT", None);
        let policies = json!({});

        let allowed = get_allowed_cities_for_policy(&claims, "read_victims", &policies);
        assert!(allowed.is_none()); // None indicates "all cities"
    }
}
