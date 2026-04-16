use log::info;
use uuid::Uuid;

use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::usecases::attendance_offenders::deps::AttendanceOffenderUseCaseDependencies;

pub struct GetAttendanceOffenderByIdUseCase {
    deps: AttendanceOffenderUseCaseDependencies,
}

impl GetAttendanceOffenderByIdUseCase {
    pub fn new(deps: AttendanceOffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<AttendanceOffenderWithAddress, AppError> {
        info!(
            "[GetAttendanceOffenderByIdUseCase] Getting attendance offender with ID: {}",
            id
        );

        match self
            .deps
            .attendance_offender_read_repository
            .get_attendance_offender_by_id(id)
            .await
        {
            Ok(attendance_with_address) => {
                let offender = self
                    .deps
                    .offender_repository
                    .get_offender_by_id(attendance_with_address.offender_id)
                    .await
                    .map_err(|e| match e {
                        RepositoryError::NotFound => AppError::NotFound(format!(
                            "Offender with id '{}' not found",
                            attendance_with_address.offender_id
                        )),
                        _ => AppError::InternalServerError,
                    })?;

                let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
                auth.check_policy(&Policy::ReadAttendances, offender.city_id)?;

                Ok(attendance_with_address)
            }
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "Attendance offender '{}' not found",
                id
            ))),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
