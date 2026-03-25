use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use log::error;
use serde::Serialize;

use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::entities::common::PaginatedResult;
use crate::core::queries::common::IncludeComplementQuery;
use crate::utils::errors::AppError;
use crate::utils::pagination::{Pagination, PaginationParams, normalize_pagination};
use crate::utils::responses::{ApiResponse, PaginatedResponse};

pub fn request_claims(req: &HttpRequest) -> Result<ClaimsToUserToken, AppError> {
    req.extensions()
        .get::<ClaimsToUserToken>()
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

pub fn include_complement(query: &IncludeComplementQuery) -> bool {
    query.include_complement_for_entities.unwrap_or(false)
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
