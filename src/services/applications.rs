use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::application::{Application, ApplicationStatus, NewApplication};
use crate::domain::errors::DomainError;
use pliq_back_db::models::pagination::Pagination;

/// Submit a new rental application. Fails if the tenant already has an active one.
pub async fn submit_application(
    pool: &PgPool,
    tenant_id: Uuid,
    listing_id: Uuid,
    input: NewApplication,
) -> Result<Application, DomainError> {
    let has_active = pliq_back_db::queries::applications::has_active_application(
        pool, tenant_id, listing_id,
    )
    .await?;

    if has_active {
        return Err(DomainError::Conflict(
            "You already have an active application for this listing".into(),
        ));
    }

    let new_app = NewApplication {
        listing_id,
        tenant_id,
        ..input
    };

    let application = pliq_back_db::queries::applications::create(pool, &new_app).await?;
    Ok(application)
}

/// List all applications submitted by a tenant.
pub async fn list_applications_for_tenant(
    pool: &PgPool,
    tenant_id: Uuid,
    pagination: &Pagination,
) -> Result<Vec<Application>, DomainError> {
    let apps = pliq_back_db::queries::applications::list_by_tenant(pool, tenant_id, pagination).await?;
    Ok(apps)
}

/// List all applications for a specific listing.
pub async fn list_applications_for_listing(
    pool: &PgPool,
    listing_id: Uuid,
    pagination: &Pagination,
) -> Result<Vec<Application>, DomainError> {
    let apps = pliq_back_db::queries::applications::list_by_listing(pool, listing_id, pagination).await?;
    Ok(apps)
}

/// Get a single application by ID.
pub async fn get_application(pool: &PgPool, id: Uuid) -> Result<Application, DomainError> {
    pliq_back_db::queries::applications::get_by_id(pool, id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("Application {id} not found")))
}

/// Update the status of an application (accept, reject, etc.).
pub async fn update_status(
    pool: &PgPool,
    id: Uuid,
    status: ApplicationStatus,
) -> Result<Application, DomainError> {
    let _existing = get_application(pool, id).await?;
    let updated = pliq_back_db::queries::applications::update_status(pool, id, status).await?;
    Ok(updated)
}

/// Withdraw an application. Only the owning tenant may withdraw.
pub async fn withdraw_application(
    pool: &PgPool,
    id: Uuid,
    tenant_id: Uuid,
) -> Result<Application, DomainError> {
    let app = get_application(pool, id).await?;

    if app.tenant_id != tenant_id {
        return Err(DomainError::Unauthorized(
            "You do not own this application".into(),
        ));
    }

    if app.status == ApplicationStatus::Withdrawn {
        return Err(DomainError::Validation(
            "Application is already withdrawn".into(),
        ));
    }

    let updated = pliq_back_db::queries::applications::update_status(
        pool,
        id,
        ApplicationStatus::Withdrawn,
    )
    .await?;
    Ok(updated)
}
