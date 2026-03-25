use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::core::entities::common::{
    AddressData, AddressType, EducationLevel, PhoneData, PhoneType,
};
use crate::core::read_models::victims::{
    VictimAddressResponse, VictimComplement, VictimPhoneResponse, VictimWithDetails,
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

impl VictimPhone {
    pub fn to_response(self) -> VictimPhoneResponse {
        VictimPhoneResponse {
            id: self.id,
            phone: self.phone,
            phone_type: self.phone_type,
        }
    }
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

impl VictimWriteResult {
    pub fn into_details(self) -> VictimWithDetails {
        self.victim.with_details(self.phones, self.addresses)
    }
}

impl VictimWithDetails {
    pub fn to_complement(self) -> VictimComplement {
        VictimComplement {
            id: self.id,
            full_name: self.full_name,
            cpf: self.cpf,
            birth_date: self.birth_date,
            city_id: self.city_id,
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
            phones: self.phones,
            addresses: self.addresses,
        }
    }
}
