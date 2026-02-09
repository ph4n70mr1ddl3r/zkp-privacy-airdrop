use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
use tracing::info;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    info!("Connecting to PostgreSQL database...");

    let pool = PgPoolOptions::new()
        .max_connections(50)
        .min_connections(5)
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
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run database migrations: {}", e))?;

    info!("Database migrations completed successfully");

    Ok(pool)
}
