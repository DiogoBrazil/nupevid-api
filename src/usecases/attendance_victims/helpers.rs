use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::work_sessions::WorkSessionReadRepository;
use crate::core::entities::auth::UserClaims;

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
