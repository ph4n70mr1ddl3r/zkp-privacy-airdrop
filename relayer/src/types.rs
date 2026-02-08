use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    pub a: [String; 2],
    pub b: [[String; 2]; 2],
    pub c: [String; 2],
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
