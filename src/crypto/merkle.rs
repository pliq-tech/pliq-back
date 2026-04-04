use sha3::{Digest, Keccak256};

/// SHA-3 Merkle tree for Proof of Rent.
pub struct MerkleTree {
    leaves: Vec<[u8; 32]>,
    layers: Vec<Vec<[u8; 32]>>,
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            leaves: Vec::new(),
            layers: Vec::new(),
        }
    }

    /// Insert a new leaf and rebuild the tree.
    pub fn insert_leaf(&mut self, data: &[u8]) {
        let hash = hash_leaf(data);
        self.leaves.push(hash);
        self.rebuild();
    }

    /// Get the current Merkle root.
    pub fn root(&self) -> Option<[u8; 32]> {
        self.layers.last().and_then(|layer| layer.first().copied())
    }

    /// Get the root as hex string.
    pub fn root_hex(&self) -> Option<String> {
        self.root().map(|r| hex::encode(r))
    }

    /// Generate a Merkle proof for the leaf at the given index.
    pub fn proof(&self, leaf_index: usize) -> Option<Vec<(bool, [u8; 32])>> {
        if leaf_index >= self.leaves.len() {
            return None;
        }

        let mut proof = Vec::new();
        let mut idx = leaf_index;

        for layer in &self.layers[..self.layers.len().saturating_sub(1)] {
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            let is_left = idx % 2 == 1;

            if sibling_idx < layer.len() {
                proof.push((is_left, layer[sibling_idx]));
            }

            idx /= 2;
        }

        Some(proof)
    }

    /// Verify a Merkle proof.
    pub fn verify(root: &[u8; 32], leaf_hash: &[u8; 32], proof: &[(bool, [u8; 32])]) -> bool {
        let mut current = *leaf_hash;

        for (is_left, sibling) in proof {
            current = if *is_left {
                hash_pair(sibling, &current)
            } else {
                hash_pair(&current, sibling)
            };
        }

        &current == root
    }

    /// Rebuild all layers from leaves.
    fn rebuild(&mut self) {
        self.layers.clear();

        if self.leaves.is_empty() {
            return;
        }

        let mut current_layer = self.leaves.clone();
        self.layers.push(current_layer.clone());

        while current_layer.len() > 1 {
            let mut next_layer = Vec::new();

            for chunk in current_layer.chunks(2) {
                if chunk.len() == 2 {
                    next_layer.push(hash_pair(&chunk[0], &chunk[1]));
                } else {
                    // Odd node: promote to next layer
                    next_layer.push(chunk[0]);
                }
            }

            self.layers.push(next_layer.clone());
            current_layer = next_layer;
        }
    }
}

impl Default for MerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash a leaf value using SHA-3 (Keccak-256).
fn hash_leaf(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(b"\x00"); // Leaf prefix
    hasher.update(data);
    hasher.finalize().into()
}

/// Hash two nodes together, sorting them for consistency.
fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(b"\x01"); // Internal node prefix

    // Sort for second-preimage resistance
    if left <= right {
        hasher.update(left);
        hasher.update(right);
    } else {
        hasher.update(right);
        hasher.update(left);
    }

    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tree() {
        let tree = MerkleTree::new();
        assert!(tree.root().is_none());
    }

    #[test]
    fn test_single_leaf() {
        let mut tree = MerkleTree::new();
        tree.insert_leaf(b"payment_1");
        assert!(tree.root().is_some());
    }

    #[test]
    fn test_proof_verification() {
        let mut tree = MerkleTree::new();
        tree.insert_leaf(b"payment_1");
        tree.insert_leaf(b"payment_2");
        tree.insert_leaf(b"payment_3");

        let root = tree.root().unwrap();
        let proof = tree.proof(1).unwrap();
        let leaf_hash = hash_leaf(b"payment_2");

        assert!(MerkleTree::verify(&root, &leaf_hash, &proof));
    }

    #[test]
    fn test_tampered_proof_fails() {
        let mut tree = MerkleTree::new();
        tree.insert_leaf(b"payment_1");
        tree.insert_leaf(b"payment_2");

        let root = tree.root().unwrap();
        let proof = tree.proof(0).unwrap();
        let wrong_leaf = hash_leaf(b"tampered");

        assert!(!MerkleTree::verify(&root, &wrong_leaf, &proof));
    }
}
