use axum::Json;
use serde_json::{json, Value};

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "healthy", "service": "pliq-back", "version": env!("CARGO_PKG_VERSION") }))
}

pub async fn ready() -> Json<Value> {
    Json(json!({ "status": "ready" }))
}
