use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateProtectiveMeasure {
    pub process_number: String,
    pub sei_process_number: Option<String>,
    pub issued_at: NaiveDate,
    pub valid_until: Option<NaiveDate>,
    pub judicial_authority: String,
    pub court_district_id: Uuid,
    pub distance_meters: Option<i32>,
    pub is_active: bool,
    pub victim_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateProtectiveMeasure {
    pub process_number: String,
    pub sei_process_number: Option<String>,
    pub issued_at: NaiveDate,
    pub valid_until: Option<NaiveDate>,
    pub judicial_authority: String,
    pub court_district_id: Uuid,
    pub distance_meters: Option<i32>,
    pub is_active: bool,
    pub victim_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProtectiveMeasure {
    pub id: Uuid,
    pub process_number: String,
    pub sei_process_number: Option<String>,
    pub issued_at: NaiveDate,
    pub valid_until: Option<NaiveDate>,
    pub judicial_authority: String,
    pub court_district_id: Uuid,
    pub distance_meters: Option<i32>,
    pub is_active: bool,
    pub victim_id: Uuid,
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
