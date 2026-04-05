use serde::{Deserialize, Serialize};
use super::events::WsEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    #[serde(rename = "type")]
    pub msg_type: WsMessageType,
    pub payload: serde_json::Value,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WsMessageType {
    Connected,
    Ping,
    Pong,
    Error,
    Notification,
    StatusUpdate,
    Chat,
    TypingIndicator,
    WebrtcOffer,
    WebrtcAnswer,
    WebrtcIceCandidate,
    WebrtcHangup,
}

impl WsMessage {
    pub fn connected() -> Self {
        Self {
            msg_type: WsMessageType::Connected,
            payload: serde_json::json!({}),
            timestamp: chrono::Utc::now().to_rfc3339(),
            room_id: None,
        }
    }

    pub fn ping() -> Self {
        Self {
            msg_type: WsMessageType::Ping,
            payload: serde_json::json!({}),
            timestamp: chrono::Utc::now().to_rfc3339(),
            room_id: None,
        }
    }

    pub fn pong() -> Self {
        Self {
            msg_type: WsMessageType::Pong,
            payload: serde_json::json!({}),
            timestamp: chrono::Utc::now().to_rfc3339(),
            room_id: None,
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            msg_type: WsMessageType::Error,
            payload: serde_json::json!({ "message": msg }),
            timestamp: chrono::Utc::now().to_rfc3339(),
            room_id: None,
        }
    }

    pub fn from_event(event: &WsEvent) -> Self {
        Self {
            msg_type: WsMessageType::StatusUpdate,
            payload: serde_json::to_value(event).unwrap_or_default(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            room_id: None,
        }
    }
}
