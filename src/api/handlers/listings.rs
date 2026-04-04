use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;
use crate::api::errors::ApiError;
use crate::api::extractors::auth::AuthenticatedUser;
use crate::AppState;
use pliq_back_db::models::{CurrencyType, Listing, ListingFilters, NewListing, PaginatedResult, UpdateListing};

#[derive(Debug, Deserialize)]
pub struct CreateListingRequest {
    pub title: String, pub description: String, pub address: String, pub city: String, pub country: String,
    pub latitude: Option<f64>, pub longitude: Option<f64>, pub rent_amount: i64, pub deposit_amount: i64,
    pub currency: CurrencyType, pub bedrooms: i32, pub bathrooms: i32, pub area_sqm: i32,
    pub amenities: serde_json::Value, pub photos: serde_json::Value, pub required_credentials: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListingFilterParams {
    pub city: Option<String>, pub min_rent: Option<i64>, pub max_rent: Option<i64>,
    pub min_bedrooms: Option<i32>, pub page: Option<i64>, pub per_page: Option<i64>,
}

pub async fn create_listing(State(state): State<AppState>, auth: AuthenticatedUser, Json(body): Json<CreateListingRequest>) -> Result<(axum::http::StatusCode, Json<Listing>), ApiError> {
    let new = NewListing {
        landlord_id: auth.user_id, title: body.title, description: body.description, address: body.address,
        city: body.city, country: body.country, latitude: body.latitude, longitude: body.longitude,
        rent_amount: body.rent_amount, deposit_amount: body.deposit_amount, currency: body.currency,
        bedrooms: body.bedrooms, bathrooms: body.bathrooms, area_sqm: body.area_sqm,
        amenities: body.amenities, photos: body.photos, required_credentials: body.required_credentials,
    };
    let listing = pliq_back_db::queries::listings::create(&state.db, &new).await?;
    Ok((axum::http::StatusCode::CREATED, Json(listing)))
}

pub async fn list_listings(State(state): State<AppState>, Query(params): Query<ListingFilterParams>) -> Result<Json<PaginatedResult<Listing>>, ApiError> {
    let filters = ListingFilters { city: params.city, min_rent: params.min_rent, max_rent: params.max_rent, min_bedrooms: params.min_bedrooms, ..Default::default() };
    let pagination = pliq_back_db::models::Pagination::new(params.page.unwrap_or(1), params.per_page.unwrap_or(20));
    let result = pliq_back_db::queries::listings::list_with_filters(&state.db, &filters, &pagination).await?;
    Ok(Json(result))
}

pub async fn get_listing(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<Listing>, ApiError> {
    let listing = pliq_back_db::queries::listings::get_by_id(&state.db, id).await?.ok_or_else(|| ApiError::NotFound("Listing not found".into()))?;
    Ok(Json(listing))
}

pub async fn update_listing(State(state): State<AppState>, auth: AuthenticatedUser, Path(id): Path<Uuid>, Json(body): Json<serde_json::Value>) -> Result<Json<Listing>, ApiError> {
    let existing = pliq_back_db::queries::listings::get_by_id(&state.db, id).await?.ok_or_else(|| ApiError::NotFound("Listing not found".into()))?;
    if existing.landlord_id != auth.user_id { return Err(ApiError::Forbidden("Not the listing owner".into())); }
    let updates = UpdateListing {
        title: body.get("title").and_then(|v| v.as_str()).map(String::from),
        description: body.get("description").and_then(|v| v.as_str()).map(String::from),
        rent_amount: body.get("rent_amount").and_then(|v| v.as_i64()),
        deposit_amount: body.get("deposit_amount").and_then(|v| v.as_i64()),
        amenities: body.get("amenities").cloned(), photos: body.get("photos").cloned(),
        required_credentials: body.get("required_credentials").cloned(),
    };
    let listing = pliq_back_db::queries::listings::update(&state.db, id, &updates).await?;
    Ok(Json(listing))
}

pub async fn delete_listing(State(state): State<AppState>, auth: AuthenticatedUser, Path(id): Path<Uuid>) -> Result<Json<Listing>, ApiError> {
    let existing = pliq_back_db::queries::listings::get_by_id(&state.db, id).await?.ok_or_else(|| ApiError::NotFound("Listing not found".into()))?;
    if existing.landlord_id != auth.user_id { return Err(ApiError::Forbidden("Not the listing owner".into())); }
    let listing = pliq_back_db::queries::listings::update_status(&state.db, id, pliq_back_db::models::ListingStatus::Archived).await?;
    Ok(Json(listing))
}
