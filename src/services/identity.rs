use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::config::Config;
use crate::domain::errors::DomainError;
use crate::domain::user::{NewUser, User, UserRole, VerificationLevel};

#[derive(Debug, Clone, Serialize)]
pub struct VerifyWorldIdRequest {
    pub merkle_root: String,
    pub nullifier_hash: String,
    pub proof: String,
    pub verification_level: String,
    pub signal: String,
    pub action: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorldIdApiResponse {
    pub success: bool,
    pub nullifier_hash: Option<String>,
    pub verification_level: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorldIdVerification {
    pub nullifier_hash: String,
    pub verification_level: VerificationLevel,
}

/// Verify a World ID proof against the cloud API.
pub async fn verify_world_id(
    client: &Client,
    config: &Config,
    request: &VerifyWorldIdRequest,
) -> Result<WorldIdVerification, DomainError> {
    let api_payload = serde_json::json!({
        "merkle_root": request.merkle_root,
        "nullifier_hash": request.nullifier_hash,
        "proof": request.proof,
        "verification_level": request.verification_level,
        "signal": request.signal,
        "action": config.world_id_action,
    });

    let response = client
        .post(&config.world_id_api_url)
        .header("Content-Type", "application/json")
        .json(&api_payload)
        .send()
        .await
        .map_err(|e| DomainError::Internal(format!("World ID request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(DomainError::Validation(format!(
            "World ID verification failed ({status}): {body}"
        )));
    }

    let api_response: WorldIdApiResponse = response
        .json()
        .await
        .map_err(|e| DomainError::Internal(format!("World ID response parse error: {e}")))?;

    if !api_response.success {
        return Err(DomainError::Validation(
            "World ID verification returned unsuccessful".into(),
        ));
    }

    let verification_level = parse_verification_level(
        api_response.verification_level.as_deref(),
    );

    Ok(WorldIdVerification {
        nullifier_hash: request.nullifier_hash.clone(),
        verification_level,
    })
}

/// Create a new user or return the existing one for a given nullifier hash.
pub async fn create_or_get_user(
    pool: &PgPool,
    nullifier_hash: &str,
    verification_level: VerificationLevel,
) -> Result<User, DomainError> {
    if let Some(user) = pliq_back_db::queries::users::get_by_nullifier(pool, nullifier_hash).await? {
        return Ok(user);
    }

    let new_user = NewUser {
        nullifier_hash: nullifier_hash.to_string(),
        wallet_address: None,
        display_name: None,
        role: UserRole::Tenant,
        verification_level,
    };

    let user = pliq_back_db::queries::users::create(pool, &new_user).await?;
    Ok(user)
}

fn parse_verification_level(level: Option<&str>) -> VerificationLevel {
    match level {
        Some("orb") => VerificationLevel::Orb,
        Some("device") => VerificationLevel::Device,
        Some("passport") => VerificationLevel::Passport,
        _ => VerificationLevel::Device,
    }
}
