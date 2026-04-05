use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::api::errors::ApiError;
use crate::api::extractors::auth::AuthenticatedUser;
use crate::AppState;
use pliq_back_db::models::ReputationScore;

#[derive(Debug, Serialize)]
pub struct ReputationResponse {
    pub score: ReputationScore,
    pub calculated_score: i32,
}

#[derive(Debug, Serialize)]
pub struct MerkleRootResponse {
    pub merkle_root: Option<String>,
    pub leaf_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct VerifyProofRequest {
    pub root: String,
    pub leaf_hash: String,
    pub siblings: Vec<String>,
    pub path_indices: Vec<bool>,
}

#[derive(Debug, Serialize)]
pub struct CredentialResponse {
    pub user_id: Uuid,
    pub score: i32,
    pub payment_count: i32,
    pub merkle_root: Option<String>,
    pub proof_count: i32,
}

pub async fn get_my_reputation(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<Json<ReputationResponse>, ApiError> {
    let score = pliq_back_db::queries::reputation::upsert(&state.db, auth.user_id).await?;
    let calculated = score.calculate_score();
    Ok(Json(ReputationResponse { score, calculated_score: calculated }))
}

pub async fn get_user_reputation(
    State(state): State<AppState>,
    _auth: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ReputationResponse>, ApiError> {
    let score = pliq_back_db::queries::reputation::upsert(&state.db, user_id).await?;
    let calculated = score.calculate_score();
    Ok(Json(ReputationResponse { score, calculated_score: calculated }))
}

pub async fn get_merkle_root(
    State(state): State<AppState>,
) -> Result<Json<MerkleRootResponse>, ApiError> {
    let meta = pliq_back_db::queries::merkle::get_tree_metadata(&state.db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let (root, count) = match meta {
        Some(m) => (m.merkle_root, m.leaf_count),
        None => (None, 0),
    };
    Ok(Json(MerkleRootResponse { merkle_root: root, leaf_count: count }))
}

pub async fn get_my_proofs(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<Json<Vec<pliq_back_db::models::MerkleLeafRow>>, ApiError> {
    Ok(Json(
        pliq_back_db::queries::merkle::get_leaves_by_tenant(&state.db, auth.user_id)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?,
    ))
}

pub async fn get_my_credential(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<Json<CredentialResponse>, ApiError> {
    let score = pliq_back_db::queries::reputation::upsert(&state.db, auth.user_id).await?;
    let calculated = score.calculate_score();
    Ok(Json(CredentialResponse {
        user_id: auth.user_id,
        score: calculated,
        payment_count: score.payment_count,
        merkle_root: score.merkle_root,
        proof_count: score.merkle_proof_count,
    }))
}

pub async fn verify_proof(
    State(_state): State<AppState>,
    Json(body): Json<VerifyProofRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let root = hex::decode(&body.root)
        .map_err(|_| ApiError::Validation("Invalid root hex".into()))?;
    let leaf_hash = hex::decode(&body.leaf_hash)
        .map_err(|_| ApiError::Validation("Invalid leaf_hash hex".into()))?;

    if root.len() != 32 || leaf_hash.len() != 32 {
        return Err(ApiError::Validation("Root and leaf_hash must be 32 bytes".into()));
    }

    let siblings: Result<Vec<[u8; 32]>, _> = body
        .siblings
        .iter()
        .map(|s| {
            hex::decode(s)
                .map_err(|_| ApiError::Validation("Invalid sibling hex".into()))
                .and_then(|b| {
                    b.try_into()
                        .map_err(|_| ApiError::Validation("Sibling must be 32 bytes".into()))
                })
        })
        .collect();
    let siblings = siblings?;

    let mut root_arr = [0u8; 32];
    let mut leaf_arr = [0u8; 32];
    root_arr.copy_from_slice(&root);
    leaf_arr.copy_from_slice(&leaf_hash);

    let proof = crate::crypto::merkle::MerkleProof {
        siblings,
        path_indices: body.path_indices.clone(),
    };

    let valid = crate::crypto::merkle::MerkleTree::verify(&root_arr, &leaf_arr, &proof);
    Ok(Json(serde_json::json!({ "valid": valid })))
}
