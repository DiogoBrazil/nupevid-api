use log::info;
use uuid::Uuid;

use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_victims::AttendanceVictimWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::usecases::attendance_victims::deps::AttendanceVictimUseCaseDependencies;

pub struct GetAttendanceVictimByIdUseCase {
    deps: AttendanceVictimUseCaseDependencies,
}

impl GetAttendanceVictimByIdUseCase {
    pub fn new(deps: AttendanceVictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<AttendanceVictimWithAddress, AppError> {
        info!(
            "[GetAttendanceVictimByIdUseCase] Getting attendance victim with ID: {}",
            id
        );

        match self
            .deps
            .attendance_victim_read_repository
            .get_attendance_victim_by_id(id)
            .await
        {
            Ok(attendance_with_address) => {
                let victim = self
                    .deps
                    .victim_repository
                    .get_victim_by_id(attendance_with_address.victim_id)
                    .await
                    .map_err(|e| match e {
                        RepositoryError::NotFound => AppError::NotFound(format!(
                            "Victim with id '{}' not found",
                            attendance_with_address.victim_id
                        )),
                        _ => AppError::InternalServerError,
                    })?;
                let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
                auth.check_policy(&Policy::ReadAttendances, victim.city_id)?;
                Ok(attendance_with_address)
            }
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "Attendance victim '{}' not found",
                id
            ))),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
