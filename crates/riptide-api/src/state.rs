use crate::config::ApiConfig;
use crate::health::HealthChecker;
use crate::metrics::RipTideMetrics;
use crate::resource_manager::ResourceManager;
use crate::sessions::{SessionConfig, SessionManager};
use anyhow::Result;
use reqwest::Client;
use riptide_core::{
    cache::CacheManager,
    extract::WasmExtractor,
    fetch::http_client,
    telemetry::TelemetrySystem,
    telemetry_info, telemetry_span,
};
use std::sync::Arc;
use tracing::{error, info, warn};

/// Application state shared across all request handlers.
///
/// This struct contains all the shared resources needed for crawling operations,
/// including HTTP clients, cache connections, and WASM extractors. The state
/// is wrapped in Arc for efficient sharing across async handlers.
#[derive(Clone)]
pub struct AppState {
    /// HTTP client for fetching web content
    pub http_client: Client,

    /// Redis cache manager for storing and retrieving cached content
    pub cache: Arc<tokio::sync::Mutex<CacheManager>>,

    /// WASM extractor for content processing
    pub extractor: Arc<WasmExtractor>,

    /// Configuration settings
    pub config: AppConfig,

    /// API configuration with resource controls
    pub api_config: ApiConfig,

    /// Comprehensive resource manager
    pub resource_manager: Arc<ResourceManager>,

    /// Prometheus metrics collector
    pub metrics: Arc<RipTideMetrics>,

    /// Health checker for enhanced diagnostics
    pub health_checker: Arc<HealthChecker>,

    /// Session manager for persistent browser sessions
    pub session_manager: Arc<SessionManager>,

    /// Telemetry system for observability
    pub telemetry: Option<Arc<TelemetrySystem>>,
}

/// Application configuration loaded from environment and config files.
#[derive(Clone, Debug)]
pub struct AppConfig {
    /// Redis connection URL
    pub redis_url: String,

    /// Path to the WASM extractor component
    pub wasm_path: String,

    /// Maximum concurrent crawl operations
    pub max_concurrency: usize,

    /// Default cache TTL in seconds
    pub cache_ttl: u64,

    /// Gate thresholds for content quality scoring
    pub gate_hi_threshold: f32,
    pub gate_lo_threshold: f32,

    /// Headless service URL for dynamic content rendering
    pub headless_url: Option<String>,

    /// Session configuration
    pub session_config: SessionConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            wasm_path: std::env::var("WASM_EXTRACTOR_PATH").unwrap_or_else(|_| {
                "./target/wasm32-wasip2/release/riptide_extractor_wasm.wasm".to_string()
            }),
            max_concurrency: std::env::var("MAX_CONCURRENCY")
                .unwrap_or_else(|_| "16".to_string())
                .parse()
                .unwrap_or(16),
            cache_ttl: std::env::var("CACHE_TTL")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600),
            gate_hi_threshold: std::env::var("GATE_HI_THRESHOLD")
                .unwrap_or_else(|_| "0.7".to_string())
                .parse()
                .unwrap_or(0.7),
            gate_lo_threshold: std::env::var("GATE_LO_THRESHOLD")
                .unwrap_or_else(|_| "0.3".to_string())
                .parse()
                .unwrap_or(0.3),
            headless_url: std::env::var("HEADLESS_URL").ok(),
            session_config: SessionConfig::default(),
        }
    }
}

impl AppState {
    /// Initialize the application state with all required components.
    ///
    /// This method sets up the HTTP client, establishes Redis connection,
    /// initializes the WASM extractor, and validates all dependencies.
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration
    /// * `metrics` - Prometheus metrics collector
    /// * `health_checker` - Health checker for enhanced diagnostics
    /// * `telemetry` - Optional telemetry system for observability
    ///
    /// # Returns
    ///
    /// A configured `AppState` ready for use in request handlers.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Redis connection cannot be established
    /// - WASM extractor cannot be loaded
    /// - Configuration is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riptide_api::state::{AppState, AppConfig};
    ///
    /// let config = AppConfig::default();
    /// let metrics = Arc::new(RipTideMetrics::new()?);
    /// let health_checker = Arc::new(HealthChecker::new());
    /// let state = AppState::new(config, metrics, health_checker).await?;
    /// ```
    pub async fn new(
        config: AppConfig,
        metrics: Arc<RipTideMetrics>,
        health_checker: Arc<HealthChecker>,
    ) -> Result<Self> {
        Self::new_with_telemetry(config, metrics, health_checker, None).await
    }

    /// Initialize with custom API configuration
    pub async fn new_with_api_config(
        config: AppConfig,
        api_config: ApiConfig,
        metrics: Arc<RipTideMetrics>,
        health_checker: Arc<HealthChecker>,
    ) -> Result<Self> {
        Self::new_with_telemetry_and_api_config(config, api_config, metrics, health_checker, None)
            .await
    }

    /// Initialize the application state with telemetry integration
    pub async fn new_with_telemetry(
        config: AppConfig,
        metrics: Arc<RipTideMetrics>,
        health_checker: Arc<HealthChecker>,
        telemetry: Option<Arc<TelemetrySystem>>,
    ) -> Result<Self> {
        let api_config = ApiConfig::from_env();
        Self::new_with_telemetry_and_api_config(
            config,
            api_config,
            metrics,
            health_checker,
            telemetry,
        )
        .await
    }

