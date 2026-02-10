use anyhow::Result;
use redis::aio::ConnectionManager;
use redis::Client;
use std::time::Duration;

const MAX_RETRIES: u32 = 8;
const BASE_RETRY_DELAY_MS: u64 = 1000;
const MAX_RETRY_DELAY_MS: u64 = 30000;
const JITTER_FACTOR: f64 = 0.2;

pub async fn connect(redis_url: &str) -> Result<ConnectionManager> {
    let client = Client::open(redis_url)?;

    let mut last_error = None;
    for attempt in 1..=MAX_RETRIES {
        match ConnectionManager::new(client.clone()).await {
            Ok(manager) => {
                tracing::info!("Successfully connected to Redis");
                return Ok(manager);
            }
            Err(e) => {
                last_error = Some(e);
                if attempt < MAX_RETRIES {
                    let delay_ms = calculate_backoff_delay(attempt);
                    tracing::warn!(
                        "Redis connection attempt {} failed, retrying in {}ms: {}",
                        attempt,
                        delay_ms,
                        last_error
                            .as_ref()
                            .map(|e| e.to_string())
                            .unwrap_or_default()
                    );
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }
    }

    Err(anyhow::anyhow!(
        "Failed to connect to Redis after {} attempts: {}",
        MAX_RETRIES,
        last_error
            .as_ref()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "Unknown error".to_string())
    ))
}

fn calculate_backoff_delay(attempt: u32) -> u64 {
    let exponential_delay = (BASE_RETRY_DELAY_MS as f64) * 2_f64.powi((attempt - 1) as i32);
    let capped_delay = exponential_delay.min(MAX_RETRY_DELAY_MS as f64);
    let jitter = (capped_delay * JITTER_FACTOR) * (rand::random::<f64>() * 2.0 - 1.0);
    (capped_delay + jitter).max(BASE_RETRY_DELAY_MS as f64) as u64
}
