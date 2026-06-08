use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::value_objects::policies::Policy;
use crate::usecases::attendance_offenders::deps::AttendanceOffenderUseCaseDependencies;
use crate::usecases::helpers_common::{
    get_attendance_offender_or_not_found, get_offender_or_not_found,
};

pub struct RemoveAttendanceOffenderMemberUseCase {
    deps: AttendanceOffenderUseCaseDependencies,
}

impl RemoveAttendanceOffenderMemberUseCase {
    pub fn new(deps: AttendanceOffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        attendance_id: Uuid,
        user_id: Uuid,
        claims: &UserClaims,
    ) -> Result<String, AppError> {
        info!(
            "[RemoveAttendanceOffenderMemberUseCase] Removing member from attendance: {}",
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
        auth.check_policy(&Policy::ManageAttendanceMembers, offender.summary.city_id)?;

        match self
            .deps
            .attendance_member_repository
            .remove_member_from_offender_attendance(attendance_id, user_id)
            .await
        {
            Ok(_) => {
                info!("[RemoveAttendanceOffenderMemberUseCase] Member removed successfully");
                Ok("Member removed successfully".to_string())
            }
            Err(e) => {
                error!(
                    "[RemoveAttendanceOffenderMemberUseCase] Failed to remove member: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
