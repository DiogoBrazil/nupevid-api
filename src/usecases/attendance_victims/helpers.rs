use log::error;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::attendance_victims::AttendanceVictimReadRepository;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::victims::VictimReadRepository;
use crate::core::contracts::repository::work_sessions::WorkSessionReadRepository;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_victims::AttendanceVictimWithAddress;
use crate::core::read_models::victims::VictimWithDetails;

pub async fn get_attendance_victim_or_not_found(
    repository: &dyn AttendanceVictimReadRepository,
    id: Uuid,
    label: &str,
) -> Result<AttendanceVictimWithAddress, AppError> {
    repository
        .get_attendance_victim_by_id(id)
        .await
        .map_err(|e| {
            if matches!(e, RepositoryError::NotFound) {
                AppError::NotFound(format!("Attendance victim '{}' not found", id))
            } else {
                error!("[{}] Database error: {:?}", label, e);
                AppError::InternalServerError
            }
        })
}

pub async fn get_active_session_members(
    claims: &UserClaims,
    work_session_repository: &dyn WorkSessionReadRepository,
) -> Result<Vec<(Uuid, Option<Uuid>)>, AppError> {
    let user_id = Uuid::parse_str(&claims.id)
        .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

    let active_session = work_session_repository
        .get_active_session_by_user(user_id)
        .await
        .map_err(|_| {
            AppError::BadRequest(
                "No active work session found. You must have an active work session to create an attendance.".to_string(),
            )
        })?;

    let session_members = work_session_repository
        .get_session_members(active_session.id)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    Ok(session_members
        .iter()
        .map(|m| (m.user_id, Some(active_session.id)))
        .collect())
}

pub async fn get_victim_or_not_found(
    victim_repository: &dyn VictimReadRepository,
    victim_id: Uuid,
) -> Result<VictimWithDetails, AppError> {
    victim_repository
        .get_victim_by_id(victim_id)
        .await
        .map_err(|e| {
            if matches!(e, RepositoryError::NotFound) {
                AppError::NotFound(format!("Victim '{}' not found", victim_id))
            } else {
                AppError::InternalServerError
            }
        })
}
