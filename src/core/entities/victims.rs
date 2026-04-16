use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::core::entities::common::{
    AddressData, AddressType, EducationLevel, PhoneData, PhoneType,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictimPhone {
    pub id: Uuid,
    pub victim_id: Uuid,
    pub phone: String,
    pub phone_type: Option<PhoneType>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictimAddress {
    pub id: Uuid,
    pub victim_id: Uuid,
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
    pub address_type: AddressType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Victim {
    pub id: Uuid,
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub city_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub education_level: Option<EducationLevel>,
    pub occupation: Option<String>,
    pub has_children: bool,
    pub children_count: Option<i32>,
    pub is_pregnant: Option<bool>,
    pub has_special_needs: bool,
    pub special_needs_type: Option<Vec<String>>,
    pub uses_alcohol: bool,
    pub uses_drugs: bool,
    pub has_psychiatric_issues: bool,
    pub psychiatric_issues_type: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictimWriteResult {
    pub victim: Victim,
    pub phones: Vec<VictimPhone>,
    pub addresses: Vec<VictimAddress>,
}
