use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddressData {
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateVictim {
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub phone: Option<String>,
    pub city_id: Uuid,
    pub address: Option<AddressData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateVictim {
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub phone: Option<String>,
    pub city_id: Uuid,
    pub address: Option<AddressData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Victim {
    pub id: Uuid,
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub phone: Option<String>,
    pub city_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictimAddressResponse {
    pub id: Uuid,
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictimWithAddress {
    pub id: Uuid,
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub phone: Option<String>,
    pub city_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub address: Option<VictimAddressResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VictimAddress {
    pub id: Uuid,
    pub victim_id: Uuid,
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

impl VictimAddress {
    pub fn to_response(&self) -> VictimAddressResponse {
        VictimAddressResponse {
            id: self.id,
            street: self.street.clone(),
            number: self.number.clone(),
            district: self.district.clone(),
            city_id: self.city_id,
            zip_code: self.zip_code.clone(),
            complement: self.complement.clone(),
        }
    }
}

impl Victim {
    pub fn with_address(self, address: Option<VictimAddress>) -> VictimWithAddress {
        VictimWithAddress {
            id: self.id,
            full_name: self.full_name,
            cpf: self.cpf,
            birth_date: self.birth_date,
            phone: self.phone,
            city_id: self.city_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
            is_deleted: self.is_deleted,
            address: address.map(|a| a.to_response()),
        }
    }
}
