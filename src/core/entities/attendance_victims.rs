use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "risk_level", rename_all = "PascalCase")]
pub enum RiskLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "offender_freedom_status", rename_all = "PascalCase")]
pub enum OffenderFreedomStatus {
    Imprisoned,
    Free,
    Monitored,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "offender_firearm_access", rename_all = "PascalCase")]
pub enum OffenderFirearmAccess {
    Yes,
    No,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttendanceAddressData {
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAttendanceVictim {
    pub victim_id: Uuid,
    pub was_victim_present: bool,
    pub attendance_date: NaiveDate,
    pub attendance_time: NaiveTime,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<AttendanceAddressData>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateAttendanceVictim {
    pub victim_id: Uuid,
    pub was_victim_present: bool,
    pub attendance_date: NaiveDate,
    pub attendance_time: NaiveTime,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<AttendanceAddressData>,
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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

impl AttendanceVictimAddress {
    pub fn to_response(self) -> AttendanceAddressResponse {
        AttendanceAddressResponse {
            id: self.id,
            street: self.street,
            number: self.number,
            district: self.district,
            city_id: self.city_id,
            zip_code: self.zip_code,
            complement: self.complement,
        }
    }
}

impl AttendanceVictim {
    pub fn with_address(
        self,
        address: Option<AttendanceVictimAddress>,
    ) -> AttendanceVictimWithAddress {
        AttendanceVictimWithAddress {
            id: self.id,
            victim_id: self.victim_id,
            was_victim_present: self.was_victim_present,
            attendance_date: self.attendance_date,
            attendance_time: self.attendance_time,
            description: self.description,
            latitude: self.latitude,
            longitude: self.longitude,
            created_at: self.created_at,
            updated_at: self.updated_at,
            is_deleted: self.is_deleted,
            address: address.map(|a| a.to_response()),
            offender_id: self.offender_id,
            protective_measure_id: self.protective_measure_id,
            is_remote: self.is_remote,
            risk_level: self.risk_level,
            offender_freedom_status: self.offender_freedom_status,
            offender_has_firearm_access: self.offender_has_firearm_access,
            needs_legal_assistance: self.needs_legal_assistance,
            needs_psychological_support: self.needs_psychological_support,
            was_instructed_about_protective_measure_procedures: self
                .was_instructed_about_protective_measure_procedures,
            offender_violated_protective_measure: self.offender_violated_protective_measure,
        }
    }
}
