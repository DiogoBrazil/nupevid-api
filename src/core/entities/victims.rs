use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateVictim {
    pub full_name: String,
    pub document_id: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub phone: Option<String>,
    pub city_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateVictim {
    pub full_name: String,
    pub document_id: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub phone: Option<String>,
    pub city_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Victim {
    pub id: Uuid,
    pub full_name: String,
    pub document_id: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub phone: Option<String>,
    pub city_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateVictimAddress {
    pub victim_id: Uuid,
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_name: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateVictimAddress {
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_name: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VictimAddress {
    pub id: Uuid,
    pub victim_id: Uuid,
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_name: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}
