mod cdp;
mod launcher;
mod models;
mod pool;

use axum::{
    routing::{get, post},
    Router,
};
use cdp::AppState; // Import AppState from cdp module
use launcher::HeadlessLauncher;
use std::{net::SocketAddr, sync::Arc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    tracing::info!("Initializing HeadlessLauncher with browser pool...");

    // Create the headless launcher (this initializes the browser pool)
    let launcher = Arc::new(HeadlessLauncher::new().await?);

    let stats = launcher.stats().await;
    tracing::info!(
        "HeadlessLauncher initialized successfully (avg response time: {:.2}ms)",
        stats.avg_response_time_ms
    );

    // Create shared app state
    let state = AppState { launcher };

    // Build router with state
    let app = Router::new()
        .route("/healthz", get(health_check))
        .route("/render", post(cdp::render))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = "0.0.0.0:9123".parse()?;
    tracing::info!("RipTide Headless listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}
