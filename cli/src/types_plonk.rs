use ethers::types::{Address, H256};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Groth16 proof format (deprecated, kept for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Groth16Proof {
    pub a: [String; 2],
    pub b: [[String; 2]; 2],
    pub c: [String; 2],
}

/// Plonk proof format (new format for universal trusted setup)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlonkProof {
    pub proof: Vec<String>, // Flat array of field elements
}

impl PlonkProof {
    /// Convert Plonk proof to flat array for API transmission
    pub fn to_flat_array(&self) -> &[String] {
        &self.proof
    }

    #[cfg(test)]
    /// Create a minimal Plonk proof for testing
    pub fn minimal() -> Self {
        Self {
            proof: vec!["0".to_string(); 8],
        }
    }
}

/// Data structure for a proof with all necessary fields for claiming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofData {
    /// The proof (either Groth16 or PLONK)
    pub proof: Proof,
    /// Public signals: [merkle_root, recipient, nullifier]
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
    Groth16(Groth16Proof),
    Plonk(PlonkProof),
}

impl Proof {
    /// Get proof type name
    pub fn type_name(&self) -> &str {
        match self {
            Proof::Groth16(_) => "Groth16",
            Proof::Plonk(_) => "Plonk",
        }
    }

    /// Get proof size in bytes (estimated)
    pub fn estimated_size_bytes(&self) -> usize {
        match self {
            Proof::Groth16(_) => 200, // ~200 bytes for Groth16
            Proof::Plonk(p) => p.proof.iter().map(|s| s.len()).sum::<usize>() + 100, // ~500 bytes for Plonk
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckStatusResponse {
    pub nullifier: String,
    pub claimed: bool,
    pub tx_hash: Option<String>,
    pub recipient: Option<String>,
    pub timestamp: Option<String>,
    pub block_number: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub services: Services,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Services {
    pub database: String,
    pub redis: String,
    pub optimism_node: String,
    pub relayer_wallet: RelayerWallet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayerWallet {
    pub address: String,
    pub balance: String,
    pub sufficient: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfoResponse {
    pub network: String,
    pub chain_id: u64,
    pub contracts: Contracts,
    pub claim_amount: String,
    pub claim_deadline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contracts {
    pub airdrop: ContractInfo,
    pub token: TokenInfo,
    pub relayer_registry: Option<ContractInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub address: String,
    pub deployed_at: Option<String>,
    pub block_number: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonateRequest {
    pub amount: String,
    pub donor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonateResponse {
    pub donation_address: String,
    pub amount_received: String,
    pub tx_hash: Option<String>,
    pub thank_you: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    pub total_claims: u64,
    pub successful_claims: u64,
    pub failed_claims: u64,
    pub total_tokens_distributed: String,
    pub unique_recipients: u64,
    pub average_gas_price: String,
    pub total_gas_used: String,
    pub relayer_balance: String,
    pub uptime_percentage: f64,
    pub response_time_ms: ResponseTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTime {
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
}

impl fmt::Display for ProofData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Proof Data:\n")?;
        write!(f, "  Nullifier: {}\n", self.nullifier)?;
        write!(f, "  Recipient: {}\n", self.recipient)?;
        write!(f, "  Merkle Root: {}\n", self.merkle_root)?;
        write!(f, "  Generated At: {}\n", self.generated_at)?;
        write!(f, "  Proof Type: {}\n", self.proof.type_name())?;
        write!(
            f,
            "  Estimated Size: {} bytes\n",
            self.proof.estimated_size_bytes()
        )
    }
}
