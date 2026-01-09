use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::Type, PartialEq)]
#[sqlx(type_name = "violence_type_enum", rename_all = "PascalCase")]
pub enum ViolenceType {
    Physical,
    Sexual,
    Psychological,
    Moral,
}

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::Type, PartialEq)]
#[sqlx(type_name = "relationship_to_victim_enum")]
pub enum RelationshipToVictim {
    #[serde(rename = "Spouse")]
    #[sqlx(rename = "Spouse")]
    Spouse,
    #[serde(rename = "Ex-Spouse")]
    #[sqlx(rename = "Ex-Spouse")]
    ExSpouse,
    #[serde(rename = "Mother")]
    #[sqlx(rename = "Mother")]
    Mother,
    #[serde(rename = "Father")]
    #[sqlx(rename = "Father")]
    Father,
    #[serde(rename = "Stepfather")]
    #[sqlx(rename = "Stepfather")]
    Stepfather,
    #[serde(rename = "Stepmother")]
    #[sqlx(rename = "Stepmother")]
    Stepmother,
    #[serde(rename = "Son/Daughter")]
    #[sqlx(rename = "Son/Daughter")]
    SonDaughter,
    #[serde(rename = "Sibling")]
    #[sqlx(rename = "Sibling")]
    Sibling,
    #[serde(rename = "Cousin")]
    #[sqlx(rename = "Cousin")]
    Cousin,
    #[serde(rename = "Grandfather")]
    #[sqlx(rename = "Grandfather")]
    Grandfather,
    #[serde(rename = "Grandmother")]
    #[sqlx(rename = "Grandmother")]
    Grandmother,
    #[serde(rename = "Brother/Sister-in-law")]
    #[sqlx(rename = "Brother/Sister-in-law")]
    BrotherSisterInLaw,
    #[serde(rename = "Father/Mother-in-law")]
    #[sqlx(rename = "Father/Mother-in-law")]
    FatherMotherInLaw,
    #[serde(rename = "Son-in-law")]
    #[sqlx(rename = "Son-in-law")]
    SonInLaw,
    #[serde(rename = "Daughter-in-law")]
    #[sqlx(rename = "Daughter-in-law")]
    DaughterInLaw,
    #[serde(rename = "Uncle/Aunt")]
    #[sqlx(rename = "Uncle/Aunt")]
    UncleAunt,
}

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::Type, PartialEq)]
#[sqlx(
    type_name = "protective_measure_status_enum",
    rename_all = "PascalCase"
)]
pub enum ProtectiveMeasureStatus {
    Valid,
    Revoked,
    Expired,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateProtectiveMeasure {
    pub process_number: String,
    pub sei_process_number: Option<String>,
    pub occurrence_report_number: Option<String>,
    pub issued_at: NaiveDate,
    pub valid_until: Option<NaiveDate>,
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
    pub extensions: Option<Vec<ExtensionUpsert>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateProtectiveMeasure {
    pub process_number: String,
    pub sei_process_number: Option<String>,
    pub occurrence_report_number: Option<String>,
    pub issued_at: NaiveDate,
    pub valid_until: Option<NaiveDate>,
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
    pub extensions: Option<Vec<ExtensionUpsert>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProtectiveMeasure {
    pub id: Uuid,
    pub process_number: String,
    pub sei_process_number: Option<String>,
    pub occurrence_report_number: Option<String>,
    pub issued_at: NaiveDate,
    pub valid_until: Option<NaiveDate>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateExtension {
    pub extension_number: i32,
    pub extension_date: NaiveDate,
    pub new_valid_until: Option<NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateExtension {
    pub extension_number: i32,
    pub extension_date: NaiveDate,
    pub new_valid_until: Option<NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtensionUpsert {
    pub id: Option<Uuid>,
    pub extension_number: i32,
    pub extension_date: NaiveDate,
    pub new_valid_until: Option<NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProtectiveMeasureExtension {
    pub id: Uuid,
    pub protective_measure_id: Uuid,
    pub extension_number: i32,
    pub extension_date: NaiveDate,
    pub new_valid_until: Option<NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProtectiveMeasureWithExtensions {
    #[serde(flatten)]
    pub measure: ProtectiveMeasure,
    pub extensions: Vec<ProtectiveMeasureExtension>,
}
