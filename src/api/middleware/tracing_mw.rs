use std::time::Instant;

use axum::body::Body;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;

use super::request_id::RequestId;

pub async fn tracing_middleware(
    req: Request<Body>,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let request_id = req
        .extensions()
        .get::<RequestId>()
        .map(|r| r.0.clone())
        .unwrap_or_default();

    let start = Instant::now();
    let response = next.run(req).await;
    let latency_ms = start.elapsed().as_millis();
    let status = response.status().as_u16();

    log_request(&request_id, &method, &path, status, latency_ms);

    response
}

fn log_request(
    request_id: &str,
    method: &axum::http::Method,
    path: &str,
    status: u16,
    latency_ms: u128,
) {
    match status {
        s if s >= 500 => tracing::error!(
            request_id, %method, path, status, latency_ms,
            "Request completed"
        ),
        s if s >= 400 => tracing::warn!(
            request_id, %method, path, status, latency_ms,
            "Request completed"
        ),
        _ => tracing::info!(
            request_id, %method, path, status, latency_ms,
            "Request completed"
        ),
    }
}
