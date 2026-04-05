use super::merkle::MerkleTree;

#[test]
fn test_empty_tree_has_no_root() {
    let tree = MerkleTree::new();
    assert!(tree.root().is_none());
}

#[test]
fn test_single_leaf_has_root() {
    let mut tree = MerkleTree::new();
    tree.insert_leaf(b"payment_1");
    assert!(tree.root().is_some());
}

#[test]
fn test_two_leaves_different_root() {
    let mut tree1 = MerkleTree::new();
    tree1.insert_leaf(b"payment_1");
    let root1 = tree1.root().unwrap();

    let mut tree2 = MerkleTree::new();
    tree2.insert_leaf(b"payment_1");
    tree2.insert_leaf(b"payment_2");
    let root2 = tree2.root().unwrap();

    assert_ne!(root1, root2);
}

#[test]
fn test_proof_roundtrip_five_leaves() {
    let mut tree = MerkleTree::new();
    let data: Vec<&[u8]> = vec![b"p1", b"p2", b"p3", b"p4", b"p5"];
    for d in &data {
        tree.insert_leaf(d);
    }
    let root = tree.root().unwrap();

    for i in 0..data.len() {
        let proof = tree.structured_proof(i).expect("proof should exist");
        let leaf_hash = sha3_leaf(data[i]);
        assert!(MerkleTree::verify(&root, &leaf_hash, &proof));
    }
}

#[test]
fn test_tampered_leaf_fails() {
    let mut tree = MerkleTree::new();
    tree.insert_leaf(b"payment_1");
    tree.insert_leaf(b"payment_2");
    let root = tree.root().unwrap();
    let proof = tree.structured_proof(0).unwrap();
    let wrong_hash = sha3_leaf(b"tampered");

    assert!(!MerkleTree::verify(&root, &wrong_hash, &proof));
}

#[test]
fn test_seven_leaves_all_verify() {
    let mut tree = MerkleTree::new();
    for i in 0..7 {
        tree.insert_leaf(format!("leaf_{i}").as_bytes());
    }
    let root = tree.root().unwrap();

    for i in 0..7 {
        let proof = tree.structured_proof(i).unwrap();
        let leaf_hash = sha3_leaf(format!("leaf_{i}").as_bytes());
        assert!(MerkleTree::verify(&root, &leaf_hash, &proof));
    }
}

#[test]
fn test_deterministic_root() {
    let mut tree1 = MerkleTree::new();
    let mut tree2 = MerkleTree::new();
    tree1.insert_leaf(b"a");
    tree1.insert_leaf(b"b");
    tree2.insert_leaf(b"a");
    tree2.insert_leaf(b"b");

    assert_eq!(tree1.root(), tree2.root());
}

fn sha3_leaf(data: &[u8]) -> [u8; 32] {
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(b"\x00");
    hasher.update(data);
    hasher.finalize().into()
}
