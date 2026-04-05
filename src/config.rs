#[derive(Debug, Clone)]
pub struct Config {
    // Database
    pub database_url: String,

    // HTTP server
    pub http_host: String,
    pub http_port: u16,

    // gRPC
    pub grpc_port: u16,
    pub grpc_ai_host: String,

    // Auth
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,

    // World ID
    pub world_id_app_id: String,
    pub world_id_action: String,
    pub world_id_api_url: String,

    // CORS
    pub cors_origins: String,

    // Rate limiting
    pub rate_limit_rps: u32,

    // Logging
    pub log_level: String,
    pub log_format: String,

    // Unlink
    pub unlink_api_key: Option<String>,
    pub unlink_engine_url: String,

    // Circle
    pub circle_api_key: Option<String>,

    // WebSocket
    pub ws_heartbeat_interval_secs: u64,
    pub ws_max_message_size: usize,

    // WebRTC
    pub stun_server: String,

    // Platform
    pub platform_fee_bps: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let required = load_required_vars()?;
        Ok(Self {
            database_url: required.database_url,
            grpc_ai_host: required.grpc_ai_host,
            jwt_secret: required.jwt_secret,
            world_id_app_id: required.world_id_app_id,
            ..Self::load_optional_vars()
        })
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.http_host, self.http_port)
    }

    fn load_optional_vars() -> Self {
        Self {
            database_url: String::new(),
            grpc_ai_host: String::new(),
            jwt_secret: String::new(),
            world_id_app_id: String::new(),
            http_host: env_or("HTTP_HOST", "0.0.0.0"),
            http_port: parse_env_or("HTTP_PORT", 3001),
            grpc_port: parse_env_or("GRPC_PORT", 50051),
            jwt_expiry_hours: parse_env_or("JWT_EXPIRY_HOURS", 24),
            world_id_action: env_or("WORLD_ID_ACTION", "pliq-verify"),
            world_id_api_url: env_or("WORLD_ID_API_URL", "https://developer.worldcoin.org/api/v2/verify"),
            cors_origins: env_or("CORS_ORIGINS", "http://localhost:3000"),
            rate_limit_rps: parse_env_or("RATE_LIMIT_RPS", 100),
            log_level: env_or("LOG_LEVEL", "info"),
            log_format: env_or("LOG_FORMAT", "pretty"),
            unlink_api_key: optional_env("UNLINK_API_KEY"),
            unlink_engine_url: env_or("UNLINK_ENGINE_URL", "https://staging-api.unlink.xyz"),
            circle_api_key: optional_env("CIRCLE_API_KEY"),
            ws_heartbeat_interval_secs: parse_env_or("WS_HEARTBEAT_INTERVAL_SECS", 30),
            ws_max_message_size: parse_env_or("WS_MAX_MESSAGE_SIZE", 65536),
            stun_server: env_or("STUN_SERVER", "stun:stun.l.google.com:19302"),
            platform_fee_bps: parse_env_or("PLATFORM_FEE_BPS", 100),
        }
    }
}

struct RequiredVars {
    database_url: String,
    grpc_ai_host: String,
    jwt_secret: String,
    world_id_app_id: String,
}

fn load_required_vars() -> Result<RequiredVars, String> {
    let mut missing = Vec::new();

    let database_url = require_env("DATABASE_URL", &mut missing);
    let grpc_ai_host = require_env("GRPC_AI_HOST", &mut missing);
    let jwt_secret = require_env("JWT_SECRET", &mut missing);
    let world_id_app_id = require_env("WORLD_ID_APP_ID", &mut missing);

    if !missing.is_empty() {
        return Err(format!(
            "Missing required environment variables: {}",
            missing.join(", ")
        ));
    }

    Ok(RequiredVars { database_url, grpc_ai_host, jwt_secret, world_id_app_id })
}

fn require_env(key: &'static str, missing: &mut Vec<&'static str>) -> String {
    std::env::var(key).unwrap_or_else(|_| {
        missing.push(key);
        String::new()
    })
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn optional_env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

fn parse_env_or<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;
