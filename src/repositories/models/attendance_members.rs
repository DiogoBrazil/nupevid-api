use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::core::entities::attendance_members::{AttendanceOffenderMember, AttendanceVictimMember};

#[derive(Debug, Clone, FromRow)]
pub struct AttendanceVictimMemberRow {
    pub id: Uuid,
    pub attendance_victim_id: Uuid,
    pub user_id: Uuid,
    pub work_session_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<AttendanceVictimMemberRow> for AttendanceVictimMember {
    fn from(row: AttendanceVictimMemberRow) -> Self {
        AttendanceVictimMember {
            id: row.id,
            attendance_victim_id: row.attendance_victim_id,
            user_id: row.user_id,
            work_session_id: row.work_session_id,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct AttendanceOffenderMemberRow {
    pub id: Uuid,
    pub attendance_offender_id: Uuid,
    pub user_id: Uuid,
    pub work_session_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<AttendanceOffenderMemberRow> for AttendanceOffenderMember {
    fn from(row: AttendanceOffenderMemberRow) -> Self {
        AttendanceOffenderMember {
            id: row.id,
            attendance_offender_id: row.attendance_offender_id,
            user_id: row.user_id,
            work_session_id: row.work_session_id,
            created_at: row.created_at,
        }
    }
}
