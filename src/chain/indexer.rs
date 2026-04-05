use std::time::Duration;

use sqlx::PgPool;

use crate::websocket::manager::WsManager;

use super::events::ChainEvent;

/// Number of confirmations before an event is considered final.
const CONFIRMATION_DEPTH: u64 = 12;

/// Base poll interval between block fetches.
const POLL_INTERVAL: Duration = Duration::from_secs(2);

/// Maximum backoff delay on RPC failures.
const MAX_BACKOFF: Duration = Duration::from_secs(60);

/// Contract addresses the indexer monitors.
#[derive(Debug, Clone)]
pub struct ContractAddresses {
    pub registry: String,
    pub escrow: String,
    pub agreement: String,
}

/// Indexes on-chain events from pliq smart contracts and syncs them
/// into the database while pushing WebSocket notifications.
#[derive(Clone)]
pub struct ChainIndexer {
    rpc_url: String,
    db: PgPool,
    ws_manager: WsManager,
    contracts: ContractAddresses,
}

impl ChainIndexer {
    pub fn new(
        rpc_url: String,
        db: PgPool,
        ws_manager: WsManager,
        contracts: ContractAddresses,
    ) -> Self {
        Self {
            rpc_url,
            db,
            ws_manager,
            contracts,
        }
    }

    /// Main loop: poll blocks, filter events, process, update cursor.
    ///
    /// Runs indefinitely until the task is cancelled.
    pub async fn run(&self) {
        let mut consecutive_failures: u32 = 0;

        loop {
            match self.poll_and_process().await {
                Ok(()) => {
                    consecutive_failures = 0;
                    tokio::time::sleep(POLL_INTERVAL).await;
                }
                Err(e) => {
                    consecutive_failures += 1;
                    let backoff = calculate_backoff(consecutive_failures);
                    tracing::warn!(
                        error = %e,
                        retry_in = ?backoff,
                        "Chain indexer RPC failure"
                    );
                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }

    /// Single poll cycle: read cursor, fetch new blocks, process events.
    async fn poll_and_process(&self) -> Result<(), IndexerError> {
        let cursor = self.read_block_cursor().await?;
        let latest = self.fetch_latest_block().await?;

        let safe_head = latest.saturating_sub(CONFIRMATION_DEPTH);
        if cursor >= safe_head {
            return Ok(());
        }

        let events = self.fetch_events(cursor + 1, safe_head).await?;

        for event in &events {
            self.process_event(event).await?;
        }

        self.update_block_cursor(safe_head).await?;

        tracing::debug!(
            from = cursor + 1,
            to = safe_head,
            count = events.len(),
            "Indexed chain events"
        );
        Ok(())
    }

    async fn read_block_cursor(&self) -> Result<u64, IndexerError> {
        // TODO: Read last processed block from DB
        // For now return 0 as scaffold
        let _pool = &self.db;
        Ok(0)
    }

    async fn fetch_latest_block(&self) -> Result<u64, IndexerError> {
        // TODO: Implement actual alloy RPC call to get latest block
        // let provider = ProviderBuilder::new().on_http(self.rpc_url.parse()?);
        // let block = provider.get_block_number().await?;
        let _rpc = &self.rpc_url;
        Ok(0)
    }

    async fn fetch_events(
        &self,
        _from_block: u64,
        _to_block: u64,
    ) -> Result<Vec<ChainEvent>, IndexerError> {
        // TODO: Implement actual alloy event filtering
        // Build log filters for each contract address, decode logs
        // into ChainEvent variants using the sol! macro types.
        let _contracts = &self.contracts;
        Ok(Vec::new())
    }

    async fn process_event(
        &self,
        event: &ChainEvent,
    ) -> Result<(), IndexerError> {
        match event {
            ChainEvent::UserRegistered { .. } => {
                self.handle_user_registered(event).await
            }
            ChainEvent::ApplicationSubmitted { .. } => {
                self.handle_application_submitted(event).await
            }
            ChainEvent::AgreementSigned { .. } => {
                self.handle_agreement_signed(event).await
            }
            ChainEvent::EscrowFunded { .. } => {
                self.handle_escrow_funded(event).await
            }
            ChainEvent::PaymentExecuted { .. } => {
                self.handle_payment_executed(event).await
            }
            ChainEvent::MerkleRootUpdated { .. } => {
                self.handle_merkle_root_updated(event).await
            }
        }
    }

    async fn handle_user_registered(
        &self,
        _event: &ChainEvent,
    ) -> Result<(), IndexerError> {
        // TODO: Update user verification status in DB
        // TODO: Push WS notification to the user
        let _db = &self.db;
        let _ws = &self.ws_manager;
        Ok(())
    }

    async fn handle_application_submitted(
        &self,
        _event: &ChainEvent,
    ) -> Result<(), IndexerError> {
        // TODO: Create application record in DB
        // TODO: Push WS notification to listing owner
        Ok(())
    }

    async fn handle_agreement_signed(
        &self,
        _event: &ChainEvent,
    ) -> Result<(), IndexerError> {
        // TODO: Update agreement status in DB
        // TODO: Push WS notification to counterparty
        Ok(())
    }

    async fn handle_escrow_funded(
        &self,
        _event: &ChainEvent,
    ) -> Result<(), IndexerError> {
        // TODO: Update escrow balance in DB
        // TODO: Push WS notification to landlord
        Ok(())
    }

    async fn handle_payment_executed(
        &self,
        _event: &ChainEvent,
    ) -> Result<(), IndexerError> {
        // TODO: Record payment in DB
        // TODO: Push WS notification to payer and recipient
        Ok(())
    }

    async fn handle_merkle_root_updated(
        &self,
        _event: &ChainEvent,
    ) -> Result<(), IndexerError> {
        // TODO: Update the in-memory Merkle tree root
        Ok(())
    }

    async fn update_block_cursor(
        &self,
        _block: u64,
    ) -> Result<(), IndexerError> {
        // TODO: Persist block cursor to DB
        let _pool = &self.db;
        Ok(())
    }
}

pub(crate) fn calculate_backoff(failures: u32) -> Duration {
    let secs = POLL_INTERVAL.as_secs() * 2u64.pow(failures.min(5));
    Duration::from_secs(secs).min(MAX_BACKOFF)
}

/// Errors that can occur during chain indexing.
#[derive(Debug, thiserror::Error)]
pub enum IndexerError {
    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Event decoding error: {0}")]
    Decode(String),
}
