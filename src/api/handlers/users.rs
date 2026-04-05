use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::api::errors::ApiError;
use crate::api::extractors::auth::AuthenticatedUser;
use crate::AppState;
use pliq_back_db::models::{UpdateUser, User};

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub wallet_address: Option<String>,
    pub role: Option<pliq_back_db::models::UserRole>,
    pub preferred_language: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublicProfile {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub role: pliq_back_db::models::UserRole,
    pub verification_level: pliq_back_db::models::VerificationLevel,
}

pub async fn get_me(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<Json<User>, ApiError> {
    let user = pliq_back_db::queries::users::get_by_id(&state.db, auth.user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("User not found".into()))?;
    Ok(Json(user))
}

pub async fn update_me(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<Json<User>, ApiError> {
    let updates = UpdateUser {
        display_name: body.display_name,
        avatar_url: body.avatar_url,
        wallet_address: body.wallet_address,
        role: body.role,
        verification_level: None,
        preferred_language: body.preferred_language,
    };
    let user = pliq_back_db::queries::users::update(&state.db, auth.user_id, &updates).await?;
    Ok(Json(user))
}

pub async fn get_public_profile(
    State(state): State<AppState>,
    _auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<PublicProfile>, ApiError> {
    let user = pliq_back_db::queries::users::get_by_id(&state.db, id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("User not found".into()))?;
    Ok(Json(PublicProfile {
        id: user.id,
        display_name: user.display_name,
        role: user.role,
        verification_level: user.verification_level,
    }))
}
