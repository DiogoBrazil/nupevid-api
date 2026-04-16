use log::{error, info};
use uuid::Uuid;

use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::protective_measures::ProtectiveMeasure;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::usecases::protective_measures::deps::ProtectiveMeasureUseCaseDependencies;

pub struct GetProtectiveMeasureByIdUseCase {
    deps: ProtectiveMeasureUseCaseDependencies,
}

impl GetProtectiveMeasureByIdUseCase {
    pub fn new(deps: ProtectiveMeasureUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<ProtectiveMeasure, AppError> {
        info!(
            "[GetProtectiveMeasureByIdUseCase] Getting protective measure by id: {}",
            id
        );

        match self
            .deps
            .measure_read_repository
            .get_protective_measure_by_id(id)
            .await
        {
            Ok(measure) => {
                let victim = self
                    .deps
                    .victim_repository
                    .get_victim_by_id(measure.victim_id)
                    .await
                    .map_err(|e| {
                        error!("[GetProtectiveMeasureByIdUseCase] Error fetching victim: {:?}", e);
                        AppError::InternalServerError
                    })?;

                let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
                auth.check_policy(&Policy::ReadProtectiveMeasures, victim.city_id)?;

                info!(
                    "[GetProtectiveMeasureByIdUseCase] Protective measure found: {}",
                    id
                );
                Ok(measure)
            }
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "Protective measure with id '{}' not found",
                id
            ))),
            Err(e) => {
                error!("[GetProtectiveMeasureByIdUseCase] Database error: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
}
