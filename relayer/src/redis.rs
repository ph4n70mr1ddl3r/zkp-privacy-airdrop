use redis::Client;
use anyhow::Result;
use redis::aio::ConnectionManager;

pub async fn connect(redis_url: &str) -> Result<ConnectionManager> {
    let client = Client::open(redis_url)?;
    let manager = ConnectionManager::new(client).await?;
    Ok(manager)
}
