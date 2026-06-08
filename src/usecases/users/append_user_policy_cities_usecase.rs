use log::error;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::User;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::helpers_common::get_user_or_not_found;
use crate::usecases::users::deps::UserUseCaseDependencies;

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
            get_user_or_not_found(self.deps.user_repository.as_ref(), target_user_id).await?;

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
            let auth = AuthContext::load(self.deps.user_repository.as_ref(), claims).await?;

            if let Some(target_city_id) = target_user.city_id {
                auth.check_policy(&Policy::UpdateUsers, target_city_id)?;
            }

            let assigned_cities = auth.policies.get(policy).cloned().unwrap_or_default();

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
        let entry = policies.entry(policy.clone()).or_default();
        for city_id in city_ids {
            if !entry.contains(city_id) {
                entry.push(*city_id);
            }
        }

        self.deps
            .user_repository
            .update_user_policies_by_id(target_user_id, policies)
            .await
            .map_err(|_| AppError::InternalServerError)
    }
}
