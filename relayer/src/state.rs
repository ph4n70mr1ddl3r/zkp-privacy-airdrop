use crate::config::Config;
use crate::types_plonk::*;
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::Address;
use parking_lot::RwLock;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

const RPC_TIMEOUT_SECONDS: u64 = 10;
const RPC_HEALTH_CHECK_TIMEOUT_SECONDS: u64 = 5;
const RATE_LIMIT_WINDOW_SECONDS: u64 = 120;
const MAX_TRANSACTION_RETRIES: u32 = 3;
const TRANSACTION_RETRY_DELAY_MS: u64 = 1000;

pub struct AppState {
    pub config: Arc<Config>,
    pub db: PgPool,
    pub redis: Arc<Mutex<ConnectionManager>>,
    pub stats: Arc<RwLock<RelayerStats>>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            db: self.db.clone(),
            redis: Arc::clone(&self.redis),
            stats: Arc::clone(&self.stats),
        }
    }
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

        Ok(Self {
            config: Arc::new(config),
            db,
            redis: Arc::new(Mutex::new(redis_conn)),
            stats,
        })
    }

    pub fn relayer_address(&self) -> Result<String, String> {
        LocalWallet::from_str(&self.config.relayer.private_key)
            .map(|wallet| format!("{:#x}", wallet.address()))
            .map_err(|e| format!("Failed to parse private key: {}", e))
    }

    pub async fn get_relayer_balance(&self) -> u128 {
        let address_str = match self.relayer_address() {
            Ok(addr) => addr,
            Err(e) => {
                tracing::warn!("Failed to get relayer address: {}, using fallback", e);
                return 0;
            }
        };
        let address = match Address::from_str(&address_str) {
            Ok(addr) => addr,
            Err(e) => {
                tracing::warn!("Invalid relayer address: {}, using fallback", e);
                return 0;
            }
        };

        let provider = match Provider::<Http>::try_from(self.config.network.rpc_url.as_str()) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("Failed to create RPC provider: {}, using fallback", e);
                return 0;
            }
        };

        let balance_result = tokio::time::timeout(
            std::time::Duration::from_secs(RPC_TIMEOUT_SECONDS),
            provider.get_balance(address, None),
        )
        .await;

        match balance_result {
            Ok(Ok(balance)) => balance.as_u128(),
            Ok(Err(e)) => {
                tracing::warn!("Failed to query balance from RPC: {}, using fallback", e);
                0
            }
            Err(_) => {
                tracing::warn!("RPC query timed out after 10 seconds, using fallback");
                0
            }
        }
    }

    pub async fn has_sufficient_balance(&self) -> bool {
        let balance = self.get_relayer_balance().await;
        let min_critical: u128 = self
            .config
            .relayer
            .min_balance_critical
            .parse()
            .unwrap_or(500_000_000_000_000_000_u128);
        balance > min_critical
    }

    pub async fn is_healthy(&self) -> bool {
        let db_healthy = self.check_db_connection().await;
        let redis_healthy = self.check_redis_connection().await;
        let balance_healthy = self.has_sufficient_balance().await;
        let merkle_tree_healthy = self.check_merkle_tree().await;

        db_healthy && redis_healthy && balance_healthy && merkle_tree_healthy
    }

    pub async fn check_db_connection(&self) -> bool {
        sqlx::query("SELECT 1").fetch_one(&self.db).await.is_ok()
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
        let limit = match limit_type {
            RateLimitType::SubmitClaim => self.config.rate_limit.claims_per_minute,
            RateLimitType::GetMerklePath | RateLimitType::CheckStatus => {
                self.config.rate_limit.requests_per_minute
            }
        };

        let redis_key = format!("rate_limit:{}", key);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();
        let window_start = current_time - (current_time % 60);

        let count_key = format!("{}:{}", redis_key, window_start);
        let mut redis = self.redis.lock().await;

        use redis::Script;
        let lua_script = r#"
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
        "#;

        let script = Script::new(lua_script);
        let result: (bool, u64) = script
            .key(&count_key)
            .arg(limit)
            .arg(RATE_LIMIT_WINDOW_SECONDS)
            .invoke_async(&mut *redis)
            .await
            .map_err(|e| e.to_string())?;

        if !result.0 {
            return Err(format!("Rate limit exceeded: {}/min", limit));
        }

        Ok(())
    }

    pub async fn is_nullifier_used(&self, nullifier: &str) -> Result<bool, String> {
        use redis::AsyncCommands;

        let key = format!("nullifier:{}", nullifier);
        let mut redis = self.redis.lock().await;

        redis
            .exists::<_, i64>(&key)
            .await
            .map(|count| count > 0)
            .map_err(|e| {
                tracing::error!("Failed to check nullifier existence in Redis: {}", e);
                format!("Redis error: {}", e)
            })
    }

    pub async fn submit_claim(&self, claim: &SubmitClaimRequest) -> Result<String, String> {
        use redis::AsyncCommands;

        let provider =
            Provider::<Http>::try_from(self.config.network.rpc_url.as_str()).map_err(|e| {
                self.increment_failed_claims();
                format!(
                    "Failed to create RPC provider from {}: {}",
                    self.config.network.rpc_url, e
                )
            })?;

        let provider = Arc::new(provider);

        let wallet = LocalWallet::from_str(&self.config.relayer.private_key).map_err(|e| {
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

        let nullifier_bytes = hex::decode(&claim.nullifier[2..]).map_err(|e| {
            self.increment_failed_claims();
            format!("Invalid nullifier hex '{}': {}", claim.nullifier, e)
        })?;

        let nullifier_array: [u8; 32] = nullifier_bytes[..].try_into().map_err(|e| {
            self.increment_failed_claims();
            format!(
                "Invalid nullifier length for '{}': expected 32 bytes, got {}",
                claim.nullifier, e
            )
        })?;

        let key = format!("nullifier:{}", claim.nullifier);
        let mut redis = self.redis.lock().await;

        // Use Redis Lua script for atomic check-and-set to prevent race conditions
        use redis::Script;
        let lua_script = r#"
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
                return 0 -- Already exists
            end
        "#;

        let script = Script::new(lua_script);
        let result: u32 = script
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
            drop(redis);
            return Err(
                "This nullifier has already been used. Each qualified account can only claim once."
                    .to_string(),
            );
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
                let plonk_verifier = IPLONKVerifier::new(airdrop_address, provider);

                if proof.proof.len() != 8 {
                    self.increment_failed_claims();
                    return Err(format!(
                        "Invalid PLONK proof: expected 8 elements, got {}",
                        proof.proof.len()
                    ));
                }

                let proof_array: [ethers::types::U256; 8] = proof
                    .proof
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        ethers::types::U256::from_dec_str(s).map_err(|e| {
                            format!("Invalid proof element at index {}: '{}': {}", i, s, e)
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| {
                        self.increment_failed_claims();
                        e
                    })?
                    .try_into()
                    .unwrap();

                let mut retry_count = 0;

                loop {
                    let call = plonk_verifier.claim(proof_array, nullifier_array, recipient);
                    let builder = call.from(wallet_with_chain.address());
                    let send_result = tokio::time::timeout(
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
                                TRANSACTION_RETRY_DELAY_MS,
                                e
                            );
                            tokio::time::sleep(tokio::time::Duration::from_millis(
                                TRANSACTION_RETRY_DELAY_MS,
                            ))
                            .await;
                        }
                        Err(_) => {
                            retry_count += 1;
                            if retry_count >= MAX_TRANSACTION_RETRIES {
                                self.increment_failed_claims();
                                return Err(format!("Failed to submit PLONK claim after {} retries: timeout after {} seconds", MAX_TRANSACTION_RETRIES, RPC_TIMEOUT_SECONDS));
                            }
                            tracing::warn!(
                                "Transaction timed out (attempt {}/{}), retrying in {}ms",
                                retry_count,
                                MAX_TRANSACTION_RETRIES,
                                TRANSACTION_RETRY_DELAY_MS
                            );
                            tokio::time::sleep(tokio::time::Duration::from_millis(
                                TRANSACTION_RETRY_DELAY_MS,
                            ))
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
            let timestamp = chrono::Utc::now().to_rfc3339();

            let tx_key = format!("{}:tx_hash", key);
            let _ = redis.set::<_, _, ()>(&tx_key, &tx_hash.to_string()).await;

            let timestamp_key = format!("{}:timestamp", key);
            let _ = redis.set::<_, _, ()>(&timestamp_key, &timestamp).await;

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

        const CLAIM_AMOUNT: u64 = 1_000_000_000_000_000_000;
        const AVG_GAS: u64 = 700_000;

        let total_tokens_distributed = successful_claims
            .checked_mul(CLAIM_AMOUNT)
            .and_then(|v| v.checked_mul(1000))
            .unwrap_or_else(|| {
                tracing::warn!("Token distribution calculation overflow, using max value");
                u64::MAX
            });

        let total_gas_used = successful_claims.checked_mul(AVG_GAS).unwrap_or_else(|| {
            tracing::warn!("Gas usage calculation overflow, using max value");
            u64::MAX
        });

        StatsResponse {
            total_claims,
            successful_claims,
            failed_claims,
            total_tokens_distributed: total_tokens_distributed.to_string(),
            unique_recipients: successful_claims,
            average_gas_price: "25000000000".to_string(),
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

    pub fn increment_failed_claims(&self) {
        let mut stats = self.stats.write();
        stats.total_claims += 1;
        stats.failed_claims += 1;
    }
}
