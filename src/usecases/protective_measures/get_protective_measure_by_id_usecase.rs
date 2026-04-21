use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::protective_measures::ProtectiveMeasure;
use crate::core::value_objects::policies::Policy;
use crate::usecases::helpers_common::{
    get_protective_measure_or_not_found, get_victim_or_not_found,
};
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

        let measure =
            get_protective_measure_or_not_found(&*self.deps.measure_read_repository, id).await?;

        let victim =
            get_victim_or_not_found(&*self.deps.victim_repository, measure.victim_id).await?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::ReadProtectiveMeasures, victim.city_id)?;

        info!(
            "[GetProtectiveMeasureByIdUseCase] Protective measure found: {}",
            id
        );
        Ok(measure)
    }
}
