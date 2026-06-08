use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_victims::AttendanceVictimWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::usecases::attendance_victims::deps::AttendanceVictimUseCaseDependencies;
use crate::usecases::helpers_common::{
    get_protective_measure_or_not_found, get_victim_or_not_found,
};

pub struct GetAttendanceVictimsByMeasureUseCase {
    deps: AttendanceVictimUseCaseDependencies,
}

impl GetAttendanceVictimsByMeasureUseCase {
    pub fn new(deps: AttendanceVictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        protective_measure_id: Uuid,
        claims: &UserClaims,
    ) -> Result<Vec<AttendanceVictimWithAddress>, AppError> {
        info!(
            "[GetAttendanceVictimsByMeasureUseCase] Getting attendance victims for protective measure: {}",
            protective_measure_id
        );

        let pm = get_protective_measure_or_not_found(
            &*self.deps.protective_measure_repository,
            protective_measure_id,
        )
        .await?;
        let victim = get_victim_or_not_found(&*self.deps.victim_repository, pm.victim_id).await?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::ReadAttendances, victim.summary.city_id)?;

        self.deps
            .attendance_victim_read_repository
            .get_attendance_victims_by_protective_measure(protective_measure_id)
            .await
            .map_err(|_| AppError::InternalServerError)
    }
}
