use actix_web::{HttpResponse, error::ResponseError, http::StatusCode};
use log::error;
use serde_json::json;

use crate::core::application_error::ApplicationError;

impl ResponseError for ApplicationError {
    fn error_response(&self) -> HttpResponse {
        error!("Error occurred: {}", self);
        let (status_code, error_type) = match self {
            ApplicationError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            ApplicationError::BadRequest(_) => (StatusCode::BAD_REQUEST, "Bad Request"),
            ApplicationError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            ApplicationError::Forbidden(_) => (StatusCode::FORBIDDEN, "Forbidden"),
            ApplicationError::NotFound(_) => (StatusCode::NOT_FOUND, "Not Found"),
            ApplicationError::Conflict(_) => (StatusCode::CONFLICT, "Conflict"),
            ApplicationError::UnprocessableEntity { .. } => {
                (StatusCode::UNPROCESSABLE_ENTITY, "UnprocessableEntity")
            }
            ApplicationError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database Error")
            }
            ApplicationError::InvalidMethodError(_) => {
                (StatusCode::METHOD_NOT_ALLOWED, "Invalid Method Error")
            }
        };

        let message = match self {
            ApplicationError::UnprocessableEntity { message, .. } => message.clone(),
            _ => self.to_string(),
        };

        let mut body = json!({
            "error": error_type,
            "message": message,
            "status_code": status_code.as_u16()
        });

        if let ApplicationError::UnprocessableEntity {
            field: Some(field), ..
        } = self
        {
            body["field"] = json!(field);
        }

        HttpResponse::build(status_code).json(body)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApplicationError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ApplicationError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApplicationError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApplicationError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApplicationError::NotFound(_) => StatusCode::NOT_FOUND,
            ApplicationError::Conflict(_) => StatusCode::CONFLICT,
            ApplicationError::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            ApplicationError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApplicationError::InvalidMethodError(_) => StatusCode::METHOD_NOT_ALLOWED,
        }
    }
}
