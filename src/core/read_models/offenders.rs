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
    #[serde(flatten)]
    pub summary: OffenderSummary,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
        let created_at = offender.created_at;
        let updated_at = offender.updated_at;
        OffenderWithDetails {
            summary: OffenderSummary {
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
                is_deleted: offender.is_deleted,
                phones: phones.into_iter().map(Into::into).collect(),
                addresses: addresses.into_iter().map(Into::into).collect(),
            },
            created_at,
            updated_at,
        }
    }

    pub fn from_write_result(result: OffenderWriteResult) -> Self {
        Self::from_entity(result.offender, result.phones, result.addresses)
    }
}

impl From<OffenderWithDetails> for OffenderSummary {
    fn from(o: OffenderWithDetails) -> Self {
        o.summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_summary() -> OffenderSummary {
        OffenderSummary {
            id: Uuid::nil(),
            full_name: "X".to_string(),
            cpf: None,
            birth_date: None,
            city_id: Uuid::nil(),
            imprisoned: false,
            occupation: None,
            is_public_security_agent: false,
            security_force: None,
            uses_alcohol: false,
            uses_drugs: false,
            has_psychiatric_issues: false,
            psychiatric_issues_type: None,
            education_level: EducationLevel::HighSchool,
            observation: None,
            is_deleted: false,
            phones: vec![],
            addresses: vec![],
        }
    }

    // Garante que `#[serde(flatten)]` mantém o JSON plano (sem chave "summary")
    // e idêntico ao comportamento anterior: campos de negócio + created_at/updated_at.
    #[test]
    fn with_details_serializes_flat_with_timestamps() {
        let now = Utc::now();
        let wd = OffenderWithDetails {
            summary: sample_summary(),
            created_at: now,
            updated_at: now,
        };
        let value = serde_json::to_value(&wd).unwrap();
        let obj = value.as_object().unwrap();
        assert!(!obj.contains_key("summary"), "flatten não deve aninhar");
        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("full_name"));
        assert!(obj.contains_key("phones"));
        assert!(obj.contains_key("addresses"));
        assert!(obj.contains_key("created_at"));
        assert!(obj.contains_key("updated_at"));
    }

    #[test]
    fn summary_serializes_without_timestamps() {
        let value = serde_json::to_value(sample_summary()).unwrap();
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("id"));
        assert!(!obj.contains_key("created_at"));
        assert!(!obj.contains_key("updated_at"));
    }
}
