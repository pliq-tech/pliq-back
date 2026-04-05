use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;
use crate::api::errors::ApiError;
use crate::api::extractors::auth::AuthenticatedUser;
use crate::api::extractors::pagination::PaginationParams;
use crate::AppState;
use pliq_back_db::models::{Application, ApplicationStatus, NewApplication};

#[derive(Debug, Deserialize)]
pub struct CreateApplicationRequest {
    pub listing_id: Uuid,
    pub cover_message: Option<String>,
    pub zk_proof_hash: Option<String>,
    pub credential_summary: Option<serde_json::Value>,
}

pub async fn create_application(State(state): State<AppState>, auth: AuthenticatedUser, Json(body): Json<CreateApplicationRequest>) -> Result<(axum::http::StatusCode, Json<Application>), ApiError> {
    pliq_back_db::queries::listings::get_by_id(&state.db, body.listing_id).await?.ok_or_else(|| ApiError::NotFound("Listing not found".into()))?;
    let has_active = pliq_back_db::queries::applications::has_active_application(&state.db, auth.user_id, body.listing_id).await.map_err(|e| ApiError::Internal(e.to_string()))?;
    if has_active { return Err(ApiError::Conflict("Active application already exists".into())); }
    let new = NewApplication { listing_id: body.listing_id, tenant_id: auth.user_id, cover_message: body.cover_message, zk_proof_hash: body.zk_proof_hash, credential_summary: body.credential_summary };
    let app = pliq_back_db::queries::applications::create(&state.db, &new).await?;
    Ok((axum::http::StatusCode::CREATED, Json(app)))
}

pub async fn list_applications(State(state): State<AppState>, auth: AuthenticatedUser, Query(params): Query<PaginationParams>) -> Result<Json<Vec<Application>>, ApiError> {
    let pagination = params.into_pagination();
    Ok(Json(pliq_back_db::queries::applications::list_by_tenant(&state.db, auth.user_id, &pagination).await?))
}

pub async fn get_application(State(state): State<AppState>, _auth: AuthenticatedUser, Path(id): Path<Uuid>) -> Result<Json<Application>, ApiError> {
    Ok(Json(pliq_back_db::queries::applications::get_by_id(&state.db, id).await?.ok_or_else(|| ApiError::NotFound("Application not found".into()))?))
}

pub async fn update_application_status(State(state): State<AppState>, _auth: AuthenticatedUser, Path(id): Path<Uuid>, Json(body): Json<serde_json::Value>) -> Result<Json<Application>, ApiError> {
    let status_str = body.get("status").and_then(|v| v.as_str()).ok_or_else(|| ApiError::Validation("status required".into()))?;
    let status = match status_str { "accepted" => ApplicationStatus::Accepted, "rejected" => ApplicationStatus::Rejected, _ => return Err(ApiError::Validation("Invalid status".into())) };
    Ok(Json(pliq_back_db::queries::applications::update_status(&state.db, id, status).await?))
}
