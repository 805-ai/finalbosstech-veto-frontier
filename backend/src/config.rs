// Configuration management
use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub default_org_id: uuid::Uuid,
    pub cors_allowed_origins: Vec<String>,
    pub signing_private_key: Option<String>,
    pub signing_public_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env file if present

        let database_url = std::env::var("DATABASE_URL")
            .context("DATABASE_URL must be set")?;

        let host = std::env::var("HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8888".to_string())
            .parse()
            .context("PORT must be a valid u16")?;

        let default_org_id = std::env::var("DEFAULT_ORG_ID")
            .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000001".to_string())
            .parse()
            .context("DEFAULT_ORG_ID must be a valid UUID")?;

        let cors_allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let signing_private_key = std::env::var("SIGNING_PRIVATE_KEY").ok();
        let signing_public_key = std::env::var("SIGNING_PUBLIC_KEY").ok();

        Ok(Config {
            database_url,
            host,
            port,
            default_org_id,
            cors_allowed_origins,
            signing_private_key,
            signing_public_key,
        })
    }
}
