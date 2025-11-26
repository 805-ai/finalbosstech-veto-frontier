// FinalBoss Veto Frontier Backend
// Patent-pending pointer orphaning system (US 19/240,581)
// High-performance Rust implementation targeting <8ms latency

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

mod api;
mod config;
mod crypto;
mod db;
mod enforcement;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("🚀 FinalBoss Veto Frontier Backend starting...");

    // Load configuration
    let config = config::Config::from_env()?;
    info!("✓ Configuration loaded");

    // Initialize database connection pool
    let db_pool = db::create_pool(&config.database_url).await?;
    info!("✓ Database connection pool created");

    // Run migrations (optional - schema should be pre-initialized)
    // sqlx::migrate!("../database/migrations").run(&db_pool).await?;

    // Initialize signing keypair
    let keypair = crypto::ed25519::load_or_generate_keypair(&config)?;
    info!("✓ Cryptographic keypair loaded");

    // Build application state
    let app_state = api::AppState {
        db_pool: db_pool.clone(),
        keypair,
        config: config.clone(),
    };

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any) // TODO: Restrict to config.cors_allowed_origins in production
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(api::handlers::health_check))
        .route("/api/pointer/create", post(api::handlers::create_pointer))
        .route("/api/pointer/resolve/:id", get(api::handlers::resolve_pointer))
        .route("/api/pointer/orphan", post(api::handlers::orphan_pointer))
        .route("/api/receipts/:pointer_id", get(api::handlers::get_receipts))
        .route("/api/audit/:subject_id", get(api::handlers::get_audit_trail))
        .layer(cors)
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    info!("🌐 Server listening on http://{}", addr);
    info!("✓ Ready to handle requests");
    info!("   POST /api/pointer/create   - Create new pointer");
    info!("   GET  /api/pointer/resolve/:id - Resolve pointer");
    info!("   POST /api/pointer/orphan    - Orphan pointer (VETO)");
    info!("   GET  /api/receipts/:id      - Get governance receipts");
    info!("   GET  /api/audit/:subject    - Get audit trail");
    info!("");
    info!("Patent: US 19/240,581 Claim 9 - Pointer orphaning with data preservation");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
