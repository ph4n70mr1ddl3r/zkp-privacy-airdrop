use redis::Client;
use anyhow::Result;
use redis::aio::Connection;

pub async fn connect(redis_url: &str) -> Result<Connection> {
    let client = Client::open(redis_url)?;
    let conn = client.get_async_connection().await?;
    Ok(conn)
}
