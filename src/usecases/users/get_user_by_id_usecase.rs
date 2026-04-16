use log::info;
use uuid::Uuid;

use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::UserRecord;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::authorization::check_policy;
use crate::usecases::users::deps::UserUseCaseDependencies;
use crate::usecases::users::helpers::{get_claims_policies_or_empty, get_existing_user};

pub struct GetUserByIdUseCase {
    deps: UserUseCaseDependencies,
}

impl GetUserByIdUseCase {
    pub fn new(deps: UserUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<UserRecord, AppError> {
        let user = get_existing_user(self.deps.user_repository.as_ref(), id).await?;

        if user.profile == Profile::Root && claims.profile != Profile::Root {
            return Err(AppError::Forbidden(
                "Only ROOT can access ROOT users".to_string(),
            ));
        }

        if claims.profile != Profile::Root
            && let Some(user_city_id) = user.city_id
        {
            let policies =
                get_claims_policies_or_empty(self.deps.user_repository.as_ref(), claims).await?;
            check_policy(claims, &Policy::ReadUsers, user_city_id, &policies)?;
        }

        info!("[GetUserByIdUseCase] User with id {} found successfully", id);
        Ok(user)
    }
}
