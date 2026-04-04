use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::api::errors::ApiError;
use crate::api::middleware::auth::Claims;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub nullifier_hash: String,
}

impl<S: Send + Sync> FromRequestParts<S> for AuthenticatedUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts.extensions.get::<Claims>()
            .ok_or_else(|| ApiError::Unauthorized("Authentication required".into()))?;
        Ok(AuthenticatedUser { user_id: claims.sub, nullifier_hash: claims.nullifier_hash.clone() })
    }
}
