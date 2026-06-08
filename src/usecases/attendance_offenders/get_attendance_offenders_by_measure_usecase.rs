use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::usecases::attendance_offenders::deps::AttendanceOffenderUseCaseDependencies;
use crate::usecases::helpers_common::{
    get_offender_or_not_found, get_protective_measure_or_not_found,
};

pub struct GetAttendanceOffendersByMeasureUseCase {
    deps: AttendanceOffenderUseCaseDependencies,
}

impl GetAttendanceOffendersByMeasureUseCase {
    pub fn new(deps: AttendanceOffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        protective_measure_id: Uuid,
        claims: &UserClaims,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, AppError> {
        info!(
            "[GetAttendanceOffendersByMeasureUseCase] Getting attendance offenders for protective measure: {}",
            protective_measure_id
        );

        let pm = get_protective_measure_or_not_found(
            &*self.deps.protective_measure_repository,
            protective_measure_id,
        )
        .await?;
        let offender =
            get_offender_or_not_found(&*self.deps.offender_repository, pm.offender_id).await?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::ReadAttendances, offender.summary.city_id)?;

        self.deps
            .attendance_offender_read_repository
            .get_attendance_offenders_by_protective_measure(protective_measure_id)
            .await
            .map_err(|_| AppError::InternalServerError)
    }
}
