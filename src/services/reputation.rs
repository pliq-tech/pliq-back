use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::crypto::merkle::MerkleTree;
use crate::domain::errors::DomainError;
use crate::domain::payment::Payment;
use crate::domain::reputation::ReputationScore;
use pliq_back_db::models::merkle::NewMerkleLeaf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProofResponse {
    pub leaf_index: usize,
    pub leaf_hash: String,
    pub proof: Vec<ProofNode>,
    pub root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofNode {
    pub is_left: bool,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleRootResponse {
    pub root: Option<String>,
    pub leaf_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoRCredential {
    pub user_id: Uuid,
    pub score: i32,
    pub payment_count: i32,
    pub on_time_count: i32,
    pub total_paid: i64,
    pub merkle_root: Option<String>,
    pub issued_at: chrono::DateTime<Utc>,
}

/// Get or create a reputation score for a user.
pub async fn get_reputation(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<ReputationScore, DomainError> {
    let score = pliq_back_db::queries::reputation::upsert(pool, user_id).await?;
    Ok(score)
}

/// Record a confirmed payment in the Merkle tree and update reputation.
pub async fn record_payment_in_tree(
    pool: &PgPool,
    merkle_tree: &tokio::sync::RwLock<MerkleTree>,
    payment: &Payment,
) -> Result<usize, DomainError> {
    let leaf_data = build_leaf_data(payment);

    // Get current leaf count before insertion to determine the new leaf's index
    let leaf_count_before = pliq_back_db::queries::merkle::get_leaf_count(pool).await? as usize;

    // Insert into the in-memory Merkle tree
    {
        let mut tree = merkle_tree.write().await;
        tree.insert_leaf(&leaf_data);
    }
    let leaf_index = leaf_count_before;

    // Persist leaf to DB
    let new_leaf = NewMerkleLeaf {
        tenant_id: payment.payer_id,
        payment_id: Some(payment.id),
        leaf_hash: hex::encode(hash_leaf_data(&leaf_data)),
        amount: payment.amount,
        payment_date: payment.paid_at.unwrap_or_else(Utc::now),
    };
    pliq_back_db::queries::merkle::insert_leaf(pool, &new_leaf).await?;

    // Update tree metadata
    let root_hex = {
        let tree = merkle_tree.read().await;
        tree.root_hex().unwrap_or_default()
    };
    let leaf_count = pliq_back_db::queries::merkle::get_leaf_count(pool).await?;
    pliq_back_db::queries::merkle::update_tree_metadata(pool, &root_hex, leaf_count).await?;

    // Set merkle leaf index on the payment record
    pliq_back_db::queries::payments::set_merkle_leaf(pool, payment.id, leaf_index as i64).await?;

    // Increment user reputation
    let on_time = payment.due_date.is_none()
        || payment
            .paid_at
            .map(|paid| paid.date_naive() <= payment.due_date.unwrap())
            .unwrap_or(true);
    pliq_back_db::queries::reputation::increment_payment(
        pool,
        payment.payer_id,
        payment.amount,
        on_time,
    )
    .await?;

    // Update user reputation merkle root
    pliq_back_db::queries::reputation::update_merkle_root(
        pool,
        payment.payer_id,
        &root_hex,
    )
    .await?;

    Ok(leaf_index)
}

/// Generate a Merkle proof for a specific leaf index.
/// Looks up the leaf hash from the DB, then builds the proof from the in-memory tree.
pub async fn get_merkle_proof(
    pool: &PgPool,
    merkle_tree: &tokio::sync::RwLock<MerkleTree>,
    leaf_index: usize,
) -> Result<MerkleProofResponse, DomainError> {
    let tree = merkle_tree.read().await;

    let proof_nodes = tree.proof(leaf_index).ok_or_else(|| {
        DomainError::NotFound(format!("Leaf index {leaf_index} not found in tree"))
    })?;

    let root = tree
        .root_hex()
        .ok_or_else(|| DomainError::Internal("Merkle tree has no root".into()))?;

    // Look up the leaf hash from the DB since the tree's leaves field is private
    let leaf_row = pliq_back_db::queries::merkle::get_leaf_by_index(pool, leaf_index as i64)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("Leaf at index {leaf_index} not found")))?;

    let proof = proof_nodes
        .into_iter()
        .map(|(is_left, hash)| ProofNode {
            is_left,
            hash: hex::encode(hash),
        })
        .collect();

    Ok(MerkleProofResponse {
        leaf_index,
        leaf_hash: leaf_row.leaf_hash,
        proof,
        root,
    })
}

/// Get the current Merkle root and leaf count.
pub async fn get_merkle_root(
    pool: &PgPool,
    merkle_tree: &tokio::sync::RwLock<MerkleTree>,
) -> Result<MerkleRootResponse, DomainError> {
    let tree = merkle_tree.read().await;
    let leaf_count = pliq_back_db::queries::merkle::get_leaf_count(pool).await? as usize;
    Ok(MerkleRootResponse {
        root: tree.root_hex(),
        leaf_count,
    })
}

/// Verify a Merkle proof against a root and leaf hash.
pub fn verify_proof(
    root: &[u8; 32],
    leaf_hash: &[u8; 32],
    proof: &crate::crypto::merkle::MerkleProof,
) -> bool {
    MerkleTree::verify(root, leaf_hash, proof)
}

/// Export a portable Proof-of-Rent credential for a user.
pub async fn export_credential(
    pool: &PgPool,
    merkle_tree: &tokio::sync::RwLock<MerkleTree>,
    user_id: Uuid,
) -> Result<PoRCredential, DomainError> {
    let rep = get_reputation(pool, user_id).await?;

    let merkle_root = {
        let tree = merkle_tree.read().await;
        tree.root_hex()
    };

    Ok(PoRCredential {
        user_id,
        score: rep.score,
        payment_count: rep.payment_count,
        on_time_count: rep.on_time_count,
        total_paid: rep.total_paid,
        merkle_root,
        issued_at: Utc::now(),
    })
}

fn build_leaf_data(payment: &Payment) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(payment.id.as_bytes());
    data.extend_from_slice(payment.payer_id.as_bytes());
    data.extend_from_slice(&payment.amount.to_le_bytes());
    data.extend_from_slice(payment.lease_id.as_bytes());
    data
}

fn hash_leaf_data(data: &[u8]) -> [u8; 32] {
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(b"\x00");
    hasher.update(data);
    hasher.finalize().into()
}
