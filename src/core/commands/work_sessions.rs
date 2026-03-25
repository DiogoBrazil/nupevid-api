use serde::{Deserialize, Serialize};

use crate::core::commands::work_session_members::AddWorkSessionMember;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateWorkSession {
    pub description: Option<String>,
    pub members: Vec<AddWorkSessionMember>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateWorkSessionMembers {
    pub members: Vec<AddWorkSessionMember>,
}
