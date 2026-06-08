use uuid::Uuid;

use crate::core::address_resolution::resolve_city_id_from_addresses;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::victims::SaveVictim;
use crate::core::entities::common::derive_flag_from_list;
use crate::validators::cpf_validator::validate_cpf;

pub fn normalize_victim_input(
    data: &mut SaveVictim,
    error_context: &str,
) -> Result<Uuid, AppError> {
    data.full_name = data.full_name.trim().to_uppercase();

    let city_id = resolve_city_id_from_addresses(&data.addresses, data.city_id)
        .map_err(|e| AppError::BadRequest(format!("{}: {}", error_context, e)))?;
    data.city_id = Some(city_id);

    let (has_special_needs, special_needs_type) = derive_flag_from_list(&data.special_needs_type);
    data.has_special_needs = has_special_needs;
    data.special_needs_type = special_needs_type;

    let (has_psychiatric_issues, psychiatric_issues_type) =
        derive_flag_from_list(&data.psychiatric_issues_type);
    data.has_psychiatric_issues = has_psychiatric_issues;
    data.psychiatric_issues_type = psychiatric_issues_type;
    data.has_children = data.children_count.is_some();

    if let Some(cpf) = data.cpf.as_ref() {
        let normalized = validate_cpf(cpf, error_context)?;
        data.cpf = Some(normalized);
    }

    Ok(city_id)
}
