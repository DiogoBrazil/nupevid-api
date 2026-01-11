use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::Type, PartialEq)]
#[sqlx(type_name = "phone_type_enum")]
pub enum PhoneType {
    #[serde(rename = "Mobile")]
    #[sqlx(rename = "Mobile")]
    Mobile,
    #[serde(rename = "Residential")]
    #[sqlx(rename = "Residential")]
    Residential,
    #[serde(rename = "Work")]
    #[sqlx(rename = "Work")]
    Work,
}

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::Type, PartialEq)]
#[sqlx(type_name = "address_type_enum", rename_all = "PascalCase")]
pub enum AddressType {
    Residential,
    Work,
    Correspondence,
    Commercial,
    Institutional,
    Temporary,
    Other,
}

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::Type, PartialEq)]
#[sqlx(type_name = "education_level_enum")]
pub enum EducationLevel {
    #[serde(rename = "Elementary")]
    #[sqlx(rename = "Elementary")]
    Elementary,
    #[serde(rename = "High School")]
    #[sqlx(rename = "High School")]
    HighSchool,
    #[serde(rename = "College")]
    #[sqlx(rename = "College")]
    College,
    #[serde(rename = "Postgraduate")]
    #[sqlx(rename = "Postgraduate")]
    Postgraduate,
    #[serde(rename = "Illiterate")]
    #[sqlx(rename = "Illiterate")]
    Illiterate,
    #[serde(rename = "Semi-illiterate")]
    #[sqlx(rename = "Semi-illiterate")]
    SemiIlliterate,
    #[serde(rename = "Master")]
    #[sqlx(rename = "Master")]
    Master,
    #[serde(rename = "Doctorate")]
    #[sqlx(rename = "Doctorate")]
    Doctorate,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PhoneData {
    pub phone: String,
    pub phone_type: Option<PhoneType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
pub struct VictimPhoneResponse {
    pub id: Uuid,
    pub phone: String,
    pub phone_type: Option<PhoneType>,
}

impl VictimPhone {
    pub fn to_response(self) -> VictimPhoneResponse {
        VictimPhoneResponse {
            id: self.id,
            phone: self.phone,
            phone_type: self.phone_type,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddressData {
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_id: Uuid,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
    pub address_type: AddressType,
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
    pub address_type: AddressType,
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
    pub address_type: AddressType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl VictimAddress {
    pub fn to_response(self) -> VictimAddressResponse {
        VictimAddressResponse {
            id: self.id,
            street: self.street,
            number: self.number,
            district: self.district,
            city_id: self.city_id,
            zip_code: self.zip_code,
            complement: self.complement,
            address_type: self.address_type,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateVictim {
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub city_id: Option<Uuid>,
    pub phones: Option<Vec<PhoneData>>,
    pub addresses: Option<Vec<AddressData>>,
    pub education_level: Option<EducationLevel>,
    pub occupation: Option<String>,
    #[serde(default)]
    pub has_children: bool,
    pub children_count: Option<i32>,
    pub is_pregnant: Option<bool>,
    #[serde(default)]
    pub has_special_needs: bool,
    pub special_needs_type: Option<Vec<String>>,
    pub uses_alcohol: bool,
    pub uses_drugs: bool,
    #[serde(default)]
    pub has_psychiatric_issues: bool,
    pub psychiatric_issues_type: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateVictim {
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub city_id: Option<Uuid>,
    pub phones: Option<Vec<PhoneData>>,
    pub addresses: Option<Vec<AddressData>>,
    pub education_level: Option<EducationLevel>,
    pub occupation: Option<String>,
    #[serde(default)]
    pub has_children: bool,
    pub children_count: Option<i32>,
    pub is_pregnant: Option<bool>,
    #[serde(default)]
    pub has_special_needs: bool,
    pub special_needs_type: Option<Vec<String>>,
    pub uses_alcohol: bool,
    pub uses_drugs: bool,
    #[serde(default)]
    pub has_psychiatric_issues: bool,
    pub psychiatric_issues_type: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
pub struct VictimWithDetails {
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
    pub phones: Vec<VictimPhoneResponse>,
    pub addresses: Vec<VictimAddressResponse>,
}

impl Victim {
    pub fn with_details(
        self,
        phones: Vec<VictimPhone>,
        addresses: Vec<VictimAddress>,
    ) -> VictimWithDetails {
        VictimWithDetails {
            id: self.id,
            full_name: self.full_name,
            cpf: self.cpf,
            birth_date: self.birth_date,
            city_id: self.city_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
            is_deleted: self.is_deleted,
            education_level: self.education_level,
            occupation: self.occupation,
            has_children: self.has_children,
            children_count: self.children_count,
            is_pregnant: self.is_pregnant,
            has_special_needs: self.has_special_needs,
            special_needs_type: self.special_needs_type,
            uses_alcohol: self.uses_alcohol,
            uses_drugs: self.uses_drugs,
            has_psychiatric_issues: self.has_psychiatric_issues,
            psychiatric_issues_type: self.psychiatric_issues_type,
            phones: phones.into_iter().map(|p| p.to_response()).collect(),
            addresses: addresses.into_iter().map(|a| a.to_response()).collect(),
        }
    }
}
