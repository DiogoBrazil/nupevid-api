use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateProtectiveMeasure {
    pub process_number: String,
    pub issued_at: NaiveDate,
    pub judicial_authority: String,
    pub court_district: String,
    pub is_active: bool,
    pub victim_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateProtectiveMeasure {
    pub process_number: String,
    pub issued_at: NaiveDate,
    pub judicial_authority: String,
    pub court_district: String,
    pub is_active: bool,
    pub victim_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProtectiveMeasure {
    pub id: Uuid,
    pub process_number: String,
    pub issued_at: NaiveDate,
    pub judicial_authority: String,
    pub court_district: String,
    pub is_active: bool,
    pub victim_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}
