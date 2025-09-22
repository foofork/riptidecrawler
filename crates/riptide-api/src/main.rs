mod handlers;
mod models;

use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use std::net::SocketAddr;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "riptide-api")]
#[command(about = "RipTide Crawler API Service")]
struct Args {
    #[arg(long, default_value = "configs/riptide.yml")]
    config: String,

    #[arg(long, default_value = "0.0.0.0:8080")]
    bind: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let args = Args::parse();
    // TODO: load configs, init Redis, WasmExtractor, Reqwest client, etc.

    let app = Router::new()
        .route("/healthz", get(handlers::health))
        .route("/crawl", post(handlers::crawl))
        .route("/deepsearch", post(handlers::deepsearch))
        .fallback(handlers::not_found)
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = args.bind.parse()?;
    tracing::info!("RipTide API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
