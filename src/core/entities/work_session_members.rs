use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TeamMemberFunction {
    Commander,
    Driver,
    Patroller,
}

impl TeamMemberFunction {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Commander => "Commander",
            Self::Driver => "Driver",
            Self::Patroller => "Patroller",
        }
    }
}

impl TryFrom<&str> for TeamMemberFunction {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Commander" => Ok(Self::Commander),
            "Driver" => Ok(Self::Driver),
            "Patroller" => Ok(Self::Patroller),
            other => Err(format!("Invalid team member function: '{}'", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSessionMember {
    pub id: Uuid,
    pub work_session_id: Uuid,
    pub user_id: Uuid,
    pub function: Option<TeamMemberFunction>,
    pub created_at: DateTime<Utc>,
}
