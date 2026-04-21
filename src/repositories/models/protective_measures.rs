use chrono::{DateTime, NaiveDate, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::core::entities::protective_measures::{
    ProtectiveMeasure, ProtectiveMeasureStatus, RelationshipToVictim, ViolenceType,
};

#[derive(Debug, Clone, FromRow)]
pub struct ProtectiveMeasureRow {
    pub id: Uuid,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl From<ProtectiveMeasureRow> for ProtectiveMeasure {
    fn from(row: ProtectiveMeasureRow) -> Self {
        ProtectiveMeasure {
            id: row.id,
            process_number: row.process_number,
            sei_process_number: row.sei_process_number,
            occurrence_report_number: row.occurrence_report_number,
            issued_at: row.issued_at,
            judicial_authority: row.judicial_authority,
            court_district_id: row.court_district_id,
            distance_meters: row.distance_meters,
            status: row.status,
            violence_types: row.violence_types,
            relationship_to_victim: row.relationship_to_victim,
            assaults_children: row.assaults_children,
            was_drunk_during_assault: row.was_drunk_during_assault,
            victim_id: row.victim_id,
            offender_id: row.offender_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
        }
    }
}
