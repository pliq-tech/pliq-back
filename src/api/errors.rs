use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::domain::errors::DomainError;

#[derive(Debug)]
pub enum ApiError {
    Validation(String),
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg),
            ApiError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "An internal error occurred".to_string())
            }
        };
        (status, Json(ErrorBody { error: ErrorDetail { code, message } })).into_response()
    }
}

impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::NotFound(msg) => ApiError::NotFound(msg),
            DomainError::Unauthorized(msg) => ApiError::Unauthorized(msg),
            DomainError::Validation(msg) => ApiError::Validation(msg),
            DomainError::Conflict(msg) => ApiError::Conflict(msg),
            DomainError::Internal(msg) => ApiError::Internal(msg),
        }
    }
}

impl From<pliq_back_db::DbError> for ApiError {
    fn from(err: pliq_back_db::DbError) -> Self {
        let domain_err: DomainError = err.into();
        domain_err.into()
    }
}
