use log::{error, info};
use uuid::Uuid;

use crate::core::commands::attendance_offenders::CreateAttendanceOffender;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::usecases::attendance_offenders::deps::AttendanceOffenderUseCaseDependencies;
use crate::usecases::attendance_offenders::helpers::{verify_offender_access, verify_victim_access};

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

        let user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

        let offender =
            verify_offender_access(&*self.deps.offender_repository, attendance.offender_id)
                .await?;

        let _victim =
            verify_victim_access(&*self.deps.victim_repository, attendance.victim_id).await?;

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

        let active_session = self
            .deps
            .work_session_repository
            .get_active_session_by_user(user_id)
            .await
            .map_err(|_| {
                AppError::BadRequest(
                    "No active work session found. You must have an active work session to create an attendance.".to_string(),
                )
            })?;

        let session_members = self
            .deps
            .work_session_repository
            .get_session_members(active_session.id)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let members_for_tx: Vec<(Uuid, Option<Uuid>)> = session_members
            .iter()
            .map(|m| (m.user_id, Some(active_session.id)))
            .collect();

        match self
            .deps
            .attendance_offender_write_repository
            .create_attendance_offender(attendance, members_for_tx)
            .await
        {
            Ok(attendance_with_address) => {
                let attendance_with_address = AttendanceOffenderWithAddress::from_write_result(attendance_with_address);
                info!(
                    "[CreateAttendanceOffenderUseCase] Attendance offender created: {}",
                    attendance_with_address.id
                );
                Ok(attendance_with_address)
            }
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => {
                Err(AppError::BadRequest(msg))
            }
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
