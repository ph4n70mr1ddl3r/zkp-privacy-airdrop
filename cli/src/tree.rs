use anyhow::{Context, Result};
use ethers::types::Address;

#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub root: [u8; 32],
    pub height: u8,
}

impl MerkleTree {
    pub fn from_file(_path: &std::path::Path) -> Result<Option<Self>> {
        Ok(None)
    }

    pub fn get_leaf_hash(&self, address: &Address) -> Result<[u8; 32]> {
        Ok([0u8; 32])
    }

    pub fn get_path(&self, address: &[u8; 32]) -> Result<(Vec<[u8; 32]>, Vec<u8>)> {
        Ok((vec![], vec![]))
    }
}
