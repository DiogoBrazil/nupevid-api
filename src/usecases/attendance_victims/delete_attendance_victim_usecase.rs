use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_victims::AttendanceVictimWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::usecases::attendance_victims::deps::AttendanceVictimUseCaseDependencies;
use crate::usecases::attendance_victims::helpers::get_attendance_victim_or_not_found;

pub struct DeleteAttendanceVictimUseCase {
    deps: AttendanceVictimUseCaseDependencies,
}

impl DeleteAttendanceVictimUseCase {
    pub fn new(deps: AttendanceVictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<AttendanceVictimWithAddress, AppError> {
        info!(
            "[DeleteAttendanceVictimUseCase] Deleting attendance victim with ID: {}",
            id
        );

        let attendance = get_attendance_victim_or_not_found(
            &*self.deps.attendance_victim_read_repository,
            id,
            "DeleteAttendanceVictimUseCase",
        )
        .await?;

        let victim = self
            .deps
            .victim_repository
            .get_victim_by_id(attendance.victim_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    attendance.victim_id
                )),
                _ => AppError::InternalServerError,
            })?;
        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::DeleteAttendances, victim.city_id)?;

        match self
            .deps
            .attendance_victim_write_repository
            .delete_attendance_victim_by_id(id)
            .await
        {
            Ok(deleted) => Ok(AttendanceVictimWithAddress::from_write_result(deleted)),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
