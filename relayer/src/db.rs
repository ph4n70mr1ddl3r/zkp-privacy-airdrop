use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{info, warn};

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    info!("Connecting to PostgreSQL database...");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .test_before_acquire(true)
        .connect(database_url)
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to create database pool: {}. \
                 Ensure PostgreSQL is running and the connection string is correct.",
                e
            )
        })?;

    info!("Database connection pool created successfully");

    info!("Running database migrations...");
    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => {
            info!("Database migrations completed successfully");
        }
        Err(e) => {
            warn!("Database migrations completed with warnings: {}", e);
        }
    }

    Ok(pool)
}
