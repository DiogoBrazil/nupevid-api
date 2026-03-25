use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::core::entities::attendance_offenders::{
    AttendanceOffender, AttendanceOffenderAddress, ViolenceAggravator,
};

#[derive(Debug, Clone, FromRow)]
pub struct AttendanceOffenderRow {
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

impl From<AttendanceOffenderRow> for AttendanceOffender {
    fn from(row: AttendanceOffenderRow) -> Self {
        AttendanceOffender {
            id: row.id,
            offender_id: row.offender_id,
            victim_id: row.victim_id,
            protective_measure_id: row.protective_measure_id,
            was_offender_present: row.was_offender_present,
            attendance_date: row.attendance_date,
            attendance_time: row.attendance_time,
            is_remote: row.is_remote,
            assaults_children: row.assaults_children,
            violence_aggravator: row.violence_aggravator,
            violence_aggravator_other: row.violence_aggravator_other,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct AttendanceOffenderAddressRow {
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

impl From<AttendanceOffenderAddressRow> for AttendanceOffenderAddress {
    fn from(row: AttendanceOffenderAddressRow) -> Self {
        AttendanceOffenderAddress {
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
