use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::api::errors::ApiError;
use crate::api::extractors::auth::AuthenticatedUser;
use crate::crypto::commitments;
use crate::AppState;
use pliq_back_db::models::{CurrencyType, EscrowCommitment, EscrowStatus, NewEscrowCommitment};

#[derive(Debug, Deserialize)]
pub struct CommitRequest {
    pub listing_id: Uuid,
    pub landlord_id: Uuid,
    pub amount: i64,
    pub currency: CurrencyType,
}

#[derive(Debug, Serialize)]
pub struct CommitResponse {
    pub escrow: EscrowCommitment,
    pub commitment_hash: String,
}

pub async fn commit_escrow(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<CommitRequest>,
) -> Result<(axum::http::StatusCode, Json<CommitResponse>), ApiError> {
    let conditions = commitments::EscrowConditions {
        amount: body.amount as u64,
        tenant_id: auth.user_id.to_string(),
        landlord_id: body.landlord_id.to_string(),
        listing_id: body.listing_id.to_string(),
    };
    let commitment = commitments::create_commitment(&conditions);
    let hash_hex = hex::encode(commitment.commitment_hash);
    let new = NewEscrowCommitment {
        listing_id: body.listing_id,
        tenant_id: auth.user_id,
        landlord_id: body.landlord_id,
        commitment_hash: hash_hex.clone(),
        amount: body.amount,
        currency: body.currency,
        conditions_hash: Some(hex::encode(conditions.hash())),
        secret: hex::encode(commitment.secret),
        nonce: hex::encode(commitment.nonce),
    };
    let escrow = pliq_back_db::queries::escrow::create(&state.db, &new).await?;
    Ok((axum::http::StatusCode::CREATED, Json(CommitResponse { escrow, commitment_hash: hash_hex })))
}

pub async fn get_escrow(
    State(state): State<AppState>,
    _auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<EscrowCommitment>, ApiError> {
    let escrow = pliq_back_db::queries::escrow::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Escrow not found".into()))?;
    Ok(Json(escrow))
}

pub async fn reveal_escrow(
    State(state): State<AppState>,
    _auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<EscrowCommitment>, ApiError> {
    Ok(Json(
        pliq_back_db::queries::escrow::update_status(&state.db, id, EscrowStatus::Revealed).await?,
    ))
}

pub async fn release_escrow(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<EscrowCommitment>, ApiError> {
    let escrow = pliq_back_db::queries::escrow::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Escrow not found".into()))?;
    if escrow.landlord_id != auth.user_id {
        return Err(ApiError::Forbidden("Only the landlord can release".into()));
    }
    Ok(Json(
        pliq_back_db::queries::escrow::update_status(&state.db, id, EscrowStatus::Released).await?,
    ))
}

pub async fn dispute_escrow(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<EscrowCommitment>, ApiError> {
    let escrow = pliq_back_db::queries::escrow::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Escrow not found".into()))?;
    if escrow.tenant_id != auth.user_id && escrow.landlord_id != auth.user_id {
        return Err(ApiError::Forbidden("Not a party to this escrow".into()));
    }
    Ok(Json(
        pliq_back_db::queries::escrow::update_status(&state.db, id, EscrowStatus::Disputed).await?,
    ))
}
