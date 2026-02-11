use anyhow::Result;
use ethers::types::Address;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::warn;
use zeroize::Zeroize;

/// Wrapper for private key string that zeroizes on drop
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SecretKey(String);

impl SecretKey {
    pub fn new(key: String) -> Self {
        SecretKey(key)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SecretKey").field(&"[REDACTED]").finish()
    }
}

impl Drop for SecretKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl From<String> for SecretKey {
    fn from(key: String) -> Self {
        SecretKey(key)
    }
}

impl AsRef<str> for SecretKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

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
    pub cors: CorsConfig,
    pub airdrop: AirdropConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age: usize,
    pub allow_credentials: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub rpc_url: String,
    pub chain_id: u64,
    pub contracts: ContractsConfig,
}

impl NetworkConfig {
    pub fn validate(&self) -> Result<()> {
        if self.rpc_url.trim().is_empty() {
            return Err(anyhow::anyhow!("RPC_URL cannot be empty"));
        }

        let url = self.rpc_url.trim();

        if !url.starts_with("https://") && !url.starts_with("http://") {
            return Err(anyhow::anyhow!(
                "RPC_URL must start with https:// or http://, got: {}",
                url
            ));
        }

        let parsed = url::Url::parse(url)
            .map_err(|e| anyhow::anyhow!("Invalid RPC URL format '{}': {}", url, e))?;

        if parsed.scheme() != "https" && parsed.scheme() != "http" {
            return Err(anyhow::anyhow!(
                "RPC_URL must use https:// or http:// scheme, got: {}",
                parsed.scheme()
            ));
        }

        if parsed.host_str().is_none_or(|h| h.is_empty()) {
            return Err(anyhow::anyhow!("RPC_URL must have a valid host"));
        }

        let is_development = url.contains("localhost") || url.contains("127.0.0.1");
        if !is_development && parsed.scheme() != "https" {
            return Err(anyhow::anyhow!(
                "RPC_URL must use HTTPS in production. Got: {}. If this is a local development endpoint, use localhost or 127.0.0.1",
                url
            ));
        }

        if self.chain_id == 0 {
            return Err(anyhow::anyhow!("CHAIN_ID must be non-zero"));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractsConfig {
    pub airdrop_address: String,
    pub token_address: String,
    pub relayer_registry_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayerConfig {
    #[serde(skip)]
    pub private_key: SecretKey,
    pub min_balance_warning: u128,
    pub min_balance_critical: u128,
    pub gas_multiplier: f64,
    pub gas_price_randomization: f64,
    pub max_gas_price: u128,
}

const MAX_GAS_RANDOMIZATION: f64 = 0.20; // 20%
const MIN_GAS_RANDOMIZATION: f64 = 0.0;

impl ContractsConfig {
    pub fn validate(&self) -> Result<()> {
        let zero_address = Address::zero();

        if self.airdrop_address.trim().is_empty() {
            return Err(anyhow::anyhow!("AIRDROP_CONTRACT_ADDRESS is required"));
        }

        if self.token_address.trim().is_empty() {
            return Err(anyhow::anyhow!("TOKEN_CONTRACT_ADDRESS is required"));
        }

        let airdrop_addr = Address::from_str(&self.airdrop_address)
            .map_err(|_| anyhow::anyhow!("Invalid airdrop contract address format"))?;

        let token_addr = Address::from_str(&self.token_address)
            .map_err(|_| anyhow::anyhow!("Invalid token contract address format"))?;

        if airdrop_addr == zero_address {
            return Err(anyhow::anyhow!(
                "AIRDROP_CONTRACT_ADDRESS cannot be zero address (0x0)"
            ));
        }

        if token_addr == zero_address {
            return Err(anyhow::anyhow!(
                "TOKEN_CONTRACT_ADDRESS cannot be zero address (0x0)"
            ));
        }

        if let Some(ref registry_addr) = self.relayer_registry_address {
            if registry_addr.trim().is_empty() {
                return Err(anyhow::anyhow!(
                    "RELAYER_REGISTRY_ADDRESS cannot be empty if provided"
                ));
            }

            let registry = Address::from_str(registry_addr)
                .map_err(|_| anyhow::anyhow!("Invalid relayer registry address format"))?;

            if registry == zero_address {
                return Err(anyhow::anyhow!(
                    "RELAYER_REGISTRY_ADDRESS cannot be zero address (0x0)"
                ));
            }
        }

        Ok(())
    }
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
    pub block_number: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirdropConfig {
    pub claim_amount: u64,
    pub avg_gas_per_claim: u64,
}

impl RateLimitConfig {
    pub fn validate(&self) -> Result<()> {
        if self.per_nullifier == 0 {
            return Err(anyhow::anyhow!(
                "RATE_LIMIT_PER_NULLIFIER must be greater than 0"
            ));
        }
        if self.per_ip == 0 {
            return Err(anyhow::anyhow!("RATE_LIMIT_PER_IP must be greater than 0"));
        }
        if self.global == 0 {
            return Err(anyhow::anyhow!("RATE_LIMIT_GLOBAL must be greater than 0"));
        }
        if self.burst_factor <= 0.0 {
            return Err(anyhow::anyhow!(
                "RATE_LIMIT_BURST_FACTOR must be greater than 0"
            ));
        }
        if self.burst_window == 0 {
            return Err(anyhow::anyhow!(
                "RATE_LIMIT_BURST_WINDOW must be greater than 0"
            ));
        }
        if self.claims_per_minute == 0 {
            return Err(anyhow::anyhow!(
                "RATE_LIMIT_CLAIMS_PER_MINUTE must be greater than 0"
            ));
        }
        if self.requests_per_minute == 0 {
            return Err(anyhow::anyhow!(
                "RATE_LIMIT_REQUESTS_PER_MINUTE must be greater than 0"
            ));
        }
        Ok(())
    }
}

impl MerkleTreeConfig {
    pub fn validate(&self) -> Result<()> {
        if self.merkle_root.is_empty() {
            return Err(anyhow::anyhow!("MERKLE_TREE_ROOT cannot be empty"));
        }

        let zero_root = "0x0000000000000000000000000000000000000000000000000000000000000000000";
        if self.merkle_root == zero_root {
            return Err(anyhow::anyhow!("MERKLE_TREE_ROOT cannot be zero root"));
        }

        if self.source.trim().is_empty() {
            return Err(anyhow::anyhow!("MERKLE_TREE_SOURCE cannot be empty"));
        }

        if self.cache_path.trim().is_empty() {
            return Err(anyhow::anyhow!("MERKLE_TREE_CACHE_PATH cannot be empty"));
        }

        Ok(())
    }
}

impl CorsConfig {
    pub fn validate(&self) -> Result<()> {
        if self.allowed_origins.is_empty() {
            return Err(anyhow::anyhow!("CORS_ALLOWED_ORIGINS cannot be empty"));
        }

        if self.allowed_methods.is_empty() {
            return Err(anyhow::anyhow!("CORS_ALLOWED_METHODS cannot be empty"));
        }

        if self.max_age == 0 {
            return Err(anyhow::anyhow!("CORS_MAX_AGE must be greater than 0"));
        }

        Ok(())
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let config = Self {
            host: std::env::var("RELAYER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("RELAYER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?,
            redis_url: std::env::var("REDIS_URL")
                .map_err(|_| anyhow::anyhow!("REDIS_URL is required"))?,
            network: NetworkConfig {
                rpc_url: std::env::var("RPC_URL")
                    .map_err(|_| anyhow::anyhow!("RPC_URL is required"))?,
                chain_id: std::env::var("CHAIN_ID")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                contracts: ContractsConfig {
                    airdrop_address: std::env::var("AIRDROP_CONTRACT_ADDRESS")
                        .unwrap_or_else(|_| "".to_string()),
                    token_address: std::env::var("TOKEN_CONTRACT_ADDRESS")
                        .unwrap_or_else(|_| "".to_string()),
                    relayer_registry_address: std::env::var("RELAYER_REGISTRY_ADDRESS").ok(),
                },
            },
            relayer: RelayerConfig {
                private_key: {
                    let mut key = std::env::var("RELAYER_PRIVATE_KEY")
                        .map_err(|_| anyhow::anyhow!("RELAYER_PRIVATE_KEY not set"))?;

                    if key.trim().is_empty() {
                        key.zeroize();
                        return Err(anyhow::anyhow!("RELAYER_PRIVATE_KEY cannot be empty"));
                    }

                    let mut normalized_key = key.trim().to_lowercase();
                    key.zeroize();

                    let insecure_keys = [
                        "0x0000000000000000000000000000000000000000000000000000000000000000",
                        "0000000000000000000000000000000000000000000000000000000000000000000",
                        "your_private_key_here",
                        "example_private_key",
                        "0000000000000000000000000000000000000000000000000000000000000000001",
                        "0x0000000000000000000000000000000000000000000000000000000000000000000001",
                    ];

                    let is_insecure = insecure_keys
                        .iter()
                        .any(|insecure_key| normalized_key == *insecure_key);
                    if is_insecure {
                        normalized_key.zeroize();
                        return Err(anyhow::anyhow!(
                            "CRITICAL ERROR: Insecure default private key detected! \
                             Please set RELAYER_PRIVATE_KEY to a secure, randomly generated private key. \
                             Never use example or all-zero private keys in production!"
                        ));
                    }

                    let mut decoded = hex::decode(normalized_key.trim_start_matches("0x"))
                        .map_err(|e| {
                            normalized_key.zeroize();
                            anyhow::anyhow!("Invalid hex private key: encoding error - {}", e)
                        })?;

                    if decoded.len() != 32 {
                        normalized_key.zeroize();
                        decoded.zeroize();
                        return Err(anyhow::anyhow!(
                            "Private key must be 32 bytes, got {}",
                            decoded.len()
                        ));
                    }

                    zkp_airdrop_utils::validate_private_key(decoded.as_slice()).map_err(|e| {
                        normalized_key.zeroize();
                        decoded.zeroize();
                        anyhow::anyhow!("{}", e)
                    })?;

                    SecretKey::new(normalized_key)
                },
                min_balance_warning: {
                    let min_balance_warning_str = std::env::var("RELAYER_MIN_BALANCE_WARNING")
                        .unwrap_or_else(|_| "1000000000000000000".to_string()); // 1 ETH
                    min_balance_warning_str.parse().map_err(|_| {
                        anyhow::anyhow!(
                            "Invalid RELAYER_MIN_BALANCE_WARNING '{}': must be a valid positive integer",
                            min_balance_warning_str
                        )
                    })?
                },
                min_balance_critical: {
                    let min_balance_critical_str = std::env::var("RELAYER_MIN_BALANCE_CRITICAL")
                        .unwrap_or_else(|_| "500000000000000000".to_string()); // 0.5 ETH
                    min_balance_critical_str.parse().map_err(|_| {
                        anyhow::anyhow!(
                            "Invalid RELAYER_MIN_BALANCE_CRITICAL '{}': must be a valid positive integer",
                            min_balance_critical_str
                        )
                    })?
                },
                gas_multiplier: {
                    let multiplier: f64 = std::env::var("RELAYER_GAS_MULTIPLIER")
                        .unwrap_or_else(|_| "1.1".to_string())
                        .parse()
                        .unwrap_or(1.1);
                    const MAX_GAS_MULTIPLIER: f64 = 2.0;
                    const MIN_GAS_MULTIPLIER: f64 = 0.1;
                    if !(MIN_GAS_MULTIPLIER..=MAX_GAS_MULTIPLIER).contains(&multiplier) {
                        return Err(anyhow::anyhow!(
                            "RELAYER_GAS_MULTIPLIER must be between {} and {}, got {}",
                            MIN_GAS_MULTIPLIER,
                            MAX_GAS_MULTIPLIER,
                            multiplier
                        ));
                    }
                    multiplier
                },
                gas_price_randomization: {
                    let randomization: f64 = std::env::var("RELAYER_GAS_RANDOMIZATION")
                        .unwrap_or_else(|_| "0.05".to_string())
                        .parse()
                        .unwrap_or(0.05);
                    if !(MIN_GAS_RANDOMIZATION..=MAX_GAS_RANDOMIZATION).contains(&randomization) {
                        return Err(anyhow::anyhow!(
                            "RELAYER_GAS_RANDOMIZATION must be between {} and {}, got {}",
                            MIN_GAS_RANDOMIZATION,
                            MAX_GAS_RANDOMIZATION,
                            randomization
                        ));
                    }
                    randomization
                },
                max_gas_price: {
                    let max_gas_price_str = std::env::var("RELAYER_MAX_GAS_PRICE")
                        .unwrap_or_else(|_| "50000000000".to_string()); // 50 gwei
                    let max_gas_price: u128 = max_gas_price_str.parse().map_err(|_| {
                        anyhow::anyhow!(
                            "Invalid RELAYER_MAX_GAS_PRICE '{}': must be a valid positive integer",
                            max_gas_price_str
                        )
                    })?;

                    const MAX_SAFE_GAS_PRICE: u128 = 200_000_000_000; // 200 gwei
                    if max_gas_price > MAX_SAFE_GAS_PRICE {
                        return Err(anyhow::anyhow!(
                            "RELAYER_MAX_GAS_PRICE '{}' ({} gwei) exceeds safe maximum of {} gwei. \
                             High gas prices can lead to massive financial loss or DoS attacks. \
                             Please use a reasonable maximum (recommended: 50-100 gwei).",
                            max_gas_price_str,
                            max_gas_price / 1_000_000_000,
                            MAX_SAFE_GAS_PRICE / 1_000_000_000
                        ));
                    }

                    if max_gas_price < 1_000_000_000 {
                        warn!(
                            "RELAYER_MAX_GAS_PRICE '{}' (<1 gwei) may be too low for network congestion",
                            max_gas_price_str
                        );
                    }

                    max_gas_price
                },
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
                source: std::env::var("MERKLE_TREE_SOURCE").unwrap_or_else(|_| "".to_string()),
                cache_path: std::env::var("MERKLE_TREE_CACHE_PATH")
                    .unwrap_or_else(|_| "/data/merkle_tree.bin".to_string()),
                merkle_root: std::env::var("MERKLE_TREE_ROOT").unwrap_or_else(|_| "".to_string()),
                block_number: std::env::var("MERKLE_TREE_BLOCK_NUMBER")
                    .unwrap_or_else(|_| "0".to_string())
                    .parse()
                    .unwrap_or(0),
            },
            cors: CorsConfig {
                allowed_origins: std::env::var("CORS_ALLOWED_ORIGINS")
                    .unwrap_or_else(|_| "https://zkp-airdrop.io".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                allowed_methods: std::env::var("CORS_ALLOWED_METHODS")
                    .unwrap_or_else(|_| "GET,POST,OPTIONS".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                allowed_headers: std::env::var("CORS_ALLOWED_HEADERS")
                    .unwrap_or_else(|_| "Authorization,Accept,Content-Type".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                max_age: std::env::var("CORS_MAX_AGE")
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse()
                    .unwrap_or(3600),
                allow_credentials: std::env::var("CORS_ALLOW_CREDENTIALS")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
            },
            airdrop: AirdropConfig {
                claim_amount: std::env::var("AIRDROP_CLAIM_AMOUNT")
                    .unwrap_or_else(|_| "1000000000000000000".to_string())
                    .parse()
                    .unwrap_or(1_000_000_000_000_000_000),
                avg_gas_per_claim: std::env::var("AIRDROP_AVG_GAS")
                    .unwrap_or_else(|_| "700000".to_string())
                    .parse()
                    .unwrap_or(700_000),
            },
        };
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.relayer.private_key.is_empty() {
            return Err(anyhow::anyhow!("RELAYER_PRIVATE_KEY cannot be empty"));
        }

        if self.host.is_empty() {
            return Err(anyhow::anyhow!("RELAYER_HOST cannot be empty"));
        }

        if self.port == 0 {
            return Err(anyhow::anyhow!("RELAYER_PORT must be greater than 0"));
        }
        if self.port < 1024 {
            return Err(anyhow::anyhow!(
                "RELAYER_PORT must be >= 1024 (non-privileged port)"
            ));
        }

        if self.database_url.trim().is_empty() {
            return Err(anyhow::anyhow!("DATABASE_URL cannot be empty"));
        }

        if self.redis_url.trim().is_empty() {
            return Err(anyhow::anyhow!("REDIS_URL cannot be empty"));
        }

        self.network.validate()?;
        self.network.contracts.validate()?;
        self.rate_limit.validate()?;
        self.merkle_tree.validate()?;
        self.cors.validate()?;

        Ok(())
    }
}
