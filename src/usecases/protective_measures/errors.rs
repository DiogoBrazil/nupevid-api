use crate::core::application_error::ApplicationError as AppError;

pub(super) fn map_reference_error(msg: String, action_prefix: &str) -> AppError {
    let detail = match msg.as_str() {
        "Court district not found" => "court_district_id not found".to_string(),
        _ => msg,
    };
    AppError::BadRequest(format!("{}: {}", action_prefix, detail))
}
