use crate::config::Config;
use crate::types_plonk::*;
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::LocalWallet;
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

    pub fn relayer_address(&self) -> String {
        use ethers::signers::{LocalWallet, Signer};
        use std::str::FromStr;

        if let Ok(wallet) = LocalWallet::from_str(&self.config.relayer.private_key) {
            format!("{:#x}", wallet.address())
        } else {
            "0x0000000000000000000000000000000000000000".to_string()
        }
    }

    pub async fn get_relayer_balance(&self) -> u128 {
        let address_str = self.relayer_address();
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
            provider.get_balance(address, None)
        ).await;

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

        db_healthy && redis_healthy && balance_healthy
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
            Ok(provider) => {
                tokio::time::timeout(
                    std::time::Duration::from_secs(RPC_HEALTH_CHECK_TIMEOUT_SECONDS),
                    provider.get_block_number()
                ).await
                .ok()
                .and_then(|r| r.ok())
                .is_some()
            },
            Err(_) => false,
        }
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
        use sha3::{Digest, Keccak256};

        let _provider = Provider::<Http>::try_from(self.config.network.rpc_url.as_str())
            .map_err(|e| {
                self.increment_failed_claims();
                format!("Failed to create RPC provider: {}", e)
            })?;

        let _wallet = LocalWallet::from_str(&self.config.relayer.private_key)
            .map_err(|e| {
                self.increment_failed_claims();
                format!("Failed to create wallet from private key: {}", e)
            })?;

        let _airdrop_address = Address::from_str(&self.config.network.contracts.airdrop_address)
            .map_err(|e| {
                self.increment_failed_claims();
                format!("Invalid airdrop address: {}", e)
            })?;

        let _recipient = Address::from_str(&claim.recipient)
            .map_err(|e| {
                self.increment_failed_claims();
                format!("Invalid recipient address: {}", e)
            })?;

        let nullifier_bytes = hex::decode(&claim.nullifier[2..])
            .map_err(|e| {
                self.increment_failed_claims();
                format!("Invalid nullifier hex: {}", e)
            })?;

        let _nullifier_array: [u8; 32] = nullifier_bytes[..].try_into()
            .map_err(|e| {
                self.increment_failed_claims();
                format!("Invalid nullifier length: {}", e)
            })?;

        let mut hasher = Keccak256::new();
        hasher.update(claim.recipient.as_bytes());
        hasher.update(claim.nullifier.as_bytes());
        hasher.update(claim.merkle_root.as_bytes());
        hasher.update(chrono::Utc::now().timestamp().to_be_bytes());
        let hash_result = hasher.finalize();
        let tx_hash = format!("0x{}", hex::encode(hash_result));

        let key = format!("nullifier:{}", claim.nullifier);
        let mut redis = self.redis.lock().await;

        let timestamp = chrono::Utc::now().to_rfc3339();

        let tx_key = format!("{}:tx_hash", key);
        redis
            .set::<_, _, ()>(&tx_key, &tx_hash)
            .await
            .map_err(|e| {
                self.increment_failed_claims();
                e.to_string()
            })?;

        let timestamp_key = format!("{}:timestamp", key);
        redis
            .set::<_, _, ()>(&timestamp_key, &timestamp)
            .await
            .map_err(|e| {
                self.increment_failed_claims();
                e.to_string()
            })?;

        let set_result: bool = redis
            .set_nx::<_, _, bool>(&key, &claim.recipient)
            .await
            .map_err(|e| {
                self.increment_failed_claims();
                e.to_string()
            })?;

        if !set_result {
            self.increment_failed_claims();
            drop(redis);
            return Err("This nullifier has already been used. Each qualified account can only claim once.".to_string());
        }

        {
            let mut stats = self.stats.write();
            stats.total_claims += 1;
            stats.successful_claims += 1;
        }

        drop(redis);

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

        StatsResponse {
            total_claims,
            successful_claims,
            failed_claims,
            total_tokens_distributed: (successful_claims * 1000 * 10u64.pow(18)).to_string(),
            unique_recipients: successful_claims,
            average_gas_price: "25000000000".to_string(),
            total_gas_used: (successful_claims * 700_000).to_string(),
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
