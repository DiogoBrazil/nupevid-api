use log::{error, info};
use uuid::Uuid;

use crate::core::commands::attendance_victims::CreateAttendanceVictim;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_victims::AttendanceVictimWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::usecases::attendance_victims::deps::AttendanceVictimUseCaseDependencies;
use crate::usecases::attendance_victims::helpers::verify_victim_access;

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

        let user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

        let victim =
            verify_victim_access(&*self.deps.victim_repository, attendance.victim_id).await?;
        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::CreateAttendances, victim.city_id)?;

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
            .attendance_victim_write_repository
            .create_attendance_victim(attendance, members_for_tx)
            .await
        {
            Ok(attendance_with_address) => {
                let attendance_with_address = AttendanceVictimWithAddress::from_write_result(attendance_with_address);
                info!(
                    "[CreateAttendanceVictimUseCase] Attendance victim created: {}",
                    attendance_with_address.id
                );
                Ok(attendance_with_address)
            }
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => {
                Err(AppError::BadRequest(msg))
            }
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
