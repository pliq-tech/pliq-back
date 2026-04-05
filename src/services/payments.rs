use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::crypto::merkle::MerkleTree;
use crate::domain::errors::DomainError;
use crate::domain::payment::{NewPayment, Payment, PaymentStatus};
use pliq_back_db::models::pagination::Pagination;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentReceipt {
    pub payment_id: Uuid,
    pub amount: i64,
    pub status: PaymentStatus,
    pub tx_hash: Option<String>,
    pub merkle_leaf_index: Option<i64>,
    pub merkle_proof: Option<Vec<MerkleProofNode>>,
    pub merkle_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProofNode {
    pub is_left: bool,
    pub hash: String,
}

/// Initiate a new payment with idempotency check.
pub async fn initiate_payment(
    pool: &PgPool,
    payer_id: Uuid,
    input: NewPayment,
) -> Result<Payment, DomainError> {
    validate_new_payment(&input)?;

    // Check idempotency: if a payment with this key exists, return it
    if let Some(existing) =
        pliq_back_db::queries::payments::get_by_idempotency_key(pool, &input.idempotency_key).await?
    {
        return Ok(existing);
    }

    let new_payment = NewPayment {
        payer_id,
        ..input
    };

    let payment = pliq_back_db::queries::payments::create(pool, &new_payment).await?;
    Ok(payment)
}

/// Process a pending payment (stub for Circle/Unlink integration).
pub async fn process_payment(
    pool: &PgPool,
    payment_id: Uuid,
) -> Result<Payment, DomainError> {
    let payment = get_payment(pool, payment_id).await?;

    if payment.status != PaymentStatus::Pending {
        return Err(DomainError::Validation(format!(
            "Cannot process payment in {:?} status",
            payment.status
        )));
    }

    // Transition to Processing
    pliq_back_db::queries::payments::update_status(
        pool,
        payment_id,
        PaymentStatus::Processing,
        None,
    )
    .await?;

    // TODO: Integrate with Circle/Unlink for actual payment processing.
    // For now, immediately confirm the payment.
    let confirmed = pliq_back_db::queries::payments::update_status(
        pool,
        payment_id,
        PaymentStatus::Confirmed,
        None,
    )
    .await?;

    Ok(confirmed)
}

/// Retrieve a single payment by ID.
pub async fn get_payment(pool: &PgPool, id: Uuid) -> Result<Payment, DomainError> {
    pliq_back_db::queries::payments::get_by_id(pool, id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("Payment {id} not found")))
}

/// List payments associated with a lease.
pub async fn list_payments_by_lease(
    pool: &PgPool,
    lease_id: Uuid,
    pagination: &Pagination,
) -> Result<Vec<Payment>, DomainError> {
    let payments = pliq_back_db::queries::payments::list_by_lease(pool, lease_id, pagination).await?;
    Ok(payments)
}

/// List payments associated with a user (as payer or payee).
pub async fn list_payments_by_user(
    pool: &PgPool,
    user_id: Uuid,
    pagination: &Pagination,
) -> Result<Vec<Payment>, DomainError> {
    let payments = pliq_back_db::queries::payments::list_by_user(pool, user_id, pagination).await?;
    Ok(payments)
}

/// Get a payment receipt including Merkle proof if the payment has been
/// recorded in the tree.
pub async fn get_payment_receipt(
    pool: &PgPool,
    payment_id: Uuid,
    merkle_tree: &tokio::sync::RwLock<MerkleTree>,
) -> Result<PaymentReceipt, DomainError> {
    let payment = get_payment(pool, payment_id).await?;

    let (merkle_proof, merkle_root) = match payment.merkle_leaf_index {
        Some(index) => build_proof_from_tree(merkle_tree, index as usize).await,
        None => (None, None),
    };

    Ok(PaymentReceipt {
        payment_id: payment.id,
        amount: payment.amount,
        status: payment.status,
        tx_hash: payment.tx_hash,
        merkle_leaf_index: payment.merkle_leaf_index,
        merkle_proof,
        merkle_root,
    })
}

async fn build_proof_from_tree(
    merkle_tree: &tokio::sync::RwLock<MerkleTree>,
    leaf_index: usize,
) -> (Option<Vec<MerkleProofNode>>, Option<String>) {
    let tree = merkle_tree.read().await;

    let proof = tree.proof(leaf_index).map(|nodes| {
        nodes
            .into_iter()
            .map(|(is_left, hash)| MerkleProofNode {
                is_left,
                hash: hex::encode(hash),
            })
            .collect()
    });

    let root = tree.root_hex();
    (proof, root)
}

fn validate_new_payment(input: &NewPayment) -> Result<(), DomainError> {
    if input.amount <= 0 {
        return Err(DomainError::Validation(
            "Payment amount must be greater than zero".into(),
        ));
    }
    if input.idempotency_key.trim().is_empty() {
        return Err(DomainError::Validation(
            "Idempotency key must not be empty".into(),
        ));
    }
    Ok(())
}
