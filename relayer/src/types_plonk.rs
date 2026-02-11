use ethers::contract::abigen;
use serde::{Deserialize, Serialize};

/// Exact number of field elements in a PLONK proof
/// PLONK proofs for this circuit contain exactly 8 elements: A, B, C, Z, T1, T2, T3, `WXi`
const PLONK_PROOF_SIZE: usize = 8;

/// Maximum length of a single proof element string (in characters)
/// BN254 field elements can be up to 78 chars when represented as hex:
/// 0x + up to 64 hex chars (32 bytes * 2 chars/byte)
const MAX_ELEMENT_LENGTH: usize = 78;

/// Maximum total bytes allowed for proof data
/// Prevents memory exhaustion from oversized proofs
const MAX_PROOF_BYTES: usize = PLONK_PROOF_SIZE * MAX_ELEMENT_LENGTH;

abigen!(
    IPLONKVerifier,
    r#"[
        function claim(uint256[8] calldata proof, bytes32 nullifier, address recipient) external returns (uint256)
        function isClaimed(bytes32 nullifier) external view returns (bool)
        function getInstanceCount() external view returns (uint256)
        function getProofElementCount() external view returns (uint256)
    ]"#,
);

/// PLONK proof format (new format for universal trusted setup)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlonkProof {
    pub proof: Vec<String>, // Flat array of 8+ field elements
}

/// Union type for different proof systems
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Proof {
    Plonk(PlonkProof),
}

impl Proof {
    #[must_use]
    pub fn type_name(&self) -> &str {
        match self {
            Proof::Plonk(_) => "Plonk",
        }
    }

    #[must_use]
    pub fn is_valid_structure(&self) -> bool {
        match self {
            Proof::Plonk(ref proof) => {
                if proof.proof.is_empty() {
                    return false;
                }
                if proof.proof.len() != PLONK_PROOF_SIZE {
                    return false;
                }

                for (idx, element) in proof.proof.iter().enumerate() {
                    if element.trim().is_empty() {
                        tracing::warn!("PLONK proof element at index {} is empty", idx);
                        return false;
                    }
                    if !element.starts_with("0x") && !element.starts_with("0X") {
                        tracing::warn!("PLONK proof element at index {} missing 0x prefix", idx);
                        return false;
                    }
                    if element.len() > MAX_ELEMENT_LENGTH {
                        tracing::warn!(
                            "PLONK proof element at index {} too long: {}",
                            idx,
                            element.len()
                        );
                        return false;
                    }
                    if !zkp_airdrop_utils::is_valid_field_element(element) {
                        tracing::warn!(
                            "PLONK proof element at index {} is invalid field element",
                            idx
                        );
                        return false;
                    }
                }

                let total_bytes: usize = proof.proof.iter().map(std::string::String::len).sum();
                if total_bytes > MAX_PROOF_BYTES {
                    return false;
                }

                true
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
    pub merkle_tree: String,
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
    HealthCheck,
}
