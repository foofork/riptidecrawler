use anyhow::Result;
use clap::Parser;
use riptide_workers::{WorkerService, WorkerServiceConfig};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "riptide-workers")]
#[command(about = "RipTide Background Worker Service")]
struct Args {
    #[arg(long, default_value = "redis://localhost:6379")]
    redis_url: String,

    #[arg(long, default_value = "4")]
    worker_count: usize,

    #[arg(long, default_value = "50")]
    max_batch_size: usize,

    #[arg(long, default_value = "10")]
    max_concurrency: usize,

    #[arg(
        long,
        default_value = "./target/wasm32-wasip2/release/riptide_extractor_wasm.wasm"
    )]
    wasm_path: String,

    #[arg(long, default_value = "true")]
    enable_scheduler: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let args = Args::parse();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        redis_url = %args.redis_url,
        worker_count = args.worker_count,
        max_batch_size = args.max_batch_size,
        max_concurrency = args.max_concurrency,
        wasm_path = %args.wasm_path,
        enable_scheduler = args.enable_scheduler,
        "Starting RipTide Worker Service"
    );

    // Create worker service configuration
    let mut config = WorkerServiceConfig::default();
    config.redis_url = args.redis_url;
    config.worker_config.worker_count = args.worker_count;
    config.max_batch_size = args.max_batch_size;
    config.max_concurrency = args.max_concurrency;
    config.wasm_path = args.wasm_path;
    config.enable_scheduler = args.enable_scheduler;

    // Initialize worker service
    tracing::info!("Initializing worker service with configuration");
    let mut worker_service = WorkerService::new(config).await?;
    tracing::info!("Worker service initialized successfully");

    // Set up graceful shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        tracing::info!("Received shutdown signal, initiating graceful shutdown");
    };

    // Start worker service
    tokio::select! {
        result = worker_service.start() => {
            if let Err(e) = result {
                tracing::error!(error = %e, "Worker service failed");
                return Err(e);
            }
        }
        _ = shutdown_signal => {
            tracing::info!("Shutdown signal received");
        }
    }

    // Stop worker service
    tracing::info!("Stopping worker service");
    worker_service.stop().await?;

    tracing::info!("RipTide Worker Service shutdown complete");
    Ok(())
}
