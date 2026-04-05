use axum::response::IntoResponse;
use super::errors::ApiError;

#[test]
fn test_validation_error_is_400() {
    let err = ApiError::Validation("bad input".into());
    let response = err.into_response();
    assert_eq!(response.status(), 400);
}

#[test]
fn test_not_found_error_is_404() {
    let err = ApiError::NotFound("missing".into());
    let response = err.into_response();
    assert_eq!(response.status(), 404);
}

#[test]
fn test_unauthorized_error_is_401() {
    let err = ApiError::Unauthorized("no auth".into());
    let response = err.into_response();
    assert_eq!(response.status(), 401);
}

#[test]
fn test_forbidden_error_is_403() {
    let err = ApiError::Forbidden("denied".into());
    let response = err.into_response();
    assert_eq!(response.status(), 403);
}

#[test]
fn test_internal_error_is_500() {
    let err = ApiError::Internal("oops".into());
    let response = err.into_response();
    assert_eq!(response.status(), 500);
}

#[test]
fn test_rate_limited_is_429() {
    let err = ApiError::RateLimited;
    let response = err.into_response();
    assert_eq!(response.status(), 429);
}
