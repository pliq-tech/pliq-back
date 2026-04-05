use axum::extract::State;
use axum::Json;
use serde::Serialize;
use crate::api::errors::ApiError;
use crate::api::extractors::auth::AuthenticatedUser;
use crate::AppState;
use pliq_back_db::models::ReputationScore;

#[derive(Debug, Serialize)]
pub struct ReputationResponse { pub score: ReputationScore, pub calculated_score: i32 }

#[derive(Debug, Serialize)]
pub struct MerkleRootResponse { pub merkle_root: Option<String>, pub leaf_count: i64 }

pub async fn get_my_reputation(State(state): State<AppState>, auth: AuthenticatedUser) -> Result<Json<ReputationResponse>, ApiError> {
    let score = pliq_back_db::queries::reputation::upsert(&state.db, auth.user_id).await?;
    let calculated = score.calculate_score();
    Ok(Json(ReputationResponse { score, calculated_score: calculated }))
}

pub async fn get_merkle_root(State(state): State<AppState>) -> Result<Json<MerkleRootResponse>, ApiError> {
    let meta = pliq_back_db::queries::merkle::get_tree_metadata(&state.db).await.map_err(|e| ApiError::Internal(e.to_string()))?;
    let (root, count) = match meta { Some(m) => (m.merkle_root, m.leaf_count), None => (None, 0) };
    Ok(Json(MerkleRootResponse { merkle_root: root, leaf_count: count }))
}

pub async fn get_my_proofs(State(state): State<AppState>, auth: AuthenticatedUser) -> Result<Json<Vec<pliq_back_db::models::MerkleLeafRow>>, ApiError> {
    Ok(Json(pliq_back_db::queries::merkle::get_leaves_by_tenant(&state.db, auth.user_id).await.map_err(|e| ApiError::Internal(e.to_string()))?))
}
