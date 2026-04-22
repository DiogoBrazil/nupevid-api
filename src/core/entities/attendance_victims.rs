use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    High,
    Medium,
    Low,
}

impl RiskLevel {
    pub fn as_str(&self) -> &str {
        match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        }
    }
}

impl TryFrom<&str> for RiskLevel {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "High" => Ok(Self::High),
            "Medium" => Ok(Self::Medium),
            "Low" => Ok(Self::Low),
            other => Err(format!("Invalid risk level: '{}'", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OffenderFreedomStatus {
    Imprisoned,
    Free,
    Monitored,
}

impl OffenderFreedomStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Imprisoned => "Imprisoned",
            Self::Free => "Free",
            Self::Monitored => "Monitored",
        }
    }
}

impl TryFrom<&str> for OffenderFreedomStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Imprisoned" => Ok(Self::Imprisoned),
            "Free" => Ok(Self::Free),
            "Monitored" => Ok(Self::Monitored),
            other => Err(format!("Invalid offender freedom status: '{}'", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OffenderFirearmAccess {
    Yes,
    No,
    Unknown,
}

impl OffenderFirearmAccess {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Yes => "Yes",
            Self::No => "No",
            Self::Unknown => "Unknown",
        }
    }
}

impl TryFrom<&str> for OffenderFirearmAccess {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Yes" => Ok(Self::Yes),
            "No" => Ok(Self::No),
            "Unknown" => Ok(Self::Unknown),
            other => Err(format!("Invalid offender firearm access: '{}'", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceVictim {
    pub id: Uuid,
    pub victim_id: Uuid,
    pub was_victim_present: bool,
    pub attendance_date: NaiveDate,
    pub attendance_time: NaiveTime,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub offender_id: Option<Uuid>,
    pub protective_measure_id: Option<Uuid>,
    pub is_remote: bool,
    pub risk_level: Option<RiskLevel>,
    pub offender_freedom_status: Option<OffenderFreedomStatus>,
    pub offender_has_firearm_access: Option<OffenderFirearmAccess>,
    pub needs_legal_assistance: bool,
    pub needs_psychological_support: bool,
    pub was_instructed_about_protective_measure_procedures: bool,
    pub offender_violated_protective_measure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceVictimAddress {
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
pub struct AttendanceVictimWriteResult {
    pub attendance: AttendanceVictim,
    pub address: Option<AttendanceVictimAddress>,
}
