use chrono::Local;
use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::commands::attendance_offenders::CreateAttendanceOffender;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::usecases::attendance_offenders::deps::AttendanceOffenderUseCaseDependencies;
use crate::usecases::attendance_victims::helpers::get_active_session_members;
use crate::usecases::helpers_common::{
    get_offender_or_not_found, get_protective_measure_or_not_found, get_victim_or_not_found,
};
use crate::validators::attendance_validator::AttendanceValidator;

pub struct CreateAttendanceOffenderUseCase {
    deps: AttendanceOffenderUseCaseDependencies,
}

impl CreateAttendanceOffenderUseCase {
    pub fn new(deps: AttendanceOffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        attendance: CreateAttendanceOffender,
        claims: &UserClaims,
    ) -> Result<AttendanceOffenderWithAddress, AppError> {
        info!("[CreateAttendanceOffenderUseCase] Creating attendance offender");

        AttendanceValidator::validate_attendance_date_not_future(
            attendance.attendance_date,
            Local::now().date_naive(),
        )?;
        AttendanceValidator::validate_violence_aggravator(
            &attendance.violence_aggravator,
            &attendance.violence_aggravator_other,
        )?;

        let pm = get_protective_measure_or_not_found(
            &*self.deps.protective_measure_repository,
            attendance.protective_measure_id,
        )
        .await?;

        let offender =
            get_offender_or_not_found(&*self.deps.offender_repository, pm.offender_id).await?;

        get_victim_or_not_found(&*self.deps.victim_repository, pm.victim_id).await?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::CreateAttendances, offender.summary.city_id)?;

        let members_for_tx =
            get_active_session_members(claims, &*self.deps.work_session_repository).await?;

        match self
            .deps
            .attendance_offender_write_repository
            .create_attendance_offender(attendance, pm.offender_id, pm.victim_id, members_for_tx)
            .await
        {
            Ok(attendance_with_address) => {
                let attendance_with_address =
                    AttendanceOffenderWithAddress::from_write_result(attendance_with_address);
                info!(
                    "[CreateAttendanceOffenderUseCase] Attendance offender created: {}",
                    attendance_with_address.id
                );
                Ok(attendance_with_address)
            }
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => Err(AppError::BadRequest(msg)),
            Err(e) => {
                error!(
                    "[CreateAttendanceOffenderUseCase] Failed to create attendance offender: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
