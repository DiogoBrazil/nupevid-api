use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::protective_measures::ProtectiveMeasure;
use crate::core::value_objects::policies::Policy;
use crate::usecases::helpers_common::get_victim_or_not_found;
use crate::usecases::protective_measures::deps::ProtectiveMeasureUseCaseDependencies;

pub struct GetMeasuresByVictimUseCase {
    deps: ProtectiveMeasureUseCaseDependencies,
}

impl GetMeasuresByVictimUseCase {
    pub fn new(deps: ProtectiveMeasureUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        victim_id: Uuid,
        claims: &UserClaims,
    ) -> Result<Vec<ProtectiveMeasure>, AppError> {
        info!(
            "[GetMeasuresByVictimUseCase] Getting measures for victim: {}",
            victim_id
        );

        let victim = get_victim_or_not_found(&*self.deps.victim_repository, victim_id).await?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::ReadProtectiveMeasures, victim.summary.city_id)?;

        match self
            .deps
            .measure_read_repository
            .get_protective_measures_by_victim(victim_id)
            .await
        {
            Ok(measures) => {
                info!(
                    "[GetMeasuresByVictimUseCase] Found {} measures for victim",
                    measures.len()
                );
                Ok(measures)
            }
            Err(e) => {
                error!(
                    "[GetMeasuresByVictimUseCase] Failed to retrieve measures: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
