use super::hybrid::{generate_keypair, open, seal, HybridPublicKey};

#[test]
fn test_generate_keypair_valid() {
    let kp = generate_keypair();
    assert_eq!(kp.x25519_public.as_bytes().len(), 32);
}

#[test]
fn test_seal_open_roundtrip() {
    let recipient = generate_keypair();
    let pk = HybridPublicKey {
        x25519_pk: *recipient.x25519_public.as_bytes(),
    };
    let plaintext = b"confidential lease data";

    let sealed = seal(plaintext, &pk).expect("seal failed");
    let recovered = open(&sealed, &recipient).expect("open failed");

    assert_eq!(recovered, plaintext);
}

#[test]
fn test_wrong_key_fails() {
    let recipient = generate_keypair();
    let wrong = generate_keypair();
    let pk = HybridPublicKey {
        x25519_pk: *recipient.x25519_public.as_bytes(),
    };

    let sealed = seal(b"secret", &pk).expect("seal failed");
    let result = open(&sealed, &wrong);

    assert!(result.is_err());
}

#[test]
fn test_empty_plaintext() {
    let recipient = generate_keypair();
    let pk = HybridPublicKey {
        x25519_pk: *recipient.x25519_public.as_bytes(),
    };

    let sealed = seal(b"", &pk).expect("seal failed");
    let recovered = open(&sealed, &recipient).expect("open failed");
    assert!(recovered.is_empty());
}

#[test]
fn test_large_plaintext() {
    let recipient = generate_keypair();
    let pk = HybridPublicKey {
        x25519_pk: *recipient.x25519_public.as_bytes(),
    };
    let plaintext = vec![0xABu8; 10 * 1024];

    let sealed = seal(&plaintext, &pk).expect("seal failed");
    let recovered = open(&sealed, &recipient).expect("open failed");
    assert_eq!(recovered, plaintext);
}
