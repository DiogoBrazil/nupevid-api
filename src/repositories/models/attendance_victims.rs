use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::core::entities::attendance_victims::{
    AttendanceVictim, AttendanceVictimAddress, OffenderFirearmAccess, OffenderFreedomStatus,
    RiskLevel,
};

#[derive(Debug, Clone, FromRow)]
pub struct AttendanceVictimRow {
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

impl From<AttendanceVictimRow> for AttendanceVictim {
    fn from(row: AttendanceVictimRow) -> Self {
        AttendanceVictim {
            id: row.id,
            victim_id: row.victim_id,
            was_victim_present: row.was_victim_present,
            attendance_date: row.attendance_date,
            attendance_time: row.attendance_time,
            description: row.description,
            latitude: row.latitude,
            longitude: row.longitude,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
            offender_id: row.offender_id,
            protective_measure_id: row.protective_measure_id,
            is_remote: row.is_remote,
            risk_level: row.risk_level,
            offender_freedom_status: row.offender_freedom_status,
            offender_has_firearm_access: row.offender_has_firearm_access,
            needs_legal_assistance: row.needs_legal_assistance,
            needs_psychological_support: row.needs_psychological_support,
            was_instructed_about_protective_measure_procedures: row
                .was_instructed_about_protective_measure_procedures,
            offender_violated_protective_measure: row.offender_violated_protective_measure,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct AttendanceVictimAddressRow {
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

impl From<AttendanceVictimAddressRow> for AttendanceVictimAddress {
    fn from(row: AttendanceVictimAddressRow) -> Self {
        AttendanceVictimAddress {
            id: row.id,
            attendance_id: row.attendance_id,
            street: row.street,
            number: row.number,
            district: row.district,
            city_id: row.city_id,
            zip_code: row.zip_code,
            complement: row.complement,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
        }
    }
}
