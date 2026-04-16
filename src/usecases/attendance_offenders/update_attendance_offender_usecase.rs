use log::error;
use uuid::Uuid;

use crate::core::commands::attendance_offenders::UpdateAttendanceOffender;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::usecases::attendance_offenders::deps::AttendanceOffenderUseCaseDependencies;
use crate::usecases::attendance_offenders::helpers::{
    get_attendance_offender_or_not_found, verify_offender_access, verify_victim_access,
};

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

        let existing = get_attendance_offender_or_not_found(
            &*self.deps.attendance_offender_read_repository,
            id,
        )
        .await?;

        let existing_offender = self
            .deps
            .offender_repository
            .get_offender_by_id(existing.offender_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    existing.offender_id
                )),
                _ => AppError::InternalServerError,
            })?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::UpdateAttendances, existing_offender.city_id)?;

        if data.offender_id != existing.offender_id {
            let new_offender =
                verify_offender_access(&*self.deps.offender_repository, data.offender_id).await?;
            auth.check_policy(&Policy::UpdateAttendances, new_offender.city_id)?;
        }

        if data.victim_id != existing.victim_id {
            let victim =
                verify_victim_access(&*self.deps.victim_repository, data.victim_id).await?;
            auth.check_policy(&Policy::UpdateAttendances, victim.city_id)?;
        }

        if data.protective_measure_id != existing.protective_measure_id
            && let Some(pm_id) = data.protective_measure_id
        {
            match self
                .deps
                .protective_measure_repository
                .get_protective_measure_by_id(pm_id)
                .await
            {
                Ok(_) => {}
                Err(RepositoryError::NotFound) => {
                    return Err(AppError::NotFound(format!(
                        "Protective measure with id '{}' not found",
                        pm_id
                    )));
                }
                Err(e) => {
                    error!(
                        "[UpdateAttendanceOffenderUseCase] Error checking protective measure: {:?}",
                        e
                    );
                    return Err(AppError::InternalServerError);
                }
            }
        }

        match self
            .deps
            .attendance_offender_write_repository
            .update_attendance_offender_by_id(data, id)
            .await
        {
            Ok(attendance_with_address) => Ok(AttendanceOffenderWithAddress::from_write_result(attendance_with_address)),
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "Attendance offender '{}' not found",
                id
            ))),
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => {
                Err(AppError::BadRequest(msg))
            }
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
