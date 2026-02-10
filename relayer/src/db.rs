use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
use tracing::info;

const MAX_DB_CONNECTIONS: u32 = 50;
const MIN_DB_CONNECTIONS: u32 = 5;
const DB_ACQUIRE_TIMEOUT_SECS: u64 = 30;
const DB_IDLE_TIMEOUT_SECS: u64 = 600;
const DB_MAX_LIFETIME_SECS: u64 = 1800;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    info!("Connecting to PostgreSQL database...");

    let pool = PgPoolOptions::new()
        .max_connections(MAX_DB_CONNECTIONS)
        .min_connections(MIN_DB_CONNECTIONS)
        .acquire_timeout(Duration::from_secs(DB_ACQUIRE_TIMEOUT_SECS))
        .idle_timeout(Duration::from_secs(DB_IDLE_TIMEOUT_SECS))
        .max_lifetime(Duration::from_secs(DB_MAX_LIFETIME_SECS))
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
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run database migrations: {}", e))?;

    info!("Database migrations completed successfully");

    Ok(pool)
}
