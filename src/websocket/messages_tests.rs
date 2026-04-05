use super::messages::{WsMessage, WsMessageType};

#[test]
fn test_connected_message_type() {
    let msg = WsMessage::connected();
    assert_eq!(msg.msg_type, WsMessageType::Connected);
    assert!(!msg.timestamp.is_empty());
}

#[test]
fn test_ping_pong_type() {
    let ping = WsMessage::ping();
    let pong = WsMessage::pong();
    assert_eq!(ping.msg_type, WsMessageType::Ping);
    assert_eq!(pong.msg_type, WsMessageType::Pong);
}

#[test]
fn test_error_message_has_payload() {
    let msg = WsMessage::error("something went wrong");
    assert_eq!(msg.msg_type, WsMessageType::Error);
    assert_eq!(msg.payload["message"], "something went wrong");
}

#[test]
fn test_message_serialization_roundtrip() {
    let msg = WsMessage::connected();
    let json = serde_json::to_string(&msg).expect("serialize failed");
    let parsed: WsMessage = serde_json::from_str(&json).expect("deserialize failed");
    assert_eq!(parsed.msg_type, WsMessageType::Connected);
}

#[test]
fn test_message_type_snake_case() {
    let json = serde_json::to_string(&WsMessageType::StatusUpdate).unwrap();
    assert_eq!(json, "\"status_update\"");
}
