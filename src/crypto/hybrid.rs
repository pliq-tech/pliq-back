//! Hybrid key encapsulation: X25519 + AES-256-GCM with HKDF.
//!
//! For hackathon: implements the classical cryptography side.
//! ML-KEM-768 integration is stubbed pending crate availability
//! (would be feature-gated behind `pqc-hybrid`).

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce as AesNonce};
use hkdf::Hkdf;
use rand::RngCore;
use sha3::Sha3_256;
use thiserror::Error;
use x25519_dalek::{PublicKey, StaticSecret};

const DOMAIN_SEPARATOR: &[u8] = b"pliq-hybrid-kem-v1";
const AES_NONCE_LEN: usize = 12;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("HKDF expansion failed")]
    HkdfExpand,
    #[error("AES-GCM encryption failed")]
    EncryptionFailed,
    #[error("AES-GCM decryption failed")]
    DecryptionFailed,
}

/// Long-term keypair for hybrid encryption.
pub struct HybridKeypair {
    pub x25519_secret: StaticSecret,
    pub x25519_public: PublicKey,
    // ML-KEM fields would go here when pqcrypto-kyber is available
}

/// Public half shared with message senders.
pub struct HybridPublicKey {
    pub x25519_pk: [u8; 32],
    // ML-KEM public key would go here
}

/// Sealed (encrypted) message with ephemeral key material.
pub struct SealedMessage {
    pub x25519_ephemeral_pk: [u8; 32],
    pub aes_nonce: [u8; AES_NONCE_LEN],
    pub aes_ciphertext: Vec<u8>,
    // ML-KEM ciphertext would go here
}

/// Generate a new hybrid keypair.
pub fn generate_keypair() -> HybridKeypair {
    let secret_bytes = random_32_bytes();
    let secret = StaticSecret::from(secret_bytes);
    let public = PublicKey::from(&secret);
    HybridKeypair {
        x25519_secret: secret,
        x25519_public: public,
    }
}

/// Encrypt plaintext for a recipient using X25519 + AES-256-GCM.
pub fn seal(
    plaintext: &[u8],
    recipient_pk: &HybridPublicKey,
) -> Result<SealedMessage, CryptoError> {
    let eph_secret_bytes = random_32_bytes();
    let eph_secret = StaticSecret::from(eph_secret_bytes);
    let eph_public = PublicKey::from(&eph_secret);

    let recipient_x25519 = PublicKey::from(recipient_pk.x25519_pk);
    let shared_secret = eph_secret.diffie_hellman(&recipient_x25519);

    let aes_key = derive_shared_key(shared_secret.as_bytes())?;
    let nonce_bytes = random_nonce();
    let ciphertext = encrypt_aes256gcm(&aes_key, &nonce_bytes, plaintext)?;

    Ok(SealedMessage {
        x25519_ephemeral_pk: eph_public.to_bytes(),
        aes_nonce: nonce_bytes,
        aes_ciphertext: ciphertext,
    })
}

/// Decrypt a sealed message using the recipient's keypair.
pub fn open(
    sealed: &SealedMessage,
    sk: &HybridKeypair,
) -> Result<Vec<u8>, CryptoError> {
    let eph_pk = PublicKey::from(sealed.x25519_ephemeral_pk);
    let shared_secret = sk.x25519_secret.diffie_hellman(&eph_pk);

    let aes_key = derive_shared_key(shared_secret.as_bytes())?;
    decrypt_aes256gcm(&aes_key, &sealed.aes_nonce, &sealed.aes_ciphertext)
}

/// Derive a 32-byte AES key from the X25519 shared secret via HKDF-SHA3-256.
fn derive_shared_key(x25519_ss: &[u8]) -> Result<[u8; 32], CryptoError> {
    let hk = Hkdf::<Sha3_256>::new(Some(DOMAIN_SEPARATOR), x25519_ss);
    let mut okm = [0u8; 32];
    hk.expand(b"aes-256-gcm-key", &mut okm)
        .map_err(|_| CryptoError::HkdfExpand)?;
    Ok(okm)
}

/// Encrypt with AES-256-GCM.
fn encrypt_aes256gcm(
    key: &[u8; 32],
    nonce: &[u8; AES_NONCE_LEN],
    plaintext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| CryptoError::EncryptionFailed)?;
    let aes_nonce = AesNonce::from_slice(nonce);
    cipher
        .encrypt(aes_nonce, plaintext)
        .map_err(|_| CryptoError::EncryptionFailed)
}

/// Decrypt with AES-256-GCM.
fn decrypt_aes256gcm(
    key: &[u8; 32],
    nonce: &[u8; AES_NONCE_LEN],
    ciphertext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| CryptoError::DecryptionFailed)?;
    let aes_nonce = AesNonce::from_slice(nonce);
    cipher
        .decrypt(aes_nonce, ciphertext)
        .map_err(|_| CryptoError::DecryptionFailed)
}

/// Generate 32 random bytes using the OS RNG.
fn random_32_bytes() -> [u8; 32] {
    let mut buf = [0u8; 32];
    rand::rng().fill_bytes(&mut buf);
    buf
}

/// Generate a random 12-byte AES-GCM nonce.
fn random_nonce() -> [u8; AES_NONCE_LEN] {
    let mut buf = [0u8; AES_NONCE_LEN];
    rand::rng().fill_bytes(&mut buf);
    buf
}
