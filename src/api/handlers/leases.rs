use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;
use crate::api::errors::ApiError;
use crate::api::extractors::auth::AuthenticatedUser;
use crate::api::extractors::pagination::PaginationParams;
use crate::AppState;
use pliq_back_db::models::{CurrencyType, Lease, LeaseStatus, NewLease};

#[derive(Debug, Deserialize)]
pub struct CreateLeaseRequest {
    pub application_id: Uuid,
    pub listing_id: Uuid,
    pub tenant_id: Uuid,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub monthly_rent: i64,
    pub deposit_amount: i64,
    pub currency: CurrencyType,
    pub contract_address: Option<String>,
    pub terms_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignLeaseRequest {
    pub signature: String,
}

pub async fn create_lease(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(body): Json<CreateLeaseRequest>,
) -> Result<(axum::http::StatusCode, Json<Lease>), ApiError> {
    let new = NewLease {
        application_id: body.application_id,
        listing_id: body.listing_id,
        tenant_id: body.tenant_id,
        landlord_id: auth.user_id,
        start_date: body.start_date,
        end_date: body.end_date,
        monthly_rent: body.monthly_rent,
        deposit_amount: body.deposit_amount,
        currency: body.currency,
        contract_address: body.contract_address,
        terms_hash: body.terms_hash,
    };
    let lease = pliq_back_db::queries::leases::create(&state.db, &new).await?;
    Ok((axum::http::StatusCode::CREATED, Json(lease)))
}

pub async fn list_leases(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<Lease>>, ApiError> {
    Ok(Json(
        pliq_back_db::queries::leases::list_by_user(
            &state.db, auth.user_id, &params.into_pagination(),
        )
        .await?,
    ))
}

pub async fn get_lease(
    State(state): State<AppState>,
    _auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Lease>, ApiError> {
    let lease = pliq_back_db::queries::leases::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Lease not found".into()))?;
    Ok(Json(lease))
}

pub async fn sign_lease(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(body): Json<SignLeaseRequest>,
) -> Result<Json<Lease>, ApiError> {
    let lease = pliq_back_db::queries::leases::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Lease not found".into()))?;

    let updated = if lease.tenant_id == auth.user_id {
        pliq_back_db::queries::leases::sign_tenant(&state.db, id, &body.signature).await?
    } else if lease.landlord_id == auth.user_id {
        pliq_back_db::queries::leases::sign_landlord(&state.db, id, &body.signature).await?
    } else {
        return Err(ApiError::Forbidden("Not a party to this lease".into()));
    };

    // Check if both parties have signed and activate
    if updated.tenant_signature.is_some() && updated.landlord_signature.is_some() {
        if matches!(updated.status, LeaseStatus::TenantSigned | LeaseStatus::LandlordSigned) {
            let activated = pliq_back_db::queries::leases::activate(&state.db, id).await?;
            return Ok(Json(activated));
        }
    }
    Ok(Json(updated))
}

pub async fn terminate_lease(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Lease>, ApiError> {
    let lease = pliq_back_db::queries::leases::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Lease not found".into()))?;
    if lease.tenant_id != auth.user_id && lease.landlord_id != auth.user_id {
        return Err(ApiError::Forbidden("Not a party to this lease".into()));
    }
    // For now just return the lease — termination workflow TBD
    Ok(Json(lease))
}
