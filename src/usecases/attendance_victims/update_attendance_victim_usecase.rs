use chrono::Local;
use log::error;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::commands::attendance_victims::UpdateAttendanceVictim;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_victims::AttendanceVictimWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::usecases::attendance_victims::deps::AttendanceVictimUseCaseDependencies;
use crate::usecases::helpers_common::{
    get_attendance_victim_or_not_found, get_protective_measure_or_not_found,
    get_victim_or_not_found,
};
use crate::validators::attendance_validator::AttendanceValidator;

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

        AttendanceValidator::validate_attendance_date_not_future(
            data.attendance_date,
            Local::now().date_naive(),
        )?;

        let existing =
            get_attendance_victim_or_not_found(&*self.deps.attendance_victim_read_repository, id)
                .await?;

        let existing_victim =
            get_victim_or_not_found(&*self.deps.victim_repository, existing.victim_id).await?;
        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::UpdateAttendances, existing_victim.summary.city_id)?;

        let pm = get_protective_measure_or_not_found(
            &*self.deps.protective_measure_repository,
            data.protective_measure_id,
        )
        .await?;
        let new_victim_id = pm.victim_id;
        let new_offender_id = Some(pm.offender_id);

        if new_victim_id != existing.victim_id {
            let new_victim =
                get_victim_or_not_found(&*self.deps.victim_repository, new_victim_id).await?;
            auth.check_policy(&Policy::UpdateAttendances, new_victim.summary.city_id)?;
        }

        match self
            .deps
            .attendance_victim_write_repository
            .update_attendance_victim_by_id(data, id, new_victim_id, new_offender_id)
            .await
        {
            Ok(attendance_with_address) => Ok(AttendanceVictimWithAddress::from_write_result(
                attendance_with_address,
            )),
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "Attendance victim '{}' not found",
                id
            ))),
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => Err(AppError::BadRequest(msg)),
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
