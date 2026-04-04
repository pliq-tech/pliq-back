use rand::RngCore;
use sha3::{Digest, Keccak256};

/// Hash-based escrow commitment using SHA-3.
#[derive(Debug, Clone)]
pub struct EscrowCommitment {
    pub commitment_hash: [u8; 32],
    pub secret: [u8; 32],
    pub nonce: [u8; 16],
}

/// Conditions that are committed to in the escrow.
#[derive(Debug, Clone)]
pub struct EscrowConditions {
    pub amount: u64,
    pub tenant_id: String,
    pub landlord_id: String,
    pub listing_id: String,
}

impl EscrowConditions {
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Keccak256::new();
        hasher.update(self.amount.to_le_bytes());
        hasher.update(self.tenant_id.as_bytes());
        hasher.update(self.landlord_id.as_bytes());
        hasher.update(self.listing_id.as_bytes());
        hasher.finalize().into()
    }
}

/// Create a new escrow commitment.
pub fn create_commitment(conditions: &EscrowConditions) -> EscrowCommitment {
    let mut rng = rand::rng();

    let mut secret = [0u8; 32];
    rng.fill_bytes(&mut secret);

    let mut nonce = [0u8; 16];
    rng.fill_bytes(&mut nonce);

    let conditions_hash = conditions.hash();

    let mut hasher = Keccak256::new();
    hasher.update(&conditions_hash);
    hasher.update(&secret);
    hasher.update(&nonce);
    let commitment_hash: [u8; 32] = hasher.finalize().into();

    EscrowCommitment {
        commitment_hash,
        secret,
        nonce,
    }
}

/// Verify that a reveal matches the original commitment.
pub fn verify_commitment(
    commitment_hash: &[u8; 32],
    conditions: &EscrowConditions,
    secret: &[u8; 32],
    nonce: &[u8; 16],
) -> bool {
    let conditions_hash = conditions.hash();

    let mut hasher = Keccak256::new();
    hasher.update(&conditions_hash);
    hasher.update(secret);
    hasher.update(nonce);
    let computed: [u8; 32] = hasher.finalize().into();

    &computed == commitment_hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment_roundtrip() {
        let conditions = EscrowConditions {
            amount: 1200,
            tenant_id: "tenant-001".to_string(),
            landlord_id: "landlord-001".to_string(),
            listing_id: "listing-001".to_string(),
        };

        let commitment = create_commitment(&conditions);

        assert!(verify_commitment(
            &commitment.commitment_hash,
            &conditions,
            &commitment.secret,
            &commitment.nonce,
        ));
    }

    #[test]
    fn test_tampered_amount_fails() {
        let conditions = EscrowConditions {
            amount: 1200,
            tenant_id: "tenant-001".to_string(),
            landlord_id: "landlord-001".to_string(),
            listing_id: "listing-001".to_string(),
        };

        let commitment = create_commitment(&conditions);

        let tampered = EscrowConditions {
            amount: 999,
            ..conditions
        };

        assert!(!verify_commitment(
            &commitment.commitment_hash,
            &tampered,
            &commitment.secret,
            &commitment.nonce,
        ));
    }
}
