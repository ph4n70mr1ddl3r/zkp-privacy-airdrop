use anyhow::{Context, Result};
use ark_bn254::Fq;
use chrono::Utc;
use ethers::types::{Address, H256};
use serde::{Deserialize, Serialize};

use crate::crypto::{address_to_field, derive_address, generate_nullifier};
use crate::tree::MerkleTree;
use std::time::Instant;

/// BN128 field element
type F = ark_bn254::Fq;

/// Public inputs for PLONK proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicInputs {
    /// Merkle root (3 field elements: padded address)
    pub merkle_root: [F; 3],
    /// Recipient address (3 field elements: padded address)
    pub recipient: [F; 3],
    /// Nullifier (1 field element)
    pub nullifier: F,
}

/// Private inputs for PLONK proof
#[derive(Debug, Clone)]
pub struct PrivateInputs {
    /// Ethereum private key (32 bytes)
    pub private_key: [u8; 32],
    /// Merkle path siblings (26 field elements)
    pub merkle_path: Vec<F>,
    /// Merkle path indices (26 bits, packed into 1 field element)
    pub merkle_path_indices: F,
}

/// Plonk proof structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlonkProof {
    /// A[2] elements
    pub a: [F; 2],
    /// B[2][2] elements
    pub b: [[F; 2]; 2],
    /// C[2] elements
    pub c: [F; 2],
}

/// Plonk proof data ready for API submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlonkProofData {
    pub proof: PlonkProof,
    pub public_inputs: PublicInputs,
    pub nullifier: H256,
    pub recipient: Address,
    pub merkle_root: H256,
    pub generated_at: String,
}

impl PlonkProof {
    #[cfg(test)]
    /// Create a minimal Plonk proof for testing
    ///
    /// # Warning
    /// This is a stub for testing only. Do not use in production as it creates
    /// an invalid proof that will fail verification.
    pub fn minimal() -> Self {
        Self {
            a: [F::zero(), F::zero()],
            b: [[F::zero(), F::zero()], [F::zero(), F::zero()]],
            c: [F::zero(), F::zero()],
        }
    }

    #[cfg(test)]
    /// Flatten proof for serialization
    pub fn to_flat_array(&self) -> Vec<String> {
        let mut proof_vec = Vec::with_capacity(8);
        proof_vec.extend_from_slice(&self.a);
        proof_vec.extend(self.b.iter().flatten());
        proof_vec.extend_from_slice(&self.c);
        proof_vec
    }
}

/// Generate a Plonk proof for the Merkle membership circuit
pub fn generate_plonk_proof(
    private_key: &[u8; 32],
    recipient: Address,
    merkle_tree: &MerkleTree,
) -> Result<PlonkProofData> {
    let start = Instant::now();

    // Step 1: Derive address from private key
    let address =
        derive_address(private_key).context("Failed to derive address from private key")?;

    // Step 2: Convert address to field elements (pad to 32 bytes)
    let recipient_field = address_to_field(&recipient);

    // Step 3: Find Merkle path for address
    let address_hash = merkle_tree.get_leaf_hash(&address)?;
    let (merkle_path, indices) = merkle_tree
        .get_path(&address_hash)
        .context("Failed to get Merkle path")?;

    // Step 4: Compute nullifier
    let nullifier_bytes =
        generate_nullifier(private_key).context("Failed to generate nullifier")?;
    let nullifier = H256::from_slice(nullifier_bytes.as_bytes());

    // Step 5: Prepare public inputs
    // Note: PLONK requires 3 field elements for each public input
    // We need to pad properly
    let merkle_root = merkle_tree.root;
    let merkle_root_field = {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&merkle_root[..]);
        F::from_be_bytes_mod_order(&bytes, true)
    };

    let recipient_f = F::from_str(&recipient_field)
        .with_context(|| format!("Failed to parse recipient field: {}", recipient_field))?;

    let public_inputs = PublicInputs {
        merkle_root: [merkle_root_field, F::zero(), F::zero()],
        recipient: [recipient_f, F::zero(), F::zero()],
        nullifier: F::from_be_bytes_mod_order(nullifier.as_bytes(), true),
    };

    // Step 6: Prepare private inputs
    let mut merkle_path_fields = Vec::with_capacity(26);
    for sibling in &merkle_path {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&sibling[..]);
        merkle_path_fields.push(F::from_be_bytes_mod_order(&bytes, true));
    }

    // Pack indices (26 bits into 1 field element)
    let mut indices_bytes = [0u8; 32];
    for (i, &bit) in indices.iter().enumerate().take(26) {
        if bit {
            indices_bytes[i / 8] |= 1 << (i % 8);
        }
    }
    let indices_field = F::from_be_bytes_mod_order(&indices_bytes, true);

    let private_inputs = PrivateInputs {
        private_key: *private_key,
        merkle_path: merkle_path_fields,
        merkle_path_indices: indices_field,
    };

    // Step 7: Generate Plonk proof
    // Note: This is a simplified version
    // In production, this would use the actual proving key
    let proof = generate_plonk_proof_internal(&private_inputs, &public_inputs)?;

    // Step 8: Zeroize sensitive data
    let mut key_copy = *private_key;
    key_copy.zeroize();

    let elapsed = start.elapsed();
    tracing::info!("Plonk proof generated in {:?}", elapsed);

    Ok(PlonkProofData {
        proof,
        public_inputs,
        nullifier,
        recipient,
        merkle_root: H256::from_slice(merkle_tree.root),
        generated_at: Utc::now().to_rfc3339(),
    })
}

fn generate_plonk_proof_internal(
    _private_inputs: &PrivateInputs,
    _public_inputs: &PublicInputs,
) -> Result<PlonkProof> {
    Err(anyhow::anyhow!(
        "PLONK proof generation not yet implemented. \
         This requires integration with a PLONK proving library (e.g., snarkjs, arkworks-plonk). \
         Steps needed:\n\
         1) Generate proving key from circuit (.ptau file)\n\
         2) Compute witness from private inputs\n\
         3) Generate PLONK proof using proving key and witness\n\
         4) Verify proof structure matches expected format (8 field elements)\n\
         \n\
         For testing purposes, see the `minimal()` method (deprecated).\n\
         For production, integrate with snarkjs CLI or arkworks PLONK implementation."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plonk_proof_structure() {
        let proof = PlonkProof::minimal();

        assert_eq!(proof.a.len(), 2);
        assert_eq!(proof.b.len(), 2);
        assert_eq!(proof.c.len(), 2);
        assert_eq!(proof.to_flat_array().len(), 8);
    }

    #[test]
    fn test_field_element_conversion() {
        let address = Address::from([0x12u8; 20]);
        let field = address_to_field(&address);

        let parsed_field = F::from_str(&field).unwrap();
        assert!(!parsed_field.is_zero());
    }
}
