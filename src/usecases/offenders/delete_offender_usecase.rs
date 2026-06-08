use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::offenders::OffenderWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::usecases::offenders::deps::OffenderUseCaseDependencies;

pub struct DeleteOffenderUseCase {
    deps: OffenderUseCaseDependencies,
}

impl DeleteOffenderUseCase {
    pub fn new(deps: OffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<OffenderWithDetails, AppError> {
        info!(
            "[DeleteOffenderUseCase] Starting process to delete offender with id: {}",
            id
        );

        match self
            .deps
            .offender_read_repository
            .get_offender_by_id(id)
            .await
        {
            Ok(offender) => {
                let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
                auth.check_policy(&Policy::DeleteOffenders, offender.summary.city_id)?;
            }
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    id
                )));
            }
            Err(e) => {
                error!("[DeleteOffenderUseCase] Error checking offender: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        }

        match self
            .deps
            .offender_write_repository
            .delete_offender_by_id(id)
            .await
        {
            Ok(deleted_offender) => {
                let deleted_offender = OffenderWithDetails::from_write_result(deleted_offender);
                info!(
                    "[DeleteOffenderUseCase] Offender with id {} deleted successfully",
                    id
                );
                Ok(deleted_offender)
            }
            Err(RepositoryError::NotFound) => {
                info!(
                    "[DeleteOffenderUseCase] Offender with id {} not found for deletion",
                    id
                );
                Err(AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    id
                )))
            }
            Err(e) => {
                error!("[DeleteOffenderUseCase] Failed to delete offender: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
}
