use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceMemberWithDetails {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub work_session_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
