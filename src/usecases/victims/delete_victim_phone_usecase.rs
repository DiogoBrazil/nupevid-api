use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::victims::VictimPhoneResponse;
use crate::core::value_objects::policies::Policy;
use crate::usecases::victims::deps::VictimUseCaseDependencies;
use crate::usecases::victims::helpers::authorize_victim_access;

pub struct DeleteVictimPhoneUseCase {
    deps: VictimUseCaseDependencies,
}

impl DeleteVictimPhoneUseCase {
    pub fn new(deps: VictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        phone_id: Uuid,
        claims: &UserClaims,
    ) -> Result<VictimPhoneResponse, AppError> {
        info!("[DeleteVictimPhoneUseCase] Deleting phone: {}", phone_id);

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;

        match self
            .deps
            .victim_write_repository
            .get_phone_by_id(phone_id)
            .await
        {
            Ok(phone) => {
                authorize_victim_access(
                    &auth,
                    &*self.deps.victim_read_repository,
                    phone.victim_id,
                    &Policy::UpdateVictims,
                )
                .await?;
            }
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Phone with id '{}' not found",
                    phone_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self
            .deps
            .victim_write_repository
            .delete_phone_by_id(phone_id)
            .await
        {
            Ok(phone) => Ok(phone.into()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
