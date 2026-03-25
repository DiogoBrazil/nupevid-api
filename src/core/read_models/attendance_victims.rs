use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::entities::attendance_victims::{
    OffenderFirearmAccess, OffenderFreedomStatus, RiskLevel,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceAddressResponse {
    pub id: Uuid,
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceVictimWithAddress {
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
    pub address: Option<AttendanceAddressResponse>,
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
