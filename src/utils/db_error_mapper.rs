use crate::utils::errors::AppError;

pub fn map_constraint(constraint: Option<&str>, mappings: &[(&str, &str)]) -> Option<AppError> {
    let constraint = constraint?;
    for (key, message) in mappings {
        if constraint == *key {
            return Some(AppError::BadRequest(message.to_string()));
        }
    }
    None
}

pub fn map_unique_constraint(
    constraint: Option<&str>,
    mappings: &[(&str, &str)],
) -> Option<AppError> {
    let constraint = constraint?;
    for (key, message) in mappings {
        if constraint == *key {
            return Some(AppError::Conflict(message.to_string()));
        }
    }
    None
}
