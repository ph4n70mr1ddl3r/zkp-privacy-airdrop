use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub network: NetworkConfig,
    pub relayer: RelayerConfig,
    pub rate_limit: RateLimitConfig,
    pub merkle_tree: MerkleTreeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub rpc_url: String,
    pub chain_id: u64,
    pub contracts: ContractsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractsConfig {
    pub airdrop_address: String,
    pub token_address: String,
    pub relayer_registry_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayerConfig {
    pub private_key: String,
    pub min_balance_warning: String,
    pub min_balance_critical: String,
    pub gas_multiplier: f64,
    pub gas_price_randomization: f64,
    pub max_gas_price: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub per_nullifier: u64,
    pub per_ip: u64,
    pub global: u64,
    pub burst_factor: f64,
    pub burst_window: u64,
    pub claims_per_minute: u64,
    pub requests_per_minute: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTreeConfig {
    pub source: String,
    pub cache_path: String,
    pub merkle_root: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: std::env::var("RELAYER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("RELAYER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://postgres:postgres@localhost:5432/zkp_airdrop".to_string()
            }),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            network: NetworkConfig {
                rpc_url: std::env::var("RPC_URL")
                    .unwrap_or_else(|_| "https://optimism.drpc.org".to_string()),
                chain_id: std::env::var("CHAIN_ID")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                contracts: ContractsConfig {
                    airdrop_address: std::env::var("AIRDROP_CONTRACT_ADDRESS").unwrap_or_else(
                        |_| "0x0000000000000000000000000000000000000000".to_string(),
                    ),
                    token_address: std::env::var("TOKEN_CONTRACT_ADDRESS").unwrap_or_else(|_| {
                        "0x0000000000000000000000000000000000000000".to_string()
                    }),
                    relayer_registry_address: std::env::var("RELAYER_REGISTRY_ADDRESS").ok(),
                },
            },
            relayer: RelayerConfig {
                private_key: std::env::var("RELAYER_PRIVATE_KEY").unwrap_or_else(|_| {
                    "0x0000000000000000000000000000000000000000000000000000000000000000".to_string()
                }),
                min_balance_warning: std::env::var("RELAYER_MIN_BALANCE_WARNING")
                    .unwrap_or_else(|_| "1000000000000000000".to_string()), // 1 ETH
                min_balance_critical: std::env::var("RELAYER_MIN_BALANCE_CRITICAL")
                    .unwrap_or_else(|_| "500000000000000000".to_string()), // 0.5 ETH
                gas_multiplier: std::env::var("RELAYER_GAS_MULTIPLIER")
                    .unwrap_or_else(|_| "1.1".to_string())
                    .parse()
                    .unwrap_or(1.1),
                gas_price_randomization: std::env::var("RELAYER_GAS_RANDOMIZATION")
                    .unwrap_or_else(|_| "0.05".to_string())
                    .parse()
                    .unwrap_or(0.05),
                max_gas_price: std::env::var("RELAYER_MAX_GAS_PRICE")
                    .unwrap_or_else(|_| "100000000".to_string()), // 0.1 gwei
            },
            rate_limit: RateLimitConfig {
                per_nullifier: std::env::var("RATE_LIMIT_PER_NULLIFIER")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .unwrap_or(60),
                per_ip: std::env::var("RATE_LIMIT_PER_IP")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
                global: std::env::var("RATE_LIMIT_GLOBAL")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()
                    .unwrap_or(1000),
                burst_factor: std::env::var("RATE_LIMIT_BURST_FACTOR")
                    .unwrap_or_else(|_| "2.0".to_string())
                    .parse()
                    .unwrap_or(2.0),
                burst_window: std::env::var("RATE_LIMIT_BURST_WINDOW")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                claims_per_minute: std::env::var("RATE_LIMIT_CLAIMS_PER_MINUTE")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
                requests_per_minute: std::env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
            },
            merkle_tree: MerkleTreeConfig {
                source: std::env::var("MERKLE_TREE_SOURCE")
                    .unwrap_or_else(|_| "https://api.merkle-tree.io/tree.json".to_string()),
                cache_path: std::env::var("MERKLE_TREE_CACHE_PATH")
                    .unwrap_or_else(|_| "/data/merkle_tree.bin".to_string()),
                merkle_root: std::env::var("MERKLE_TREE_ROOT").unwrap_or_else(|_| {
                    "0x0000000000000000000000000000000000000000000000000000000000000000".to_string()
                }),
            },
        })
    }
}
