use uuid::Uuid;

use crate::core::entities::common::{AddressData, AddressType};
use crate::core::errors::DomainError;

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
