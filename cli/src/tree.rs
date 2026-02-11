use anyhow::{Context, Result};
use ethers::types::Address;
use sha3::{Digest, Keccak256};

#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub root: [u8; 32],
    pub height: u8,
}

impl MerkleTree {
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read(path)
            .with_context(|| format!("Failed to read Merkle tree file: {}", path.display()))?;

        if content.len() < 33 {
            return Err(anyhow::anyhow!(
                "Invalid Merkle tree file: expected at least 33 bytes (root + height), got {}",
                content.len()
            ));
        }

        let mut root = [0u8; 32];
        root.copy_from_slice(&content[..32]);
        let height = content[32];

        if height > 32 {
            return Err(anyhow::anyhow!(
                "Invalid Merkle tree height: {}, maximum is 32",
                height
            ));
        }

        Ok(Self { root, height })
    }

    pub fn get_leaf_hash(&self, address: &Address) -> Result<[u8; 32]> {
        let mut hasher = Keccak256::new();
        hasher.update(address.as_bytes());
        let hash = hasher.finalize();

        let mut result = [0u8; 32];
        result.copy_from_slice(&hash);
        Ok(result)
    }

    /// Get the Merkle path for an address in the tree.
    ///
    /// This method is not supported in the current minimal implementation which only
    /// loads the Merkle root and height from the binary file.
    ///
    /// For generating proofs with Merkle paths, use the tree-builder CLI:
    /// ```bash
    /// tree-builder generate-proof --private-key <KEY> --tree <PATH> --output proof.json
    /// ```
    ///
    /// # Errors
    /// Always returns an error as this feature is not implemented in the minimal tree loader.
    pub fn get_path(&self, _address: &[u8; 32]) -> Result<(Vec<[u8; 32]>, Vec<u8>)> {
        Err(anyhow::anyhow!(
            "Merkle path retrieval is not supported in this minimal MerkleTree implementation. \
            The tree-builder CLI must be used to generate proofs with Merkle paths. \
            Run: tree-builder generate-proof --help"
        ))
    }
}
