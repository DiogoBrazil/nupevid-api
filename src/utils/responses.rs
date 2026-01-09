use actix_web::{HttpResponse, http::StatusCode};
use serde::Serialize;
use serde_json::json;

#[derive(Debug)]
pub struct ApiResponse<T: Serialize> {
    message: String,
    status_code: StatusCode,
    data: Option<T>,
}

#[derive(Debug)]
pub struct PaginatedResponse<T: Serialize> {
    message: String,
    status_code: StatusCode,
    data: Vec<T>,
    page: i64,
    page_size: i64,
    total_items: i64,
    total_pages: i64,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn success(data: Vec<T>, page: i64, page_size: i64, total_items: i64) -> Self {
        let total_pages = if total_items == 0 {
            0
        } else {
            (total_items + page_size - 1) / page_size
        };

        Self {
            message: "Operation successful".to_string(),
            status_code: StatusCode::OK,
            data,
            page,
            page_size,
            total_items,
            total_pages,
        }
    }

    pub fn into_response(self) -> HttpResponse {
        HttpResponse::build(self.status_code)
            .json(json!({
                "message": self.message,
                "status": self.status_code.as_u16(),
                "data": self.data,
                "page": self.page,
                "page_size": self.page_size,
                "total_items": self.total_items,
                "total_pages": self.total_pages
            }))
    }
}
impl<T: Serialize> ApiResponse<T> {
    pub fn new(message: String, status_code: StatusCode, data: Option<T>) -> Self {
        Self {
            message,
            status_code,
            data,
        }
    }

    pub fn success(data: T) -> Self {
        Self {
            message: "Operation successful".to_string(),
            status_code: StatusCode::OK,
            data: Some(data),
        }
    }

    pub fn created(data: T) -> Self {
        Self {
            message: "Resource created successfully".to_string(),
            status_code: StatusCode::CREATED,
            data: Some(data),
        }
    }

    pub fn into_response(self) -> HttpResponse {
        HttpResponse::build(self.status_code)
            .json(json!({
                "message": self.message,
                "status": self.status_code.as_u16(),
                "data": self.data
            }))
    }
}
