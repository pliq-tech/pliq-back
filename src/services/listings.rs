use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::errors::DomainError;
use crate::domain::listing::{
    Listing, ListingFilters, ListingStatus, NewListing, UpdateListing,
};
use pliq_back_db::models::pagination::{PaginatedResult, Pagination};

/// Create a new listing after validating required fields.
pub async fn create_listing(
    pool: &PgPool,
    landlord_id: Uuid,
    input: NewListing,
) -> Result<Listing, DomainError> {
    validate_new_listing(&input)?;

    let listing = NewListing {
        landlord_id,
        ..input
    };

    let created = pliq_back_db::queries::listings::create(pool, &listing).await?;
    Ok(created)
}

/// Retrieve a single listing by ID.
pub async fn get_listing(pool: &PgPool, id: Uuid) -> Result<Listing, DomainError> {
    pliq_back_db::queries::listings::get_by_id(pool, id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("Listing {id} not found")))
}

/// List listings with optional filters and pagination.
pub async fn list_listings(
    pool: &PgPool,
    filters: &ListingFilters,
    pagination: &Pagination,
) -> Result<PaginatedResult<Listing>, DomainError> {
    let result = pliq_back_db::queries::listings::list_with_filters(pool, filters, pagination).await?;
    Ok(result)
}

/// Update a listing. Only the owning landlord may update.
pub async fn update_listing(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    input: UpdateListing,
) -> Result<Listing, DomainError> {
    let listing = get_listing(pool, id).await?;
    verify_ownership(listing.landlord_id, user_id)?;

    let updated = pliq_back_db::queries::listings::update(pool, id, &input).await?;
    Ok(updated)
}

/// Soft-delete a listing by setting its status to Archived.
pub async fn delete_listing(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
) -> Result<(), DomainError> {
    let listing = get_listing(pool, id).await?;
    verify_ownership(listing.landlord_id, user_id)?;

    pliq_back_db::queries::listings::update_status(pool, id, ListingStatus::Archived).await?;
    Ok(())
}

fn validate_new_listing(input: &NewListing) -> Result<(), DomainError> {
    if input.title.trim().is_empty() {
        return Err(DomainError::Validation("Title must not be empty".into()));
    }
    if input.rent_amount <= 0 {
        return Err(DomainError::Validation(
            "Rent amount must be greater than zero".into(),
        ));
    }
    if input.deposit_amount < 0 {
        return Err(DomainError::Validation(
            "Deposit amount must not be negative".into(),
        ));
    }
    Ok(())
}

fn verify_ownership(owner_id: Uuid, user_id: Uuid) -> Result<(), DomainError> {
    if owner_id != user_id {
        return Err(DomainError::Unauthorized(
            "You do not own this listing".into(),
        ));
    }
    Ok(())
}
