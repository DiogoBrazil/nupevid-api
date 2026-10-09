use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::attendance_offenders::AttendanceOffenderReadRepository;
use crate::core::contracts::repository::attendance_victims::AttendanceVictimReadRepository;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::offenders::OffenderReadRepository;
use crate::core::contracts::repository::protective_measures::ProtectiveMeasureReadRepository;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::victims::VictimReadRepository;
use crate::core::entities::protective_measures::ProtectiveMeasure;
use crate::core::entities::users::User;
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;
use crate::core::read_models::attendance_victims::AttendanceVictimWithAddress;
use crate::core::read_models::offenders::OffenderWithDetails;
use crate::core::read_models::victims::VictimWithDetails;

// "Not found" constructors shared between genuine lookups and the
// out-of-scope-by-ID paths (check_policy_or_not_found), so both responses are
// byte-identical and cannot be used to probe for resource existence.

pub fn victim_not_found_error(id: Uuid) -> AppError {
    AppError::NotFound(format!("Victim with id '{}' not found", id))
}

pub fn offender_not_found_error(id: Uuid) -> AppError {
    AppError::NotFound(format!("Offender with id '{}' not found", id))
}

pub fn attendance_victim_not_found_error(id: Uuid) -> AppError {
    AppError::NotFound(format!("Attendance victim '{}' not found", id))
}

pub fn attendance_offender_not_found_error(id: Uuid) -> AppError {
    AppError::NotFound(format!("Attendance offender '{}' not found", id))
}

pub fn protective_measure_not_found_error(id: Uuid) -> AppError {
    AppError::NotFound(format!("Protective measure '{}' not found", id))
}

pub fn user_not_found_error(id: Uuid) -> AppError {
    AppError::NotFound(format!("User with id '{}' not found", id))
}

pub async fn get_victim_or_not_found(
    repo: &dyn VictimReadRepository,
    id: Uuid,
) -> Result<VictimWithDetails, AppError> {
    repo.get_victim_by_id(id).await.map_err(|e| match e {
        RepositoryError::NotFound => victim_not_found_error(id),
        _ => AppError::InternalServerError,
    })
}

pub async fn get_offender_or_not_found(
    repo: &dyn OffenderReadRepository,
    id: Uuid,
) -> Result<OffenderWithDetails, AppError> {
    repo.get_offender_by_id(id).await.map_err(|e| match e {
        RepositoryError::NotFound => offender_not_found_error(id),
        _ => AppError::InternalServerError,
    })
}

pub async fn get_attendance_victim_or_not_found(
    repo: &dyn AttendanceVictimReadRepository,
    id: Uuid,
) -> Result<AttendanceVictimWithAddress, AppError> {
    repo.get_attendance_victim_by_id(id)
        .await
        .map_err(|e| match e {
            RepositoryError::NotFound => attendance_victim_not_found_error(id),
            _ => AppError::InternalServerError,
        })
}

pub async fn get_attendance_offender_or_not_found(
    repo: &dyn AttendanceOffenderReadRepository,
    id: Uuid,
) -> Result<AttendanceOffenderWithAddress, AppError> {
    repo.get_attendance_offender_by_id(id)
        .await
        .map_err(|e| match e {
            RepositoryError::NotFound => attendance_offender_not_found_error(id),
            _ => AppError::InternalServerError,
        })
}

pub async fn get_protective_measure_or_not_found(
    repo: &dyn ProtectiveMeasureReadRepository,
    id: Uuid,
) -> Result<ProtectiveMeasure, AppError> {
    repo.get_protective_measure_by_id(id)
        .await
        .map_err(|e| match e {
            RepositoryError::NotFound => protective_measure_not_found_error(id),
            _ => AppError::InternalServerError,
        })
}

pub async fn get_user_or_not_found(repo: &dyn UserRepository, id: Uuid) -> Result<User, AppError> {
    repo.get_user_by_id(id).await.map_err(|e| match e {
        RepositoryError::NotFound => user_not_found_error(id),
        _ => AppError::InternalServerError,
    })
}
