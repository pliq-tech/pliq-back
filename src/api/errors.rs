use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::api::middleware::request_id::RequestId;
use crate::domain::errors::DomainError;

#[derive(Debug)]
pub enum ApiError {
    Validation(String),
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    UnprocessableEntity(String),
    RateLimited,
    BadGateway { service: String },
    ServiceUnavailable { reason: String },
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
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
}

impl ApiError {
    fn status_and_code(&self) -> (StatusCode, &'static str, String) {
        match self {
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg.clone()),
            Self::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg.clone()),
            Self::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
            Self::UnprocessableEntity(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "UNPROCESSABLE_ENTITY", msg.clone())
            }
            Self::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMITED",
                "Too many requests".to_string(),
            ),
            Self::BadGateway { service } => (
                StatusCode::BAD_GATEWAY,
                "BAD_GATEWAY",
                format!("Upstream service unavailable: {service}"),
            ),
            Self::ServiceUnavailable { reason } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "SERVICE_UNAVAILABLE",
                reason.clone(),
            ),
            Self::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "An internal error occurred".to_string(),
                )
            }
        }
    }

    /// Build the response, optionally including request_id.
    pub fn into_response_with_id(self, request_id: Option<String>) -> Response {
        let (status, code, message) = self.status_and_code();
        let body = ErrorBody {
            error: ErrorDetail { code, message, request_id },
        };
        (status, Json(body)).into_response()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        self.into_response_with_id(None)
    }
}

/// Extension trait: convert ApiError using request extensions for request_id.
pub trait ApiErrorExt {
    fn with_request_id(self, extensions: &axum::http::Extensions) -> Response;
}

impl ApiErrorExt for ApiError {
    fn with_request_id(self, extensions: &axum::http::Extensions) -> Response {
        let id = extensions.get::<RequestId>().map(|r| r.0.clone());
        self.into_response_with_id(id)
    }
}

impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::NotFound(msg) => Self::NotFound(msg),
            DomainError::Unauthorized(msg) => Self::Unauthorized(msg),
            DomainError::Validation(msg) => Self::Validation(msg),
            DomainError::Conflict(msg) => Self::Conflict(msg),
            DomainError::Internal(msg) => Self::Internal(msg),
        }
    }
}

impl From<pliq_back_db::DbError> for ApiError {
    fn from(err: pliq_back_db::DbError) -> Self {
        let domain_err: DomainError = err.into();
        domain_err.into()
    }
}
