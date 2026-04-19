use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_helpers::PolicyMap;
use crate::core::commands::users::PermissionPolicies;
use crate::core::entities::auth::UserClaims;
use crate::core::value_objects::profiles::Profile;

pub struct PolicyValidator;

impl PolicyValidator {
    pub fn validate_assignment_permission(
        claims: &UserClaims,
        target_profile: &Profile,
        policies: &PermissionPolicies,
        claims_policies: Option<&PolicyMap>,
    ) -> Result<(), AppError> {
        match &claims.profile {
            Profile::Root => Ok(()),
            Profile::CityUser => {
                if !policies.is_empty() {
                    return Err(AppError::Forbidden(
                        "CITY_USER profile is not allowed to assign permission policies"
                            .to_string(),
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
                        let has_policy = claims_policies
                            .and_then(|cp| cp.get(policy))
                            .is_some_and(|cities| cities.contains(city_id));
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
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_claims(profile: Profile, user_id: Uuid) -> UserClaims {
        UserClaims {
            id: user_id.to_string(),
            exp: 9999999999,
            iss: "nupevid-api".to_string(),
            aud: "nupevid-api".to_string(),
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

        let mut claims_policies: PolicyMap = HashMap::new();
        claims_policies.insert(Policy::ReadVictims, vec![city_id]);

        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert(Policy::ReadVictims, vec![city_id]);

        let result = PolicyValidator::validate_assignment_permission(
            &claims,
            &Profile::CityUser,
            &policies,
            Some(&claims_policies),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_assignment_permission_city_admin_lacks_policy() {
        let city_id = Uuid::new_v4();
        let claims = create_test_claims(Profile::CityAdmin, Uuid::new_v4());

        let claims_policies: PolicyMap = HashMap::new();

        let mut policies: PermissionPolicies = HashMap::new();
        policies.insert(Policy::ReadVictims, vec![city_id]);

        let result = PolicyValidator::validate_assignment_permission(
            &claims,
            &Profile::CityUser,
            &policies,
            Some(&claims_policies),
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
