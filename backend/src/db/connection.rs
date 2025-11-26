// Database connection pool management
use anyhow::{Context, Result};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .context("Failed to connect to PostgreSQL")?;

    info!("Database connection pool established");

    Ok(pool)
}
