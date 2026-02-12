use serde::{Deserialize, Serialize};
use std::fmt;

/// Exact number of field elements in a PLONK proof
/// PLONK proofs for this circuit contain exactly 8 elements: A, B, C, Z, T1, T2, T3, `WXi`
const PLONK_PROOF_SIZE: usize = 8;

/// Plonk proof format (new format for universal trusted setup)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlonkProof {
    pub proof: Vec<String>, // Flat array of field elements
}

impl PlonkProof {
    /// Convert Plonk proof to flat array for API transmission
    ///
    /// # Returns
    /// Returns a reference to proof array after validating it
    ///
    /// # Errors
    /// Returns an error if proof is empty or invalid
    #[allow(dead_code)]
    pub fn to_flat_array(&self) -> Result<&[String], String> {
        if self.proof.is_empty() {
            return Err("PlonkProof is empty".to_string());
        }
        if self.proof.len() != PLONK_PROOF_SIZE {
            return Err(format!(
                "PlonkProof must have exactly {} elements, found {}",
                PLONK_PROOF_SIZE,
                self.proof.len()
            ));
        }
        for (idx, element) in self.proof.iter().enumerate() {
            if element.trim().is_empty() {
                return Err(format!("PlonkProof element at index {idx} is empty"));
            }
            if !element.starts_with("0x") && !element.starts_with("0X") {
                return Err(format!(
                    "PlonkProof element at index {idx} missing 0x prefix"
                ));
            }
            if !zkp_airdrop_utils::is_valid_field_element(element) {
                return Err(format!(
                    "PlonkProof element at index {idx} is not a valid field element"
                ));
            }
        }
        Ok(&self.proof)
    }

    #[cfg(test)]
    /// Create a minimal Plonk proof for testing
    #[allow(dead_code)]
    pub fn minimal() -> Self {
        let test_element =
            "0x0000000000000000000000000000000000000000000000000000000000000000001".to_string();
        Self {
            proof: vec![test_element; PLONK_PROOF_SIZE],
        }
    }
}

/// Data structure for a proof with all necessary fields for claiming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofData {
    /// The proof (either Groth16 or PLONK)
    pub proof: Proof,
    /// Public signals: [`merkle_root`, recipient, nullifier]
    pub public_signals: [String; 3],
    /// Unique identifier derived from private key
    pub nullifier: String,
    /// Address to receive tokens
    pub recipient: String,
    /// Merkle root of the tree
    pub merkle_root: String,
    /// Timestamp when proof was generated
    pub generated_at: String,
}

/// Union type for different proof systems
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Proof {
    Plonk(PlonkProof),
}

impl Proof {
    /// Get proof type name
    pub fn type_name(&self) -> &str {
        match self {
            Proof::Plonk(_) => "Plonk",
        }
    }

    /// Get proof size in bytes (estimated)
    pub fn estimated_size_bytes(&self) -> usize {
        match self {
            Proof::Plonk(p) => p.proof.iter().map(std::string::String::len).sum::<usize>() + 100, // ~500 bytes for Plonk
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitClaimRequest {
    pub proof: Proof,
    pub recipient: String,
    pub nullifier: String,
    pub merkle_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitClaimResponse {
    pub success: bool,
    pub tx_hash: Option<String>,
    pub status: Option<String>,
    pub estimated_confirmation: Option<String>,
    pub error: Option<String>,
    pub code: Option<String>,
}

impl fmt::Display for ProofData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Proof Data:")?;
        writeln!(f, "  Nullifier: {}", self.nullifier)?;
        writeln!(f, "  Recipient: {}", self.recipient)?;
        writeln!(f, "  Merkle Root: {}", self.merkle_root)?;
        writeln!(f, "  Generated At: {}", self.generated_at)?;
        writeln!(f, "  Proof Type: {}", self.proof.type_name())?;
        writeln!(
            f,
            "  Estimated Size: {} bytes",
            self.proof.estimated_size_bytes()
        )
    }
}
