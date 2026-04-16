use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::protective_measures::ProtectiveMeasure;
use crate::core::value_objects::policies::Policy;
use crate::usecases::protective_measures::deps::ProtectiveMeasureUseCaseDependencies;

pub struct DeleteProtectiveMeasureUseCase {
    deps: ProtectiveMeasureUseCaseDependencies,
}

impl DeleteProtectiveMeasureUseCase {
    pub fn new(deps: ProtectiveMeasureUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<ProtectiveMeasure, AppError> {
        info!(
            "[DeleteProtectiveMeasureUseCase] Deleting protective measure: {}",
            id
        );

        let measure = match self
            .deps
            .measure_read_repository
            .get_protective_measure_by_id(id)
            .await
        {
            Ok(m) => m,
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Protective measure with id '{}' not found",
                    id
                )));
            }
            Err(e) => {
                error!(
                    "[DeleteProtectiveMeasureUseCase] Error fetching measure: {:?}",
                    e
                );
                return Err(AppError::InternalServerError);
            }
        };

        let victim = self
            .deps
            .victim_repository
            .get_victim_by_id(measure.victim_id)
            .await
            .map_err(|e| {
                error!(
                    "[DeleteProtectiveMeasureUseCase] Error fetching victim: {:?}",
                    e
                );
                AppError::InternalServerError
            })?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::DeleteProtectiveMeasures, victim.city_id)?;

        match self
            .deps
            .measure_write_repository
            .delete_protective_measure_by_id(id)
            .await
        {
            Ok(deleted_measure) => {
                info!(
                    "[DeleteProtectiveMeasureUseCase] Protective measure deleted successfully: {}",
                    id
                );
                Ok(deleted_measure)
            }
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "Protective measure with id '{}' not found",
                id
            ))),
            Err(e) => {
                error!(
                    "[DeleteProtectiveMeasureUseCase] Failed to delete measure: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
