use anyhow::Result;
use redis::aio::ConnectionManager;
use redis::Client;
use std::time::Duration;

const MAX_RETRIES: u32 = 5;
const RETRY_DELAY_MS: u64 = 2000;

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
                    tracing::warn!(
                        "Redis connection attempt {} failed, retrying in {}ms...",
                        attempt,
                        RETRY_DELAY_MS * attempt as u64
                    );
                    tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS * attempt as u64))
                        .await;
                }
            }
        }
    }

    Err(anyhow::anyhow!(
        "Failed to connect to Redis after {} attempts: {}",
        MAX_RETRIES,
        last_error.unwrap()
    ))
}
