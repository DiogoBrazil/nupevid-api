use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::usecases::attendance_offenders::deps::AttendanceOffenderUseCaseDependencies;
use crate::usecases::helpers_common::get_victim_or_not_found;

pub struct GetAttendanceOffendersByVictimUseCase {
    deps: AttendanceOffenderUseCaseDependencies,
}

impl GetAttendanceOffendersByVictimUseCase {
    pub fn new(deps: AttendanceOffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        victim_id: Uuid,
        claims: &UserClaims,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, AppError> {
        info!(
            "[GetAttendanceOffendersByVictimUseCase] Getting attendance offenders for victim: {}",
            victim_id
        );

        let victim = get_victim_or_not_found(&*self.deps.victim_repository, victim_id).await?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::ReadAttendances, victim.city_id)?;

        match self
            .deps
            .attendance_offender_read_repository
            .get_attendance_offenders_by_victim(victim_id)
            .await
        {
            Ok(attendances_list) => Ok(attendances_list),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
