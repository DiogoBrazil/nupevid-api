use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::commands::attendance_victims::CreateAttendanceVictim;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_victims::AttendanceVictimWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::usecases::attendance_offenders::helpers::load_pm_or_not_found;
use crate::usecases::attendance_victims::deps::AttendanceVictimUseCaseDependencies;
use crate::usecases::attendance_victims::helpers::{
    get_active_session_members, get_victim_or_not_found,
};

pub struct CreateAttendanceVictimUseCase {
    deps: AttendanceVictimUseCaseDependencies,
}

impl CreateAttendanceVictimUseCase {
    pub fn new(deps: AttendanceVictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        attendance: CreateAttendanceVictim,
        claims: &UserClaims,
    ) -> Result<AttendanceVictimWithAddress, AppError> {
        info!("[CreateAttendanceVictimUseCase] Creating attendance victim");

        let pm = load_pm_or_not_found(
            &*self.deps.protective_measure_repository,
            attendance.protective_measure_id,
        )
        .await?;
        let victim = get_victim_or_not_found(&*self.deps.victim_repository, pm.victim_id).await?;
        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::CreateAttendances, victim.city_id)?;

        let members_for_tx =
            get_active_session_members(claims, &*self.deps.work_session_repository).await?;

        match self
            .deps
            .attendance_victim_write_repository
            .create_attendance_victim(
                attendance,
                pm.victim_id,
                Some(pm.offender_id),
                members_for_tx,
            )
            .await
        {
            Ok(attendance_with_address) => {
                let attendance_with_address =
                    AttendanceVictimWithAddress::from_write_result(attendance_with_address);
                info!(
                    "[CreateAttendanceVictimUseCase] Attendance victim created: {}",
                    attendance_with_address.id
                );
                Ok(attendance_with_address)
            }
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => Err(AppError::BadRequest(msg)),
            Err(e) => {
                error!(
                    "[CreateAttendanceVictimUseCase] Failed to create attendance victim: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
