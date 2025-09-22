use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    tracing::info!("RipTide Workers starting...");

    // TODO: Implement background workers for batch processing
    // - Queue consumer
    // - Batch crawler
    // - Result processor

    // Keep running
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down workers...");

    Ok(())
}
