use axum::extract::State;
use axum::Json;
use serde::Serialize;
use crate::api::errors::ApiError;
use crate::api::extractors::auth::AuthenticatedUser;
use crate::AppState;

#[derive(Serialize)]
pub struct WebrtcConfig {
    pub ice_servers: Vec<IceServer>,
}

#[derive(Serialize)]
pub struct IceServer {
    pub urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

pub async fn get_webrtc_config(
    State(state): State<AppState>,
    _auth: AuthenticatedUser,
) -> Result<Json<WebrtcConfig>, ApiError> {
    Ok(Json(WebrtcConfig {
        ice_servers: vec![IceServer {
            urls: vec![state.config.stun_server.clone()],
            username: None,
            credential: None,
        }],
    }))
}
