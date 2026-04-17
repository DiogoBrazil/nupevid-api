use log::error;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_helpers::get_user_policies_strict;
use crate::core::authorization::check_policy;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::User;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::users::deps::UserUseCaseDependencies;
use crate::usecases::users::helpers::get_existing_user;

pub struct AppendUserPolicyCitiesUseCase {
    deps: UserUseCaseDependencies,
}

impl AppendUserPolicyCitiesUseCase {
    pub fn new(deps: UserUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        target_user_id: Uuid,
        policy: &Policy,
        city_ids: &[Uuid],
        claims: &UserClaims,
    ) -> Result<User, AppError> {
        if !policy.is_assignable() {
            return Err(AppError::BadRequest(format!(
                "Policy '{}' is not assignable; only ROOT can perform this action",
                policy
            )));
        }

        let target_user =
            get_existing_user(self.deps.user_repository.as_ref(), target_user_id).await?;

        if target_user.profile == Profile::Root && claims.profile != Profile::Root {
            return Err(AppError::Forbidden(
                "Only ROOT can modify ROOT users".to_string(),
            ));
        }

        if claims.profile == Profile::CityUser {
            error!("[AppendUserPolicyCitiesUseCase] CITY_USER cannot assign policies");
            return Err(AppError::Forbidden(
                "CITY_USER profile is not allowed to assign permission policies".to_string(),
            ));
        }

        if claims.profile != Profile::Root {
            let policies = get_user_policies_strict(self.deps.user_repository.as_ref(), claims)
                .await?
                .ok_or_else(|| AppError::Forbidden("User has no policies".to_string()))?;

            if let Some(target_city_id) = target_user.city_id {
                check_policy(claims, &Policy::UpdateUsers, target_city_id, &policies)?;
            }

            let assigned_cities = policies.get(policy).cloned().unwrap_or_default();

            for city_id in city_ids {
                if !assigned_cities.contains(city_id) {
                    return Err(AppError::Forbidden(format!(
                        "You cannot assign policy '{}' for city '{}' you do not possess",
                        policy, city_id
                    )));
                }
            }
        }

        let mut policies = target_user.permission_policies.clone();
        let mut list = policies
            .get(policy.as_str())
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();

        for city_id in city_ids {
            let city_id = city_id.to_string();
            if !list.iter().any(|value| value.as_str() == Some(&city_id)) {
                list.push(serde_json::Value::String(city_id));
            }
        }

        policies[policy.as_str()] = serde_json::Value::Array(list);

        self.deps
            .user_repository
            .update_user_policies_by_id(target_user_id, policies)
            .await
            .map_err(|_| AppError::InternalServerError)
    }
}
