use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;
use crate::api::errors::ApiError;
use crate::api::extractors::auth::AuthenticatedUser;
use crate::api::extractors::pagination::PaginationParams;
use crate::AppState;
use pliq_back_db::models::{CurrencyType, NewPayment, Payment, PaymentType};

#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest { pub lease_id: Uuid, pub payee_id: Uuid, pub amount: i64, pub currency: CurrencyType, pub payment_type: PaymentType, pub idempotency_key: String, pub due_date: Option<chrono::NaiveDate> }

pub async fn create_payment(State(state): State<AppState>, auth: AuthenticatedUser, Json(body): Json<CreatePaymentRequest>) -> Result<(axum::http::StatusCode, Json<Payment>), ApiError> {
    if let Some(existing) = pliq_back_db::queries::payments::get_by_idempotency_key(&state.db, &body.idempotency_key).await.map_err(|e| ApiError::Internal(e.to_string()))? {
        return Ok((axum::http::StatusCode::CREATED, Json(existing)));
    }
    let new = NewPayment { lease_id: body.lease_id, payer_id: auth.user_id, payee_id: body.payee_id, amount: body.amount, currency: body.currency, payment_type: body.payment_type, idempotency_key: body.idempotency_key, due_date: body.due_date };
    Ok((axum::http::StatusCode::CREATED, Json(pliq_back_db::queries::payments::create(&state.db, &new).await?)))
}

pub async fn list_payments(State(state): State<AppState>, auth: AuthenticatedUser, Query(params): Query<PaginationParams>) -> Result<Json<Vec<Payment>>, ApiError> {
    Ok(Json(pliq_back_db::queries::payments::list_by_user(&state.db, auth.user_id, &params.into_pagination()).await?))
}

pub async fn get_payment(State(state): State<AppState>, _auth: AuthenticatedUser, Path(id): Path<Uuid>) -> Result<Json<Payment>, ApiError> {
    Ok(Json(pliq_back_db::queries::payments::get_by_id(&state.db, id).await?.ok_or_else(|| ApiError::NotFound("Payment not found".into()))?))
}
