use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::entities::common::{AddressType, EducationLevel, PhoneType};
use crate::core::entities::victims::{Victim, VictimAddress, VictimPhone, VictimWriteResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictimPhoneResponse {
    pub id: Uuid,
    pub phone: String,
    pub phone_type: Option<PhoneType>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictimSummary {
    pub id: Uuid,
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub city_id: Uuid,
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

impl From<VictimPhone> for VictimPhoneResponse {
    fn from(phone: VictimPhone) -> Self {
        VictimPhoneResponse {
            id: phone.id,
            phone: phone.phone,
            phone_type: phone.phone_type,
        }
    }
}

impl From<VictimAddress> for VictimAddressResponse {
    fn from(address: VictimAddress) -> Self {
        VictimAddressResponse {
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

impl VictimWithDetails {
    pub fn from_entity(
        victim: Victim,
        phones: Vec<VictimPhone>,
        addresses: Vec<VictimAddress>,
    ) -> Self {
        VictimWithDetails {
            id: victim.id,
            full_name: victim.full_name,
            cpf: victim.cpf,
            birth_date: victim.birth_date,
            city_id: victim.city_id,
            created_at: victim.created_at,
            updated_at: victim.updated_at,
            is_deleted: victim.is_deleted,
            education_level: victim.education_level,
            occupation: victim.occupation,
            has_children: victim.has_children,
            children_count: victim.children_count,
            is_pregnant: victim.is_pregnant,
            has_special_needs: victim.has_special_needs,
            special_needs_type: victim.special_needs_type,
            uses_alcohol: victim.uses_alcohol,
            uses_drugs: victim.uses_drugs,
            has_psychiatric_issues: victim.has_psychiatric_issues,
            psychiatric_issues_type: victim.psychiatric_issues_type,
            phones: phones.into_iter().map(Into::into).collect(),
            addresses: addresses.into_iter().map(Into::into).collect(),
        }
    }

    pub fn from_write_result(result: VictimWriteResult) -> Self {
        Self::from_entity(result.victim, result.phones, result.addresses)
    }
}

impl From<VictimWithDetails> for VictimSummary {
    fn from(v: VictimWithDetails) -> Self {
        VictimSummary {
            id: v.id,
            full_name: v.full_name,
            cpf: v.cpf,
            birth_date: v.birth_date,
            city_id: v.city_id,
            is_deleted: v.is_deleted,
            education_level: v.education_level,
            occupation: v.occupation,
            has_children: v.has_children,
            children_count: v.children_count,
            is_pregnant: v.is_pregnant,
            has_special_needs: v.has_special_needs,
            special_needs_type: v.special_needs_type,
            uses_alcohol: v.uses_alcohol,
            uses_drugs: v.uses_drugs,
            has_psychiatric_issues: v.has_psychiatric_issues,
            psychiatric_issues_type: v.psychiatric_issues_type,
            phones: v.phones,
            addresses: v.addresses,
        }
    }
}
