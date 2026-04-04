use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use crate::api::errors::ApiError;
use crate::api::middleware::auth::encode_jwt;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct VerifyWorldIdRequest {
    pub nullifier_hash: String,
    pub merkle_root: String,
    pub proof: String,
    pub verification_level: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: uuid::Uuid,
    pub is_new_user: bool,
}

pub async fn verify_world_id(
    State(state): State<AppState>,
    Json(body): Json<VerifyWorldIdRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Check for existing user
    let existing = pliq_back_db::queries::users::get_by_nullifier(&state.db, &body.nullifier_hash)
        .await.map_err(|e| ApiError::Internal(e.to_string()))?;

    let (user, is_new_user) = match existing {
        Some(user) => (user, false),
        None => {
            let level = match body.verification_level.as_str() {
                "orb" => pliq_back_db::models::VerificationLevel::Orb,
                "device" => pliq_back_db::models::VerificationLevel::Device,
                _ => pliq_back_db::models::VerificationLevel::None,
            };
            let new_user = pliq_back_db::models::NewUser {
                nullifier_hash: body.nullifier_hash.clone(),
                wallet_address: None,
                display_name: None,
                role: pliq_back_db::models::UserRole::Tenant,
                verification_level: level,
            };
            let user = pliq_back_db::queries::users::create(&state.db, &new_user)
                .await.map_err(|e| ApiError::Internal(e.to_string()))?;
            (user, true)
        }
    };

    let token = encode_jwt(user.id, &user.nullifier_hash, &state.config.jwt_secret)
        .map_err(|e| ApiError::Internal(format!("JWT error: {}", e)))?;

    Ok(Json(AuthResponse { token, user_id: user.id, is_new_user }))
}
