use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::User;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::helpers_common::get_user_or_not_found;
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

        if user.profile == Profile::Root && claims.profile != Profile::Root {
            return Err(AppError::Forbidden(
                "Only ROOT can access ROOT users".to_string(),
            ));
        }

        if claims.profile != Profile::Root
            && let Some(user_city_id) = user.city_id
        {
            let auth = AuthContext::load(self.deps.user_repository.as_ref(), claims).await?;
            auth.check_policy(&Policy::ReadUsers, user_city_id)?;
        }

        info!(
            "[GetUserByIdUseCase] User with id {} found successfully",
            id
        );
        Ok(user)
    }
}
