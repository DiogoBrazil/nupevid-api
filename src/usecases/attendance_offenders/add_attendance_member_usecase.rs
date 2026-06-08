use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::attendance_members::AddAttendanceMember;
use crate::core::entities::auth::UserClaims;
use crate::core::value_objects::policies::Policy;
use crate::usecases::attendance_offenders::deps::AttendanceOffenderUseCaseDependencies;
use crate::usecases::helpers_common::{
    get_attendance_offender_or_not_found, get_offender_or_not_found,
};

pub struct AddAttendanceOffenderMemberUseCase {
    deps: AttendanceOffenderUseCaseDependencies,
}

impl AddAttendanceOffenderMemberUseCase {
    pub fn new(deps: AttendanceOffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        attendance_id: Uuid,
        data: AddAttendanceMember,
        claims: &UserClaims,
    ) -> Result<String, AppError> {
        info!(
            "[AddAttendanceOffenderMemberUseCase] Adding member to attendance: {}",
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

        let user_to_add = self
            .deps
            .user_repository
            .get_user_by_id(data.user_id)
            .await
            .map_err(|e| {
                if matches!(e, RepositoryError::NotFound) {
                    AppError::NotFound(format!("User '{}' not found", data.user_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        if user_to_add.city_id != Some(offender.summary.city_id) {
            return Err(AppError::BadRequest(
                "User must be from the same city as the offender".to_string(),
            ));
        }

        match self
            .deps
            .attendance_member_repository
            .add_member_to_offender_attendance(attendance_id, data.user_id, None)
            .await
        {
            Ok(_) => {
                info!("[AddAttendanceOffenderMemberUseCase] Member added successfully");
                Ok("Member added successfully".to_string())
            }
            Err(RepositoryError::DuplicateEntry(msg)) => Err(AppError::Conflict(msg)),
            Err(e) => {
                error!(
                    "[AddAttendanceOffenderMemberUseCase] Failed to add member: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
