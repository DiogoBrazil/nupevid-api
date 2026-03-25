use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::errors::DomainError;

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

#[derive(Debug)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub page: i64,
    pub page_size: i64,
    pub total_items: i64,
}

/// Derive city_id from addresses with priority: Residential > Work
fn derive_city_id_from_addresses(addresses: &Option<Vec<AddressData>>) -> Option<Uuid> {
    let addresses = addresses.as_ref()?;

    for address in addresses {
        if address.address_type == AddressType::Residential {
            return Some(address.city_id);
        }
    }

    for address in addresses {
        if address.address_type == AddressType::Work {
            return Some(address.city_id);
        }
    }

    None
}

/// Resolve city_id from addresses: Residential > Work > fallback
pub fn resolve_city_id_from_addresses(
    addresses: &Option<Vec<AddressData>>,
    fallback_city_id: Option<Uuid>,
) -> Result<Uuid, DomainError> {
    if let Some(city_id) = derive_city_id_from_addresses(addresses) {
        return Ok(city_id);
    }

    if let Some(city_id) = fallback_city_id {
        return Ok(city_id);
    }

    Err(DomainError::ValidationError(
        "no Residential or Work address provided; please send city_id in the request body"
            .to_string(),
    ))
}

/// Normalize boolean flag from optional list presence
pub fn normalize_flag_from_list(values: &Option<Vec<String>>) -> (bool, Option<Vec<String>>) {
    match values {
        Some(list) if !list.is_empty() => (true, Some(list.clone())),
        _ => (false, None),
    }
}

/// Derive security agent flag from the presence of a security force value
pub fn is_security_agent<T>(security_force: &Option<T>) -> bool {
    security_force.is_some()
}
