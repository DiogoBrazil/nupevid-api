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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PhoneData {
    pub phone: String,
    pub phone_type: Option<PhoneType>,
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
    pub fn to_response(&self) -> OffenderPhoneResponse {
        OffenderPhoneResponse {
            id: self.id,
            phone: self.phone.clone(),
            phone_type: self.phone_type.clone(),
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl OffenderAddress {
    pub fn to_response(&self) -> OffenderAddressResponse {
        OffenderAddressResponse {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkAddressData {
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_id: Uuid,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffenderWorkAddressResponse {
    pub id: Uuid,
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OffenderWorkAddress {
    pub id: Uuid,
    pub offender_id: Uuid,
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

impl OffenderWorkAddress {
    pub fn to_response(&self) -> OffenderWorkAddressResponse {
        OffenderWorkAddressResponse {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateOffender {
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub city_id: Uuid,
    pub victim_id: Uuid,
    pub imprisoned: bool,
    pub occupation: Option<String>,
    pub workplace: Option<String>,
    pub is_public_security_agent: bool,
    pub security_force: Option<SecurityForce>,
    pub relationship_to_victim: RelationshipToVictim,
    pub uses_alcohol: bool,
    pub uses_drugs: bool,
    pub has_psychiatric_issues: bool,
    pub psychiatric_issues_type: Option<String>,
    pub was_drunk_during_assault: bool,
    pub observation: Option<String>,
    pub phones: Option<Vec<PhoneData>>,
    pub addresses: Option<Vec<AddressData>>,
    pub work_addresses: Option<Vec<WorkAddressData>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateOffender {
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub city_id: Uuid,
    pub victim_id: Uuid,
    pub imprisoned: bool,
    pub occupation: Option<String>,
    pub workplace: Option<String>,
    pub is_public_security_agent: bool,
    pub security_force: Option<SecurityForce>,
    pub relationship_to_victim: RelationshipToVictim,
    pub uses_alcohol: bool,
    pub uses_drugs: bool,
    pub has_psychiatric_issues: bool,
    pub psychiatric_issues_type: Option<String>,
    pub was_drunk_during_assault: bool,
    pub observation: Option<String>,
    pub phones: Option<Vec<PhoneData>>,
    pub addresses: Option<Vec<AddressData>>,
    pub work_addresses: Option<Vec<WorkAddressData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Offender {
    pub id: Uuid,
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub city_id: Uuid,
    pub victim_id: Uuid,
    pub imprisoned: bool,
    pub occupation: Option<String>,
    pub workplace: Option<String>,
    pub is_public_security_agent: bool,
    pub security_force: Option<SecurityForce>,
    pub relationship_to_victim: RelationshipToVictim,
    pub uses_alcohol: bool,
    pub uses_drugs: bool,
    pub has_psychiatric_issues: bool,
    pub psychiatric_issues_type: Option<String>,
    pub was_drunk_during_assault: bool,
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
    pub victim_id: Uuid,
    pub imprisoned: bool,
    pub occupation: Option<String>,
    pub workplace: Option<String>,
    pub is_public_security_agent: bool,
    pub security_force: Option<SecurityForce>,
    pub relationship_to_victim: RelationshipToVictim,
    pub uses_alcohol: bool,
    pub uses_drugs: bool,
    pub has_psychiatric_issues: bool,
    pub psychiatric_issues_type: Option<String>,
    pub was_drunk_during_assault: bool,
    pub observation: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub phones: Vec<OffenderPhoneResponse>,
    pub addresses: Vec<OffenderAddressResponse>,
    pub work_addresses: Vec<OffenderWorkAddressResponse>,
}

impl Offender {
    pub fn with_details(
        self,
        phones: Vec<OffenderPhone>,
        addresses: Vec<OffenderAddress>,
        work_addresses: Vec<OffenderWorkAddress>,
    ) -> OffenderWithDetails {
        OffenderWithDetails {
            id: self.id,
            full_name: self.full_name,
            cpf: self.cpf,
            birth_date: self.birth_date,
            city_id: self.city_id,
            victim_id: self.victim_id,
            imprisoned: self.imprisoned,
            occupation: self.occupation,
            workplace: self.workplace,
            is_public_security_agent: self.is_public_security_agent,
            security_force: self.security_force,
            relationship_to_victim: self.relationship_to_victim,
            uses_alcohol: self.uses_alcohol,
            uses_drugs: self.uses_drugs,
            has_psychiatric_issues: self.has_psychiatric_issues,
            psychiatric_issues_type: self.psychiatric_issues_type,
            was_drunk_during_assault: self.was_drunk_during_assault,
            observation: self.observation,
            created_at: self.created_at,
            updated_at: self.updated_at,
            is_deleted: self.is_deleted,
            phones: phones.into_iter().map(|p| p.to_response()).collect(),
            addresses: addresses.into_iter().map(|a| a.to_response()).collect(),
            work_addresses: work_addresses.into_iter().map(|wa| wa.to_response()).collect(),
        }
    }
}
