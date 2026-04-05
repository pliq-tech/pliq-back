use super::auth::{encode_jwt, Claims};
use chrono;
use jsonwebtoken::{self, decode, DecodingKey, Validation};
use uuid::Uuid;

#[test]
fn test_encode_decode_roundtrip() {
    let user_id = Uuid::new_v4();
    let secret = "test-secret";

    let token = encode_jwt(user_id, "nullifier", "tenant", "orb", secret, 24)
        .expect("encode failed");

    let data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .expect("decode failed");

    assert_eq!(data.claims.sub, user_id);
    assert_eq!(data.claims.role, "tenant");
    assert_eq!(data.claims.verification_level, "orb");
}

#[test]
fn test_expired_token_rejected() {
    // Manually create a token with exp in the past
    let now = chrono::Utc::now().timestamp();
    let claims = Claims {
        sub: Uuid::new_v4(),
        nullifier_hash: "null".to_string(),
        role: "tenant".to_string(),
        verification_level: "orb".to_string(),
        exp: now - 3600, // 1 hour ago
        iat: now - 7200,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(b"secret"),
    )
    .expect("encode failed");

    let result = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(b"secret"),
        &Validation::default(),
    );
    assert!(result.is_err());
}

#[test]
fn test_invalid_secret_rejected() {
    let token = encode_jwt(Uuid::new_v4(), "null", "tenant", "orb", "secret1", 24)
        .expect("encode failed");

    let result = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(b"wrong-secret"),
        &Validation::default(),
    );
    assert!(result.is_err());
}
