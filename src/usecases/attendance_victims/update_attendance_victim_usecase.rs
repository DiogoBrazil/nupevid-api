use log::error;
use uuid::Uuid;

use crate::core::commands::attendance_victims::UpdateAttendanceVictim;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_victims::AttendanceVictimWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::usecases::attendance_victims::deps::AttendanceVictimUseCaseDependencies;
use crate::usecases::attendance_victims::helpers::{
    get_attendance_victim_or_not_found, verify_victim_access,
};

pub struct UpdateAttendanceVictimUseCase {
    deps: AttendanceVictimUseCaseDependencies,
}

impl UpdateAttendanceVictimUseCase {
    pub fn new(deps: AttendanceVictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        data: UpdateAttendanceVictim,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<AttendanceVictimWithAddress, AppError> {
        log::info!(
            "[UpdateAttendanceVictimUseCase] Updating attendance victim with ID: {}",
            id
        );

        let existing = get_attendance_victim_or_not_found(
            &*self.deps.attendance_victim_read_repository,
            id,
            "UpdateAttendanceVictimUseCase",
        )
        .await?;

        let existing_victim = self
            .deps
            .victim_repository
            .get_victim_by_id(existing.victim_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    AppError::NotFound(format!("Victim with id '{}' not found", existing.victim_id))
                }
                _ => AppError::InternalServerError,
            })?;
        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::UpdateAttendances, existing_victim.city_id)?;

        if data.victim_id != existing.victim_id {
            let new_victim =
                verify_victim_access(&*self.deps.victim_repository, data.victim_id).await?;
            auth.check_policy(&Policy::UpdateAttendances, new_victim.city_id)?;
        }

        match self
            .deps
            .attendance_victim_write_repository
            .update_attendance_victim_by_id(data, id)
            .await
        {
            Ok(attendance_with_address) => Ok(AttendanceVictimWithAddress::from_write_result(attendance_with_address)),
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "Attendance victim '{}' not found",
                id
            ))),
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => {
                Err(AppError::BadRequest(msg))
            }
            Err(e) => {
                error!(
                    "[UpdateAttendanceVictimUseCase] Error updating attendance victim: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
