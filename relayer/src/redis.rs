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
        match ConnectionManager::new_with_connection_config(
            client.clone(),
            redis::aio::ConnectionConfig {
                connection_timeout: Duration::from_secs(5),
                exponent_base: 2,
                max_retries: MAX_RETRIES,
                retry_factor: 2,
                wait_initial: Duration::from_millis(RETRY_DELAY_MS),
                ..Default::default()
            },
        )
        .await
        {
            Ok(manager) => return Ok(manager),
            Err(e) => {
                last_error = Some(e);
                tracing::warn!(
                    "Redis connection attempt {} failed, retrying in {}ms...",
                    attempt,
                    RETRY_DELAY_MS * attempt as u64
                );
                tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS * attempt as u64)).await;
            }
        }
    }

    Err(anyhow::anyhow!(
        "Failed to connect to Redis after {} attempts: {}",
        MAX_RETRIES,
        last_error.unwrap()
    ))
}
