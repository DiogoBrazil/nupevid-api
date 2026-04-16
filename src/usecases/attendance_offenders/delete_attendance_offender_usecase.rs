use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::usecases::attendance_offenders::deps::AttendanceOffenderUseCaseDependencies;
use crate::usecases::attendance_offenders::helpers::get_attendance_offender_or_not_found;

pub struct DeleteAttendanceOffenderUseCase {
    deps: AttendanceOffenderUseCaseDependencies,
}

impl DeleteAttendanceOffenderUseCase {
    pub fn new(deps: AttendanceOffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<AttendanceOffenderWithAddress, AppError> {
        info!(
            "[DeleteAttendanceOffenderUseCase] Deleting attendance offender with ID: {}",
            id
        );

        let attendance = get_attendance_offender_or_not_found(
            &*self.deps.attendance_offender_read_repository,
            id,
        )
        .await?;

        let offender = self
            .deps
            .offender_repository
            .get_offender_by_id(attendance.offender_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    attendance.offender_id
                )),
                _ => AppError::InternalServerError,
            })?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::DeleteAttendances, offender.city_id)?;

        match self
            .deps
            .attendance_offender_write_repository
            .delete_attendance_offender_by_id(id)
            .await
        {
            Ok(deleted) => Ok(AttendanceOffenderWithAddress::from_write_result(deleted)),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
