use std::time::Duration;

use super::events::ChainEvent;
use super::indexer::{calculate_backoff, ContractAddresses};

#[test]
fn backoff_increases_exponentially() {
    let base = Duration::from_secs(2);

    assert_eq!(calculate_backoff(0), base); // 2^0 * 2 = 2
    assert_eq!(calculate_backoff(1), Duration::from_secs(4)); // 2^1 * 2
    assert_eq!(calculate_backoff(2), Duration::from_secs(8)); // 2^2 * 2
    assert_eq!(calculate_backoff(3), Duration::from_secs(16));
}

#[test]
fn backoff_caps_at_max() {
    let max = Duration::from_secs(60);

    // 2^5 * 2 = 64, capped at 60
    assert_eq!(calculate_backoff(5), max);
    assert_eq!(calculate_backoff(10), max);
    assert_eq!(calculate_backoff(100), max);
}

#[test]
fn chain_event_serializes_user_registered() {
    let event = ChainEvent::UserRegistered {
        address: "0xabc".to_string(),
        nullifier_hash: "0xdef".to_string(),
        verification_level: "orb".to_string(),
    };

    let json = serde_json::to_value(&event).unwrap();
    assert_eq!(json["UserRegistered"]["address"], "0xabc");
    assert_eq!(json["UserRegistered"]["verification_level"], "orb");
}

#[test]
fn chain_event_serializes_payment_executed() {
    let event = ChainEvent::PaymentExecuted {
        agreement_id: "agr-1".to_string(),
        amount: 1_000_000,
        payer: "0x123".to_string(),
    };

    let json = serde_json::to_value(&event).unwrap();
    assert_eq!(json["PaymentExecuted"]["amount"], 1_000_000);
    assert_eq!(json["PaymentExecuted"]["payer"], "0x123");
}

#[test]
fn contract_addresses_debug() {
    let addrs = ContractAddresses {
        registry: "0x111".to_string(),
        escrow: "0x222".to_string(),
        agreement: "0x333".to_string(),
    };
    let debug = format!("{addrs:?}");
    assert!(debug.contains("0x111"));
    assert!(debug.contains("0x222"));
    assert!(debug.contains("0x333"));
}
