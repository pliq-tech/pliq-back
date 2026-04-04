use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<pliq_back_db::DbError> for DomainError {
    fn from(err: pliq_back_db::DbError) -> Self {
        match err {
            pliq_back_db::DbError::NotFound => DomainError::NotFound("Resource not found".into()),
            pliq_back_db::DbError::DuplicateNullifier => DomainError::Conflict("User already registered".into()),
            pliq_back_db::DbError::DuplicateApplication => DomainError::Conflict("Active application already exists".into()),
            pliq_back_db::DbError::DuplicateIdempotencyKey => DomainError::Conflict("Payment already processed".into()),
            pliq_back_db::DbError::InvalidStatusTransition { from, to } => {
                DomainError::Validation(format!("Invalid status transition from {from} to {to}"))
            }
            other => DomainError::Internal(other.to_string()),
        }
    }
}
