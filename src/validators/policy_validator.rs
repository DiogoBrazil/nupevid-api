use crate::core::commands::users::PermissionPolicies;
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::value_objects::profiles::Profile;
use crate::utils::errors::AppError;
use uuid::Uuid;

pub struct PolicyValidator;

impl PolicyValidator {
    pub fn validate_assignment_permission(
        claims: &ClaimsToUserToken,
        target_profile: &Profile,
        policies: &PermissionPolicies,
        claims_policies_json: Option<&serde_json::Value>,
    ) -> Result<(), AppError> {
        match &claims.profile {
            Profile::Root => Ok(()),
            Profile::CityUser => {
                if !policies.is_empty() {
                    return Err(AppError::Forbidden(
                        "CITY_USER profile is not allowed to assign permission policies".to_string(),
                    ));
                }
                Ok(())
            }
            Profile::CityAdmin => {
                if *target_profile != Profile::CityUser {
                    if !policies.is_empty() {
                        return Err(AppError::Forbidden(
                            "CITY_ADMIN can only assign permission policies to CITY_USER profiles"
                                .to_string(),
                        ));
                    }
                    return Ok(());
                }

                for (policy, city_ids) in policies.iter() {
                    for city_id in city_ids {
                        let mut has_policy = false;
                        if let Some(pjson) = claims_policies_json
                            && let Some(arr) =
                                pjson.get(policy.as_str()).and_then(|v| v.as_array())
                        {
                            has_policy = arr
                                .iter()
                                .filter_map(|v| v.as_str())
                                .any(|cid| Uuid::parse_str(cid).ok() == Some(*city_id));
                        }
                        if !has_policy {
                            return Err(AppError::Forbidden(format!(
                                "CITY_ADMIN cannot assign policy '{}' for city '{}' that they do not possess",
                                policy, city_id
                            )));
                        }
                    }
                }
                Ok(())
            }
        }
    }

    pub fn validate_policies_are_assignable(policies: &PermissionPolicies) -> Result<(), AppError> {
        for policy in policies.keys() {
            if !policy.is_assignable() {
                return Err(AppError::Forbidden(format!(
                    "Policy '{}' cannot be assigned as it is reserved for ROOT users",
                    policy
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value_objects::policies::Policy;
    use crate::core::value_objects::ranks::Rank;
    use serde_json::json;
    use std::collections::HashMap;

    fn create_test_claims(profile: Profile, user_id: Uuid) -> ClaimsToUserToken {
        ClaimsToUserToken {
            id: user_id.to_string(),
            exp: 9999999999,
            rank: Rank::CelPm,
            registration: "100012345".to_string(),
            full_name: "Test User".to_string(),
            profile,
            email: "test@example.com".to_string(),
            city_id: None,
        }
    }

    #[test]
    fn test_validate_assignment_permission_root_allowed() {
        let claims = create_test_claims(Profile::Root, Uuid::new_v4());
        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert(Policy::CreateCities, vec![Uuid::new_v4()]);

        let result = PolicyValidator::validate_assignment_permission(
            &claims,
            &Profile::CityAdmin,
            &policies,
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_assignment_permission_city_user_forbidden() {
        let claims = create_test_claims(Profile::CityUser, Uuid::new_v4());
        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert(Policy::ReadVictims, vec![Uuid::new_v4()]);

        let result = PolicyValidator::validate_assignment_permission(
            &claims,
            &Profile::CityUser,
            &policies,
            None,
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("CITY_USER profile is not allowed")
        );
    }

    #[test]
    fn test_validate_assignment_permission_city_admin_to_city_admin_forbidden() {
        let claims = create_test_claims(Profile::CityAdmin, Uuid::new_v4());
        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert(Policy::ReadVictims, vec![Uuid::new_v4()]);

        let result = PolicyValidator::validate_assignment_permission(
            &claims,
            &Profile::CityAdmin,
            &policies,
            None,
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("can only assign permission policies to CITY_USER")
        );
    }

    #[test]
    fn test_validate_assignment_permission_city_admin_success() {
        let city_id = Uuid::new_v4();
        let claims = create_test_claims(Profile::CityAdmin, Uuid::new_v4());

        let policies_json = json!({
            "read_victims": [city_id.to_string()]
        });

        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert(Policy::ReadVictims, vec![city_id]);

        let result = PolicyValidator::validate_assignment_permission(
            &claims,
            &Profile::CityUser,
            &policies,
            Some(&policies_json),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_assignment_permission_city_admin_lacks_policy() {
        let city_id = Uuid::new_v4();
        let claims = create_test_claims(Profile::CityAdmin, Uuid::new_v4());

        let policies_json = json!({});

        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert(Policy::ReadVictims, vec![city_id]);

        let result = PolicyValidator::validate_assignment_permission(
            &claims,
            &Profile::CityUser,
            &policies,
            Some(&policies_json),
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot assign policy")
        );
    }

    #[test]
    fn test_validate_policies_are_assignable_success() {
        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert(Policy::ReadVictims, vec![Uuid::new_v4()]);

        let result = PolicyValidator::validate_policies_are_assignable(&policies);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_policies_are_assignable_forbidden() {
        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert(Policy::CreateCities, vec![Uuid::new_v4()]);

        let result = PolicyValidator::validate_policies_are_assignable(&policies);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("reserved for ROOT")
        );
    }
}
