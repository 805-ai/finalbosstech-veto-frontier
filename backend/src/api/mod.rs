// API module
pub mod errors;
pub mod handlers;

pub use errors::*;
pub use handlers::*;

// Application state shared across handlers
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub keypair: crate::crypto::Ed25519Keypair,
    pub config: crate::config::Config,
}
