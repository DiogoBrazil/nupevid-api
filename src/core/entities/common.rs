use serde::{Deserialize, Serialize};
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
