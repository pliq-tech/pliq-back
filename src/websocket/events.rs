use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsEvent {
    ApplicationReceived { application_id: Uuid, listing_id: Uuid, tenant_id: Uuid },
    ApplicationStatusChanged { application_id: Uuid, status: String },
    LeaseActivated { lease_id: Uuid },
    PaymentConfirmed { payment_id: Uuid, amount: i64 },
    ReputationUpdated { user_id: Uuid, new_score: i32 },
}
