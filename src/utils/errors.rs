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
            ApplicationError::PayloadTooLarge(_) => {
                (StatusCode::PAYLOAD_TOO_LARGE, "Payload Too Large")
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
            ApplicationError::DatabaseError(_) => "Database error".to_string(),
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
            ApplicationError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            ApplicationError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApplicationError::InvalidMethodError(_) => StatusCode::METHOD_NOT_ALLOWED,
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::body::to_bytes;
    use actix_web::error::ResponseError;

    use super::*;

    #[actix_rt::test]
    async fn database_error_response_does_not_expose_internal_detail() {
        let response =
            ApplicationError::DatabaseError("secret db detail".to_string()).error_response();
        let body = to_bytes(response.into_body()).await.unwrap();
        let text = std::str::from_utf8(&body).unwrap();

        assert!(!text.contains("secret db detail"));
        assert!(text.contains("Database error"));
    }
}
