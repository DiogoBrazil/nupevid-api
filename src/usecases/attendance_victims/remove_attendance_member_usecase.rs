use log::{error, info};
use uuid::Uuid;

use crate::core::entities::auth::UserClaims;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::usecases::attendance_victims::deps::AttendanceVictimUseCaseDependencies;
use crate::usecases::attendance_victims::helpers::{
    get_attendance_victim_or_not_found, verify_victim_access,
};

pub struct RemoveAttendanceMemberUseCase {
    deps: AttendanceVictimUseCaseDependencies,
}

impl RemoveAttendanceMemberUseCase {
    pub fn new(deps: AttendanceVictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        attendance_id: Uuid,
        user_id: Uuid,
        claims: &UserClaims,
    ) -> Result<String, AppError> {
        info!(
            "[RemoveAttendanceMemberUseCase] Removing member from attendance: {}",
            attendance_id
        );

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;

        let attendance = get_attendance_victim_or_not_found(
            &*self.deps.attendance_victim_read_repository,
            attendance_id,
            "RemoveAttendanceMemberUseCase",
        )
        .await?;

        let victim =
            verify_victim_access(&*self.deps.victim_repository, attendance.victim_id).await?;
        auth.check_policy(&Policy::ManageAttendanceMembers, victim.city_id)?;

        match self
            .deps
            .attendance_member_repository
            .remove_member_from_victim_attendance(attendance_id, user_id)
            .await
        {
            Ok(_) => {
                info!("[RemoveAttendanceMemberUseCase] Member removed successfully");
                Ok("Member removed successfully".to_string())
            }
            Err(e) => {
                error!(
                    "[RemoveAttendanceMemberUseCase] Failed to remove member: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
