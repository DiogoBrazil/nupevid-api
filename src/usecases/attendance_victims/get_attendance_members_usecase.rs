use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_members::AttendanceMemberWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::usecases::attendance_victims::deps::AttendanceVictimUseCaseDependencies;
use crate::usecases::helpers_common::{
    get_attendance_victim_or_not_found, get_victim_or_not_found,
};

pub struct GetAttendanceMembersUseCase {
    deps: AttendanceVictimUseCaseDependencies,
}

impl GetAttendanceMembersUseCase {
    pub fn new(deps: AttendanceVictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        attendance_id: Uuid,
        claims: &UserClaims,
    ) -> Result<Vec<AttendanceMemberWithDetails>, AppError> {
        info!(
            "[GetAttendanceMembersUseCase] Getting members for attendance: {}",
            attendance_id
        );

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;

        let attendance = get_attendance_victim_or_not_found(
            &*self.deps.attendance_victim_read_repository,
            attendance_id,
        )
        .await?;

        let victim =
            get_victim_or_not_found(&*self.deps.victim_repository, attendance.victim_id).await?;
        auth.check_policy(&Policy::ReadAttendances, victim.summary.city_id)?;

        let members = self
            .deps
            .attendance_member_repository
            .get_victim_attendance_members(attendance_id)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        info!(
            "[GetAttendanceMembersUseCase] Found {} members",
            members.len()
        );
        Ok(members)
    }
}
