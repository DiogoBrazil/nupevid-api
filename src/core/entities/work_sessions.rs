use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use super::work_session_members::{TeamMemberFunction, AddWorkSessionMember};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateWorkSession {
    pub description: Option<String>,
    pub members: Vec<AddWorkSessionMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkSessionMemberWithDetails {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub function: Option<TeamMemberFunction>,
    pub created_at: DateTime<Utc>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateWorkSessionMembers {
    pub members: Vec<AddWorkSessionMember>,
}

impl WorkSession {
    pub fn with_members(self, members: Vec<WorkSessionMemberWithDetails>) -> WorkSessionWithMembers {
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
}

#[derive(Deserialize, Debug)]
pub struct ListWorkSessionsQuery {
    pub user_id: Option<Uuid>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub city_id: Option<Uuid>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}
