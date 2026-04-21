use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::entities::protective_measures::{
    ProtectiveMeasureStatus, RelationshipToVictim, ViolenceType,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CreateProtectiveMeasure {
    pub process_number: String,
    pub sei_process_number: Option<String>,
    pub occurrence_report_number: Option<String>,
    pub issued_at: NaiveDate,
    pub judicial_authority: String,
    pub court_district_id: Uuid,
    pub distance_meters: Option<i32>,
    pub status: ProtectiveMeasureStatus,
    pub violence_types: Vec<ViolenceType>,
    pub relationship_to_victim: RelationshipToVictim,
    pub assaults_children: bool,
    pub was_drunk_during_assault: bool,
    pub victim_id: Uuid,
    pub offender_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct UpdateProtectiveMeasure {
    pub process_number: String,
    pub sei_process_number: Option<String>,
    pub occurrence_report_number: Option<String>,
    pub issued_at: NaiveDate,
    pub judicial_authority: String,
    pub court_district_id: Uuid,
    pub distance_meters: Option<i32>,
    pub status: ProtectiveMeasureStatus,
    pub violence_types: Vec<ViolenceType>,
    pub relationship_to_victim: RelationshipToVictim,
    pub assaults_children: bool,
    pub was_drunk_during_assault: bool,
    pub victim_id: Uuid,
    pub offender_id: Uuid,
}
