use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViolenceAggravator {
    AlcoholUse,
    DrugUse,
    PsychiatricIssues,
    Other,
}

impl ViolenceAggravator {
    pub fn as_str(&self) -> &str {
        match self {
            Self::AlcoholUse => "AlcoholUse",
            Self::DrugUse => "DrugUse",
            Self::PsychiatricIssues => "PsychiatricIssues",
            Self::Other => "Other",
        }
    }
}

impl TryFrom<&str> for ViolenceAggravator {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "AlcoholUse" => Ok(Self::AlcoholUse),
            "DrugUse" => Ok(Self::DrugUse),
            "PsychiatricIssues" => Ok(Self::PsychiatricIssues),
            "Other" => Ok(Self::Other),
            other => Err(format!("Invalid violence aggravator: '{}'", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceOffender {
    pub id: Uuid,
    pub offender_id: Uuid,
    pub victim_id: Uuid,
    pub protective_measure_id: Option<Uuid>,
    pub was_offender_present: bool,
    pub attendance_date: NaiveDate,
    pub attendance_time: NaiveTime,
    pub is_remote: bool,
    pub assaults_children: bool,
    pub violence_aggravator: ViolenceAggravator,
    pub violence_aggravator_other: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceOffenderAddress {
    pub id: Uuid,
    pub attendance_id: Uuid,
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceOffenderWriteResult {
    pub attendance: AttendanceOffender,
    pub address: Option<AttendanceOffenderAddress>,
}
