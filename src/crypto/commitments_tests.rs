use super::commitments::{create_commitment, verify_commitment, EscrowConditions};

fn test_conditions() -> EscrowConditions {
    EscrowConditions {
        amount: 1200_00,
        tenant_id: "tenant-uuid".to_string(),
        landlord_id: "landlord-uuid".to_string(),
        listing_id: "listing-uuid".to_string(),
    }
}

#[test]
fn test_create_commitment_non_zero() {
    let conditions = test_conditions();
    let commitment = create_commitment(&conditions);

    assert_ne!(commitment.commitment_hash, [0u8; 32]);
    assert_ne!(commitment.secret, [0u8; 32]);
    assert_ne!(commitment.nonce, [0u8; 16]);
}

#[test]
fn test_verify_roundtrip() {
    let conditions = test_conditions();
    let commitment = create_commitment(&conditions);

    let valid = verify_commitment(
        &commitment.commitment_hash,
        &conditions,
        &commitment.secret,
        &commitment.nonce,
    );
    assert!(valid);
}

#[test]
fn test_tampered_amount_fails() {
    let conditions = test_conditions();
    let commitment = create_commitment(&conditions);

    let mut tampered = test_conditions();
    tampered.amount = 9999_00;

    let valid = verify_commitment(
        &commitment.commitment_hash,
        &tampered,
        &commitment.secret,
        &commitment.nonce,
    );
    assert!(!valid);
}

#[test]
fn test_different_conditions_different_hash() {
    let c1 = EscrowConditions {
        amount: 1000,
        tenant_id: "a".into(),
        landlord_id: "b".into(),
        listing_id: "c".into(),
    };
    let c2 = EscrowConditions {
        amount: 2000,
        tenant_id: "a".into(),
        landlord_id: "b".into(),
        listing_id: "c".into(),
    };
    assert_ne!(c1.hash(), c2.hash());
}

#[test]
fn test_conditions_hash_deterministic() {
    let c1 = test_conditions();
    let c2 = test_conditions();
    assert_eq!(c1.hash(), c2.hash());
}
