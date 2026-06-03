use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::entities::attendance_offenders::{
    AttendanceOffender, AttendanceOffenderAddress, AttendanceOffenderWriteResult,
    ViolenceAggravator,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceOffenderAddressResponse {
    pub id: Uuid,
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceOffenderWithAddress {
    pub id: Uuid,
    pub offender_id: Uuid,
    pub victim_id: Uuid,
    pub protective_measure_id: Uuid,
    pub was_offender_present: bool,
    pub attendance_date: NaiveDate,
    pub attendance_time: NaiveTime,
    pub is_remote: bool,
    pub assaults_children: bool,
    pub violence_aggravator: Option<ViolenceAggravator>,
    pub violence_aggravator_other: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub address: Option<AttendanceOffenderAddressResponse>,
}

impl From<AttendanceOffenderAddress> for AttendanceOffenderAddressResponse {
    fn from(address: AttendanceOffenderAddress) -> Self {
        AttendanceOffenderAddressResponse {
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

impl AttendanceOffenderWithAddress {
    pub fn from_entity(
        attendance: AttendanceOffender,
        address: Option<AttendanceOffenderAddress>,
    ) -> Self {
        AttendanceOffenderWithAddress {
            id: attendance.id,
            offender_id: attendance.offender_id,
            victim_id: attendance.victim_id,
            protective_measure_id: attendance.protective_measure_id,
            was_offender_present: attendance.was_offender_present,
            attendance_date: attendance.attendance_date,
            attendance_time: attendance.attendance_time,
            is_remote: attendance.is_remote,
            assaults_children: attendance.assaults_children,
            violence_aggravator: attendance.violence_aggravator,
            violence_aggravator_other: attendance.violence_aggravator_other,
            description: attendance.description,
            created_at: attendance.created_at,
            updated_at: attendance.updated_at,
            is_deleted: attendance.is_deleted,
            address: address.map(Into::into),
        }
    }

    pub fn from_write_result(result: AttendanceOffenderWriteResult) -> Self {
        Self::from_entity(result.attendance, result.address)
    }
}
