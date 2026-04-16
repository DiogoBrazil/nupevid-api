pub use crate::core::entities::common::{
    AddressData, AddressType, EducationLevel, PhoneData, PhoneType,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SecurityForce {
    #[serde(rename = "Military Police")]
    MilitaryPolice,
    #[serde(rename = "Civil Police")]
    CivilPolice,
    #[serde(rename = "Penal Police")]
    PenalPolice,
    #[serde(rename = "Fire Department")]
    FireDepartment,
    #[serde(rename = "Federal Highway Police")]
    FederalHighwayPolice,
    #[serde(rename = "Federal Police")]
    FederalPolice,
    #[serde(rename = "Municipal Guard")]
    MunicipalGuard,
}

impl SecurityForce {
    pub fn as_str(&self) -> &str {
        match self {
            Self::MilitaryPolice => "Military Police",
            Self::CivilPolice => "Civil Police",
            Self::PenalPolice => "Penal Police",
            Self::FireDepartment => "Fire Department",
            Self::FederalHighwayPolice => "Federal Highway Police",
            Self::FederalPolice => "Federal Police",
            Self::MunicipalGuard => "Municipal Guard",
        }
    }
}

impl TryFrom<&str> for SecurityForce {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Military Police" => Ok(Self::MilitaryPolice),
            "Civil Police" => Ok(Self::CivilPolice),
            "Penal Police" => Ok(Self::PenalPolice),
            "Fire Department" => Ok(Self::FireDepartment),
            "Federal Highway Police" => Ok(Self::FederalHighwayPolice),
            "Federal Police" => Ok(Self::FederalPolice),
            "Municipal Guard" => Ok(Self::MunicipalGuard),
            other => Err(format!("Invalid security force: '{}'", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffenderPhone {
    pub id: Uuid,
    pub offender_id: Uuid,
    pub phone: String,
    pub phone_type: Option<PhoneType>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffenderAddress {
    pub id: Uuid,
    pub offender_id: Uuid,
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
pub struct Offender {
    pub id: Uuid,
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub city_id: Uuid,
    pub imprisoned: bool,
    pub occupation: Option<String>,
    pub is_public_security_agent: bool,
    pub security_force: Option<SecurityForce>,
    pub uses_alcohol: bool,
    pub uses_drugs: bool,
    pub has_psychiatric_issues: bool,
    pub psychiatric_issues_type: Option<Vec<String>>,
    pub education_level: EducationLevel,
    pub observation: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffenderWriteResult {
    pub offender: Offender,
    pub phones: Vec<OffenderPhone>,
    pub addresses: Vec<OffenderAddress>,
}

