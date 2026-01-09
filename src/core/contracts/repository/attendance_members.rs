use async_trait::async_trait;
use uuid::Uuid;

use crate::core::entities::attendance_members::{
    AttendanceMemberWithDetails, AttendanceOffenderMember, AttendanceVictimMember,
};

#[async_trait]
pub trait AttendanceMemberRepository: Send + Sync {
    async fn add_member_to_victim_attendance(
        &self,
        attendance_id: Uuid,
        user_id: Uuid,
        work_session_id: Option<Uuid>,
    ) -> Result<AttendanceVictimMember, sqlx::Error>;

    async fn get_victim_attendance_members(
        &self,
        attendance_id: Uuid,
    ) -> Result<Vec<AttendanceMemberWithDetails>, sqlx::Error>;

    async fn remove_member_from_victim_attendance(
        &self,
        attendance_id: Uuid,
        user_id: Uuid,
    ) -> Result<AttendanceVictimMember, sqlx::Error>;

    async fn add_member_to_offender_attendance(
        &self,
        attendance_id: Uuid,
        user_id: Uuid,
        work_session_id: Option<Uuid>,
    ) -> Result<AttendanceOffenderMember, sqlx::Error>;

    async fn get_offender_attendance_members(
        &self,
        attendance_id: Uuid,
    ) -> Result<Vec<AttendanceMemberWithDetails>, sqlx::Error>;

    async fn remove_member_from_offender_attendance(
        &self,
        attendance_id: Uuid,
        user_id: Uuid,
    ) -> Result<AttendanceOffenderMember, sqlx::Error>;
}
