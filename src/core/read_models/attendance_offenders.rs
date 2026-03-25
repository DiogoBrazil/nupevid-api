use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::entities::attendance_offenders::ViolenceAggravator;

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
