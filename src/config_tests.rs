use super::*;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn set_required_vars() {
    unsafe {
        std::env::set_var("DATABASE_URL", "postgres://test:test@localhost/pliq");
        std::env::set_var("GRPC_AI_HOST", "http://localhost:50052");
        std::env::set_var("JWT_SECRET", "test-jwt-secret-key");
        std::env::set_var("WORLD_ID_APP_ID", "app_test_123");
    }
}

fn clear_required_vars() {
    for key in ["DATABASE_URL", "GRPC_AI_HOST", "JWT_SECRET", "WORLD_ID_APP_ID"] {
        unsafe { std::env::remove_var(key); }
    }
}

#[test]
fn test_missing_required_vars() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_required_vars();

    let result = Config::from_env();
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(msg.contains("DATABASE_URL"));
    assert!(msg.contains("JWT_SECRET"));

    set_required_vars();
}

#[test]
fn test_defaults_applied() {
    let _guard = ENV_LOCK.lock().unwrap();
    set_required_vars();

    let config = Config::from_env().expect("from_env failed");
    assert_eq!(config.http_host, "0.0.0.0");
    assert_eq!(config.http_port, 3001);
    assert_eq!(config.grpc_port, 50051);
    assert_eq!(config.log_level, "info");
}

#[test]
fn test_bind_address_format() {
    let _guard = ENV_LOCK.lock().unwrap();
    set_required_vars();

    let config = Config::from_env().expect("from_env failed");
    assert_eq!(config.bind_address(), "0.0.0.0:3001");
}
