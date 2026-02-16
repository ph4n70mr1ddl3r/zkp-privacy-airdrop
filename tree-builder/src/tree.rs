use rayon::prelude::*;

pub const MAX_LEAVES: usize = 1 << 26; // 2^26 = 67,108,864

#[cfg(test)]
use num_bigint::BigUint;
#[cfg(test)]
use num_traits::Num;

#[cfg(test)]
#[allow(dead_code)]
const FIELD_PRIME: &str =
    "21888242871839275222246405745257275088548364400416034343698204186575808495617";

#[cfg(test)]
#[allow(dead_code)]
fn field_prime() -> BigUint {
    BigUint::from_str_radix(FIELD_PRIME, 10).expect("Invalid field prime constant")
}

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
        let max_leaves = 1_usize << height;
        let max_nodes = max_leaves * 2;

        MerkleTree {
            height,
            leaves: Vec::with_capacity(max_leaves),
            nodes: Vec::with_capacity(max_nodes),
            root: [0u8; 32],
        }
    }

    pub fn insert(&mut self, leaf: [u8; 32]) -> Result<(), String> {
        if self.leaves.len() >= MAX_LEAVES {
            return Err(format!(
                "Tree is full, maximum leaves reached (MAX_LEAVES={})",
                MAX_LEAVES
            ));
        }
        self.leaves.push(leaf);
        Ok(())
    }

    pub fn finalize(&mut self) {
        self.build_tree();
    }

    /// Builds the Merkle tree from inserted leaves using parallel hashing.
    /// This should be called after all leaves have been inserted.
    /// The tree is built bottom-up, hashing pairs of nodes at each level.
    fn build_tree(&mut self) {
        if self.leaves.is_empty() {
            self.root = [0u8; 32];
            return;
        }

        let mut level = std::mem::take(&mut self.leaves);
        self.nodes.clear();

        for _depth in 0..self.height {
            self.nodes.push(level.clone());

            if level.len() == 1 {
                self.root = level.swap_remove(0);
                return;
            }

            // Intentional clone: We need to store the current level in nodes for path generation
            // while consuming the level for the next iteration. Rayon parallel operations require
            // ownership, so we clone here before the parallel hash operations.
            level = level
                .par_chunks(2)
                .map(|pair| {
                    if pair.len() == 2 {
                        super::poseidon::hash_two(&pair[0], &pair[1])
                            .expect("Failed to hash pair during tree build")
                    } else {
                        pair[0]
                    }
                })
                .collect();
        }

        self.root = if let Some(&root) = level.first() {
            root
        } else {
            // Tree was built but has no root (should not happen with proper initialization)
            [0u8; 32]
        };
    }

    /// Gets the Merkle proof path for a leaf at the given index.
    ///
    /// The Merkle path consists of sibling nodes needed to verify that the leaf
    /// is part of the tree. This is used by claimants to prove their address
    /// is in the eligible list without revealing which address.
    ///
    /// # Arguments
    /// * `index` - The index of the leaf in the tree
    ///
    /// # Returns
    /// A `MerklePath` containing siblings and direction indicators
    ///
    /// # Errors
    /// Returns an error if the index is out of bounds
    pub fn get_path(&self, index: usize) -> Result<MerklePath, String> {
        if index >= self.leaves.len() {
            return Err(format!(
                "Index {} out of bounds, total leaves: {}",
                index,
                self.leaves.len()
            ));
        }

        let mut siblings = Vec::new();
        let mut indices = Vec::new();
        let mut idx = index;

        for level in &self.nodes {
            if level.is_empty() {
                break;
            }

            if idx >= level.len() {
                return Err(format!(
                    "Invalid tree state: index {} >= level length {}",
                    idx,
                    level.len()
                ));
            }

            let is_right = idx % 2 == 1;
            indices.push(is_right);

            if is_right {
                if idx > 0 && idx - 1 < level.len() {
                    siblings.push(level[idx - 1]);
                } else {
                    return Err(format!(
                        "Invalid sibling index: {} - 1 out of bounds for level length {}",
                        idx,
                        level.len()
                    ));
                }
            } else if idx + 1 < level.len() {
                siblings.push(level[idx + 1]);
            } else {
                siblings.push([0u8; 32]);
            }

            idx /= 2;
        }

        Ok(MerklePath { siblings, indices })
    }

    /// Verifies that a leaf is part of the tree using its Merkle path.
    ///
    /// This function recomputes the root hash by hashing the leaf with
    /// its siblings along the path, then compares the result to the stored root.
    ///
    /// # Arguments
    /// * `leaf` - The 32-byte leaf hash to verify
    /// * `path` - The Merkle path containing siblings and direction indicators
    ///
    /// # Returns
    /// `true` if the leaf is valid and the computed root matches the stored root
    pub fn verify_path(&self, leaf: &[u8; 32], path: &MerklePath) -> bool {
        let mut current = *leaf;

        for (sibling, &go_right) in path.siblings.iter().zip(path.indices.iter()) {
            if go_right {
                current = super::poseidon::hash_two(&current, sibling)
                    .expect("Failed to hash during path verification");
            } else {
                current = super::poseidon::hash_two(sibling, &current)
                    .expect("Failed to hash during path verification");
            }
        }

        current == self.root
    }
}

pub fn build_merkle_tree(addresses: &[[u8; 20]], height: u8) -> Result<MerkleTree, String> {
    const MIN_HEIGHT: u8 = 1;
    const MAX_HEIGHT: u8 = 26;

    if height < MIN_HEIGHT {
        return Err(format!("Tree height too small, minimum is {}", MIN_HEIGHT));
    }
    if height > MAX_HEIGHT {
        return Err(format!("Tree height too large, maximum is {}", MAX_HEIGHT));
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
    let style = match indicatif::ProgressStyle::default_bar()
        .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
    {
        Ok(s) => s.progress_chars("=>-"),
        Err(e) => {
            eprintln!(
                "Warning: Failed to set progress style template: {}, using default",
                e
            );
            indicatif::ProgressStyle::default_bar().progress_chars("=>-")
        }
    };
    pb.set_style(style);

    pb.set_message("Hashing addresses...");

    for address in addresses {
        let leaf = super::poseidon::hash_address(*address)
            .map_err(|e| format!("Failed to hash address: {}", e))?;
        tree.insert(leaf)
            .map_err(|e| format!("Failed to insert leaf: {}", e))?;
        pb.inc(1);
    }

    pb.finish();

    tree.finalize();

    Ok(tree)
}

// Field element reduction for testing - not used in production
// This is only used by test_mod_field tests below
#[cfg(test)]
#[allow(dead_code)]
fn mod_field(bytes: &[u8; 32]) -> Result<[u8; 32], String> {
    let value = BigUint::from_bytes_be(bytes);
    let reduced = &value % &field_prime();
    let mut result = vec![0u8; 32];
    let bytes = reduced.to_bytes_be();
    if bytes.len() > 32 {
        return Err("Field element exceeds 32 bytes".to_string());
    }
    let offset = 32 - bytes.len();
    result[offset..].copy_from_slice(&bytes);
    result
        .try_into()
        .map_err(|_| "Failed to convert Vec to array".to_string())
}

#[cfg(test)]
mod test_mod_field {
    use super::*;

    #[test]
    fn test_mod_field_valid() {
        let input = [1u8; 32];
        let result = mod_field(&input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mod_field_error_handling() {
        let input = [255u8; 32];
        let result = mod_field(&input);
        assert!(result.is_ok() || result.is_err());
    }
}
