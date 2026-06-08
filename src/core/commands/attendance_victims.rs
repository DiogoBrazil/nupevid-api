use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::entities::attendance_victims::{
    OffenderFirearmAccess, OffenderFreedomStatus, RiskLevel,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AttendanceAddressData {
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CreateAttendanceVictim {
    pub protective_measure_id: Uuid,
    pub was_victim_present: bool,
    pub attendance_date: NaiveDate,
    pub attendance_time: NaiveTime,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<AttendanceAddressData>,
    pub is_remote: bool,
    pub risk_level: Option<RiskLevel>,
    pub offender_freedom_status: Option<OffenderFreedomStatus>,
    pub offender_has_firearm_access: Option<OffenderFirearmAccess>,
    pub needs_legal_assistance: bool,
    pub needs_psychological_support: bool,
    pub was_instructed_about_protective_measure_procedures: bool,
    pub offender_violated_protective_measure: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct UpdateAttendanceVictim {
    pub protective_measure_id: Uuid,
    pub was_victim_present: bool,
    pub attendance_date: NaiveDate,
    pub attendance_time: NaiveTime,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<AttendanceAddressData>,
    pub is_remote: bool,
    pub risk_level: Option<RiskLevel>,
    pub offender_freedom_status: Option<OffenderFreedomStatus>,
    pub offender_has_firearm_access: Option<OffenderFirearmAccess>,
    pub needs_legal_assistance: bool,
    pub needs_psychological_support: bool,
    pub was_instructed_about_protective_measure_procedures: bool,
    pub offender_violated_protective_measure: bool,
}
