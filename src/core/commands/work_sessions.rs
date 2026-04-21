use serde::{Deserialize, Serialize};

use crate::core::commands::work_session_members::AddWorkSessionMember;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CreateWorkSession {
    pub description: Option<String>,
    pub members: Vec<AddWorkSessionMember>,
}

pub type UpdateWorkSession = CreateWorkSession;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct UpdateWorkSessionMembers {
    pub members: Vec<AddWorkSessionMember>,
}
