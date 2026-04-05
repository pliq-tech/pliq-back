use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// Standard API response envelope wrapping data with optional pagination.
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<PaginationMeta>,
}

/// Pagination metadata included in paginated responses.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PaginationMeta {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
}

/// Return `200 OK` with the given data wrapped in an `ApiResponse` envelope.
pub fn json_data<T: Serialize>(data: T) -> Response {
    (StatusCode::OK, Json(ApiResponse { data, meta: None })).into_response()
}

/// Return `201 Created` with the given data wrapped in an `ApiResponse` envelope.
pub fn json_created<T: Serialize>(data: T) -> Response {
    (
        StatusCode::CREATED,
        Json(ApiResponse { data, meta: None }),
    )
        .into_response()
}

/// Return `200 OK` with data and pagination metadata.
pub fn json_paginated<T: Serialize>(
    data: T,
    page: i64,
    limit: i64,
    total: i64,
) -> Response {
    (
        StatusCode::OK,
        Json(ApiResponse {
            data,
            meta: Some(PaginationMeta { page, limit, total }),
        }),
    )
        .into_response()
}

#[cfg(test)]
#[path = "response_tests.rs"]
mod response_tests;
