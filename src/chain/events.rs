use alloy::sol;
use serde::{Deserialize, Serialize};

/// On-chain events emitted by pliq smart contracts.
///
/// Each variant maps to a Solidity event the indexer monitors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainEvent {
    UserRegistered {
        address: String,
        nullifier_hash: String,
        verification_level: String,
    },
    ApplicationSubmitted {
        listing_id: String,
        applicant: String,
        zk_proof_hash: String,
    },
    AgreementSigned {
        agreement_id: String,
        signer: String,
    },
    EscrowFunded {
        agreement_id: String,
        amount: u128,
    },
    PaymentExecuted {
        agreement_id: String,
        amount: u128,
        payer: String,
    },
    MerkleRootUpdated {
        root: String,
    },
}

// Solidity event ABI definitions for alloy log decoding.
sol! {
    event UserRegistered(
        address indexed user,
        bytes32 nullifierHash,
        string verificationLevel
    );
    event ApplicationSubmitted(
        bytes32 indexed listingId,
        address indexed applicant,
        bytes32 zkProofHash
    );
    event AgreementSigned(
        bytes32 indexed agreementId,
        address indexed signer
    );
    event EscrowFunded(
        bytes32 indexed agreementId,
        uint256 amount
    );
    event PaymentExecuted(
        bytes32 indexed agreementId,
        uint256 amount,
        address indexed payer
    );
    event MerkleRootUpdated(
        bytes32 root
    );
}
