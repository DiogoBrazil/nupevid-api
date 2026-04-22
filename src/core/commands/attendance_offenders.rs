use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::commands::attendance_victims::AttendanceAddressData;
use crate::core::entities::attendance_offenders::ViolenceAggravator;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CreateAttendanceOffender {
    pub protective_measure_id: Uuid,
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
#[serde(deny_unknown_fields)]
pub struct UpdateAttendanceOffender {
    pub protective_measure_id: Uuid,
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
