use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::usecases::attendance_offenders::deps::AttendanceOffenderUseCaseDependencies;
use crate::usecases::helpers_common::{
    get_attendance_offender_or_not_found, get_offender_or_not_found,
};

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

        let attendance_with_address = get_attendance_offender_or_not_found(
            &*self.deps.attendance_offender_read_repository,
            id,
        )
        .await?;

        let offender = get_offender_or_not_found(
            &*self.deps.offender_repository,
            attendance_with_address.offender_id,
        )
        .await?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::ReadAttendances, offender.city_id)?;

        Ok(attendance_with_address)
    }
}
