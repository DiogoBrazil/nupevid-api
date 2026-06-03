use chrono::Local;
use log::error;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::commands::attendance_offenders::UpdateAttendanceOffender;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::usecases::attendance_offenders::deps::AttendanceOffenderUseCaseDependencies;
use crate::usecases::helpers_common::{
    get_attendance_offender_or_not_found, get_offender_or_not_found,
    get_protective_measure_or_not_found, get_victim_or_not_found,
};
use crate::validators::attendance_validator::AttendanceValidator;

pub struct UpdateAttendanceOffenderUseCase {
    deps: AttendanceOffenderUseCaseDependencies,
}

impl UpdateAttendanceOffenderUseCase {
    pub fn new(deps: AttendanceOffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        data: UpdateAttendanceOffender,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<AttendanceOffenderWithAddress, AppError> {
        log::info!(
            "[UpdateAttendanceOffenderUseCase] Updating attendance offender with ID: {}",
            id
        );

        AttendanceValidator::validate_attendance_date_not_future(
            data.attendance_date,
            Local::now().date_naive(),
        )?;
        AttendanceValidator::validate_violence_aggravator(
            &data.violence_aggravator,
            &data.violence_aggravator_other,
        )?;

        let existing = get_attendance_offender_or_not_found(
            &*self.deps.attendance_offender_read_repository,
            id,
        )
        .await?;

        let existing_offender =
            get_offender_or_not_found(&*self.deps.offender_repository, existing.offender_id)
                .await?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::UpdateAttendances, existing_offender.city_id)?;

        let pm = get_protective_measure_or_not_found(
            &*self.deps.protective_measure_repository,
            data.protective_measure_id,
        )
        .await?;
        let new_offender_id = pm.offender_id;
        let new_victim_id = pm.victim_id;

        if new_offender_id != existing.offender_id {
            let new_offender =
                get_offender_or_not_found(&*self.deps.offender_repository, new_offender_id).await?;
            auth.check_policy(&Policy::UpdateAttendances, new_offender.city_id)?;
        }

        if new_victim_id != existing.victim_id {
            let victim =
                get_victim_or_not_found(&*self.deps.victim_repository, new_victim_id).await?;
            auth.check_policy(&Policy::UpdateAttendances, victim.city_id)?;
        }

        match self
            .deps
            .attendance_offender_write_repository
            .update_attendance_offender_by_id(data, id, new_offender_id, new_victim_id)
            .await
        {
            Ok(attendance_with_address) => Ok(AttendanceOffenderWithAddress::from_write_result(
                attendance_with_address,
            )),
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "Attendance offender '{}' not found",
                id
            ))),
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => Err(AppError::BadRequest(msg)),
            Err(e) => {
                error!(
                    "[UpdateAttendanceOffenderUseCase] Error updating attendance offender: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
