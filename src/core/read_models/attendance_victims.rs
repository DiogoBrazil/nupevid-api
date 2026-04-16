use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::entities::attendance_victims::{
    AttendanceVictim, AttendanceVictimAddress, AttendanceVictimWriteResult,
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

impl From<AttendanceVictimAddress> for AttendanceAddressResponse {
    fn from(address: AttendanceVictimAddress) -> Self {
        AttendanceAddressResponse {
            id: address.id,
            street: address.street,
            number: address.number,
            district: address.district,
            city_id: address.city_id,
            zip_code: address.zip_code,
            complement: address.complement,
        }
    }
}

impl AttendanceVictimWithAddress {
    pub fn from_entity(
        attendance: AttendanceVictim,
        address: Option<AttendanceVictimAddress>,
    ) -> Self {
        AttendanceVictimWithAddress {
            id: attendance.id,
            victim_id: attendance.victim_id,
            was_victim_present: attendance.was_victim_present,
            attendance_date: attendance.attendance_date,
            attendance_time: attendance.attendance_time,
            description: attendance.description,
            latitude: attendance.latitude,
            longitude: attendance.longitude,
            created_at: attendance.created_at,
            updated_at: attendance.updated_at,
            is_deleted: attendance.is_deleted,
            address: address.map(Into::into),
            offender_id: attendance.offender_id,
            protective_measure_id: attendance.protective_measure_id,
            is_remote: attendance.is_remote,
            risk_level: attendance.risk_level,
            offender_freedom_status: attendance.offender_freedom_status,
            offender_has_firearm_access: attendance.offender_has_firearm_access,
            needs_legal_assistance: attendance.needs_legal_assistance,
            needs_psychological_support: attendance.needs_psychological_support,
            was_instructed_about_protective_measure_procedures: attendance
                .was_instructed_about_protective_measure_procedures,
            offender_violated_protective_measure: attendance.offender_violated_protective_measure,
        }
    }

    pub fn from_write_result(result: AttendanceVictimWriteResult) -> Self {
        Self::from_entity(result.attendance, result.address)
    }
}
