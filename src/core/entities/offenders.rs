pub use crate::core::entities::common::{
    AddressData, AddressType, EducationLevel, PhoneData, PhoneType,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::Type, PartialEq)]
#[sqlx(type_name = "security_force_enum")]
pub enum SecurityForce {
    #[serde(rename = "Military Police")]
    #[sqlx(rename = "Military Police")]
    MilitaryPolice,
    #[serde(rename = "Civil Police")]
    #[sqlx(rename = "Civil Police")]
    CivilPolice,
    #[serde(rename = "Penal Police")]
    #[sqlx(rename = "Penal Police")]
    PenalPolice,
    #[serde(rename = "Fire Department")]
    #[sqlx(rename = "Fire Department")]
    FireDepartment,
    #[serde(rename = "Federal Highway Police")]
    #[sqlx(rename = "Federal Highway Police")]
    FederalHighwayPolice,
    #[serde(rename = "Federal Police")]
    #[sqlx(rename = "Federal Police")]
    FederalPolice,
    #[serde(rename = "Municipal Guard")]
    #[sqlx(rename = "Municipal Guard")]
    MunicipalGuard,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
pub struct OffenderPhoneResponse {
    pub id: Uuid,
    pub phone: String,
    pub phone_type: Option<PhoneType>,
}

impl OffenderPhone {
    pub fn to_response(self) -> OffenderPhoneResponse {
        OffenderPhoneResponse {
            id: self.id,
            phone: self.phone,
            phone_type: self.phone_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffenderAddressResponse {
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

impl OffenderAddress {
    pub fn to_response(self) -> OffenderAddressResponse {
        OffenderAddressResponse {
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
pub struct CreateOffender {
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub city_id: Option<Uuid>,
    pub imprisoned: bool,
    pub occupation: Option<String>,
    #[serde(default, skip_deserializing)]
    pub is_public_security_agent: bool,
    pub security_force: Option<SecurityForce>,
    pub uses_alcohol: bool,
    pub uses_drugs: bool,
    #[serde(default)]
    pub has_psychiatric_issues: bool,
    pub psychiatric_issues_type: Option<Vec<String>>,
    pub education_level: EducationLevel,
    pub observation: Option<String>,
    pub phones: Option<Vec<PhoneData>>,
    pub addresses: Option<Vec<AddressData>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateOffender {
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub city_id: Option<Uuid>,
    pub imprisoned: bool,
    pub occupation: Option<String>,
    #[serde(default, skip_deserializing)]
    pub is_public_security_agent: bool,
    pub security_force: Option<SecurityForce>,
    pub uses_alcohol: bool,
    pub uses_drugs: bool,
    #[serde(default)]
    pub has_psychiatric_issues: bool,
    pub psychiatric_issues_type: Option<Vec<String>>,
    pub education_level: EducationLevel,
    pub observation: Option<String>,
    pub phones: Option<Vec<PhoneData>>,
    pub addresses: Option<Vec<AddressData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
pub struct OffenderWithDetails {
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
    pub phones: Vec<OffenderPhoneResponse>,
    pub addresses: Vec<OffenderAddressResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffenderComplement {
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
    pub is_deleted: bool,
    pub phones: Vec<OffenderPhoneResponse>,
    pub addresses: Vec<OffenderAddressResponse>,
}

impl Offender {
    pub fn with_details(
        self,
        phones: Vec<OffenderPhone>,
        addresses: Vec<OffenderAddress>,
    ) -> OffenderWithDetails {
        OffenderWithDetails {
            id: self.id,
            full_name: self.full_name,
            cpf: self.cpf,
            birth_date: self.birth_date,
            city_id: self.city_id,
            imprisoned: self.imprisoned,
            occupation: self.occupation,
            is_public_security_agent: self.is_public_security_agent,
            security_force: self.security_force,
            uses_alcohol: self.uses_alcohol,
            uses_drugs: self.uses_drugs,
            has_psychiatric_issues: self.has_psychiatric_issues,
            psychiatric_issues_type: self.psychiatric_issues_type,
            education_level: self.education_level,
            observation: self.observation,
            created_at: self.created_at,
            updated_at: self.updated_at,
            is_deleted: self.is_deleted,
            phones: phones.into_iter().map(|p| p.to_response()).collect(),
            addresses: addresses.into_iter().map(|a| a.to_response()).collect(),
        }
    }
}

impl OffenderWithDetails {
    pub fn to_complement(self) -> OffenderComplement {
        OffenderComplement {
            id: self.id,
            full_name: self.full_name,
            cpf: self.cpf,
            birth_date: self.birth_date,
            city_id: self.city_id,
            imprisoned: self.imprisoned,
            occupation: self.occupation,
            is_public_security_agent: self.is_public_security_agent,
            security_force: self.security_force,
            uses_alcohol: self.uses_alcohol,
            uses_drugs: self.uses_drugs,
            has_psychiatric_issues: self.has_psychiatric_issues,
            psychiatric_issues_type: self.psychiatric_issues_type,
            education_level: self.education_level,
            observation: self.observation,
            is_deleted: self.is_deleted,
            phones: self.phones,
            addresses: self.addresses,
        }
    }
}
