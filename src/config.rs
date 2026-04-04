use std::collections::Vec;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub backend_host: String,
    pub backend_port: u16,
    pub grpc_ai_url: String,
    pub jwt_secret: String,
    pub world_id_app_id: String,
    pub world_id_action_id: String,
    pub world_id_api_url: String,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let mut missing = Vec::new();

        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            missing.push("DATABASE_URL");
            String::new()
        });
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            missing.push("JWT_SECRET");
            String::new()
        });

        if !missing.is_empty() {
            return Err(format!("Missing required environment variables: {}", missing.join(", ")));
        }

        Ok(Self {
            database_url,
            backend_host: std::env::var("BACKEND_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            backend_port: std::env::var("BACKEND_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            grpc_ai_url: std::env::var("GRPC_AI_URL").unwrap_or_else(|_| "http://localhost:50051".to_string()),
            jwt_secret,
            world_id_app_id: std::env::var("WORLD_ID_APP_ID").unwrap_or_default(),
            world_id_action_id: std::env::var("WORLD_ID_ACTION_ID").unwrap_or_default(),
            world_id_api_url: std::env::var("WORLD_ID_API_URL").unwrap_or_else(|_| "https://developer.worldcoin.org".to_string()),
            rust_log: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        })
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.backend_host, self.backend_port)
    }
}
