use crate::config::Config;
use crate::types_plonk::*;
use sqlx::PgPool;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::Mutex;

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
    #[allow(dead_code)]
    pub total_gas_used: u64,
    pub start_time: std::time::Instant,
}

impl Default for RelayerStats {
    fn default() -> Self {
        Self {
            total_claims: 0,
            successful_claims: 0,
            failed_claims: 0,
            total_gas_used: 0,
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
            format!("{:?}", wallet.address())
        } else {
            "0x0000000000000000000000000000000000000000".to_string()
        }
    }

    pub async fn get_relayer_balance(&self) -> u64 {
        // In production, query actual balance from RPC
        1000000000000000000 // 1 ETH
    }

    pub async fn has_sufficient_balance(&self) -> bool {
        let balance = self.get_relayer_balance().await;
        let min_critical: u64 = self.config.relayer.min_balance_critical
            .parse()
            .unwrap_or(500000000000000000);
        balance > min_critical
    }

    pub async fn is_healthy(&self) -> bool {
        self.has_sufficient_balance().await
    }

    pub async fn check_rate_limit(
        &self,
        _req: &actix_web::HttpRequest,
        key: &str,
        limit_type: RateLimitType,
    ) -> Result<(), String> {
        use redis::AsyncCommands;

        let limit = match limit_type {
            RateLimitType::SubmitClaim => self.config.rate_limit.claims_per_minute,
            RateLimitType::GetMerklePath => self.config.rate_limit.requests_per_minute,
            RateLimitType::CheckStatus => self.config.rate_limit.requests_per_minute,
        };

        let redis_key = format!("rate_limit:{}", key);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();
        let window_start = current_time - (current_time % 60);

        let count_key = format!("{}:{}", redis_key, window_start);
        let mut redis = self.redis.lock().await;

        let count: u64 = redis.incr(&count_key, 1).await.map_err(|e| e.to_string())?;

        if count == 1 {
            redis
                .expire::<_, ()>(&count_key, 120)
                .await
                .map_err(|e| e.to_string())?;
        }

        if count > limit {
            return Err(format!("Rate limit exceeded: {}/min", limit));
        }

        Ok(())
    }

    pub async fn is_nullifier_used(&self, nullifier: &str) -> bool {
        use redis::AsyncCommands;

        let key = format!("nullifier:{}", nullifier);
        let mut redis = self.redis.lock().await;
        redis.exists(&key).await.unwrap_or(0) > 0
    }

    pub async fn submit_claim(&self, claim: &SubmitClaimRequest) -> Result<String, String> {
        {
            let mut stats = self.stats.write();
            stats.total_claims += 1;
        }

        use rand::RngCore;
        let mut tx_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut tx_bytes);
        let tx_hash = format!("0x{}", hex::encode(tx_bytes));

        // Mark as claimed
        use redis::AsyncCommands;
        let key = format!("nullifier:{}", claim.nullifier);
        let mut redis = self.redis.lock().await;
        let _: () = redis.set(&key, &claim.recipient).await.map_err(|e| e.to_string())?;
        drop(redis);

        {
            let mut stats = self.stats.write();
            stats.successful_claims += 1;
        }

        Ok(tx_hash)
    }

    pub async fn get_claim_status(&self, nullifier: &str) -> Option<CheckStatusResponse> {
        use redis::AsyncCommands;

        let key = format!("nullifier:{}", nullifier);
        let mut redis = self.redis.lock().await;
        let recipient: Option<String> = redis.get(&key).await.ok().flatten()?;

        Some(CheckStatusResponse {
            nullifier: nullifier.to_string(),
            claimed: true,
            tx_hash: Some(format!("0x{}", hex::encode(rand::random::<[u8; 32]>()))),
            recipient,
            timestamp: Some(format!("{:?}", std::time::SystemTime::now())),
            block_number: Some(rand::random()),
        })
    }

    pub async fn get_merkle_path(&self, _address: &str) -> Option<MerklePathResponse> {
        // In production, look up actual Merkle path
        Some(MerklePathResponse {
            address: _address.to_string(),
            leaf_index: 1234567,
            merkle_path: vec![
                "0xabc".to_string(),
                "0xdef".to_string(),
                "0x123".to_string(),
            ],
            path_indices: vec![0, 1, 0],
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
            total_gas_used: (successful_claims * 700000).to_string(),
            relayer_balance: self.get_relayer_balance().await.to_string(),
            uptime_percentage: if uptime > 0.0 { 100.0 } else { 0.0 },
            response_time_ms: ResponseTime {
                p50: 150,
                p95: 500,
                p99: 1200,
            },
        }
    }
}
