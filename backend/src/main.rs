use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod auth;
mod config;
mod db;
mod error;
mod executor;
mod validation;

use config::Config;
use db::Database;

async fn db_middleware(
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let db = req.extensions().get::<Database>().cloned();
    if let Some(db) = db {
        req.extensions_mut().insert(db);
    }
    next.run(req).await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mapf_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    // Connect to database
    let db = Database::connect(&config.database_url).await?;
    
    // Run migrations
    db.migrate().await?;

    tracing::info!("Database connected and migrations applied");

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build application state
    let state = api::AppState::new(config.clone(), db);

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/register", post(api::auth::register))
        .route("/api/verify", post(api::solver::verify))
        .route("/api/submit", post(api::solver::submit))
        .route("/api/leaderboard", get(api::leaderboard::list))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
