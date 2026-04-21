use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::entities::work_session_members::TeamMemberFunction;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AddWorkSessionMember {
    pub user_id: Uuid,
    pub function: Option<TeamMemberFunction>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct UpdateMemberFunction {
    pub function: Option<TeamMemberFunction>,
}
