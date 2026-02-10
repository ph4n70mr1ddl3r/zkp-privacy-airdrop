use ethers::contract::abigen;
use num_bigint::BigUint;
use num_traits::Num;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

const MAX_PROOF_SIZE: usize = 8;
const MAX_ELEMENT_LENGTH: usize = 78;
const MAX_PROOF_BYTES: usize = MAX_PROOF_SIZE * MAX_ELEMENT_LENGTH;

const BN254_FIELD_MODULUS: &str =
    "21888242871839275222246405745257275088548364400416034343698204186575808495617";

static FIELD_MODULUS: OnceLock<BigUint> = OnceLock::new();

fn get_field_modulus() -> &'static BigUint {
    FIELD_MODULUS.get_or_init(|| {
        BigUint::from_str_radix(BN254_FIELD_MODULUS, 10).expect("Invalid field modulus constant")
    })
}

abigen!(
    IPLONKVerifier,
    r#"[
        function claim(uint256[8] calldata proof, bytes32 nullifier, address recipient) external returns (uint256)
        function isClaimed(bytes32 nullifier) external view returns (bool)
        function getInstanceCount() external view returns (uint256)
        function getProofElementCount() external view returns (uint256)
    ]"#,
);

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

/// Validates that a string represents a valid field element in BN254 scalar field
fn is_valid_field_element(hex_str: &str) -> bool {
    let hex = hex_str.trim_start_matches("0x");
    if hex.is_empty() {
        return false;
    }

    let bytes = match hex::decode(hex) {
        Ok(b) => b,
        Err(_) => return false,
    };

    if bytes.len() != 32 {
        return false;
    }

    let value = BigUint::from_bytes_be(&bytes);
    value < *get_field_modulus()
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
                let valid_a = proof.a.iter().all(|s| is_valid_field_element(s));
                let valid_c = proof.c.iter().all(|s| is_valid_field_element(s));
                let valid_b = proof
                    .b
                    .iter()
                    .all(|row| row.iter().all(|s| is_valid_field_element(s)));
                valid_a && valid_c && valid_b
            }
            Proof::Plonk(ref proof) => {
                if proof.proof.len() != MAX_PROOF_SIZE {
                    return false;
                }

                let total_bytes: usize = proof.proof.iter().map(|s| s.len()).sum();
                if total_bytes > MAX_PROOF_BYTES {
                    return false;
                }

                let content_ok = proof.proof.iter().all(|s| {
                    !s.is_empty() && s.len() <= MAX_ELEMENT_LENGTH && is_valid_field_element(s)
                });
                content_ok
            }
        }
    }

    /// Estimate proof size in bytes for logging purposes
    #[must_use]
    #[allow(dead_code)]
    pub fn estimated_size_bytes(&self) -> usize {
        match self {
            Proof::Groth16(ref proof) => {
                proof.a.iter().map(|s| s.len()).sum::<usize>()
                    + proof
                        .b
                        .iter()
                        .flat_map(|row| row.iter())
                        .map(|s| s.len())
                        .sum::<usize>()
                    + proof.c.iter().map(|s| s.len()).sum::<usize>()
            }
            Proof::Plonk(ref proof) => proof.proof.iter().map(|s| s.len()).sum(),
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
    #[allow(dead_code)]
    pub fn plonk_minimal() -> Self {
        Self {
            proof: Proof::Plonk(PlonkProof {
                proof: vec!["0".to_string(); 8],
            }),
            recipient: "0x1234567890123456789012345678901234567890".to_string(),
            nullifier: "0x0000000000000000000000000000000000000000000000000000000000".to_string(),
            merkle_root: "0x0000000000000000000000000000000000000000000000000000000000".to_string(),
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
}
