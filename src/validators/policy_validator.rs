use crate::core::commands::users::PermissionPolicies;
use crate::core::entities::auth::ClaimsToUserToken;
use crate::utils::errors::AppError;
use crate::validators::common::*;
use uuid::Uuid;

pub struct PolicyValidator;

impl PolicyValidator {
    pub fn validate_policy_names(policies: &PermissionPolicies) -> Result<(), AppError> {
        for policy_name in policies.keys() {
            if !is_valid_policy(policy_name) {
                return Err(AppError::BadRequest(format!(
                    "Invalid policy name '{}'. Valid policies are: {:?}",
                    policy_name, VALID_POLICIES
                )));
            }
        }
        Ok(())
    }

    pub fn validate_assignment_permission(
        claims: &ClaimsToUserToken,
        target_profile: &str,
        policies: &PermissionPolicies,
        claims_policies_json: Option<&serde_json::Value>,
    ) -> Result<(), AppError> {
        if claims.profile == PROFILE_ROOT {
            return Ok(());
        }

        if claims.profile == PROFILE_CITY_USER {
            if !policies.is_empty() {
                return Err(AppError::Forbidden(
                    "CITY_USER profile is not allowed to assign permission policies".to_string(),
                ));
            }
            return Ok(());
        }

        if claims.profile == PROFILE_CITY_ADMIN {
            if target_profile != PROFILE_CITY_USER {
                if !policies.is_empty() {
                    return Err(AppError::Forbidden(
                        "CITY_ADMIN can only assign permission policies to CITY_USER profiles"
                            .to_string(),
                    ));
                }
                return Ok(());
            }

            for (policy_name, city_ids) in policies.iter() {
                for city_id in city_ids {
                    let mut has_policy = false;
                    if let Some(pjson) = claims_policies_json
                        && let Some(arr) = pjson.get(policy_name).and_then(|v| v.as_array())
                    {
                        has_policy = arr
                            .iter()
                            .filter_map(|v| v.as_str())
                            .any(|cid| Uuid::parse_str(cid).ok() == Some(*city_id));
                    }
                    if !has_policy {
                        return Err(AppError::Forbidden(format!(
                            "CITY_ADMIN cannot assign policy '{}' for city '{}' that they do not possess",
                            policy_name, city_id
                        )));
                    }
                }
            }
            return Ok(());
        }

        Err(AppError::Forbidden("Permission denied".to_string()))
    }

    pub fn validate_policies_are_assignable(policies: &PermissionPolicies) -> Result<(), AppError> {
        for policy_name in policies.keys() {
            if !is_assignable_policy(policy_name) {
                return Err(AppError::Forbidden(format!(
                    "Policy '{}' cannot be assigned as it is reserved for ROOT users",
                    policy_name
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    fn create_test_claims(profile: &str, user_id: Uuid) -> ClaimsToUserToken {
        ClaimsToUserToken {
            id: user_id.to_string(),
            exp: 9999999999,
            rank: "CEL PM".to_string(),
            registration: "100012345".to_string(),
            full_name: "Test User".to_string(),
            profile: profile.to_string(),
            email: "test@example.com".to_string(),
            city_id: None,
        }
    }

    #[test]
    fn test_validate_policy_names_success() {
        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert(POLICY_READ_VICTIMS.to_string(), vec![Uuid::new_v4()]);
        policies.insert(POLICY_CREATE_VICTIMS.to_string(), vec![Uuid::new_v4()]);

        let result = PolicyValidator::validate_policy_names(&policies);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_policy_names_invalid() {
        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert("invalid_policy".to_string(), vec![Uuid::new_v4()]);

        let result = PolicyValidator::validate_policy_names(&policies);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid policy name")
        );
    }

    #[test]
    fn test_validate_assignment_permission_root_allowed() {
        let claims = create_test_claims(PROFILE_ROOT, Uuid::new_v4());
        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert(POLICY_CREATE_CITIES.to_string(), vec![Uuid::new_v4()]);

        let result = PolicyValidator::validate_assignment_permission(
            &claims,
            PROFILE_CITY_ADMIN,
            &policies,
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_assignment_permission_city_user_forbidden() {
        let claims = create_test_claims(PROFILE_CITY_USER, Uuid::new_v4());
        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert(POLICY_READ_VICTIMS.to_string(), vec![Uuid::new_v4()]);

        let result = PolicyValidator::validate_assignment_permission(
            &claims,
            PROFILE_CITY_USER,
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
        let claims = create_test_claims(PROFILE_CITY_ADMIN, Uuid::new_v4());
        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert(POLICY_READ_VICTIMS.to_string(), vec![Uuid::new_v4()]);

        let result = PolicyValidator::validate_assignment_permission(
            &claims,
            PROFILE_CITY_ADMIN,
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
        let claims = create_test_claims(PROFILE_CITY_ADMIN, Uuid::new_v4());

        let policies_json = json!({
            POLICY_READ_VICTIMS: [city_id.to_string()]
        });

        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert(POLICY_READ_VICTIMS.to_string(), vec![city_id]);

        let result = PolicyValidator::validate_assignment_permission(
            &claims,
            PROFILE_CITY_USER,
            &policies,
            Some(&policies_json),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_assignment_permission_city_admin_lacks_policy() {
        let city_id = Uuid::new_v4();
        let claims = create_test_claims(PROFILE_CITY_ADMIN, Uuid::new_v4());

        let policies_json = json!({});

        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert(POLICY_READ_VICTIMS.to_string(), vec![city_id]);

        let result = PolicyValidator::validate_assignment_permission(
            &claims,
            PROFILE_CITY_USER,
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
        policies.insert(POLICY_READ_VICTIMS.to_string(), vec![Uuid::new_v4()]);

        let result = PolicyValidator::validate_policies_are_assignable(&policies);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_policies_are_assignable_forbidden() {
        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert(POLICY_CREATE_CITIES.to_string(), vec![Uuid::new_v4()]);

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
