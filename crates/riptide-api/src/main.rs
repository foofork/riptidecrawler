mod config;
mod dto;
mod errors;
mod handlers;
mod health;
mod metrics;
mod middleware;
mod models;
mod pipeline;
mod pipeline_enhanced;
mod reliability_integration;
mod resource_manager;
mod routes;
mod rpc_client;
mod rpc_session_context;
mod sessions;
mod state;
mod strategies_pipeline;
mod streaming;
mod telemetry_config;
mod tests;
mod validation;

// Configure jemalloc allocator for memory profiling (non-MSVC targets only)
#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
use tikv_jemallocator::Jemalloc;

#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use crate::health::HealthChecker;
use crate::metrics::{create_metrics_layer, RipTideMetrics};
use crate::middleware::{
    auth_middleware, rate_limit_middleware, request_validation_middleware, PayloadLimitLayer,
};
use crate::sessions::middleware::SessionLayer;
use crate::state::{AppConfig, AppState};
use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use riptide_monitoring::TelemetrySystem;
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
    #[arg(long, default_value = "config/application/riptide.yml")]
    config: String,

    #[arg(long, default_value = "0.0.0.0:8080")]
    bind: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber FIRST (before any tracing calls)
    // Check if OpenTelemetry should be enabled
    let otel_enabled = std::env::var("OTEL_ENDPOINT").is_ok();

    if otel_enabled {
        // Initialize with OpenTelemetry support
        // Note: TelemetrySystem::init() will set up the subscriber
        let _telemetry_system = Arc::new(TelemetrySystem::init()?);
        tracing::info!("OTEL_ENDPOINT detected, OpenTelemetry initialized");
    } else {
        // Initialize basic tracing subscriber without OpenTelemetry
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| {
                        "info,cranelift=warn,cranelift_codegen=warn,wasmtime=warn,wasmtime_cranelift=warn"
                            .into()
                    }),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
        tracing::info!("OTEL_ENDPOINT not set, using basic tracing");
    }

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

    // Log jemalloc initialization and memory stats if enabled
    #[cfg(feature = "jemalloc")]
    {
        tracing::info!("jemalloc allocator enabled for memory profiling");

        // Collect and log initial memory stats using jemalloc
        #[cfg(not(target_env = "msvc"))]
        {
            use riptide_api::jemalloc_stats::JemallocStats;

            if let Some(stats) = JemallocStats::collect() {
                tracing::info!(
                    allocated_mb = stats.allocated_mb(),
                    resident_mb = stats.resident_mb(),
                    metadata_mb = stats.metadata_mb(),
                    fragmentation_ratio = stats.fragmentation_ratio(),
                    metadata_overhead = stats.metadata_overhead_ratio(),
                    "Initial jemalloc memory statistics"
                );

                // Update metrics with initial stats
                metrics.update_jemalloc_stats();
            } else {
                tracing::warn!("jemalloc allocator enabled but stats collection failed - ensure jemalloc feature is enabled");
            }
        }
    }

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
        // Health endpoints - standardized on /healthz
        .route("/healthz", get(handlers::health))
        .route("/api/health/detailed", get(handlers::health_detailed))
        .route(
            "/health/:component",
            get(handlers::health::component_health_check),
        )
        .route(
            "/health/metrics",
            get(handlers::health::health_metrics_check),
        )
        // Metrics - both root and v1 paths
        .route("/metrics", get(handlers::metrics))
        .route("/api/v1/metrics", get(handlers::metrics)) // v1 alias
        // Crawl endpoints - both root and v1 paths
        .route("/crawl", post(handlers::crawl))
        .route("/api/v1/crawl", post(handlers::crawl)) // v1 alias
        .route("/crawl/stream", post(handlers::crawl_stream))
        .route("/api/v1/crawl/stream", post(handlers::crawl_stream)) // v1 alias
        // Extract endpoint - NEW v1.1 feature
        .route("/api/v1/extract", post(handlers::extract))
        .route("/extract", post(handlers::extract)) // Root alias for backward compatibility
        // Search endpoint - NEW v1.1 feature
        .route("/api/v1/search", get(handlers::search))
        .route("/search", get(handlers::search)) // Root alias for backward compatibility
        // DeepSearch
        .route("/deepsearch", post(handlers::deepsearch))
        .route("/deepsearch/stream", post(handlers::deepsearch_stream))
        .route(
            "/api/v1/deepsearch/stream",
            post(handlers::deepsearch_stream),
        ) // v1 alias
        // PDF processing endpoints with progress tracking
        .nest("/pdf", routes::pdf::pdf_routes())
        // Stealth configuration and testing endpoints
        .nest("/stealth", routes::stealth::stealth_routes())
        // Table extraction endpoints
        .nest("/api/v1/tables", routes::tables::table_routes())
        // LLM provider management endpoints
        .nest("/api/v1/llm", routes::llm::llm_routes())
        // Content chunking endpoints
        .nest("/api/v1/content", routes::chunking::chunking_routes())
        // Engine selection endpoints (Phase 10)
        .nest("/engine", routes::engine::engine_routes())
        // Domain profile management endpoints (Phase 10.4: Warm-Start Caching)
        .nest("/api/v1/profiles", routes::profiles::profile_routes())
        // Strategies endpoints for advanced extraction
        .route(
            "/strategies/crawl",
            post(handlers::strategies::strategies_crawl),
        )
        .route(
            "/strategies/info",
            get(handlers::strategies::get_strategies_info),
        )
        // Spider endpoints for deep crawling
        .route("/spider/crawl", post(handlers::spider::spider_crawl))
        .route("/spider/status", post(handlers::spider::spider_status))
        .route("/spider/control", post(handlers::spider::spider_control))
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
        .route("/workers/jobs", get(handlers::workers::list_jobs))
        // Browser management endpoints
        .route(
            "/api/v1/browser/session",
            post(handlers::browser::create_browser_session),
        )
        .route(
            "/api/v1/browser/action",
            post(handlers::browser::execute_browser_action),
        )
        .route(
            "/api/v1/browser/pool/status",
            get(handlers::browser::get_browser_pool_status),
        )
        .route(
            "/api/v1/browser/session/:id",
            axum::routing::delete(handlers::browser::close_browser_session),
        )
        // Resource monitoring endpoints
        .route(
            "/resources/status",
            get(handlers::resources::get_resource_status),
        )
        .route(
            "/resources/browser-pool",
            get(handlers::resources::get_browser_pool_status),
        )
        .route(
            "/resources/rate-limiter",
            get(handlers::resources::get_rate_limiter_status),
        )
        .route(
            "/resources/memory",
            get(handlers::resources::get_memory_status),
        )
        .route(
            "/resources/performance",
            get(handlers::resources::get_performance_status),
        )
        .route(
            "/resources/pdf/semaphore",
            get(handlers::resources::get_pdf_semaphore_status),
        )
        // Memory profiling endpoint for production observability
        .route(
            "/api/v1/memory/profile",
            get(handlers::memory::memory_profile_handler),
        )
        // Memory leak detection endpoint
        .route(
            "/api/v1/memory/leaks",
            get(handlers::resources::get_memory_leaks),
        )
        // Fetch engine metrics endpoint
        .route("/fetch/metrics", get(handlers::fetch::get_fetch_metrics))
        .route(
            "/workers/jobs/:job_id",
            get(handlers::workers::get_job_status),
        )
        .route(
            "/workers/jobs/:job_id/result",
            get(handlers::workers::get_job_result),
        )
        .route(
            "/workers/stats/queue",
            get(handlers::workers::get_queue_stats),
        )
        .route(
            "/workers/stats/workers",
            get(handlers::workers::get_worker_stats),
        )
        .route(
            "/workers/metrics",
            get(handlers::workers::get_worker_metrics),
        )
        .route(
            "/workers/schedule",
            post(handlers::workers::create_scheduled_job),
        )
        .route(
            "/workers/schedule",
            get(handlers::workers::list_scheduled_jobs),
        )
        .route(
            "/workers/schedule/:job_id",
            axum::routing::delete(handlers::workers::delete_scheduled_job),
        )
        // Monitoring system endpoints
        .route(
            "/monitoring/health-score",
            get(handlers::monitoring::get_health_score),
        )
        .route(
            "/monitoring/performance-report",
            get(handlers::monitoring::get_performance_report),
        )
        .route(
            "/monitoring/metrics/current",
            get(handlers::monitoring::get_current_metrics),
        )
        .route(
            "/monitoring/alerts/rules",
            get(handlers::monitoring::get_alert_rules),
        )
        // Enhanced pipeline phase visualization endpoints
        .route("/pipeline/phases", get(handlers::get_pipeline_phases))
        .route(
            "/monitoring/alerts/active",
            get(handlers::monitoring::get_active_alerts),
        )
        // Performance profiling endpoints (riptide-performance integration)
        .route(
            "/api/profiling/memory",
            get(handlers::profiling::get_memory_profile),
        )
        .route(
            "/api/profiling/cpu",
            get(handlers::profiling::get_cpu_profile),
        )
        .route(
            "/api/profiling/bottlenecks",
            get(handlers::profiling::get_bottleneck_analysis),
        )
        .route(
            "/api/profiling/allocations",
            get(handlers::profiling::get_allocation_metrics),
        )
        .route(
            "/api/profiling/leak-detection",
            post(handlers::profiling::trigger_leak_detection),
        )
        .route(
            "/api/profiling/snapshot",
            post(handlers::profiling::trigger_heap_snapshot),
        )
        // Legacy monitoring profiling endpoints (deprecated, kept for compatibility)
        .route(
            "/monitoring/profiling/memory",
            get(handlers::monitoring::get_memory_metrics),
        )
        .route(
            "/monitoring/profiling/leaks",
            get(handlers::monitoring::get_leak_analysis),
        )
        .route(
            "/monitoring/profiling/allocations",
            get(handlers::monitoring::get_allocation_metrics),
        )
        // WASM instance health monitoring
        .route(
            "/monitoring/wasm-instances",
            get(handlers::monitoring::get_wasm_health),
        )
        // Resource management status endpoint
        .route(
            "/api/resources/status",
            get(handlers::monitoring::get_resource_status),
        )
        // Telemetry and trace visualization endpoints (TELEM-005)
        .route(
            "/api/telemetry/status",
            get(handlers::telemetry::get_telemetry_status),
        )
        .route(
            "/api/telemetry/traces",
            get(handlers::telemetry::list_traces),
        )
        .route(
            "/api/telemetry/traces/:trace_id",
            get(handlers::telemetry::get_trace_tree),
        );

    // Persistence and Multi-tenancy Admin Endpoints (feature-gated)
    #[cfg(feature = "persistence")]
    let app = app
        // Tenant management
        .route("/admin/tenants", post(handlers::admin::create_tenant))
        .route("/admin/tenants", get(handlers::admin::list_tenants))
        .route("/admin/tenants/:id", get(handlers::admin::get_tenant))
        .route(
            "/admin/tenants/:id",
            axum::routing::put(handlers::admin::update_tenant),
        )
        .route(
            "/admin/tenants/:id",
            axum::routing::delete(handlers::admin::delete_tenant),
        )
        .route(
            "/admin/tenants/:id/usage",
            get(handlers::admin::get_tenant_usage),
        )
        .route(
            "/admin/tenants/:id/billing",
            get(handlers::admin::get_tenant_billing),
        )
        // Cache management
        .route("/admin/cache/warm", post(handlers::admin::warm_cache))
        .route(
            "/admin/cache/invalidate",
            post(handlers::admin::invalidate_cache),
        )
        .route("/admin/cache/stats", get(handlers::admin::get_cache_stats))
        // State management
        .route("/admin/state/reload", post(handlers::admin::reload_state))
        .route(
            "/admin/state/checkpoint",
            post(handlers::admin::create_checkpoint),
        )
        .route(
            "/admin/state/restore/:id",
            post(handlers::admin::restore_checkpoint),
        );

    let app = app.fallback(handlers::not_found);

    // Create separate router for session-aware routes
    // SessionLayer must be applied BEFORE with_state for proper type inference
    let session_routes = Router::new()
        .route("/render", post(handlers::render))
        .route("/api/v1/render", post(handlers::render))
        .layer(SessionLayer::new(app_state.session_manager.clone()))
        .with_state(app_state.clone());

    // Merge session routes into main app
    let app = app
        .merge(session_routes)
        .with_state(app_state.clone())
        .layer(axum::middleware::from_fn(request_validation_middleware)) // Request validation - rejects malformed payloads and unsupported methods (400/405)
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        )) // Authentication - validates API keys
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            rate_limit_middleware,
        )) // Rate limiting and concurrency control
        .layer(PayloadLimitLayer::with_limit(50 * 1024 * 1024)) // 50MB limit for large PDF/HTML payloads
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

    // Clone app_state for shutdown handler
    let shutdown_state = app_state.clone();

    // Start the server
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(shutdown_state))
        .await?;

    tracing::info!("RipTide API server shutdown complete");
    Ok(())
}

/// Graceful shutdown signal handler.
///
/// Listens for SIGTERM and SIGINT signals to gracefully shutdown the server.
/// This allows for proper cleanup of connections and resources including
/// the session cleanup background task.
async fn shutdown_signal(app_state: Arc<AppState>) {
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

    // Shutdown session cleanup task
    tracing::info!("Shutting down session cleanup task");
    app_state.session_manager.shutdown();

    // Give background tasks a moment to complete final cleanup
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    tracing::info!("All cleanup tasks completed");
}
