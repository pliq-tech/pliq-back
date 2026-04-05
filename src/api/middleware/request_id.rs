use axum::body::Body;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

/// Newtype wrapper stored in request extensions for downstream access.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

pub async fn request_id_middleware(
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let id = extract_or_generate(&req);
    req.extensions_mut().insert(RequestId(id.clone()));

    let mut response = next.run(req).await;
    if let Ok(value) = id.parse() {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}

fn extract_or_generate(req: &Request<Body>) -> String {
    req.headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}
