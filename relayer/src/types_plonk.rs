use serde::{Deserialize, Serialize};

/// Groth16 proof format (deprecated, kept for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Groth16Proof {
    pub a: [String; 2],
    pub b: [[String; 2]; 2],
    pub c: [String; 2],
}

/// PLONK proof format (new format for universal trusted setup)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlonkProof {
    pub proof: Vec<String>, // Flat array of 8+ field elements
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
    #[must_use]
    pub fn type_name(&self) -> &str {
        match self {
            Proof::Groth16(_) => "Groth16",
            Proof::Plonk(_) => "Plonk",
        }
    }

    /// Validate proof structure
    #[must_use]
    pub fn is_valid_structure(&self) -> bool {
        match self {
            Proof::Groth16(ref proof) => {
                proof.a.iter().all(|s| !s.is_empty())
                    && proof.c.iter().all(|s| !s.is_empty())
                    && proof.b.iter().all(|row| row.iter().all(|s| !s.is_empty()))
            }
            Proof::Plonk(ref proof) => {
                proof.proof.len() >= 8 && proof.proof.iter().all(|s| !s.is_empty())
            }
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

impl SubmitClaimRequest {
    /// Create a minimal PLONK request for testing
    #[must_use]
    pub fn plonk_minimal() -> Self {
        Self {
            proof: Proof::Plonk(PlonkProof {
                proof: vec!["0".to_string(); 8],
            }),
            recipient: "0x1234567890123456789012345678901234567890".to_string(),
            nullifier: "0x0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            merkle_root: "0x0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
        }
    }
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
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub code: Option<String>,
    pub retry_after: Option<u64>,
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
pub struct MerkleRootResponse {
    pub merkle_root: String,
    pub block_number: u64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDetails {
    pub address: String,
    pub deployed_at: Option<String>,
    pub block_number: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenDetails {
    pub address: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractsInfo {
    pub airdrop: ContractDetails,
    pub token: TokenDetails,
    pub relayer_registry: Option<ContractDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfoResponse {
    pub network: String,
    pub chain_id: u64,
    pub contracts: ContractsInfo,
    pub claim_amount: String,
    pub claim_deadline: String,
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
pub struct ResponseTime {
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
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
pub struct Services {
    pub database: String,
    pub redis: String,
    pub optimism_node: String,
    pub relayer_wallet: RelayerWalletInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayerWalletInfo {
    pub address: String,
    pub balance: String,
    pub sufficient: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub services: Services,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerklePathResponse {
    pub address: String,
    pub leaf_index: u64,
    pub merkle_path: Vec<String>,
    pub path_indices: Vec<u8>,
    pub root: String,
}

pub enum RateLimitType {
    SubmitClaim,
    GetMerklePath,
    CheckStatus,
}
