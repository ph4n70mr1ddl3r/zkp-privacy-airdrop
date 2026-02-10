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

    pub fn get_path(&self, _address: &[u8; 32]) -> Result<(Vec<[u8; 32]>, Vec<u8>)> {
        Err(anyhow::anyhow!(
            "MerkleTree::get_path requires loading the full tree from the binary format. \
            The current implementation only loads the root and height. \
            Please use the tree-builder CLI to generate proof data with merkle paths."
        ))
    }
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

    pub fn get_path(&self, _address: &[u8; 32]) -> Result<(Vec<[u8; 32]>, Vec<u8>)> {
        Err(anyhow::anyhow!(
            "MerkleTree::get_path requires loading the full tree from the binary format. \
            The current implementation only loads the root and height. \
            Please use the tree-builder CLI to generate proof data with merkle paths."
        ))
    }
}

impl MerkleTree {
    #[allow(dead_code)]
    pub fn from_file(_path: &std::path::Path) -> Result<Option<Self>> {
        Err(anyhow::anyhow!("MerkleTree::from_file not implemented. Please use tree-builder CLI to generate and serialize the tree."))
    }

    pub fn get_leaf_hash(&self, address: &Address) -> Result<[u8; 32]> {
        Err(anyhow::anyhow!("MerkleTree::get_leaf_hash not implemented. Please use tree-builder CLI for tree operations."))
    }

    pub fn get_path(&self, address: &[u8; 32]) -> Result<(Vec<[u8; 32]>, Vec<u8>)> {
        Err(anyhow::anyhow!("MerkleTree::get_path not implemented. Please use tree-builder CLI for tree operations."))
    }
}
