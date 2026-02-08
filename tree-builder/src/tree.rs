use num_bigint::BigUint;
use num_traits::{Num, Zero};
use rayon::prelude::*;
use std::collections::HashMap;

pub const TREE_HEIGHT: u8 = 26;
pub const MAX_LEAVES: usize = 1 << 26; // 2^26 = 67,108,864

const FIELD_PRIME: &str =
    "21888242871839275222246405745257275088548364400416034343698204186575808495617";
const FIELD_PRIME_BIG: BigUint = BigUint::from_str_radix(FIELD_PRIME, 10).unwrap();

#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub height: u8,
    pub leaves: Vec<[u8; 32]>,
    pub nodes: Vec<Vec<[u8; 32]>>,
    pub root: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct MerklePath {
    pub siblings: Vec<[u8; 32]>,
    pub indices: Vec<bool>,
}

impl MerkleTree {
    pub fn new(height: u8) -> Self {
        MerkleTree {
            height,
            leaves: Vec::new(),
            nodes: Vec::new(),
            root: [0u8; 32],
        }
    }

    pub fn insert(&mut self, leaf: [u8; 32]) {
        if self.leaves.len() >= MAX_LEAVES {
            panic!("Tree is full, maximum leaves reached");
        }
        self.leaves.push(leaf);
    }

    pub fn finalize(&mut self) {
        self.build_tree();
    }

    fn build_tree(&mut self) {
        if self.leaves.is_empty() {
            self.root = [0u8; 32];
            return;
        }

        let mut level = self.leaves.clone();
        self.nodes.clear();

        for _ in 0..self.height {
            self.nodes.push(level.clone());

            if level.len() == 1 {
                self.root = level[0];
                return;
            }

            let next_level: Vec<[u8; 32]> = level
                .par_chunks(2)
                .map(|pair| {
                    if pair.len() == 2 {
                        super::poseidon::hash_two(&pair[0], &pair[1])
                    } else {
                        pair[0]
                    }
                })
                .collect();

            level = next_level;
        }

        self.root = *level.first().unwrap_or(&[0u8; 32]);
    }

    pub fn get_path(&self, index: usize) -> Result<MerklePath, String> {
        if index >= self.leaves.len() {
            return Err("Index out of bounds".to_string());
        }

        let mut siblings = Vec::new();
        let mut indices = Vec::new();
        let mut idx = index;

        for level in &self.nodes {
            if level.is_empty() {
                break;
            }

            let is_right = idx % 2 == 1;
            indices.push(is_right);

            if is_right {
                if idx > 0 {
                    siblings.push(level[idx - 1]);
                } else {
                    siblings.push([0u8; 32]);
                }
            } else {
                if idx + 1 < level.len() {
                    siblings.push(level[idx + 1]);
                } else {
                    siblings.push([0u8; 32]);
                }
            }

            idx /= 2;
        }

        Ok(MerklePath { siblings, indices })
    }

    pub fn verify_path(&self, leaf: &[u8; 32], path: &MerklePath) -> bool {
        let mut current = *leaf;

        for (sibling, &go_right) in path.siblings.iter().zip(path.indices.iter()) {
            if go_right {
                current = super::poseidon::hash_two(&current, sibling);
            } else {
                current = super::poseidon::hash_two(sibling, &current);
            }
        }

        current == self.root
    }

    pub fn get_leaf_index(&self, leaf_hash: &[u8; 32]) -> Option<usize> {
        self.leaves.iter().position(|l| l == leaf_hash)
    }
}

pub fn build_merkle_tree(addresses: &[[u8; 20]], height: u8) -> Result<MerkleTree, String> {
    if height > 26 {
        return Err("Tree height too large, maximum is 26".to_string());
    }

    if addresses.len() > 1 << height {
        return Err(format!(
            "Too many addresses for tree height {}: {} > {}",
            height,
            addresses.len(),
            1 << height
        ));
    }

    let mut tree = MerkleTree::new(height);

    let pb = indicatif::ProgressBar::new(addresses.len() as u64);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .progress_chars("=>-"),
    );
    pb.set_message("Hashing addresses...");

    for address in addresses {
        let leaf = super::poseidon::hash_address(*address);
        tree.insert(leaf);
        pb.inc(1);
    }

    pb.finish();

    tree.finalize();

    Ok(tree)
}

fn mod_field(bytes: &[u8; 32]) -> [u8; 32] {
    let value = BigUint::from_bytes_be(bytes);
    let reduced = &value % &FIELD_PRIME_BIG;
    let mut result = [0u8; 32];
    reduced.to_bytes_be(&mut result);
    result
}
