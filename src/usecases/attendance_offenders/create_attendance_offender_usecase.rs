use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::commands::attendance_offenders::CreateAttendanceOffender;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::usecases::attendance_offenders::deps::AttendanceOffenderUseCaseDependencies;
use crate::usecases::attendance_offenders::helpers::{
    get_offender_or_not_found, get_victim_or_not_found,
};
use crate::usecases::attendance_victims::helpers::get_active_session_members;

pub struct CreateAttendanceOffenderUseCase {
    deps: AttendanceOffenderUseCaseDependencies,
}

impl CreateAttendanceOffenderUseCase {
    pub fn new(deps: AttendanceOffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        attendance: CreateAttendanceOffender,
        claims: &UserClaims,
    ) -> Result<AttendanceOffenderWithAddress, AppError> {
        info!("[CreateAttendanceOffenderUseCase] Creating attendance offender");

        let offender =
            get_offender_or_not_found(&*self.deps.offender_repository, attendance.offender_id)
                .await?;

        get_victim_or_not_found(&*self.deps.victim_repository, attendance.victim_id).await?;

        if let Some(pm_id) = attendance.protective_measure_id {
            match self
                .deps
                .protective_measure_repository
                .get_protective_measure_by_id(pm_id)
                .await
            {
                Ok(_) => {}
                Err(RepositoryError::NotFound) => {
                    return Err(AppError::NotFound(format!(
                        "Protective measure with id '{}' not found",
                        pm_id
                    )));
                }
                Err(e) => {
                    error!(
                        "[CreateAttendanceOffenderUseCase] Error checking protective measure: {:?}",
                        e
                    );
                    return Err(AppError::InternalServerError);
                }
            }
        }

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::CreateAttendances, offender.city_id)?;

        let members_for_tx =
            get_active_session_members(claims, &*self.deps.work_session_repository).await?;

        match self
            .deps
            .attendance_offender_write_repository
            .create_attendance_offender(attendance, members_for_tx)
            .await
        {
            Ok(attendance_with_address) => {
                let attendance_with_address =
                    AttendanceOffenderWithAddress::from_write_result(attendance_with_address);
                info!(
                    "[CreateAttendanceOffenderUseCase] Attendance offender created: {}",
                    attendance_with_address.id
                );
                Ok(attendance_with_address)
            }
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => Err(AppError::BadRequest(msg)),
            Err(e) => {
                error!(
                    "[CreateAttendanceOffenderUseCase] Failed to create attendance offender: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
