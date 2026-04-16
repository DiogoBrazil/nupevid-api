use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use log::error;
use serde::Serialize;

use crate::core::entities::auth::UserClaims;
use crate::core::pagination::PaginatedResult;
use crate::core::filters::common::IncludeRelatedQuery;
use crate::core::application_error::ApplicationError as AppError;
use crate::utils::pagination::{Pagination, PaginationParams, normalize_pagination};
use crate::utils::responses::{ApiResponse, PaginatedResponse};

pub fn request_claims(req: &HttpRequest) -> Result<UserClaims, AppError> {
    req.extensions()
        .get::<UserClaims>()
        .cloned()
        .ok_or_else(|| {
            error!("[ServiceHelper] No claims found in request");
            AppError::Unauthorized("Unauthorized".to_string())
        })
}

pub fn request_pagination(params: &PaginationParams) -> Pagination {
    normalize_pagination(params)
}

pub fn request_pagination_from_parts(page: Option<i64>, page_size: Option<i64>) -> Pagination {
    request_pagination(&PaginationParams { page, page_size })
}

pub fn include_related(query: &IncludeRelatedQuery) -> bool {
    query.include_related_entities.unwrap_or(false)
}

pub fn created<T: Serialize>(data: T) -> HttpResponse {
    ApiResponse::created(data).into_response()
}

pub fn success<T: Serialize>(data: T) -> HttpResponse {
    ApiResponse::success(data).into_response()
}

pub fn paginated<T: Serialize>(result: PaginatedResult<T>) -> HttpResponse {
    PaginatedResponse::success(
        result.items,
        result.page,
        result.page_size,
        result.total_items,
    )
    .into_response()
}
