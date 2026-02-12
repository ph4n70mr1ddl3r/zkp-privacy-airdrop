use serde::{Deserialize, Serialize};
use std::fmt;

pub use zkp_airdrop_utils::types::{Proof, SubmitClaimRequest, SubmitClaimResponse};

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
