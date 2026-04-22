use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceVictimMember {
    pub id: Uuid,
    pub attendance_victim_id: Uuid,
    pub user_id: Uuid,
    pub work_session_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceOffenderMember {
    pub id: Uuid,
    pub attendance_offender_id: Uuid,
    pub user_id: Uuid,
    pub work_session_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddAttendanceMember {
    pub user_id: Uuid,
}
