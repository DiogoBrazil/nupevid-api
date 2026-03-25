use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use crate::core::entities::attendance_members::{AttendanceOffenderMember, AttendanceVictimMember};
use crate::core::read_models::attendance_members::AttendanceMemberWithDetails;

#[async_trait]
pub trait AttendanceMemberRepository: Send + Sync {
    async fn add_member_to_victim_attendance(
        &self,
        attendance_id: Uuid,
        user_id: Uuid,
        work_session_id: Option<Uuid>,
    ) -> Result<AttendanceVictimMember, RepositoryError>;

    async fn get_victim_attendance_members(
        &self,
        attendance_id: Uuid,
    ) -> Result<Vec<AttendanceMemberWithDetails>, RepositoryError>;

    async fn remove_member_from_victim_attendance(
        &self,
        attendance_id: Uuid,
        user_id: Uuid,
    ) -> Result<AttendanceVictimMember, RepositoryError>;

    async fn add_member_to_offender_attendance(
        &self,
        attendance_id: Uuid,
        user_id: Uuid,
        work_session_id: Option<Uuid>,
    ) -> Result<AttendanceOffenderMember, RepositoryError>;

    async fn get_offender_attendance_members(
        &self,
        attendance_id: Uuid,
    ) -> Result<Vec<AttendanceMemberWithDetails>, RepositoryError>;

    async fn remove_member_from_offender_attendance(
        &self,
        attendance_id: Uuid,
        user_id: Uuid,
    ) -> Result<AttendanceOffenderMember, RepositoryError>;
}
