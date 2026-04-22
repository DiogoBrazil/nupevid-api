use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_members::AttendanceMemberWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::usecases::attendance_offenders::deps::AttendanceOffenderUseCaseDependencies;
use crate::usecases::helpers_common::{
    get_attendance_offender_or_not_found, get_offender_or_not_found,
};

pub struct GetAttendanceOffenderMembersUseCase {
    deps: AttendanceOffenderUseCaseDependencies,
}

impl GetAttendanceOffenderMembersUseCase {
    pub fn new(deps: AttendanceOffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        attendance_id: Uuid,
        claims: &UserClaims,
    ) -> Result<Vec<AttendanceMemberWithDetails>, AppError> {
        info!(
            "[GetAttendanceOffenderMembersUseCase] Getting members for attendance: {}",
            attendance_id
        );

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;

        let attendance = get_attendance_offender_or_not_found(
            &*self.deps.attendance_offender_read_repository,
            attendance_id,
        )
        .await?;

        let offender =
            get_offender_or_not_found(&*self.deps.offender_repository, attendance.offender_id)
                .await?;
        auth.check_policy(&Policy::ReadAttendances, offender.city_id)?;

        let members = self
            .deps
            .attendance_member_repository
            .get_offender_attendance_members(attendance_id)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        info!(
            "[GetAttendanceOffenderMembersUseCase] Found {} members",
            members.len()
        );
        Ok(members)
    }
}
