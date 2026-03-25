use serde::{Deserialize, Serialize};

use crate::core::read_models::work_sessions::{
    WorkSessionWithMembers, WorkSessionWithMembersComplement,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkSessionResponse {
    Simple(WorkSessionWithMembers),
    WithEntities(WorkSessionWithMembersComplement),
}
