use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::attendance_offenders::AttendanceOffenderReadRepository;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::offenders::OffenderReadRepository;
use crate::core::contracts::repository::victims::VictimReadRepository;
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;
use crate::core::read_models::offenders::OffenderWithDetails;
use crate::core::read_models::victims::VictimWithDetails;

pub async fn get_attendance_offender_or_not_found(
    repository: &dyn AttendanceOffenderReadRepository,
    id: Uuid,
) -> Result<AttendanceOffenderWithAddress, AppError> {
    repository
        .get_attendance_offender_by_id(id)
        .await
        .map_err(|e| {
            if matches!(e, RepositoryError::NotFound) {
                AppError::NotFound(format!("Attendance offender '{}' not found", id))
            } else {
                AppError::InternalServerError
            }
        })
}

pub async fn get_offender_or_not_found(
    offender_repository: &dyn OffenderReadRepository,
    offender_id: Uuid,
) -> Result<OffenderWithDetails, AppError> {
    offender_repository
        .get_offender_by_id(offender_id)
        .await
        .map_err(|e| {
            if matches!(e, RepositoryError::NotFound) {
                AppError::NotFound(format!("Offender '{}' not found", offender_id))
            } else {
                AppError::InternalServerError
            }
        })
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
