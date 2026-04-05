use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::Response;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::middleware::auth::Claims;
use crate::AppState;
use super::messages::WsMessage;

#[derive(Deserialize)]
pub struct WsQuery {
    pub token: String,
}

pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> Response {
    let user_id = match validate_ws_token(&query.token, &state.config.jwt_secret) {
        Some(id) => id,
        None => {
            return Response::builder()
                .status(401)
                .body("Unauthorized".into())
                .unwrap();
        }
    };
    ws.on_upgrade(move |socket| handle_socket(socket, user_id, state))
}

fn validate_ws_token(token: &str, secret: &str) -> Option<Uuid> {
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .ok()?;
    Some(token_data.claims.sub)
}

async fn handle_socket(socket: WebSocket, user_id: Uuid, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Send connected message
    let connected = serde_json::to_string(&WsMessage::connected()).unwrap_or_default();
    if sender.send(Message::Text(connected.into())).await.is_err() {
        return;
    }

    // Subscribe to broadcast events for this user
    let mut event_rx = state.ws_manager.subscribe(user_id);

    let heartbeat_secs = state.config.ws_heartbeat_interval_secs;

    // Main loop: relay broadcast events and handle incoming messages
    loop {
        tokio::select! {
            // Broadcast events from the server
            event = event_rx.recv() => {
                match event {
                    Ok(ws_event) => {
                        let msg = WsMessage::from_event(&ws_event);
                        let json = serde_json::to_string(&msg).unwrap_or_default();
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            // Incoming messages from client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        handle_incoming(&text, user_id, &state).await;
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            // Heartbeat ping
            _ = tokio::time::sleep(std::time::Duration::from_secs(heartbeat_secs)) => {
                let ping = serde_json::to_string(&WsMessage::ping()).unwrap_or_default();
                if sender.send(Message::Text(ping.into())).await.is_err() {
                    break;
                }
            }
        }
    }

    state.ws_manager.remove_user(&user_id);
    tracing::debug!("WebSocket disconnected: {user_id}");
}

async fn handle_incoming(text: &str, _user_id: Uuid, _state: &AppState) {
    if let Ok(msg) = serde_json::from_str::<WsMessage>(text) {
        match msg.msg_type {
            super::messages::WsMessageType::Pong => {}
            super::messages::WsMessageType::Chat => {
                // TODO: relay to room members
            }
            _ => {
                tracing::debug!("Unhandled WS message type: {:?}", msg.msg_type);
            }
        }
    }
}
