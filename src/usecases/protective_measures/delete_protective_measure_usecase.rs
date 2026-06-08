use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::protective_measures::ProtectiveMeasure;
use crate::core::value_objects::policies::Policy;
use crate::usecases::helpers_common::{
    get_protective_measure_or_not_found, get_victim_or_not_found,
};
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

        let measure =
            get_protective_measure_or_not_found(&*self.deps.measure_read_repository, id).await?;

        let victim =
            get_victim_or_not_found(&*self.deps.victim_repository, measure.victim_id).await?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::DeleteProtectiveMeasures, victim.summary.city_id)?;

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
