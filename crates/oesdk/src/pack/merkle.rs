//! Domain-separated Merkle tree over `(path, raw bytes)` leaves.

use sha2::{Digest, Sha256};

pub const LEAF_PREFIX: u8 = 0x00;
pub const NODE_PREFIX: u8 = 0x01;
pub const SIGN_DOMAIN: &[u8] = b"FNFM1\x00";
pub const TOC_PATH: &str = "__toc__";

pub fn leaf_hash(path: &str, raw: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([LEAF_PREFIX]);
    hasher.update(path.as_bytes());
    hasher.update([0u8]);
    hasher.update((raw.len() as u64).to_le_bytes());
    hasher.update(Sha256::digest(raw));
    hasher.finalize().into()
}

fn node_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([NODE_PREFIX]);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

/// Fold leaves in order; an odd node is promoted unchanged (no duplication,
/// which would make a second preimage trivially forgeable).
pub fn root(leaves: &[[u8; 32]]) -> [u8; 32] {
    if leaves.is_empty() {
        return [0u8; 32];
    }
    let mut level: Vec<[u8; 32]> = leaves.to_vec();
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            next.push(match pair {
                [a, b] => node_hash(a, b),
                [a] => *a,
                _ => unreachable!(),
            });
        }
        level = next;
    }
    level[0]
}

pub fn sign_message(root: &[u8; 32]) -> Vec<u8> {
    let mut message = SIGN_DOMAIN.to_vec();
    message.extend_from_slice(root);
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaves_are_path_bound() {
        assert_ne!(leaf_hash("a", b"x"), leaf_hash("b", b"x"));
        assert_ne!(leaf_hash("a", b"x"), leaf_hash("a", b"y"));
    }

    #[test]
    fn odd_node_is_promoted() {
        let a = leaf_hash("a", b"1");
        let b = leaf_hash("b", b"2");
        let c = leaf_hash("c", b"3");
        assert_eq!(root(&[a]), a);
        assert_eq!(root(&[a, b]), node_hash(&a, &b));
        assert_eq!(root(&[a, b, c]), node_hash(&node_hash(&a, &b), &c));
    }
}
