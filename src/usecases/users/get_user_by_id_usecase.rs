use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::User;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::helpers_common::{get_user_or_not_found, user_not_found_error};
use crate::usecases::users::deps::UserUseCaseDependencies;

pub struct GetUserByIdUseCase {
    deps: UserUseCaseDependencies,
}

impl GetUserByIdUseCase {
    pub fn new(deps: UserUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(&self, id: Uuid, claims: &UserClaims) -> Result<User, AppError> {
        let user = get_user_or_not_found(self.deps.user_repository.as_ref(), id).await?;

        // Out-of-scope access is answered with the same "not found" as a
        // missing user, so user existence cannot be probed by ID.
        if user.profile == Profile::Root && claims.profile != Profile::Root {
            return Err(user_not_found_error(id));
        }

        if claims.profile != Profile::Root
            && let Some(user_city_id) = user.city_id
        {
            let auth = AuthContext::load(self.deps.user_repository.as_ref(), claims).await?;
            auth.check_policy_or_not_found(&Policy::ReadUsers, user_city_id, || {
                user_not_found_error(id)
            })?;
        }

        info!(
            "[GetUserByIdUseCase] User with id {} found successfully",
            id
        );
        Ok(user)
    }
}
