use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::Type;
use uuid::Uuid;

use crate::core::entities::attendance_victims::AttendanceAddressData;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "violence_aggravator_enum", rename_all = "PascalCase")]
pub enum ViolenceAggravator {
    AlcoholUse,
    DrugUse,
    PsychiatricIssues,
    Other,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAttendanceOffender {
    pub offender_id: Uuid,
    pub victim_id: Uuid,
    pub protective_measure_id: Option<Uuid>,
    pub was_offender_present: bool,
    pub attendance_date: NaiveDate,
    pub attendance_time: NaiveTime,
    pub address: Option<AttendanceAddressData>,
    pub is_remote: bool,
    pub assaults_children: bool,
    pub violence_aggravator: ViolenceAggravator,
    pub violence_aggravator_other: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateAttendanceOffender {
    pub offender_id: Uuid,
    pub victim_id: Uuid,
    pub protective_measure_id: Option<Uuid>,
    pub was_offender_present: bool,
    pub attendance_date: NaiveDate,
    pub attendance_time: NaiveTime,
    pub address: Option<AttendanceAddressData>,
    pub is_remote: bool,
    pub assaults_children: bool,
    pub violence_aggravator: ViolenceAggravator,
    pub violence_aggravator_other: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
    pub address: Option<AttendanceOffenderAddressResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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

impl AttendanceOffenderAddress {
    pub fn to_response(self) -> AttendanceOffenderAddressResponse {
        AttendanceOffenderAddressResponse {
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

impl AttendanceOffender {
    pub fn with_address(self, address: Option<AttendanceOffenderAddress>) -> AttendanceOffenderWithAddress {
        AttendanceOffenderWithAddress {
            id: self.id,
            offender_id: self.offender_id,
            victim_id: self.victim_id,
            protective_measure_id: self.protective_measure_id,
            was_offender_present: self.was_offender_present,
            attendance_date: self.attendance_date,
            attendance_time: self.attendance_time,
            is_remote: self.is_remote,
            assaults_children: self.assaults_children,
            violence_aggravator: self.violence_aggravator,
            violence_aggravator_other: self.violence_aggravator_other,
            description: self.description,
            created_at: self.created_at,
            updated_at: self.updated_at,
            is_deleted: self.is_deleted,
            address: address.map(|a| a.to_response()),
        }
    }
}
