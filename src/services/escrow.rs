use sqlx::PgPool;
use uuid::Uuid;

use crate::crypto::commitments::{self, EscrowConditions};
use crate::domain::errors::DomainError;
use crate::domain::escrow::{EscrowCommitment, EscrowStatus, NewEscrowCommitment};
use crate::domain::listing::CurrencyType;

/// Input for creating a new escrow commitment.
#[derive(Debug, Clone)]
pub struct CreateEscrowInput {
    pub listing_id: Uuid,
    pub landlord_id: Uuid,
    pub amount: i64,
    pub currency: CurrencyType,
}

/// Create a new escrow commitment using cryptographic hash commitment.
/// Returns the persisted escrow record and the hex-encoded commitment hash.
pub async fn create_escrow(
    pool: &PgPool,
    tenant_id: Uuid,
    input: CreateEscrowInput,
) -> Result<(EscrowCommitment, String), DomainError> {
    if input.amount <= 0 {
        return Err(DomainError::Validation(
            "Escrow amount must be greater than zero".into(),
        ));
    }

    let conditions = EscrowConditions {
        amount: input.amount as u64,
        tenant_id: tenant_id.to_string(),
        landlord_id: input.landlord_id.to_string(),
        listing_id: input.listing_id.to_string(),
    };

    let crypto_commitment = commitments::create_commitment(&conditions);
    let commitment_hex = hex::encode(crypto_commitment.commitment_hash);
    let conditions_hex = hex::encode(conditions.hash());

    let new_escrow = NewEscrowCommitment {
        listing_id: input.listing_id,
        tenant_id,
        landlord_id: input.landlord_id,
        commitment_hash: commitment_hex.clone(),
        amount: input.amount,
        currency: input.currency,
        conditions_hash: Some(conditions_hex),
        secret: hex::encode(crypto_commitment.secret),
        nonce: hex::encode(crypto_commitment.nonce),
    };

    let escrow = pliq_back_db::queries::escrow::create(pool, &new_escrow).await?;
    Ok((escrow, commitment_hex))
}

/// Verify the escrow commitment by revealing the secret and nonce.
pub async fn reveal_escrow(
    pool: &PgPool,
    escrow_id: Uuid,
    secret: &[u8; 32],
    nonce: &[u8; 16],
) -> Result<EscrowCommitment, DomainError> {
    let escrow = get_escrow(pool, escrow_id).await?;

    if escrow.status != EscrowStatus::Committed {
        return Err(DomainError::Validation(format!(
            "Cannot reveal escrow in {:?} status",
            escrow.status
        )));
    }

    let commitment_bytes = parse_hex_32(&escrow.commitment_hash)?;

    let conditions = EscrowConditions {
        amount: escrow.amount as u64,
        tenant_id: escrow.tenant_id.to_string(),
        landlord_id: escrow.landlord_id.to_string(),
        listing_id: escrow.listing_id.to_string(),
    };

    let valid = commitments::verify_commitment(&commitment_bytes, &conditions, secret, nonce);
    if !valid {
        return Err(DomainError::Validation(
            "Commitment verification failed: secret/nonce do not match".into(),
        ));
    }

    let updated = pliq_back_db::queries::escrow::update_status(
        pool,
        escrow_id,
        EscrowStatus::Revealed,
    )
    .await?;
    Ok(updated)
}

/// Release an escrow deposit. Only the landlord may release.
pub async fn release_escrow(
    pool: &PgPool,
    escrow_id: Uuid,
    user_id: Uuid,
) -> Result<EscrowCommitment, DomainError> {
    let escrow = get_escrow(pool, escrow_id).await?;

    if escrow.landlord_id != user_id {
        return Err(DomainError::Unauthorized(
            "Only the landlord can release an escrow".into(),
        ));
    }

    if escrow.status != EscrowStatus::Active && escrow.status != EscrowStatus::Revealed {
        return Err(DomainError::Validation(format!(
            "Cannot release escrow in {:?} status",
            escrow.status
        )));
    }

    let updated = pliq_back_db::queries::escrow::update_status(
        pool,
        escrow_id,
        EscrowStatus::Released,
    )
    .await?;
    Ok(updated)
}

/// Dispute an escrow. Either party may file a dispute.
pub async fn dispute_escrow(
    pool: &PgPool,
    escrow_id: Uuid,
    user_id: Uuid,
) -> Result<EscrowCommitment, DomainError> {
    let escrow = get_escrow(pool, escrow_id).await?;

    if escrow.tenant_id != user_id && escrow.landlord_id != user_id {
        return Err(DomainError::Unauthorized(
            "You are not a party to this escrow".into(),
        ));
    }

    if escrow.status == EscrowStatus::Released || escrow.status == EscrowStatus::Refunded {
        return Err(DomainError::Validation(format!(
            "Cannot dispute escrow in {:?} status",
            escrow.status
        )));
    }

    let updated = pliq_back_db::queries::escrow::update_status(
        pool,
        escrow_id,
        EscrowStatus::Disputed,
    )
    .await?;
    Ok(updated)
}

/// Retrieve a single escrow commitment by ID.
pub async fn get_escrow(pool: &PgPool, id: Uuid) -> Result<EscrowCommitment, DomainError> {
    pliq_back_db::queries::escrow::get_by_id(pool, id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("Escrow {id} not found")))
}

fn parse_hex_32(hex_str: &str) -> Result<[u8; 32], DomainError> {
    let bytes = hex::decode(hex_str)
        .map_err(|e| DomainError::Internal(format!("Invalid hex: {e}")))?;
    bytes
        .try_into()
        .map_err(|_| DomainError::Internal("Invalid commitment hash length".into()))
}
