use axum::http::{header, HeaderName, HeaderValue, Method};
use tower_http::cors::CorsLayer;

pub fn cors_layer(origins: &str) -> CorsLayer {
    let allowed_origins: Vec<HeaderValue> = origins
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            HeaderName::from_static("x-request-id"),
        ])
        .allow_credentials(true)
}
