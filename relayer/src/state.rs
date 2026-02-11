use crate::config::Config;
use crate::types_plonk::*;
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::Address;
use parking_lot::RwLock;
use rand::rngs::OsRng;
use rand::Rng;
use redis::aio::ConnectionManager;
use redis::Script;
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;

mod privacy_airdrop_plonk {
    ethers::contract::abigen!(PrivacyAirdropPLONK, "./PrivacyAirdropPLONK.json");
}

const RPC_TIMEOUT_SECONDS: u64 = 10;

/// Pre-compiled Lua script for rate limiting
static RATE_LIMIT_SCRIPT: LazyLock<Script> = LazyLock::new(|| {
    Script::new(
        r#"
        local key = KEYS[1]
        local limit = tonumber(ARGV[1])
        local window_size = tonumber(ARGV[2])
        local current = tonumber(redis.call("GET", key) or "0")

        if current < limit then
            local new_count = redis.call("INCR", key)
            if new_count == 1 then
                redis.call("EXPIRE", key, window_size)
            end
            return {true, new_count}
        else
            return {false, current}
        end
    "#,
    )
});

/// Pre-compiled Lua script for nullifier check-and-set (atomic)
/// Validates recipient matches if nullifier already exists to prevent proof reuse attacks
static NULLIFIER_CHECK_SCRIPT: LazyLock<Script> = LazyLock::new(|| {
    Script::new(
        r#"
        local key = KEYS[1]
        local recipient = ARGV[1]

        -- Check if key exists
        local existing = redis.call("GET", key)

        -- If key doesn't exist, set it atomically
        if not existing then
            redis.call("SET", key, recipient)
            redis.call("EXPIRE", key, 31536000) -- 1 year expiry
            return 1 -- Success
        else
            -- Validate recipient matches to prevent proof reuse across different addresses
            if existing == recipient then
                return 0 -- Already exists with same recipient (valid duplicate)
            else
                -- Different recipient trying to reuse the same nullifier
                redis.call("SET", key, "BLOCKED:"..existing)
                return -1 -- Nullifier already used by different recipient
            end
        end
    "#,
    )
});

const RPC_HEALTH_CHECK_TIMEOUT_SECONDS: u64 = 5;
const RATE_LIMIT_WINDOW_SECONDS: u64 = 120;
const MAX_TRANSACTION_RETRIES: u32 = 3;
const TRANSACTION_RETRY_DELAY_MS: u64 = 1000;
const TOTAL_TRANSACTION_TIMEOUT_SECONDS: u64 = 60;
const BALANCE_CACHE_TTL_SECONDS: u64 = 30;

#[derive(Clone, Copy)]
struct BalanceCache {
    balance: u128,
    timestamp: std::time::Instant,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: PgPool,
    pub redis: Arc<Mutex<ConnectionManager>>,
    pub stats: Arc<RwLock<RelayerStats>>,
    balance_cache: Arc<RwLock<Option<BalanceCache>>>,
}

pub struct RelayerStats {
    pub total_claims: u64,
    pub successful_claims: u64,
    pub failed_claims: u64,
    pub start_time: std::time::Instant,
}

impl Default for RelayerStats {
    fn default() -> Self {
        Self {
            total_claims: 0,
            successful_claims: 0,
            failed_claims: 0,
            start_time: std::time::Instant::now(),
        }
    }
}

impl AppState {
    pub async fn new(
        config: Config,
        db: PgPool,
        redis_conn: ConnectionManager,
    ) -> Result<Self, sqlx::Error> {
        let stats = Arc::new(RwLock::new(RelayerStats::default()));
        let balance_cache = Arc::new(RwLock::new(None));

        Ok(Self {
            config: Arc::new(config),
            db,
            redis: Arc::new(Mutex::new(redis_conn)),
            stats,
            balance_cache,
        })
    }

