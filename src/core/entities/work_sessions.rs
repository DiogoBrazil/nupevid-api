use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::work_session_members::WorkSessionMemberWithUser;
use crate::core::read_models::work_sessions::{
    WorkSessionMemberWithDetails, WorkSessionWithMembers, WorkSessionWithMembersComplement,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSession {
    pub id: Uuid,
    pub created_by_user_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WorkSession {
    pub fn with_members(
        self,
        members: Vec<WorkSessionMemberWithDetails>,
    ) -> WorkSessionWithMembers {
        WorkSessionWithMembers {
            id: self.id,
            created_by_user_id: self.created_by_user_id,
            started_at: self.started_at,
            ended_at: self.ended_at,
            is_active: self.is_active,
            description: self.description,
            created_at: self.created_at,
            updated_at: self.updated_at,
            members,
        }
    }

    pub fn with_members_complement(
        self,
        members: Vec<WorkSessionMemberWithUser>,
    ) -> WorkSessionWithMembersComplement {
        WorkSessionWithMembersComplement {
            id: self.id,
            created_by_user_id: self.created_by_user_id,
            started_at: self.started_at,
            ended_at: self.ended_at,
            is_active: self.is_active,
            description: self.description,
            created_at: self.created_at,
            updated_at: self.updated_at,
            members,
        }
    }
}
