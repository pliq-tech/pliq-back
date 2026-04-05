use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::api::errors::ApiError;
use crate::api::extractors::auth::AuthenticatedUser;
use crate::api::extractors::pagination::PaginationParams;
use crate::AppState;
use pliq_back_db::models::{CurrencyType, NewPayment, Payment, PaymentType};

#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub payee_id: Uuid,
    pub amount: i64,
    pub currency: CurrencyType,
    pub payment_type: PaymentType,
    pub idempotency_key: String,
    pub due_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct PaymentReceipt {
    pub payment: Payment,
    pub merkle_proof: Option<serde_json::Value>,
}

pub async fn create_payment(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(lease_id): Path<Uuid>,
    Json(body): Json<CreatePaymentRequest>,
) -> Result<(axum::http::StatusCode, Json<Payment>), ApiError> {
    if let Some(existing) = pliq_back_db::queries::payments::get_by_idempotency_key(
        &state.db, &body.idempotency_key,
    )
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        return Ok((axum::http::StatusCode::OK, Json(existing)));
    }
    let new = NewPayment {
        lease_id,
        payer_id: auth.user_id,
        payee_id: body.payee_id,
        amount: body.amount,
        currency: body.currency,
        payment_type: body.payment_type,
        idempotency_key: body.idempotency_key,
        due_date: body.due_date,
    };
    let payment = pliq_back_db::queries::payments::create(&state.db, &new).await?;
    Ok((axum::http::StatusCode::CREATED, Json(payment)))
}

pub async fn list_lease_payments(
    State(state): State<AppState>,
    _auth: AuthenticatedUser,
    Path(lease_id): Path<Uuid>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<Payment>>, ApiError> {
    Ok(Json(
        pliq_back_db::queries::payments::list_by_lease(
            &state.db, lease_id, &params.into_pagination(),
        )
        .await?,
    ))
}

pub async fn get_payment(
    State(state): State<AppState>,
    _auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Payment>, ApiError> {
    let payment = pliq_back_db::queries::payments::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Payment not found".into()))?;
    Ok(Json(payment))
}

pub async fn get_payment_receipt(
    State(state): State<AppState>,
    _auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<PaymentReceipt>, ApiError> {
    let payment = pliq_back_db::queries::payments::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Payment not found".into()))?;

    let merkle_proof = if let Some(leaf_index) = payment.merkle_leaf_index {
        let tree = state.merkle_tree.read().await;
        tree.structured_proof(leaf_index as usize)
            .map(|p| serde_json::json!({
                "leaf_index": leaf_index,
                "siblings": p.siblings.iter().map(hex::encode).collect::<Vec<_>>(),
                "path_indices": p.path_indices,
                "root": tree.root_hex(),
            }))
    } else {
        None
    };

    Ok(Json(PaymentReceipt { payment, merkle_proof }))
}

pub async fn initiate_payment(
    State(_state): State<AppState>,
    _auth: AuthenticatedUser,
    Json(_body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Payment intent creation — stub for Circle/Unlink integration
    Ok(Json(serde_json::json!({
        "message": "Payment intent created",
        "status": "pending",
    })))
}

pub async fn payment_history(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<Payment>>, ApiError> {
    Ok(Json(
        pliq_back_db::queries::payments::list_by_user(
            &state.db, auth.user_id, &params.into_pagination(),
        )
        .await?,
    ))
}
