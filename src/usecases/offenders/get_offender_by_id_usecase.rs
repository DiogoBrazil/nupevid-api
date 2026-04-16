use log::{error, info};
use uuid::Uuid;

use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::offenders::OffenderWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::usecases::offenders::deps::OffenderUseCaseDependencies;

pub struct GetOffenderByIdUseCase {
    deps: OffenderUseCaseDependencies,
}

impl GetOffenderByIdUseCase {
    pub fn new(deps: OffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<OffenderWithDetails, AppError> {
        info!(
            "[GetOffenderByIdUseCase] Starting find offender by id process for id: {}",
            id
        );

        match self.deps.offender_read_repository.get_offender_by_id(id).await {
            Ok(offender_with_details) => {
                let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
                auth.check_policy(&Policy::ReadOffenders, offender_with_details.city_id)?;

                info!(
                    "[GetOffenderByIdUseCase] Offender with id {} found successfully",
                    id
                );
                Ok(offender_with_details)
            }
            Err(RepositoryError::NotFound) => {
                info!("[GetOffenderByIdUseCase] Offender with id {} not found", id);
                Err(AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    id
                )))
            }
            Err(e) => {
                error!(
                    "[GetOffenderByIdUseCase] Database error while finding offender: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