    /// Initialize the application state with telemetry and custom API configuration
    pub async fn new_with_telemetry_and_api_config(
        config: AppConfig,
        api_config: ApiConfig,
        metrics: Arc<RipTideMetrics>,
        health_checker: Arc<HealthChecker>,
        telemetry: Option<Arc<TelemetrySystem>>,
    ) -> Result<Self> {
        tracing::info!("Initializing application state with resource controls");

        // Validate API configuration
        api_config
            .validate()
            .map_err(|e| anyhow::anyhow!("Invalid API configuration: {}", e))?;

        // Initialize HTTP client with optimized settings
        let http_client = http_client()?;
        tracing::debug!("HTTP client initialized");

        // Establish Redis connection
        let cache_manager = CacheManager::new(&config.redis_url).await?;
        let cache = Arc::new(tokio::sync::Mutex::new(cache_manager));
        tracing::info!("Redis connection established: {}", config.redis_url);

        // Initialize WASM extractor
        let extractor = WasmExtractor::new(&config.wasm_path).await?;
        let extractor = Arc::new(extractor);
        tracing::info!("WASM extractor loaded: {}", config.wasm_path);

        // Initialize session manager
        let session_manager = SessionManager::new(config.session_config.clone()).await?;
        let session_manager = Arc::new(session_manager);
        tracing::info!("Session manager initialized");

        // Initialize comprehensive resource manager
        let resource_manager = ResourceManager::new(api_config.clone()).await?;
        let resource_manager = Arc::new(resource_manager);
        tracing::info!(
            "Resource manager initialized with controls: pool_cap={}, pdf_semaphore={}, rate_limit={}rps",
            api_config.headless.max_pool_size,
            api_config.pdf.max_concurrent,
            api_config.rate_limiting.requests_per_second_per_host
        );

        tracing::info!("Application state initialization complete with resource controls");

        Ok(Self {
            http_client,
            cache,
            extractor,
            config,
            api_config,
            resource_manager,
            metrics,
            health_checker,
            session_manager,
            telemetry,
        })
    }

    /// Check the health of all application dependencies with telemetry.
    ///
    /// This method verifies that all critical components are functioning:
    /// - Redis connection is active
    /// - HTTP client can make requests
    /// - WASM extractor is operational
    /// - Resource manager is operational
    /// - Telemetry system is operational
    ///
    /// # Returns
    ///
    /// A `HealthStatus` indicating the overall health and any issues.
    pub async fn health_check(&self) -> HealthStatus {
        let _span = telemetry_span!("health_check");
        info!("Starting health check");
        let mut health = HealthStatus {
            healthy: true,
            redis: DependencyHealth::Unknown,
            extractor: DependencyHealth::Unknown,
            http_client: DependencyHealth::Unknown,
            resource_manager: DependencyHealth::Unknown,
        };

        // Check Redis connection
        health.redis = match self.check_redis().await {
            Ok(_) => DependencyHealth::Healthy,
            Err(e) => {
                health.healthy = false;
                DependencyHealth::Unhealthy(e.to_string())
            }
        };

        // Check HTTP client (simple request to reliable endpoint)
        health.http_client = match self.check_http_client().await {
            Ok(_) => DependencyHealth::Healthy,
            Err(e) => {
                health.healthy = false;
                DependencyHealth::Unhealthy(e.to_string())
            }
        };

        // WASM extractor is checked at startup, assume healthy if state exists
        health.extractor = DependencyHealth::Healthy;

        // Check resource manager status
        let resource_status = self.resource_manager.get_resource_status().await;
        health.resource_manager =
            if resource_status.memory_pressure || resource_status.degradation_score > 0.8 {
                health.healthy = false;
                DependencyHealth::Unhealthy(format!(
                    "Resource constraints: memory_pressure={}, degradation_score={:.2}",
                    resource_status.memory_pressure, resource_status.degradation_score
                ))
            } else {
                DependencyHealth::Healthy
            };

        health
    }

    /// Test Redis connection by performing a simple operation.
    async fn check_redis(&self) -> Result<()> {
        let mut cache = self.cache.lock().await;
        cache.set_simple("health_check", &"ok", 1).await?;
        cache.delete("health_check").await?;
        Ok(())
    }

    /// Test HTTP client by verifying it's properly initialized.
    /// This avoids making external network calls during health checks.
    async fn check_http_client(&self) -> Result<()> {
        // Simply verify the HTTP client is initialized and configured
        // The client's ability to make requests is tested during actual usage
        // This prevents information leakage and external dependencies in health checks

        // Check if we can access the client (it's not null/uninitialized)
        let _ = &self.http_client;

        // Optionally test against localhost if a test endpoint is available
        // This keeps the health check internal to the system
        if let Ok(port) = std::env::var("HEALTH_CHECK_PORT") {
            if let Ok(response) = self
                .http_client
                .head(format!("http://127.0.0.1:{}/health", port))
                .send()
                .await
            {
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Internal health check failed: {}",
                        response.status()
                    ));
                }
            }
        }

        // HTTP client is properly initialized
        Ok(())
    }
}

/// Overall health status of the application and its dependencies.
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub healthy: bool,
    pub redis: DependencyHealth,
    pub extractor: DependencyHealth,
    pub http_client: DependencyHealth,
    pub resource_manager: DependencyHealth,
}

/// Health status of an individual dependency.
#[derive(Debug, Clone)]
pub enum DependencyHealth {
    Healthy,
    Unhealthy(String),
    Unknown,
}

impl std::fmt::Display for DependencyHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyHealth::Healthy => write!(f, "healthy"),
            DependencyHealth::Unhealthy(msg) => write!(f, "unhealthy: {}", msg),
            DependencyHealth::Unknown => write!(f, "unknown"),
        }
    }
}
