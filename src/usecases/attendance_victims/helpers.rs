use log::error;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::attendance_victims::AttendanceVictimReadRepository;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::victims::VictimReadRepository;
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

pub async fn verify_victim_access(
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
