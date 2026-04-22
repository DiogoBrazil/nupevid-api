use uuid::Uuid;

use crate::core::address_resolution::resolve_city_id_from_addresses;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::offenders::SaveOffender;
use crate::core::entities::common::{derive_flag_from_list, is_security_agent};
use crate::validators::cpf_validator::validate_cpf;

pub fn normalize_offender_input(
    data: &mut SaveOffender,
    error_context: &str,
) -> Result<Uuid, AppError> {
    let city_id = resolve_city_id_from_addresses(&data.addresses, data.city_id)
        .map_err(|e| AppError::BadRequest(format!("{}: {}", error_context, e)))?;
    data.city_id = Some(city_id);
    data.is_public_security_agent = is_security_agent(&data.security_force);

    let (has_psychiatric_issues, psychiatric_issues_type) =
        derive_flag_from_list(&data.psychiatric_issues_type);
    data.has_psychiatric_issues = has_psychiatric_issues;
    data.psychiatric_issues_type = psychiatric_issues_type;

    if let Some(cpf) = data.cpf.as_ref() {
        let normalized = validate_cpf(cpf, error_context)?;
        data.cpf = Some(normalized);
    }

    Ok(city_id)
}
