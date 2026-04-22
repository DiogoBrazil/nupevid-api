use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::entities::common::{AddressType, EducationLevel, PhoneType};
use crate::core::entities::offenders::{
    Offender, OffenderAddress, OffenderPhone, OffenderWriteResult, SecurityForce,
};

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
pub struct OffenderSummary {
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

impl From<OffenderPhone> for OffenderPhoneResponse {
    fn from(phone: OffenderPhone) -> Self {
        OffenderPhoneResponse {
            id: phone.id,
            phone: phone.phone,
            phone_type: phone.phone_type,
        }
    }
}

impl From<OffenderAddress> for OffenderAddressResponse {
    fn from(address: OffenderAddress) -> Self {
        OffenderAddressResponse {
            id: address.id,
            street: address.street,
            number: address.number,
            district: address.district,
            city_id: address.city_id,
            zip_code: address.zip_code,
            complement: address.complement,
            address_type: address.address_type,
        }
    }
}

impl OffenderWithDetails {
    pub fn from_entity(
        offender: Offender,
        phones: Vec<OffenderPhone>,
        addresses: Vec<OffenderAddress>,
    ) -> Self {
        OffenderWithDetails {
            id: offender.id,
            full_name: offender.full_name,
            cpf: offender.cpf,
            birth_date: offender.birth_date,
            city_id: offender.city_id,
            imprisoned: offender.imprisoned,
            occupation: offender.occupation,
            is_public_security_agent: offender.is_public_security_agent,
            security_force: offender.security_force,
            uses_alcohol: offender.uses_alcohol,
            uses_drugs: offender.uses_drugs,
            has_psychiatric_issues: offender.has_psychiatric_issues,
            psychiatric_issues_type: offender.psychiatric_issues_type,
            education_level: offender.education_level,
            observation: offender.observation,
            created_at: offender.created_at,
            updated_at: offender.updated_at,
            is_deleted: offender.is_deleted,
            phones: phones.into_iter().map(Into::into).collect(),
            addresses: addresses.into_iter().map(Into::into).collect(),
        }
    }

    pub fn from_write_result(result: OffenderWriteResult) -> Self {
        Self::from_entity(result.offender, result.phones, result.addresses)
    }
}

impl From<OffenderWithDetails> for OffenderSummary {
    fn from(o: OffenderWithDetails) -> Self {
        OffenderSummary {
            id: o.id,
            full_name: o.full_name,
            cpf: o.cpf,
            birth_date: o.birth_date,
            city_id: o.city_id,
            imprisoned: o.imprisoned,
            occupation: o.occupation,
            is_public_security_agent: o.is_public_security_agent,
            security_force: o.security_force,
            uses_alcohol: o.uses_alcohol,
            uses_drugs: o.uses_drugs,
            has_psychiatric_issues: o.has_psychiatric_issues,
            psychiatric_issues_type: o.psychiatric_issues_type,
            education_level: o.education_level,
            observation: o.observation,
            is_deleted: o.is_deleted,
            phones: o.phones,
            addresses: o.addresses,
        }
    }
}
