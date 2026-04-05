use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::domain::errors::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlinkTxId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateTransferRequest {
    pub amount: u64,
    pub currency: String,
    pub recipient: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnlinkTxStatus {
    Pending,
    Confirmed,
    Failed,
}

/// Initiate a private deposit through the Unlink privacy engine.
///
/// TODO: Integrate with the Unlink API once available.
pub async fn private_deposit(
    _client: &Client,
    _config: &Config,
    _request: &PrivateTransferRequest,
) -> Result<UnlinkTxId, DomainError> {
    Err(DomainError::Internal(
        "Unlink private deposit not yet implemented".into(),
    ))
}

/// Initiate a private transfer through the Unlink privacy engine.
///
/// TODO: Integrate with the Unlink API once available.
pub async fn private_transfer(
    _client: &Client,
    _config: &Config,
    _request: &PrivateTransferRequest,
) -> Result<UnlinkTxId, DomainError> {
    Err(DomainError::Internal(
        "Unlink private transfer not yet implemented".into(),
    ))
}

/// Initiate a private withdrawal through the Unlink privacy engine.
///
/// TODO: Integrate with the Unlink API once available.
pub async fn private_withdraw(
    _client: &Client,
    _config: &Config,
    _request: &PrivateTransferRequest,
) -> Result<UnlinkTxId, DomainError> {
    Err(DomainError::Internal(
        "Unlink private withdrawal not yet implemented".into(),
    ))
}

/// Check the status of a previously submitted Unlink transaction.
///
/// TODO: Integrate with the Unlink API once available.
pub async fn get_tx_status(
    _client: &Client,
    _config: &Config,
    _tx_id: &str,
) -> Result<UnlinkTxStatus, DomainError> {
    Err(DomainError::Internal(
        "Unlink transaction status check not yet implemented".into(),
    ))
}
