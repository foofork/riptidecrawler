mod config;
mod errors;
mod handlers;
mod health;
mod metrics;
mod models;
mod pipeline;
mod pipeline_enhanced;
mod strategies_pipeline;
mod reliability_integration;
mod resource_manager;
mod routes;
mod rpc_client;
mod sessions;
mod state;
mod streaming;
mod telemetry_config;
mod tests;
mod validation;

use crate::health::HealthChecker;
use crate::metrics::{create_metrics_layer, RipTideMetrics};
use crate::sessions::middleware::SessionLayer;
use crate::state::{AppConfig, AppState};
use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use riptide_core::telemetry::TelemetrySystem;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer,
};

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
    // Initialize telemetry system with OpenTelemetry
    let _telemetry_system = TelemetrySystem::init()?;

    let args = Args::parse();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        bind_address = %args.bind,
        "Starting RipTide API Server"
    );

    // Initialize startup time tracking
    handlers::init_startup_time();

    // Load application configuration
    let config = AppConfig::default();
    tracing::info!(
        redis_url = %config.redis_url,
        wasm_path = %config.wasm_path,
        max_concurrency = config.max_concurrency,
        cache_ttl = config.cache_ttl,
        gate_hi_threshold = config.gate_hi_threshold,
        gate_lo_threshold = config.gate_lo_threshold,
        headless_url = ?config.headless_url,
        "Application configuration loaded"
    );

    // Initialize metrics
    tracing::info!("Initializing Prometheus metrics");
    let metrics = Arc::new(RipTideMetrics::new()?);
    let (prometheus_layer, _metric_handle) = create_metrics_layer()?;
    tracing::info!("Prometheus metrics initialized");

    // Initialize health checker
    let health_checker = Arc::new(HealthChecker::new());

    // Initialize application state with all dependencies
    tracing::info!("Initializing application state and dependencies");
    let app_state = AppState::new(config, metrics.clone(), health_checker.clone()).await?;
    tracing::info!("Application state initialization complete");

    // Worker service is initialized and ready to process jobs
    // Note: The worker pool starts automatically when jobs are submitted
    // No explicit background task is needed as the service manages its own lifecycle
    tracing::info!("Worker service initialized and ready for job processing");

    // Perform initial health check
    let initial_health = app_state.health_check().await;
    if !initial_health.healthy {
        tracing::error!(
            redis_status = %initial_health.redis,
            extractor_status = %initial_health.extractor,
            http_client_status = %initial_health.http_client,
            worker_service_status = %initial_health.worker_service,
            "Initial health check failed, but continuing startup"
        );
        // Note: We continue startup even if some deps are unhealthy
        // The health endpoint will report the actual status
    } else {
        tracing::info!("Initial health check passed - all dependencies healthy");
    }

    // Build the application router with middleware stack
    let app = Router::new()
        .route("/healthz", get(handlers::health))
        .route("/metrics", get(handlers::metrics))
        .route("/render", post(handlers::render))
        .route("/crawl", post(handlers::crawl))
        .route("/crawl/stream", post(streaming::ndjson_crawl_stream))
        .route("/crawl/sse", post(streaming::crawl_sse))
        .route("/crawl/ws", get(streaming::crawl_websocket))
        .route("/deepsearch", post(handlers::deepsearch))
        // PDF processing endpoints with progress tracking
        .nest("/pdf", routes::pdf::pdf_routes())
        // Stealth configuration and testing endpoints
        .nest("/stealth", routes::stealth::stealth_routes())
        // Table extraction endpoints
        .nest("/api/v1/tables", routes::tables::table_routes())
        // LLM provider management endpoints
        .nest("/api/v1/llm", routes::llm::llm_routes())
        // Strategies endpoints for advanced extraction
        .route("/strategies/crawl", post(handlers::strategies::strategies_crawl))
        .route("/strategies/info", get(handlers::strategies::get_strategies_info))
        // Spider endpoints for deep crawling
        .route("/spider/crawl", post(handlers::spider::spider_crawl))
        .route("/spider/status", post(handlers::spider::spider_status))
        .route("/spider/control", post(handlers::spider::spider_control))
        .route(
            "/deepsearch/stream",
            post(streaming::ndjson_deepsearch_stream),
        )
        // Session management endpoints
        .route("/sessions", post(handlers::sessions::create_session))
        .route("/sessions", get(handlers::sessions::list_sessions))
        .route(
            "/sessions/stats",
            get(handlers::sessions::get_session_stats),
        )
        .route(
            "/sessions/cleanup",
            post(handlers::sessions::cleanup_expired_sessions),
        )
        .route(
            "/sessions/:session_id",
            get(handlers::sessions::get_session_info),
        )
        .route(
            "/sessions/:session_id",
            axum::routing::delete(handlers::sessions::delete_session),
        )
        .route(
            "/sessions/:session_id/extend",
            post(handlers::sessions::extend_session),
        )
        .route(
            "/sessions/:session_id/cookies",
            post(handlers::sessions::set_cookie),
        )
        .route(
            "/sessions/:session_id/cookies",
            axum::routing::delete(handlers::sessions::clear_cookies),
        )
        .route(
            "/sessions/:session_id/cookies/:domain",
            get(handlers::sessions::get_cookies_for_domain),
        )
        .route(
            "/sessions/:session_id/cookies/:domain/:name",
            get(handlers::sessions::get_cookie),
        )
        .route(
            "/sessions/:session_id/cookies/:domain/:name",
            axum::routing::delete(handlers::sessions::delete_cookie),
        )
        // Worker management endpoints
        .route("/workers/jobs", post(handlers::workers::submit_job))
        .route("/workers/jobs/:job_id", get(handlers::workers::get_job_status))
        .route("/workers/jobs/:job_id/result", get(handlers::workers::get_job_result))
        .route("/workers/stats/queue", get(handlers::workers::get_queue_stats))
        .route("/workers/stats/workers", get(handlers::workers::get_worker_stats))
        .route("/workers/metrics", get(handlers::workers::get_worker_metrics))
        .route("/workers/schedule", post(handlers::workers::create_scheduled_job))
        .route("/workers/schedule", get(handlers::workers::list_scheduled_jobs))
        .route("/workers/schedule/:job_id", axum::routing::delete(handlers::workers::delete_scheduled_job))
        // Monitoring system endpoints
        .route("/monitoring/health-score", get(handlers::monitoring::get_health_score))
        .route("/monitoring/performance-report", get(handlers::monitoring::get_performance_report))
        .route("/monitoring/metrics/current", get(handlers::monitoring::get_current_metrics))
        .route("/monitoring/alerts/rules", get(handlers::monitoring::get_alert_rules))
        // Enhanced pipeline phase visualization endpoints
        .route("/pipeline/phases", get(handlers::get_pipeline_phases))
        .route("/monitoring/alerts/active", get(handlers::monitoring::get_active_alerts))
        // Telemetry and trace visualization endpoints (TELEM-005)
        // Telemetry routes temporarily disabled due to API compatibility issues
        // .route("/telemetry/status", get(handlers::telemetry::get_telemetry_status))
        // .route("/telemetry/traces", get(handlers::telemetry::list_traces))
        // .route("/telemetry/traces/:trace_id", get(handlers::telemetry::get_trace_tree))
        .fallback(handlers::not_found)
        .with_state(app_state.clone())
        .layer(SessionLayer::new(app_state.session_manager.clone()))
        .layer(prometheus_layer)
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new());

    // Parse bind address
    let addr: SocketAddr = args.bind.parse()?;
    tracing::info!("RipTide API server starting on {}", addr);

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Log successful startup
    tracing::info!(
        bind_address = %addr,
        version = env!("CARGO_PKG_VERSION"),
        "RipTide API server successfully started and ready to accept connections"
    );

    // Start the server
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("RipTide API server shutdown complete");
    Ok(())
}

/// Graceful shutdown signal handler.
///
/// Listens for SIGTERM and SIGINT signals to gracefully shutdown the server.
/// This allows for proper cleanup of connections and resources.
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        if let Err(e) = signal::ctrl_c().await {
            tracing::error!("Failed to install Ctrl+C handler: {}", e);
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut signal_handler) => {
                signal_handler.recv().await;
            }
            Err(e) => {
                tracing::error!("Failed to install SIGTERM handler: {}", e);
                // Wait indefinitely if signal handler fails
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, initiating graceful shutdown");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, initiating graceful shutdown");
        },
    }
}