    pub fn relayer_address(&self) -> Result<String, String> {
        let wallet = LocalWallet::from_str(self.config.relayer.private_key.as_str())
            .map_err(|e| format!("Failed to parse private key: {}", e))?;
        Ok(format!("{:#x}", wallet.address()))
    }

    pub async fn get_relayer_balance(&self) -> u128 {
        {
            let cache_read = self.balance_cache.read();
            if let Some(cached) = *cache_read {
                if cached.timestamp.elapsed().as_secs() < BALANCE_CACHE_TTL_SECONDS {
                    return cached.balance;
                }
            }
        }

        let address_str = match self.relayer_address() {
            Ok(addr) => addr,
            Err(e) => {
                tracing::warn!(
                    "Failed to get relayer address: {}, using fallback balance 0",
                    e
                );
                return 0;
            }
        };
        let address = match Address::from_str(&address_str) {
            Ok(addr) => addr,
            Err(e) => {
                tracing::warn!("Invalid relayer address: {}, using fallback balance 0", e);
                return 0;
            }
        };

        let provider = match Provider::<Http>::try_from(self.config.network.rpc_url.as_str()) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(
                    "Failed to create RPC provider: {}, using fallback balance 0",
                    e
                );
                return 0;
            }
        };

        let balance_result = tokio::time::timeout(
            std::time::Duration::from_secs(RPC_TIMEOUT_SECONDS),
            provider.get_balance(address, None),
        )
        .await;

        let balance = match balance_result {
            Ok(Ok(balance)) => balance.as_u128(),
            Ok(Err(e)) => {
                tracing::warn!("Failed to query balance from RPC: {}, using fallback", e);
                0
            }
            Err(_) => {
                tracing::warn!(
                    "RPC query timed out after {} seconds, using fallback",
                    RPC_TIMEOUT_SECONDS
                );
                0
            }
        };

        {
            let mut cache_write = self.balance_cache.write();
            *cache_write = Some(BalanceCache {
                balance,
                timestamp: std::time::Instant::now(),
            });
        }

        balance
    }

    pub async fn has_sufficient_balance(&self) -> bool {
        let balance = self.get_relayer_balance().await;
        balance > self.config.relayer.min_balance_critical
    }

    pub async fn is_healthy(&self) -> bool {
        let db_healthy = self.check_db_connection().await;
        let redis_healthy = self.check_redis_connection().await;
        let balance_healthy = self.has_sufficient_balance().await;
        let merkle_tree_healthy = self.check_merkle_tree().await;

        db_healthy && redis_healthy && balance_healthy && merkle_tree_healthy
    }

    pub async fn check_db_connection(&self) -> bool {
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            sqlx::query("SELECT 1").fetch_one(&self.db),
        )
        .await
        .is_ok()
    }

    pub async fn check_redis_connection(&self) -> bool {
        use redis::AsyncCommands;
        let mut redis = self.redis.lock().await;
        redis.set::<_, _, ()>("__health_check__", "1").await.is_ok()
    }

    pub async fn get_db_status(&self) -> &'static str {
        if self.check_db_connection().await {
            "connected"
        } else {
            "disconnected"
        }
    }

    pub async fn get_redis_status(&self) -> &'static str {
        if self.check_redis_connection().await {
            "connected"
        } else {
            "disconnected"
        }
    }

    pub async fn get_node_status(&self) -> &'static str {
        if self.check_rpc_connection().await {
            "connected"
        } else {
            "disconnected"
        }
    }

    async fn check_rpc_connection(&self) -> bool {
        use ethers::providers::Provider;
        match Provider::<Http>::try_from(self.config.network.rpc_url.as_str()) {
            Ok(provider) => tokio::time::timeout(
                std::time::Duration::from_secs(RPC_HEALTH_CHECK_TIMEOUT_SECONDS),
                provider.get_block_number(),
            )
            .await
            .ok()
            .and_then(|r| r.ok())
            .is_some(),
            Err(_) => false,
        }
    }

    pub async fn check_merkle_tree(&self) -> bool {
        let root = &self.config.merkle_tree.merkle_root;
        if root.is_empty() {
            tracing::error!("Merkle tree root is empty");
            return false;
        }

        if !root.starts_with("0x") {
            tracing::error!("Merkle tree root must start with 0x");
            return false;
        }

        if root.len() < 2 {
            tracing::error!(
                "Merkle tree root must be at least 2 characters, got {}",
                root.len()
            );
            return false;
        }

        match hex::decode(&root[2..]) {
            Ok(bytes) => {
                if bytes.len() != 32 {
                    tracing::error!("Merkle tree root must be 32 bytes, got {}", bytes.len());
                    return false;
                }
            }
            Err(e) => {
                tracing::error!("Failed to decode merkle tree root: {}", e);
                return false;
            }
        }

        true
    }

    pub async fn check_rate_limit(
        &self,
        key: &str,
        limit_type: RateLimitType,
    ) -> Result<(), String> {
        use redis::AsyncCommands;

        let limit = match limit_type {
            RateLimitType::SubmitClaim => self.config.rate_limit.claims_per_minute,
            RateLimitType::GetMerklePath
            | RateLimitType::CheckStatus
            | RateLimitType::HealthCheck => self.config.rate_limit.requests_per_minute,
        };

        let nullifier_prefix = match limit_type {
            RateLimitType::SubmitClaim => "submit_claim",
            RateLimitType::GetMerklePath => "get_merkle_path",
            RateLimitType::CheckStatus => "check_status",
            RateLimitType::HealthCheck => "health_check",
        };
        let redis_key = format!("rate_limit:{}:{}", nullifier_prefix, key);

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();
        let window_start = current_time - (current_time % 60);

        let count_key = format!("{}:{}", redis_key, window_start);
        let mut redis = self.redis.lock().await;

        let result: (bool, u64) = RATE_LIMIT_SCRIPT
            .key(&count_key)
            .arg(limit)
            .arg(RATE_LIMIT_WINDOW_SECONDS)
            .invoke_async(&mut *redis)
            .await
            .map_err(|e| e.to_string())?;

        if !result.0 {
            let exponential_backoff_key = format!("{}:backoff", redis_key);
            let backoff_count: Option<u64> =
                redis.get(&exponential_backoff_key).await.ok().flatten();

            let base_wait_seconds = 60u64;
            let multiplier = backoff_count.unwrap_or(0).min(5);
            let wait_seconds = base_wait_seconds * (1 << multiplier);

            let new_backoff_count = backoff_count.map(|c| c + 1).unwrap_or(1);
            let _: Result<(), _> = redis
                .set_ex(&exponential_backoff_key, new_backoff_count, 300)
                .await;

            return Err(format!(
                "Rate limit exceeded: {}/min. Please wait {} seconds before retrying.",
                limit, wait_seconds
            ));
        }

        let exponential_backoff_key = format!("{}:backoff", redis_key);
        let _: Result<(), _> = redis.del(&exponential_backoff_key).await;

        Ok(())
    }

    pub async fn submit_claim(&self, claim: &SubmitClaimRequest) -> Result<String, String> {
        let provider = Provider::<Http>::try_from(self.config.network.rpc_url.as_str())
            .inspect_err(|e| {
                self.increment_failed_claims();
                tracing::error!(
                    "Failed to create RPC provider from {}: {}",
                    self.config.network.rpc_url,
                    e
                );
            })
            .map_err(|e| e.to_string())?;

        let provider = Arc::new(provider);

        let wallet =
            LocalWallet::from_str(self.config.relayer.private_key.as_str()).map_err(|e| {
                self.increment_failed_claims();
                format!("Failed to create wallet from private key: {}", e)
            })?;

        let airdrop_address = Address::from_str(&self.config.network.contracts.airdrop_address)
            .map_err(|e| {
                self.increment_failed_claims();
                format!(
                    "Invalid airdrop address '{}': {}",
                    self.config.network.contracts.airdrop_address, e
                )
            })?;

        let recipient = Address::from_str(&claim.recipient).map_err(|e| {
            self.increment_failed_claims();
            format!("Invalid recipient address '{}': {}", claim.recipient, e)
        })?;

        let nullifier_str = claim.nullifier.trim();
        if nullifier_str.len() != 66 {
            self.increment_failed_claims();
            return Err(format!(
                "Invalid nullifier length: expected 66 characters, got {}",
                nullifier_str.len()
            ));
        }

        if !nullifier_str.starts_with("0x") && !nullifier_str.starts_with("0X") {
            self.increment_failed_claims();
            return Err("Invalid nullifier format: must start with 0x".to_string());
        }

        let nullifier_bytes = hex::decode(&nullifier_str[2..]).map_err(|e| {
            self.increment_failed_claims();
            format!("Invalid nullifier hex encoding: {}", e)
        })?;

        let nullifier_array: [u8; 32] = nullifier_bytes[..].try_into().map_err(|e| {
            self.increment_failed_claims();
            format!("Invalid nullifier length: expected 32 bytes, got {}", e)
        })?;

        let zero_nullifier = [0u8; 32];
        if nullifier_array == zero_nullifier {
            self.increment_failed_claims();
            return Err("Invalid nullifier: cannot be all zeros".to_string());
        }

        let key = format!("nullifier:{}", claim.nullifier);
        let mut redis = self.redis.lock().await;

        let result: i32 = NULLIFIER_CHECK_SCRIPT
            .key(&key)
            .arg(&claim.recipient)
            .invoke_async(&mut *redis)
            .await
            .map_err(|e| {
                self.increment_failed_claims();
                format!("Redis error during nullifier check: {}", e)
            })?;

        if result != 1 {
            self.increment_failed_claims();
            return Err(if result == -1 {
                "Security violation: This nullifier has already been used by a different address. Proof reuse detected and blocked."
                    .to_string()
            } else {
                "This nullifier has already been used. Each qualified account can only claim once."
                    .to_string()
            });
        }

        drop(redis);

        let tx_hash: ethers::types::H256 = match &claim.proof {
            crate::types_plonk::Proof::Plonk(proof) => {
                let chain_id = tokio::time::timeout(
                    std::time::Duration::from_secs(RPC_TIMEOUT_SECONDS),
                    provider.get_chainid(),
                )
                .await
                .map_err(|_| {
                    self.increment_failed_claims();
                    format!(
                        "Failed to get chain ID: timeout after {} seconds",
                        RPC_TIMEOUT_SECONDS
                    )
                })?
                .map_err(|e| {
                    self.increment_failed_claims();
                    format!("Failed to get chain ID: {}", e)
                })?;

                let wallet_with_chain = wallet.with_chain_id(chain_id.as_u32());
                let plonk_verifier = privacy_airdrop_plonk::PrivacyAirdropPLONK::new(
                    airdrop_address,
                    provider.clone(),
                );

                if proof.proof.len() != 8 {
                    self.increment_failed_claims();
                    return Err(format!(
                        "Invalid PLONK proof: expected 8 elements, got {}",
                        proof.proof.len()
                    ));
                }

                let parsed_elements: Vec<ethers::types::U256> = proof
                    .proof
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        ethers::types::U256::from_str_radix(s, 16).map_err(|e| {
                            format!("Invalid proof element at index {}: '{}': {}", i, s, e)
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .inspect_err(|_| {
                        self.increment_failed_claims();
                    })?;

                let parsed_len = parsed_elements.len();

                if parsed_len != 8 {
                    self.increment_failed_claims();
                    return Err(format!(
                        "Invalid PLONK proof length: expected 8 elements, got {} after parsing",
                        parsed_len
                    ));
                }

                let proof_array: [ethers::types::U256; 8] =
                    parsed_elements.try_into().map_err(|_e| {
                        self.increment_failed_claims();
                        format!(
                            "Failed to convert proof to array: expected 8 elements, got {}",
                            parsed_len
                        )
                    })?;

                let nonce = tokio::time::timeout(
                    std::time::Duration::from_secs(RPC_TIMEOUT_SECONDS),
                    provider.get_transaction_count(wallet_with_chain.address(), None),
                )
                .await
                .map_err(|_| {
                    self.increment_failed_claims();
                    format!(
                        "Failed to get transaction nonce: timeout after {} seconds",
                        RPC_TIMEOUT_SECONDS
                    )
                })?
                .map_err(|e| {
                    self.increment_failed_claims();
                    format!("Failed to get transaction nonce: {}", e)
                })?;

                let mut retry_count = 0;
                let mut current_nonce = nonce;
                let total_start_time = std::time::Instant::now();

                loop {
                    if total_start_time.elapsed().as_secs() > TOTAL_TRANSACTION_TIMEOUT_SECONDS {
                        self.increment_failed_claims();
                        return Err(format!(
                            "Transaction submission exceeded total timeout of {} seconds",
                            TOTAL_TRANSACTION_TIMEOUT_SECONDS
                        ));
                    }

                    let retry_delay = std::cmp::min(
                        TRANSACTION_RETRY_DELAY_MS.saturating_mul(1u64 << retry_count.min(10)),
                        TRANSACTION_RETRY_DELAY_MS * 8,
                    );

                    if retry_count > 0 {
                        current_nonce = tokio::time::timeout(
                            std::time::Duration::from_secs(RPC_TIMEOUT_SECONDS),
                            provider.get_transaction_count(wallet_with_chain.address(), None),
                        )
                        .await
                        .map_err(|_| {
                            self.increment_failed_claims();
                            format!(
                                "Failed to get transaction nonce on retry: timeout after {} seconds",
                                RPC_TIMEOUT_SECONDS
                            )
                        })?
                        .map_err(|e| {
                            self.increment_failed_claims();
                            format!("Failed to get transaction nonce on retry: {}", e)
                        })?;
                    }

                    let call = plonk_verifier.claim(
                        privacy_airdrop_plonk::Plonkproof { proof: proof_array },
                        nullifier_array,
                        recipient,
                    );

                    let base_gas_price = tokio::time::timeout(
                        std::time::Duration::from_secs(RPC_TIMEOUT_SECONDS),
                        provider.get_gas_price(),
                    )
                    .await
                    .map_err(|_e| {
                        self.increment_failed_claims();
                        format!(
                            "Failed to get gas price: timeout after {} seconds",
                            RPC_TIMEOUT_SECONDS
                        )
                    })?
                    .map_err(|e| {
                        self.increment_failed_claims();
                        format!("Failed to get gas price: {}", e)
                    })?;

                    let base_gas_price_u128 = base_gas_price.as_u128();
                    if base_gas_price_u128 == 0 {
                        self.increment_failed_claims();
                        return Err("Invalid gas price: base gas price is zero".to_string());
                    }

                    const MAX_GAS_RANDOMIZATION_PERCENT: u64 = 20;
                    let gas_randomization_percent = ((self.config.relayer.gas_price_randomization
                        * 100.0) as u64)
                        .min(MAX_GAS_RANDOMIZATION_PERCENT);
                    let random_factor = OsRng.gen_range(0..=gas_randomization_percent);
                    let adjustment_multiplier = 100u64 + random_factor;

                    let adjusted_price = base_gas_price_u128
                        .checked_mul(adjustment_multiplier as u128)
                        .and_then(|v| v.checked_div(100))
                        .ok_or_else(|| {
                            "Gas price calculation overflow: multiplication or division failed"
                                .to_string()
                        })?;

                    if adjusted_price == 0 {
                        self.increment_failed_claims();
                        return Err("Gas price calculation resulted in zero value".to_string());
                    }

                    let max_gas_price =
                        ethers::types::U256::from(self.config.relayer.max_gas_price);

                    let mut gas_price = ethers::types::U256::from(adjusted_price);

                    if gas_price > max_gas_price {
                        tracing::warn!(
                            "Computed gas price {} gwei exceeds max {} gwei, using max",
                            gas_price.as_u128() / 1_000_000_000,
                            max_gas_price.as_u128() / 1_000_000_000
                        );
                        gas_price = max_gas_price;
                    }

                    if gas_price.as_u128() < 1_000_000_000 {
                        tracing::warn!(
                            "Final gas price {} wei is less than 1 gwei, using 1 gwei minimum",
                            gas_price.as_u128()
                        );
                        gas_price = ethers::types::U256::from(1_000_000_000u128);
                    }

                    let builder = call
                        .from(wallet_with_chain.address())
                        .nonce(current_nonce)
                        .gas_price(gas_price);

                    let send_result: Result<_, tokio::time::error::Elapsed> = tokio::time::timeout(
                        std::time::Duration::from_secs(RPC_TIMEOUT_SECONDS),
                        builder.send(),
                    )
                    .await;

                    match send_result {
                        Ok(Ok(pending_tx)) => {
                            break pending_tx.tx_hash();
                        }
                        Ok(Err(e)) => {
                            retry_count += 1;
                            if retry_count >= MAX_TRANSACTION_RETRIES {
                                self.increment_failed_claims();
                                return Err(format!(
                                    "Failed to submit PLONK claim after {} retries: {}",
                                    MAX_TRANSACTION_RETRIES, e
                                ));
                            }
                            tracing::warn!(
                                "Transaction failed (attempt {}/{}), retrying in {}ms: {}",
                                retry_count,
                                MAX_TRANSACTION_RETRIES,
                                retry_delay,
                                e
                            );
                            tokio::time::sleep(tokio::time::Duration::from_millis(retry_delay))
                                .await;
                        }
                        Err(_) => {
                            retry_count += 1;
                            if retry_count >= MAX_TRANSACTION_RETRIES {
                                self.increment_failed_claims();
                                return Err(format!(
                                    "Failed to submit PLONK claim after {} retries: timeout after {} seconds",
                                    MAX_TRANSACTION_RETRIES, RPC_TIMEOUT_SECONDS));
                            }
                            tracing::warn!(
                                "Transaction timed out (attempt {}/{}), retrying in {}ms",
                                retry_count,
                                MAX_TRANSACTION_RETRIES,
                                retry_delay
                            );
                            tokio::time::sleep(tokio::time::Duration::from_millis(retry_delay))
                                .await;
                        }
                    }
                }
            }
            crate::types_plonk::Proof::Groth16(_) => {
                self.increment_failed_claims();
                return Err(
                    "Groth16 proofs are no longer supported. Please use PLONK proofs.".to_string(),
                );
            }
        };

        {
            let mut redis = self.redis.lock().await;
            use redis::AsyncCommands;
            let timestamp = chrono::Utc::now().to_rfc3339();

            let tx_key = format!("{}:tx_hash", key);
            let timestamp_key = format!("{}:timestamp", key);

            redis
                .set::<_, _, ()>(&tx_key, tx_hash.to_string())
                .await
                .map_err(|e| {
                    tracing::error!("Failed to store tx_hash in Redis: {}", e);
                    "Internal storage error: failed to record transaction".to_string()
                })?;

            redis
                .set::<_, _, ()>(&timestamp_key, &timestamp)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to store timestamp in Redis: {}", e);
                    "Internal storage error: failed to record transaction".to_string()
                })?;

            let mut stats = self.stats.write();
            stats.total_claims += 1;
            stats.successful_claims += 1;
        }

        Ok(tx_hash.to_string())
    }

    pub async fn get_claim_status(&self, nullifier: &str) -> Option<CheckStatusResponse> {
        use redis::AsyncCommands;
        let key = format!("nullifier:{}", nullifier);
        let mut redis = self.redis.lock().await;

        let recipient: Option<String> = redis.get(&key).await.ok().flatten()?;

        let tx_key = format!("{}:tx_hash", key);
        let tx_hash: Option<String> = redis.get(&tx_key).await.ok().flatten();

        let block_key = format!("{}:block", key);
        let block_number: Option<u64> = redis.get(&block_key).await.ok().flatten();

        let timestamp_key = format!("{}:timestamp", key);
        let timestamp: Option<String> = redis.get(&timestamp_key).await.ok().flatten();

        Some(CheckStatusResponse {
            nullifier: nullifier.to_string(),
            claimed: true,
            tx_hash,
            recipient,
            timestamp,
            block_number,
        })
    }

    pub async fn get_merkle_path(&self, address: &str) -> Option<MerklePathResponse> {
        use redis::AsyncCommands;
        let key = format!("merkle_path:{}", address);
        let mut redis = self.redis.lock().await;

        let leaf_index: Option<u64> = redis.get(format!("{}:index", key)).await.ok().flatten()?;
        let merkle_path: Option<String> =
            redis.get(format!("{}:path", key)).await.ok().flatten()?;
        let path_indices: Option<String> =
            redis.get(format!("{}:indices", key)).await.ok().flatten()?;

        let path_data: Vec<String> = serde_json::from_str(&merkle_path?).ok()?;
        let indices_data: Vec<u8> = serde_json::from_str(&path_indices?).ok()?;

        Some(MerklePathResponse {
            address: address.to_string(),
            leaf_index: leaf_index?,
            merkle_path: path_data,
            path_indices: indices_data,
            root: self.config.merkle_tree.merkle_root.clone(),
        })
    }

    pub async fn get_stats(&self) -> StatsResponse {
        let (total_claims, successful_claims, failed_claims, uptime) = {
            let stats = self.stats.read();
            (
                stats.total_claims,
                stats.successful_claims,
                stats.failed_claims,
                stats.start_time.elapsed().as_secs_f64(),
            )
        };

        let claim_amount = self.config.airdrop.claim_amount;
        let avg_gas = self.config.airdrop.avg_gas_per_claim;

        let total_tokens_distributed = successful_claims.saturating_mul(claim_amount);

        let total_gas_used = successful_claims.saturating_mul(avg_gas);

        let avg_gas_price = self.get_average_gas_price().await;

        StatsResponse {
            total_claims,
            successful_claims,
            failed_claims,
            total_tokens_distributed: total_tokens_distributed.to_string(),
            unique_recipients: successful_claims,
            average_gas_price: avg_gas_price.to_string(),
            total_gas_used: total_gas_used.to_string(),
            relayer_balance: self.get_relayer_balance().await.to_string(),
            uptime_percentage: if uptime > 0.0 { 100.0 } else { 0.0 },
            response_time_ms: ResponseTime {
                p50: 150,
                p95: 500,
                p99: 1200,
            },
        }
    }

    async fn get_average_gas_price(&self) -> u128 {
        match Provider::<Http>::try_from(self.config.network.rpc_url.as_str()) {
            Ok(provider) => tokio::time::timeout(
                std::time::Duration::from_secs(RPC_TIMEOUT_SECONDS),
                provider.get_gas_price(),
            )
            .await
            .ok()
            .and_then(|r| r.ok())
            .map(|p| p.as_u128())
            .unwrap_or(25_000_000_000),
            Err(_) => 25_000_000_000,
        }
    }

    pub fn increment_failed_claims(&self) {
        let mut stats = self.stats.write();
        stats.total_claims += 1;
        stats.failed_claims += 1;
    }
}
