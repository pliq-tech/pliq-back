use axum::body::Body;
use axum::extract::State;
use axum::http::{header, Request};
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::errors::ApiError;
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub nullifier_hash: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn encode_jwt(user_id: Uuid, nullifier_hash: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now().timestamp();
    let claims = Claims { sub: user_id, nullifier_hash: nullifier_hash.to_string(), exp: now + 86400, iat: now };
    jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()))
}

pub async fn auth_middleware(State(state): State<AppState>, mut req: Request<Body>, next: Next) -> Result<Response, ApiError> {
    let auth_header = req.headers().get(header::AUTHORIZATION).and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".into()))?;
    let token = auth_header.strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::Unauthorized("Invalid Authorization format".into()))?;
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()), &Validation::default())
        .map_err(|e| ApiError::Unauthorized(format!("Invalid token: {}", e)))?;
    req.extensions_mut().insert(token_data.claims);
    Ok(next.run(req).await)
}
