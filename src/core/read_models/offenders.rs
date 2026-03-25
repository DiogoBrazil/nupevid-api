use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::entities::common::{AddressType, EducationLevel, PhoneType};
use crate::core::entities::offenders::SecurityForce;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffenderPhoneResponse {
    pub id: Uuid,
    pub phone: String,
    pub phone_type: Option<PhoneType>,
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
