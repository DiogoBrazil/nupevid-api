use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::entities::work_session_members::TeamMemberFunction;
use crate::core::entities::work_sessions::WorkSession;
use crate::core::read_models::users::UserSummary;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSessionMemberWithDetails {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub function: Option<TeamMemberFunction>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSessionMemberWithUser {
    pub id: Uuid,
    pub function: Option<TeamMemberFunction>,
    pub user: UserSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSessionWithMembers {
    pub id: Uuid,
    pub created_by_user_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub members: Vec<WorkSessionMemberWithDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSessionWithMembersSummary {
    pub id: Uuid,
    pub created_by_user_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub members: Vec<WorkSessionMemberWithUser>,
}

impl WorkSessionWithMembers {
    pub fn from_entity(
        session: WorkSession,
        members: Vec<WorkSessionMemberWithDetails>,
    ) -> Self {
        WorkSessionWithMembers {
            id: session.id,
            created_by_user_id: session.created_by_user_id,
            started_at: session.started_at,
            ended_at: session.ended_at,
            is_active: session.is_active,
            description: session.description,
            created_at: session.created_at,
            updated_at: session.updated_at,
            members,
        }
    }
}

impl WorkSessionWithMembersSummary {
    pub fn from_entity(
        session: WorkSession,
        members: Vec<WorkSessionMemberWithUser>,
    ) -> Self {
        WorkSessionWithMembersSummary {
            id: session.id,
            created_by_user_id: session.created_by_user_id,
            started_at: session.started_at,
            ended_at: session.ended_at,
            is_active: session.is_active,
            description: session.description,
            created_at: session.created_at,
            updated_at: session.updated_at,
            members,
        }
    }
}
