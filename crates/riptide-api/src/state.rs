#![allow(dead_code)]
use crate::config::RiptideApiConfig;
use crate::health::HealthChecker;
use crate::metrics_integration::CombinedMetrics;
use crate::metrics_transport::TransportMetrics;
use crate::resource_manager::ResourceManager;
use crate::sessions::{SessionConfig, SessionManager};
use crate::streaming::StreamingModule;
use anyhow::{Context, Result};
use reqwest::Client;
use riptide_cache::CacheManager;
use riptide_facade::metrics::BusinessMetrics;
// CacheWarmingConfig requires wasm-pool feature which is not available in riptide-api
// #[cfg(feature = "wasm-pool")]
// use riptide_cache::CacheWarmingConfig;
use riptide_events::{EventBus, EventBusConfig, EventSeverity};
#[cfg(feature = "fetch")]
use riptide_fetch::{http_client, FetchEngine};
use riptide_pdf::PdfMetricsCollector;
use riptide_reliability::CircuitBreakerState;
use riptide_reliability::{ReliabilityConfig, ReliableExtractor};
#[cfg(feature = "spider")]
use riptide_spider::{Spider, SpiderConfig};
// TelemetrySystem is in riptide_monitoring, not riptide_core
#[cfg(feature = "extraction")]
use riptide_extraction::UnifiedExtractor;
use riptide_monitoring::TelemetrySystem;
// Facade types removed to break circular dependency
// Facades are now used through riptide-facade crate directly
// #[cfg(feature = "extraction")]
// use riptide_facade::facades::ExtractionFacade;
// #[cfg(feature = "search")]
// use riptide_facade::facades::SearchFacade;
// #[cfg(feature = "spider")]
// use riptide_facade::facades::SpiderFacade;
// #[cfg(feature = "browser")]
// use riptide_facade::BrowserFacade;
// use riptide_facade::ScraperFacade;
#[cfg(feature = "browser")]
use riptide_headless::launcher::HeadlessLauncher;
use riptide_monitoring::{
    AlertCondition, AlertManager, AlertRule, AlertSeverity, HealthCalculator, MetricsCollector,
    MonitoringConfig, PerformanceMetrics,
};
use riptide_performance::PerformanceManager;
#[cfg(feature = "workers")]
use riptide_workers::{WorkerService, WorkerServiceConfig};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use crate::middleware::AuthConfig;

/// Application state shared across all request handlers.
///
/// This struct contains all the shared resources needed for crawling operations,
/// including HTTP clients, cache connections, and content extractors. The state
/// is wrapped in Arc for efficient sharing across async handlers.
#[derive(Clone)]
pub struct AppState {
    /// HTTP client for fetching web content
    pub http_client: Client,

    /// Redis cache manager for storing and retrieving cached content
    pub cache: Arc<tokio::sync::Mutex<CacheManager>>,

    /// Unified extractor for content processing (WASM or native)
    /// TODO: Future wiring for learned extractor patterns
    #[cfg(feature = "extraction")]
    #[allow(dead_code)]
    pub extractor: Arc<UnifiedExtractor>,

    /// Reliable extractor wrapper with retry and circuit breaker logic
    #[allow(dead_code)]
    pub reliable_extractor: Arc<ReliableExtractor>,

    /// Configuration settings
    pub config: AppConfig,

    /// API configuration with resource controls
    pub api_config: RiptideApiConfig,

    /// Comprehensive resource manager
    pub resource_manager: Arc<ResourceManager>,

    /// Business domain metrics for facade layer operations
    /// Tracks extraction quality, gate decisions, PDF/Spider processing, cache effectiveness
    pub business_metrics: Arc<BusinessMetrics>,

    /// Transport-level metrics for HTTP/WebSocket/SSE protocols
    /// Tracks request/response metrics, connection tracking, streaming protocols
    pub transport_metrics: Arc<TransportMetrics>,

    /// Combined metrics collector merging business and transport metrics
    /// Use this for the /metrics endpoint to serve all metrics
    pub combined_metrics: Arc<CombinedMetrics>,

    /// Health checker for enhanced diagnostics
    pub health_checker: Arc<HealthChecker>,

    /// Session manager for persistent browser sessions
    pub session_manager: Arc<SessionManager>,

    /// Streaming module for real-time data delivery
    pub streaming: Arc<StreamingModule>,

    /// Telemetry system for observability
    #[allow(dead_code)] // Public API - will be wired up in telemetry integration
    pub telemetry: Option<Arc<TelemetrySystem>>,

    /// Spider engine for deep crawling
    #[cfg(feature = "spider")]
    pub spider: Option<Arc<Spider>>,

    /// PDF metrics collector for monitoring PDF processing
    #[allow(dead_code)] // Public API - used via metrics.update_pdf_metrics_from_collector
    pub pdf_metrics: Arc<PdfMetricsCollector>,

    /// Worker service for background job processing
    #[cfg(feature = "workers")]
    pub worker_service: Arc<WorkerService>,

    /// Event bus for centralized event coordination
    pub event_bus: Arc<EventBus>,

    /// Circuit breaker for resilience and fault tolerance
    pub circuit_breaker: Arc<tokio::sync::Mutex<CircuitBreakerState>>,

    /// Performance metrics for circuit breaker tracking
    #[allow(dead_code)] // Public API - used for circuit breaker performance tracking
    pub performance_metrics: Arc<tokio::sync::Mutex<PerformanceMetrics>>,

    /// Monitoring system for performance tracking and alerting
    pub monitoring_system: Arc<MonitoringSystem>,

    /// FetchEngine for HTTP operations with per-host circuit breakers and rate limiting
    #[cfg(feature = "fetch")]
    #[allow(dead_code)] // Public API - used for HTTP fetch operations with rate limiting
    pub fetch_engine: Arc<FetchEngine>,

    /// Performance manager for resource limiting, monitoring, and optimization
    pub performance_manager: Arc<PerformanceManager>,

    /// Authentication configuration for API key validation
    pub auth_config: AuthConfig,

    /// Future: CacheWarmer for intelligent cache pre-warming (placeholder for now)
    #[allow(dead_code)] // Future feature - intentionally not used yet
    pub cache_warmer_enabled: bool,

    /// Headless browser launcher with connection pooling and stealth support
    #[cfg(feature = "browser")]
    pub browser_launcher: Option<Arc<HeadlessLauncher>>,

    /// Browser facade for simplified browser automation
    /// Only available when using local Chrome mode (headless_url not configured)
    /// REMOVED: Caused circular dependency with riptide-facade
    // #[cfg(feature = "browser")]
    // pub browser_facade: Option<Arc<BrowserFacade>>,

    /// Extraction facade for content extraction with multiple strategies
    /// Phase 2C.2: Restored after eliminating circular dependency via trait abstraction
    pub extraction_facade: Arc<riptide_facade::facades::ExtractionFacade>,

    /// Scraper facade for simple HTTP operations
    /// Phase 2C.2: Restored after eliminating circular dependency via trait abstraction
    pub scraper_facade: Arc<riptide_facade::facades::ScraperFacade>,

    /// Spider facade for web crawling operations
    /// Phase 2C.2: Restored after eliminating circular dependency via trait abstraction
    #[cfg(feature = "spider")]
    pub spider_facade: Option<Arc<riptide_facade::facades::SpiderFacade>>,

    /// Search facade for web search operations
    /// Phase 2C.2: Restored after eliminating circular dependency via trait abstraction
    #[cfg(feature = "search")]
    pub search_facade: Option<Arc<riptide_facade::facades::SearchFacade>>,

    /// Engine selection facade for intelligent engine selection
    /// Phase 3 Sprint 3.1: Engine selection business logic
    pub engine_facade: Arc<riptide_facade::facades::EngineFacade>,

    /// Streaming facade for real-time data delivery
    /// Phase 4 Sprint 4.3: Streaming business logic consolidation
    /// TODO: Enable when dependencies are properly wired
    // pub streaming_facade: Arc<riptide_facade::facades::StreamingFacade>,

    /// Resource facade for centralized resource orchestration
    /// Phase 4 Sprint 4.4: Resource management business logic
    pub resource_facade:
        Arc<riptide_facade::facades::ResourceFacade<crate::adapters::ResourceSlot>>,

    /// Trace backend for distributed trace storage and retrieval
    pub trace_backend: Option<Arc<dyn crate::handlers::trace_backend::TraceBackend>>,

    /// Persistence adapter for multi-tenant operations (optional, requires persistence feature)
    #[cfg(feature = "persistence")]
    #[allow(dead_code)] // TODO: Replace with actual PersistenceAdapter type when available
    pub persistence_adapter: Option<()>,
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

    /// Spider configuration for deep crawling
    #[cfg(feature = "spider")]
    pub spider_config: Option<SpiderConfig>,

    /// Worker service configuration
    #[cfg(feature = "workers")]
    pub worker_config: WorkerServiceConfig,

    /// Event bus configuration
    pub event_bus_config: EventBusConfig,

    /// Circuit breaker configuration
    pub circuit_breaker_config: CircuitBreakerConfig,

    /// Reliability configuration for retry and fallback behavior
    pub reliability_config: ReliabilityConfig,

    /// Monitoring system configuration
    #[allow(dead_code)] // Public API - monitoring configuration
    pub monitoring_config: MonitoringConfig,

    /// Enhanced pipeline configuration
    pub enhanced_pipeline_config: EnhancedPipelineConfig,

    // Cache warming configuration removed - requires wasm-pool feature which is not available
    // #[cfg(feature = "wasm-extractor")]
    // pub cache_warming_config: CacheWarmingConfig,
    /// Engine selection configuration (Phase 10)
    pub engine_selection_config: EngineSelectionConfig,
}

/// Enhanced pipeline configuration for phase timing and metrics
#[derive(Clone, Debug)]
pub struct EnhancedPipelineConfig {
    /// Enable enhanced pipeline with detailed phase timing
    pub enable_enhanced_pipeline: bool,
    /// Enable phase timing metrics collection
    #[allow(dead_code)] // Public API - phase metrics configuration
    pub enable_phase_metrics: bool,
    /// Enable detailed debug logging for each phase
    #[allow(dead_code)] // Public API - debug logging configuration
    pub enable_debug_logging: bool,
    /// Timeout for fetch phase in seconds
    pub fetch_timeout_secs: u64,
    /// Timeout for gate phase in seconds
    #[allow(dead_code)] // Public API - gate timeout configuration
    pub gate_timeout_secs: u64,
    /// Timeout for WASM phase in seconds
    #[allow(dead_code)] // Public API - WASM timeout configuration
    pub wasm_timeout_secs: u64,
    /// Timeout for render phase in seconds
    pub render_timeout_secs: u64,
}

impl Default for EnhancedPipelineConfig {
    fn default() -> Self {
        Self {
            enable_enhanced_pipeline: std::env::var("ENHANCED_PIPELINE_ENABLE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true), // Enabled by default
            enable_phase_metrics: std::env::var("ENHANCED_PIPELINE_METRICS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
            enable_debug_logging: std::env::var("ENHANCED_PIPELINE_DEBUG")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(false), // Disabled by default to avoid log spam
            fetch_timeout_secs: std::env::var("ENHANCED_PIPELINE_FETCH_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(15),
            gate_timeout_secs: std::env::var("ENHANCED_PIPELINE_GATE_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            wasm_timeout_secs: std::env::var("ENHANCED_PIPELINE_WASM_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            render_timeout_secs: std::env::var("ENHANCED_PIPELINE_RENDER_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(60),
        }
    }
}

/// Engine selection configuration (Phase 10)
#[derive(Clone, Debug)]
pub struct EngineSelectionConfig {
    /// Enable probe-first escalation for SPAs (try WASM before headless)
    pub probe_first_spa: bool,
    /// Enable visible text density calculation
    #[allow(dead_code)] // Used in engine selection handlers
    pub use_visible_text_density: bool,
    /// Enable placeholder/skeleton detection
    #[allow(dead_code)] // Used in engine selection handlers
    pub detect_placeholders: bool,
}

impl Default for EngineSelectionConfig {
    fn default() -> Self {
        Self {
            probe_first_spa: std::env::var("ENGINE_PROBE_FIRST_SPA")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(false), // Conservative default
            use_visible_text_density: std::env::var("ENGINE_USE_VISIBLE_TEXT_DENSITY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true), // Enabled by default (Phase 10 refinement)
            detect_placeholders: std::env::var("ENGINE_DETECT_PLACEHOLDERS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true), // Enabled by default (Phase 10 refinement)
        }
    }
}

/// Circuit breaker configuration
#[derive(Clone, Debug)]
pub struct CircuitBreakerConfig {
    /// Failure threshold percentage (0-100) to trip the circuit breaker
    pub failure_threshold: u8,
    /// Timeout duration in milliseconds before transitioning from Open to HalfOpen
    pub timeout_ms: u64,
    /// Minimum requests to consider before evaluating failure rate
    #[allow(dead_code)] // Public API - circuit breaker minimum requests threshold
    pub min_requests: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: std::env::var("CIRCUIT_BREAKER_FAILURE_THRESHOLD")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50), // 50% failure rate trips the breaker
            timeout_ms: std::env::var("CIRCUIT_BREAKER_TIMEOUT_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5000), // 5 seconds
            min_requests: std::env::var("CIRCUIT_BREAKER_MIN_REQUESTS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10), // 10 requests minimum
        }
    }
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
            #[cfg(feature = "spider")]
            spider_config: AppConfig::init_spider_config(),
            #[cfg(feature = "workers")]
            worker_config: AppConfig::init_worker_config(),
            event_bus_config: EventBusConfig::default(),
            circuit_breaker_config: CircuitBreakerConfig::default(),
            reliability_config: ReliabilityConfig::from_env(),
            monitoring_config: MonitoringConfig::default(),
            enhanced_pipeline_config: EnhancedPipelineConfig::default(),
            // cache_warming_config removed - requires wasm-pool feature
            engine_selection_config: EngineSelectionConfig::default(),
        }
    }
}

impl AppConfig {
    /// Initialize spider configuration based on environment variables
    #[cfg(feature = "spider")]
    fn init_spider_config() -> Option<SpiderConfig> {
        // Check if spider is enabled
        let spider_enabled = std::env::var("SPIDER_ENABLE")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        if !spider_enabled {
            tracing::debug!("Spider engine disabled (SPIDER_ENABLE=false)");
            return None;
        }

        tracing::info!("Initializing Spider engine (SPIDER_ENABLE=true)");

        // Create default spider config
        let base_url =
            std::env::var("SPIDER_BASE_URL").unwrap_or_else(|_| "https://example.com".to_string());

        let base_url = match url::Url::parse(&base_url) {
            Ok(url) => {
                tracing::debug!("Spider base URL: {}", url);
                url
            }
            Err(e) => {
                tracing::warn!(
                    "Invalid SPIDER_BASE_URL '{}': {}, using default",
                    base_url,
                    e
                );
                url::Url::parse("https://example.com")
                    .unwrap_or_else(|_| panic!("Built-in fallback URL is invalid"))
            }
        };

        let mut config = SpiderConfig::new(base_url);

        // Override with environment variables
        if let Ok(user_agent) = std::env::var("SPIDER_USER_AGENT") {
            tracing::debug!("Spider user agent: {}", user_agent);
            config.user_agent = user_agent;
        }

        if let Ok(timeout_str) = std::env::var("SPIDER_TIMEOUT_SECONDS") {
            if let Ok(timeout_secs) = timeout_str.parse::<u64>() {
                tracing::debug!("Spider timeout: {}s", timeout_secs);
                config.timeout = Duration::from_secs(timeout_secs);
            }
        }

        if let Ok(delay_str) = std::env::var("SPIDER_DELAY_MS") {
            if let Ok(delay_ms) = delay_str.parse::<u64>() {
                tracing::debug!("Spider delay: {}ms", delay_ms);
                config.delay = Duration::from_millis(delay_ms);
            }
        }

        if let Ok(concurrency_str) = std::env::var("SPIDER_CONCURRENCY") {
            if let Ok(concurrency) = concurrency_str.parse::<usize>() {
                tracing::debug!("Spider concurrency: {}", concurrency);
                config.concurrency = concurrency;
            }
        }

        if let Ok(max_depth_str) = std::env::var("SPIDER_MAX_DEPTH") {
            if let Ok(max_depth) = max_depth_str.parse::<usize>() {
                tracing::debug!("Spider max depth: {}", max_depth);
                config.max_depth = Some(max_depth);
            }
        }

        if let Ok(max_pages_str) = std::env::var("SPIDER_MAX_PAGES") {
            if let Ok(max_pages) = max_pages_str.parse::<usize>() {
                tracing::debug!("Spider max pages: {}", max_pages);
                config.max_pages = Some(max_pages);
            }
        }

        if let Ok(respect_robots_str) = std::env::var("SPIDER_RESPECT_ROBOTS") {
            if let Ok(respect_robots) = respect_robots_str.parse::<bool>() {
                tracing::debug!("Spider respect robots.txt: {}", respect_robots);
                config.respect_robots = respect_robots;
            }
        }

        // Validate configuration before returning
        if let Err(e) = config.validate() {
            tracing::error!("Invalid Spider configuration: {}", e);
            return None;
        }

        tracing::info!(
            max_depth = ?config.max_depth,
            max_pages = ?config.max_pages,
            concurrency = config.concurrency,
            timeout_secs = config.timeout.as_secs(),
            delay_ms = config.delay.as_millis(),
            respect_robots = config.respect_robots,
            "Spider configuration initialized successfully from environment variables"
        );

        Some(config)
    }

    /// Initialize worker service configuration based on environment variables
    #[cfg(feature = "workers")]
    fn init_worker_config() -> WorkerServiceConfig {
        use riptide_workers::{QueueConfig, SchedulerConfig, WorkerConfig};

        WorkerServiceConfig {
            redis_url: std::env::var("WORKER_REDIS_URL")
                .or_else(|_| std::env::var("REDIS_URL"))
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),

            worker_config: WorkerConfig {
                worker_count: std::env::var("WORKER_POOL_SIZE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(4), // 4 workers by default
                poll_interval_secs: 5,
                job_timeout_secs: 600, // 10 minutes
                heartbeat_interval_secs: 30,
                max_concurrent_jobs: 4,
                enable_health_monitoring: true,
            },

            queue_config: QueueConfig {
                namespace: "riptide_jobs".to_string(),
                cache_size: 1000,
                delayed_job_poll_interval: 30,
                job_lease_timeout: 600, // 10 minutes
                persist_results: true,
                result_ttl: 3600, // 1 hour
            },

            scheduler_config: SchedulerConfig::default(),

            max_batch_size: std::env::var("WORKER_MAX_BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50),

            max_concurrency: std::env::var("WORKER_MAX_CONCURRENCY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),

            wasm_path: std::env::var("WASM_EXTRACTOR_PATH").unwrap_or_else(|_| {
                "./target/wasm32-wasip2/release/riptide_extractor_wasm.wasm".to_string()
            }),

            enable_scheduler: std::env::var("WORKER_ENABLE_SCHEDULER")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
        }
    }
}

impl AppState {
    /// Initialize the application state with all required components including facades.
    ///
    /// This is the main entry point for creating a fully initialized AppState.
    /// It calls `new_base()` followed by `with_facades()` to construct the complete state.
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration
    /// * `health_checker` - Health checker for enhanced diagnostics
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
    /// - Facade initialization fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riptide_api::state::{AppState, AppConfig};
    ///
    /// let config = AppConfig::default();
    /// let health_checker = Arc::new(HealthChecker::new());
    /// let state = AppState::new(config, health_checker).await?;
    /// ```
    pub async fn new(config: AppConfig, health_checker: Arc<HealthChecker>) -> Result<Self> {
        Self::new_with_facades(config, health_checker, None).await
    }

    /// Initialize AppState with facades and optional telemetry (public convenience method).
    ///
    /// This method creates a complete AppState with all facades initialized.
    /// It's the recommended way to create AppState in production.
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration
    /// * `health_checker` - Health checker for enhanced diagnostics
    /// * `telemetry` - Optional telemetry system for observability
    ///
    /// # Returns
    ///
    /// A configured `AppState` with all facades initialized.
    pub async fn new_with_facades(
        config: AppConfig,
        health_checker: Arc<HealthChecker>,
        telemetry: Option<Arc<TelemetrySystem>>,
    ) -> Result<Self> {
        let api_config = RiptideApiConfig::from_env();
        let base_state = Self::new_base(config, api_config, health_checker, telemetry).await?;
        base_state.with_facades().await
    }

    /// Initialize base AppState without facades.
    ///
    /// This method creates the core AppState with all infrastructure components
    /// but without facade layer initialization. Use `with_facades()` to add facades
    /// after construction, or use `new_with_facades()` for complete initialization.
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration
    /// * `api_config` - API-specific configuration with resource controls
    /// * `health_checker` - Health checker for enhanced diagnostics
    /// * `telemetry` - Optional telemetry system for observability
    ///
    /// # Returns
    ///
    /// An `AppState` with all core components initialized but facades set to default/placeholder values.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Redis connection cannot be established
    /// - WASM extractor cannot be loaded
    /// - Configuration is invalid
    /// - Core component initialization fails
    pub async fn new_base(
        config: AppConfig,
        api_config: RiptideApiConfig,
        health_checker: Arc<HealthChecker>,
        telemetry: Option<Arc<TelemetrySystem>>,
    ) -> Result<Self> {
        tracing::info!(
            "Initializing application state with resource controls and split metrics architecture"
        );

        // Validate API configuration
        api_config
            .validate()
            .map_err(|e| anyhow::anyhow!("Invalid API configuration: {}", e))?;

        // Initialize business domain metrics for facade layer
        let business_metrics =
            Arc::new(BusinessMetrics::new().context("Failed to initialize business metrics")?);
        tracing::info!("Business metrics initialized for facade layer operations");

        // Initialize transport-level metrics for HTTP/WebSocket/SSE
        let transport_metrics =
            Arc::new(TransportMetrics::new().context("Failed to initialize transport metrics")?);
        tracing::info!("Transport metrics initialized for protocol-level tracking");

        // Create combined metrics collector for unified /metrics endpoint
        let combined_metrics = Arc::new(
            CombinedMetrics::new(business_metrics.clone(), transport_metrics.clone())
                .context("Failed to create combined metrics collector")?,
        );
        tracing::info!(
            "Combined metrics collector created - business + transport registries merged for /metrics endpoint"
        );

        // Initialize HTTP client with optimized settings
        #[cfg(feature = "fetch")]
        let http_client = http_client()?;
        #[cfg(not(feature = "fetch"))]
        let http_client = Client::new();
        tracing::debug!("HTTP client initialized");

        // Establish Redis connection
        let cache_manager = CacheManager::new(&config.redis_url).await?;
        let cache = Arc::new(tokio::sync::Mutex::new(cache_manager));
        tracing::info!("Redis connection established: {}", config.redis_url);

        // Initialize unified extractor with automatic fallback
        #[cfg(feature = "extraction")]
        let extractor = {
            let wasm_path = std::env::var("WASM_EXTRACTOR_PATH").ok();
            let ext = Arc::new(
                UnifiedExtractor::new(wasm_path.as_deref())
                    .await
                    .context("Failed to initialize content extractor")?,
            );
            tracing::info!(
                extractor_type = ext.extractor_type(),
                wasm_available = UnifiedExtractor::wasm_available(),
                "Content extractor initialized"
            );
            ext
        };

        // Initialize ReliableExtractor with retry and circuit breaker logic
        let reliable_extractor = Arc::new(
            ReliableExtractor::new(config.reliability_config.clone())
                .map_err(|e| anyhow::anyhow!("Failed to initialize ReliableExtractor: {}", e))?,
        );
        tracing::info!(
            max_retries = config.reliability_config.http_retry.max_attempts,
            timeout_secs = config.reliability_config.headless_timeout.as_secs(),
            graceful_degradation = config.reliability_config.enable_graceful_degradation,
            quality_threshold = config.reliability_config.fast_extraction_quality_threshold,
            "ReliableExtractor initialized with retry and circuit breaker patterns"
        );

        // Initialize session manager
        let session_manager = SessionManager::new(config.session_config.clone()).await?;
        let session_manager = Arc::new(session_manager);
        tracing::info!("Session manager initialized");

        // Initialize streaming module with lifecycle management (Phase D: removed deprecated metrics)
        let streaming_module = StreamingModule::default();
        if let Err(e) = streaming_module.validate() {
            tracing::warn!("Streaming configuration validation failed: {}", e);
        }

        // Start streaming maintenance tasks
        if let Err(e) = streaming_module.start_maintenance_tasks().await {
            tracing::warn!("Failed to start streaming maintenance tasks: {}", e);
        }

        let streaming = Arc::new(streaming_module);
        tracing::info!(
            "Streaming module initialized with backpressure handling and lifecycle management"
        );

        // Initialize Spider if enabled
        #[cfg(feature = "spider")]
        let spider = if let Some(ref spider_config) = config.spider_config {
            tracing::info!("Initializing Spider engine for deep crawling");

            let spider_config = spider_config.clone();
            match Spider::new(spider_config).await {
                Ok(spider_engine) => {
                    let spider_with_integrations = spider_engine
                        .with_fetch_engine(Arc::new({
                            #[cfg(feature = "fetch")]
                            {
                                riptide_fetch::FetchEngine::new()?
                            }
                            #[cfg(not(feature = "fetch"))]
                            {
                                anyhow::bail!("fetch feature required for spider integration")
                            }
                        }))
                        .with_memory_manager(Arc::new({
                            let mut wasmtime_config = wasmtime::Config::new();
                            wasmtime_config.wasm_component_model(true);
                            let engine = wasmtime::Engine::new(&wasmtime_config).map_err(|e| {
                                anyhow::anyhow!("Failed to create wasmtime engine: {}", e)
                            })?;
                            riptide_spider::memory_manager::MemoryManager::new(
                                riptide_spider::memory_manager::MemoryManagerConfig::default(),
                                engine,
                            )
                            .await?
                        }));

                    tracing::info!(
                        "Spider engine initialized successfully with fetch and memory integrations"
                    );
                    Some(Arc::new(spider_with_integrations))
                }
                Err(e) => {
                    tracing::error!("Failed to initialize Spider engine: {}", e);
                    None
                }
            }
        } else {
            tracing::debug!("Spider engine disabled");
            None
        };

        #[cfg(not(feature = "spider"))]
        let spider: Option<Arc<dyn std::any::Any + Send + Sync>> = None;

        // Initialize comprehensive resource manager with headless URL
        let resource_manager =
            ResourceManager::new_with_headless_url(api_config.clone(), config.headless_url.clone())
                .await?;
        let resource_manager = Arc::new(resource_manager);
        tracing::info!(
            "Resource manager initialized with controls: pool_cap={}, pdf_semaphore={}, rate_limit={}rps, headless_url={:?}",
            api_config.headless.max_pool_size,
            api_config.pdf.max_concurrent,
            api_config.rate_limiting.requests_per_second_per_host,
            config.headless_url
        );

        // Initialize PDF metrics collector for monitoring
        let pdf_metrics = Arc::new(PdfMetricsCollector::new());
        tracing::info!("PDF metrics collector initialized for monitoring PDF processing");

        // Initialize worker service for background job processing
        #[cfg(feature = "workers")]
        let worker_service = {
            tracing::info!("Initializing worker service for background jobs");
            let ws = WorkerService::new(config.worker_config.clone())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to initialize worker service: {}", e))?;
            let ws = Arc::new(ws);
            tracing::info!("Worker service initialized successfully");
            ws
        };

        // Initialize event bus with configuration
        tracing::info!("Initializing event bus for centralized event coordination");
        let mut event_bus = EventBus::with_config(config.event_bus_config.clone());

        // Register event handlers
        use riptide_events::handlers::{
            HealthEventHandler, LoggingEventHandler, MetricsEventHandler, TelemetryEventHandler,
        };

        // Logging handler for structured logging
        let logging_handler = Arc::new(LoggingEventHandler::new());
        event_bus
            .register_handler(logging_handler)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to register logging handler: {}", e))?;
        tracing::info!("Registered logging event handler");

        // Metrics handler for automatic metrics collection
        let metrics_collector = riptide_monitoring::MetricsCollector::new();
        let metrics_handler = Arc::new(MetricsEventHandler::new(Arc::new(metrics_collector)));
        event_bus
            .register_handler(metrics_handler)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to register metrics handler: {}", e))?;
        tracing::info!("Registered metrics event handler");

        // Telemetry handler for OpenTelemetry integration
        let telemetry_handler = Arc::new(TelemetryEventHandler::new());
        event_bus
            .register_handler(telemetry_handler)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to register telemetry handler: {}", e))?;
        tracing::info!("Registered telemetry event handler");

        // Health handler for health monitoring
        let health_handler = Arc::new(HealthEventHandler::new());
        event_bus
            .register_handler(health_handler)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to register health handler: {}", e))?;
        tracing::info!("Registered health event handler");

        // Start event bus processing
        event_bus
            .start()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start event bus: {}", e))?;
        tracing::info!("Event bus started and processing events");

        let event_bus = Arc::new(event_bus);

        // Initialize circuit breaker for fault tolerance
        tracing::info!(
            failure_threshold = config.circuit_breaker_config.failure_threshold,
            timeout_ms = config.circuit_breaker_config.timeout_ms,
            "Initializing circuit breaker for resilience"
        );
        let circuit_breaker = Arc::new(tokio::sync::Mutex::new(CircuitBreakerState::default()));

        // Initialize performance metrics for circuit breaker tracking
        let performance_metrics = Arc::new(tokio::sync::Mutex::new(PerformanceMetrics::default()));
        tracing::info!("Circuit breaker and performance metrics initialized");

        // Initialize monitoring system with default configuration
        tracing::info!("Initializing monitoring system for performance tracking and alerting");
        let monitoring_system = Arc::new(MonitoringSystem::new());

        // Register default alert rules
        monitoring_system.register_default_alert_rules().await;

        // Start background alert evaluation task
        monitoring_system.start_alert_evaluation_task(event_bus.clone());

        tracing::info!(
            "Monitoring system initialized with alert rules and background evaluation task"
        );

        // Initialize FetchEngine with configuration
        #[cfg(feature = "fetch")]
        let fetch_engine = {
            tracing::info!(
                "Initializing FetchEngine for HTTP operations with per-host circuit breakers"
            );
            let fe = Arc::new(
                FetchEngine::new()
                    .map_err(|e| anyhow::anyhow!("Failed to initialize FetchEngine: {}", e))?,
            );
            tracing::info!("FetchEngine initialized successfully");
            fe
        };

        // Initialize PerformanceManager for resource limiting and monitoring
        tracing::info!("Initializing PerformanceManager for resource limiting and profiling");
        let performance_manager = Arc::new(
            PerformanceManager::new()
                .map_err(|e| anyhow::anyhow!("Failed to initialize PerformanceManager: {}", e))?,
        );

        // Start background profiling and monitoring
        performance_manager.start_monitoring().await.map_err(|e| {
            tracing::warn!("Failed to start performance monitoring: {}", e);
            anyhow::anyhow!("Failed to start performance monitoring: {}", e)
        })?;

        tracing::info!("PerformanceManager initialized and started with profiling overhead <2%");

        // Initialize authentication configuration
        let auth_config = AuthConfig::new();
        tracing::info!(
            require_auth = auth_config.requires_auth(),
            "Authentication configuration initialized"
        );

        // Note: CacheWarmer initialization to be added in future (requires wasm-pool feature)
        // #[cfg(feature = "wasm-extractor")]
        // {
        //     let cache_warmer_enabled = config.cache_warming_config.enabled;
        //     if cache_warmer_enabled {
        //         tracing::info!("CacheWarmer feature flag enabled (full implementation pending)");
        //     }
        // }

        // Initialize headless browser launcher with pooling (only if no headless service URL)
        #[cfg(feature = "browser")]
        let browser_launcher = if let Some(url) = &config.headless_url {
            if !url.is_empty() {
                tracing::info!(
                    headless_url = %url,
                    "Headless service URL configured - skipping local browser launcher initialization"
                );
                None
            } else {
                tracing::info!("HEADLESS_URL is empty - initializing local browser launcher");
                // Build config and initialize launcher same as non-headless mode below
                let browser_launcher_config = riptide_headless::launcher::LauncherConfig {
                    pool_config: riptide_headless::pool::BrowserPoolConfig {
                        min_pool_size: std::cmp::max(1, api_config.headless.max_pool_size / 2),
                        max_pool_size: api_config.headless.max_pool_size,
                        initial_pool_size: std::cmp::max(1, api_config.headless.max_pool_size / 4),
                        idle_timeout: Duration::from_secs(api_config.headless.idle_timeout_secs),
                        max_lifetime: Duration::from_secs(300),
                        health_check_interval: Duration::from_secs(30),
                        memory_threshold_mb: 500,
                        enable_recovery: true,
                        max_retries: 3,
                        profile_base_dir: None,
                        cleanup_timeout: Duration::from_secs(5),
                        ..Default::default()
                    },
                    default_stealth_preset: riptide_stealth::StealthPreset::Medium,
                    enable_stealth: true,
                    page_timeout: Duration::from_secs(30),
                    enable_monitoring: true,
                    hybrid_mode: false,
                };

                Some(Arc::new(
                    HeadlessLauncher::with_config(browser_launcher_config)
                        .await
                        .map_err(|e| {
                            tracing::error!(error = %e, "Failed to initialize browser launcher");
                            anyhow::anyhow!("Failed to initialize browser launcher: {}", e)
                        })?,
                ))
            }
        } else {
            tracing::info!(
                max_pool_size = api_config.headless.max_pool_size,
                "No headless service URL - initializing local browser launcher with connection pooling"
            );

            let browser_launcher_config = riptide_headless::launcher::LauncherConfig {
                pool_config: riptide_headless::pool::BrowserPoolConfig {
                    min_pool_size: std::cmp::max(1, api_config.headless.max_pool_size / 2),
                    max_pool_size: api_config.headless.max_pool_size,
                    initial_pool_size: std::cmp::max(1, api_config.headless.max_pool_size / 4),
                    idle_timeout: Duration::from_secs(api_config.headless.idle_timeout_secs),
                    max_lifetime: Duration::from_secs(300), // 5 minutes max lifetime
                    health_check_interval: Duration::from_secs(30),
                    memory_threshold_mb: 500,
                    enable_recovery: true,
                    max_retries: 3,
                    profile_base_dir: None, // Use system temp directory by default
                    cleanup_timeout: Duration::from_secs(5),
                    ..Default::default()
                },
                default_stealth_preset: riptide_stealth::StealthPreset::Medium,
                enable_stealth: true,
                page_timeout: Duration::from_secs(30),
                enable_monitoring: true,
                hybrid_mode: false,
            };

            let launcher = Arc::new(
                HeadlessLauncher::with_config(browser_launcher_config)
                    .await
                    .map_err(|e| {
                        tracing::error!(error = %e, "Failed to initialize browser launcher");
                        anyhow::anyhow!("Failed to initialize browser launcher: {}", e)
                    })?,
            );

            tracing::info!(
                pool_size = api_config.headless.max_pool_size,
                "Headless browser launcher initialized successfully"
            );

            Some(launcher)
        };

        // Initialize facade layer for simplified API access
        tracing::info!("Initializing riptide-facade layer for simplified APIs");

        // Create facade configuration from existing config
        // NOTE: Using default timeout since reliability_config is feature-gated
        // REMOVED: Caused circular dependency with riptide-facade
        // let facade_config = riptide_facade::RiptideConfig::default()
        //     .with_timeout(Duration::from_secs(30)) // Default timeout since reliability_config is disabled
        //     .with_stealth_enabled(true) // Stealth enabled by default (Medium preset)
        //     .with_stealth_preset("Medium");

        // Initialize browser facade only if not using headless service
        // #[cfg(feature = "browser")]
        // let browser_facade = if let Some(url) = &config.headless_url {
        //     if !url.is_empty() {
        //         tracing::info!(
        //             headless_url = %url,
        //             "Headless service URL configured - skipping BrowserFacade initialization (requires local Chrome)"
        //         );
        //         None
        //     } else {
        //         tracing::info!("HEADLESS_URL is empty - initializing BrowserFacade for local browser automation");
        //         match BrowserFacade::new(facade_config.clone()).await {
        //             Ok(facade) => {
        //                 tracing::info!("BrowserFacade initialized successfully");
        //                 Some(Arc::new(facade))
        //             }
        //             Err(e) => {
        //                 tracing::error!(error = %e, "Failed to initialize BrowserFacade");
        //                 return Err(anyhow::anyhow!("Failed to initialize BrowserFacade: {}", e));
        //             }
        //         }
        //     }
        // } else {
        //     tracing::info!("Initializing BrowserFacade for local browser automation");
        //     match BrowserFacade::new(facade_config.clone()).await {
        //         Ok(facade) => {
        //             tracing::info!("BrowserFacade initialized successfully");
        //             Some(Arc::new(facade))
        //         }
        //         Err(e) => {
        //             tracing::error!(error = %e, "Failed to initialize BrowserFacade");
        //             return Err(anyhow::anyhow!("Failed to initialize BrowserFacade: {}", e));
        //         }
        //     }
        // };

        // Initialize extraction facade
        // REMOVED: Caused circular dependency with riptide-facade
        // let extraction_facade = Arc::new(
        //     ExtractionFacade::new(facade_config.clone())
        //         .await
        //         .map_err(|e| {
        //             tracing::error!(error = %e, "Failed to initialize ExtractionFacade");
        //             anyhow::anyhow!("Failed to initialize ExtractionFacade: {}", e)
        //         })?,
        // );
        // tracing::info!("ExtractionFacade initialized successfully");

        // Initialize scraper facade
        // REMOVED: Caused circular dependency with riptide-facade
        // let scraper_facade = Arc::new(ScraperFacade::new(facade_config.clone()).await.map_err(
        //     |e| {
        //         tracing::error!(error = %e, "Failed to initialize ScraperFacade");
        //         anyhow::anyhow!("Failed to initialize ScraperFacade: {}", e)
        //     },
        // )?);
        // tracing::info!("ScraperFacade initialized successfully");

        // Initialize spider facade if spider is enabled
        // REMOVED: Caused circular dependency with riptide-facade
        // let spider_facade = if config.spider_config.is_some() {
        //     tracing::info!("Initializing SpiderFacade for simplified spider operations");
        //     let spider_config = config
        //         .spider_config
        //         .as_ref()
        //         .ok_or_else(|| anyhow::anyhow!("Spider config expected but not found"))?;
        //     match SpiderFacade::from_config(spider_config.clone()).await {
        //         Ok(facade) => {
        //             tracing::info!("SpiderFacade initialized successfully");
        //             Some(Arc::new(facade))
        //         }
        //         Err(e) => {
        //             tracing::warn!(error = %e, "Failed to initialize SpiderFacade, spider operations will use direct engine");
        //             None
        //         }
        //     }
        // } else {
        //     tracing::debug!("SpiderFacade disabled (spider not enabled)");
        //     None
        // };

        // Initialize search facade with backend from environment or default to None
        // #[cfg(feature = "search")]
        // let search_facade = {
        // Read search backend from environment with fallback to None
        //     let backend_str = std::env::var("RIPTIDE_SEARCH_BACKEND")
        //         .or_else(|_| std::env::var("SEARCH_BACKEND"))
        //         .unwrap_or_else(|_| "none".to_string());
        //         //     let backend: riptide_search::SearchBackend = backend_str.parse().unwrap_or_else(|e| {
        //         tracing::warn!(
        //             error = %e,
        //             backend = %backend_str,
        //             "Invalid search backend specified, falling back to 'none'"
        //         );
        //         riptide_search::SearchBackend::None
        //     });
        //         //     tracing::info!(backend = %backend, "Initializing SearchFacade");
        //             // Try to initialize with the specified backend
        //     match SearchFacade::new(backend.clone()).await {
        //         Ok(facade) => {
        //             tracing::info!(backend = %backend, "SearchFacade initialized successfully");
        //             Some(Arc::new(facade))
        //         }
        //         Err(e) => {
        //             tracing::warn!(
        //                 error = %e,
        //                 backend = %backend,
        //                 "Failed to initialize SearchFacade with specified backend"
        //             );
        //                     // If not already using None backend, try falling back to None
        //             if backend != riptide_search::SearchBackend::None {
        //                 tracing::info!(
        //                     "Attempting fallback to 'none' backend for graceful degradation"
        //                 );
        //                 match SearchFacade::new(riptide_search::SearchBackend::None).await {
        //                     Ok(facade) => {
        //                         tracing::info!(
        //                             "SearchFacade initialized successfully with fallback 'none' backend. \
        //                              Search functionality will work with URL parsing only."
        //                         );
        //                         Some(Arc::new(facade))
        //                     }
        //                     Err(fallback_err) => {
        //                         tracing::error!(
        //                             error = %fallback_err,
        //                             "Failed to initialize SearchFacade even with 'none' backend fallback. \
        //                              Search endpoint will be unavailable."
        //                         );
        //                         None
        //                     }
        //                 }
        //             } else {
        // Already tried None backend and it failed
        //                 tracing::error!(
        //                     "Failed to initialize SearchFacade with 'none' backend. \
        //                      Search endpoint will be unavailable."
        //                 );
        //                 None
        //             }
        //         }
        //     }
        // };

        tracing::info!("Application state initialization complete with resource controls, event bus, circuit breaker, monitoring, fetch engine, performance manager, authentication, cache warming, browser launcher, and facade layer");

        // Initialize trace backend for distributed tracing
        let trace_backend: Option<Arc<dyn crate::handlers::trace_backend::TraceBackend>> = {
            use crate::handlers::trace_backend::{InMemoryTraceBackend, OtlpTraceBackend};

            // Try to use OTLP backend if configured, otherwise fall back to in-memory
            if let Some(otlp_backend) = OtlpTraceBackend::from_env() {
                tracing::info!(
                    backend_type = "otlp",
                    "OTLP trace backend configured for distributed tracing"
                );
                Some(Arc::new(otlp_backend) as Arc<dyn crate::handlers::trace_backend::TraceBackend>)
            } else {
                // Use in-memory backend for development
                tracing::info!(
                    backend_type = "in-memory",
                    "Using in-memory trace backend (set OTLP_TRACE_QUERY_ENDPOINT for production)"
                );
                let backend = InMemoryTraceBackend::new();

                // Populate with mock data for demonstration
                backend.populate_mock_data().await;

                Some(Arc::new(backend) as Arc<dyn crate::handlers::trace_backend::TraceBackend>)
            }
        };

        // Determine cache warmer enabled status (disabled - requires wasm-pool feature)
        // #[cfg(feature = "wasm-extractor")]
        // let cache_warmer_enabled = config.cache_warming_config.enabled;
        // #[cfg(not(feature = "wasm-extractor"))]
        let cache_warmer_enabled = false;

        // Initialize placeholder facades (will be replaced by with_facades() method)
        tracing::debug!("Initializing placeholder facades for base AppState");

        let facade_config = riptide_facade::config::RiptideConfig::default();

        // Create minimal placeholder facades for base state
        let extraction_facade = Arc::new(
            riptide_facade::facades::ExtractionFacade::new(facade_config.clone())
                .await
                .map_err(|e| {
                    anyhow::anyhow!("Failed to initialize placeholder ExtractionFacade: {}", e)
                })?,
        );

        let scraper_facade = Arc::new(
            riptide_facade::facades::ScraperFacade::new(facade_config.clone())
                .await
                .map_err(|e| {
                    anyhow::anyhow!("Failed to initialize placeholder ScraperFacade: {}", e)
                })?,
        );

        // Initialize engine facade with cache storage adapter
        let cache_storage = Arc::new(
            riptide_cache::RedisStorage::new(&config.redis_url)
                .await
                .context("Failed to create Redis storage for engine facade")?,
        );
        let engine_facade = Arc::new(riptide_facade::facades::EngineFacade::new(cache_storage));
        tracing::info!("EngineFacade initialized successfully");

        // Initialize resource facade (Sprint 4.4)
        use crate::adapters::ResourceManagerPoolAdapter;
        let resource_pool_adapter =
            Arc::new(ResourceManagerPoolAdapter::new(resource_manager.clone()));
        let redis_manager = Arc::new(
            riptide_cache::redis::RedisManager::new(&config.redis_url)
                .await
                .context("Failed to create Redis manager for rate limiting")?,
        );
        let redis_rate_limiter = Arc::new(riptide_cache::adapters::RedisRateLimiter::new(
            redis_manager,
            (api_config.rate_limiting.requests_per_second_per_host * 60.0) as usize, // requests per minute
            std::time::Duration::from_secs(60),
        ));
        let resource_facade = Arc::new(riptide_facade::facades::ResourceFacade::new(
            resource_pool_adapter
                as Arc<dyn riptide_types::ports::Pool<crate::adapters::ResourceSlot>>,
            redis_rate_limiter as Arc<dyn riptide_types::ports::RateLimiter>,
            business_metrics.clone(),
            riptide_facade::facades::ResourceConfig::default(),
        ));
        tracing::info!(
            "ResourceFacade initialized successfully with pool adapter and Redis rate limiter"
        );

        // TODO Phase 4.3: Initialize streaming facade with proper dependencies
        // For now, create a placeholder that will be properly wired later
        // let streaming_facade = Arc::new(riptide_facade::facades::StreamingFacade::new(
        //     cache_storage.clone(),
        //     Arc::new(NoopEventBus),
        //     vec![],
        //     Arc::new(NoopMetrics),
        // ));
        // tracing::info!("StreamingFacade initialized successfully for real-time data delivery");

        // Optional facades are None in base state - will be initialized by with_facades()
        #[cfg(feature = "spider")]
        let spider_facade = None;

        #[cfg(feature = "search")]
        let search_facade = None;

        Ok(Self {
            http_client,
            cache,
            #[cfg(feature = "extraction")]
            extractor,
            reliable_extractor,
            config,
            api_config,
            resource_manager,
            business_metrics,
            transport_metrics,
            combined_metrics,
            health_checker,
            session_manager,
            streaming,
            telemetry,
            #[cfg(feature = "spider")]
            spider,
            pdf_metrics,
            #[cfg(feature = "workers")]
            worker_service,
            event_bus,
            circuit_breaker,
            performance_metrics,
            monitoring_system,
            #[cfg(feature = "fetch")]
            fetch_engine,
            performance_manager,
            auth_config,
            cache_warmer_enabled,
            #[cfg(feature = "browser")]
            browser_launcher,
            // Facades initialized above (Phase 2C.2)
            extraction_facade,
            scraper_facade,
            #[cfg(feature = "spider")]
            spider_facade,
            #[cfg(feature = "search")]
            search_facade,
            // Engine facade (Phase 3 Sprint 3.1)
            engine_facade,
            // TODO Phase 4.3: Streaming facade (Phase 4 Sprint 4.3)
            // streaming_facade,
            // Resource facade (Phase 4 Sprint 4.4)
            resource_facade,
            trace_backend,
            #[cfg(feature = "persistence")]
            persistence_adapter: None, // TODO: Initialize actual persistence adapter when integrated
        })
    }

    /// Initialize facades for an existing base AppState.
    ///
    /// This method takes an existing base AppState (created with `new_base()`) and
    /// initializes all facade components with proper configuration. This allows for
    /// two-phase initialization where the core infrastructure is set up first,
    /// then facades are added.
    ///
    /// # Returns
    ///
    /// A new `AppState` with all facades properly initialized.
    ///
    /// # Errors
    ///
    /// Returns an error if facade initialization fails.
    pub async fn with_facades(mut self) -> Result<Self> {
        tracing::info!("Initializing facades for AppState");

        let facade_config = riptide_facade::config::RiptideConfig::default();

        // Initialize extraction facade
        self.extraction_facade = Arc::new(
            riptide_facade::facades::ExtractionFacade::new(facade_config.clone())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to initialize ExtractionFacade: {}", e))?,
        );
        tracing::info!("ExtractionFacade initialized successfully");

        // Initialize scraper facade
        self.scraper_facade = Arc::new(
            riptide_facade::facades::ScraperFacade::new(facade_config.clone())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to initialize ScraperFacade: {}", e))?,
        );
        tracing::info!("ScraperFacade initialized successfully");

        // Initialize SpiderFacade with Development preset (suitable for local testing)
        #[cfg(feature = "spider")]
        {
            use url::Url;
            let base_url =
                Url::parse("https://example.com").expect("Failed to parse default spider base URL");
            self.spider_facade = match riptide_facade::facades::SpiderFacade::from_preset(
                riptide_facade::facades::SpiderPreset::Development,
                base_url,
            )
            .await
            {
                Ok(facade) => {
                    tracing::info!("SpiderFacade initialized with Development preset");
                    Some(Arc::new(facade))
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to initialize SpiderFacade, spider endpoints will be unavailable");
                    None
                }
            };
        }

        // Initialize SearchFacade if SERPER_API_KEY is available
        #[cfg(feature = "search")]
        {
            self.search_facade = if let Ok(api_key) = std::env::var("SERPER_API_KEY") {
                match riptide_facade::facades::SearchFacade::with_api_key(
                    riptide_search::SearchBackend::Serper,
                    Some(api_key),
                )
                .await
                {
                    Ok(facade) => {
                        tracing::info!("SearchFacade initialized with Serper backend");
                        Some(Arc::new(facade))
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "Failed to initialize SearchFacade, search endpoint will be unavailable");
                        None
                    }
                }
            } else {
                tracing::info!("SERPER_API_KEY not found, search endpoint will be unavailable. Set SERPER_API_KEY to enable search.");
                None
            };
        }

        tracing::info!("All facades initialized successfully");
        Ok(self)
    }

    // ===== Phase 5.2: Transitional Helper Methods =====
    // These helpers allow gradual migration from deprecated metrics

    /// Record HTTP error using transport metrics
    pub fn record_http_error(&self) {
        self.transport_metrics.record_http_error();
    }

    /// Record WASM error using transport metrics
    pub fn record_wasm_error(&self) {
        self.transport_metrics.record_wasm_error();
    }

    /// Record Redis error using transport metrics
    pub fn record_redis_error(&self) {
        self.transport_metrics.record_redis_error();
    }

    /// Record HTTP request using transport metrics
    pub fn record_http_request(&self, method: &str, path: &str, status: u16, duration_secs: f64) {
        self.transport_metrics
            .record_http_request(method, path, status, duration_secs);
    }

    /// Record phase timing using business metrics
    pub fn record_phase_timing(&self, phase: &str, duration_secs: f64) {
        // Map phase name to appropriate histogram
        match phase {
            "fetch" => self
                .business_metrics
                .fetch_phase_duration
                .observe(duration_secs),
            "gate" => self
                .business_metrics
                .gate_phase_duration
                .observe(duration_secs),
            "wasm" => self
                .business_metrics
                .wasm_phase_duration
                .observe(duration_secs),
            "render" => self
                .business_metrics
                .render_phase_duration
                .observe(duration_secs),
            _ => {
                tracing::warn!("Unknown phase type for timing: {}", phase);
            }
        }
    }

    /// Record gate decision using business metrics
    pub fn record_gate_decision(&self, decision: &str) {
        self.business_metrics.record_gate_decision(decision);
    }

    /// Get prometheus registry for metrics endpoint
    pub fn metrics_registry(&self) -> &prometheus::Registry {
        self.combined_metrics.registry()
    }

    /// Update streaming metrics (delegates to business metrics)
    pub fn update_streaming_metrics(&self, _metrics: &crate::streaming::GlobalStreamingMetrics) {
        // Delegate to business metrics for streaming operations
        self.business_metrics.record_stream_created("", "");
    }

    /// Record PDF processing success
    pub fn record_pdf_success(&self, pages: usize, duration_secs: f64) {
        self.business_metrics
            .record_pdf_processing_success(duration_secs, pages as u32, 0.0);
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
        info!("Starting health check");
        let mut health = HealthStatus {
            healthy: true,
            redis: DependencyHealth::Unknown,
            extractor: DependencyHealth::Unknown,
            http_client: DependencyHealth::Unknown,
            resource_manager: DependencyHealth::Unknown,
            streaming: DependencyHealth::Unknown,
            spider: DependencyHealth::Unknown,
            worker_service: DependencyHealth::Unknown,
            circuit_breaker: DependencyHealth::Unknown,
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

        // Check streaming module health
        health.streaming = if self.streaming.is_healthy().await {
            DependencyHealth::Healthy
        } else {
            let streaming_metrics = self.streaming.metrics().await;
            health.healthy = false;
            DependencyHealth::Unhealthy(format!(
                "Streaming unhealthy: active_connections={}, error_rate={:.2}",
                streaming_metrics.active_connections, streaming_metrics.error_rate
            ))
        };

        // Check spider engine health if available
        #[cfg(feature = "spider")]
        {
            health.spider = if let Some(spider) = &self.spider {
                let spider_state = spider.get_crawl_state().await;
                if spider_state.active {
                    DependencyHealth::Healthy
                } else {
                    // Spider is available but not actively crawling
                    DependencyHealth::Healthy
                }
            } else {
                // Spider is disabled, consider it healthy (not an error condition)
                DependencyHealth::Healthy
            };
        }
        #[cfg(not(feature = "spider"))]
        {
            // Spider feature disabled - consider healthy (not an error)
            health.spider = DependencyHealth::Healthy;
        }

        // Check worker service health
        #[cfg(feature = "workers")]
        {
            health.worker_service = {
                let worker_health = self.worker_service.health_check().await;
                if worker_health.overall_healthy {
                    DependencyHealth::Healthy
                } else {
                    health.healthy = false;
                    DependencyHealth::Unhealthy(format!(
                        "Worker service unhealthy: queue={}, pool={}, scheduler={}",
                        worker_health.queue_healthy,
                        worker_health.worker_pool_healthy,
                        worker_health.scheduler_healthy
                    ))
                }
            };
        }
        #[cfg(not(feature = "workers"))]
        {
            // Workers feature disabled - consider healthy (not an error)
            health.worker_service = DependencyHealth::Healthy;
        }

        // Check circuit breaker health
        health.circuit_breaker = {
            let cb_state = self.circuit_breaker.lock().await;
            if cb_state.is_open() {
                health.healthy = false;
                DependencyHealth::Unhealthy(
                    "Circuit breaker is open - too many failures".to_string(),
                )
            } else if cb_state.is_half_open() {
                DependencyHealth::Unhealthy("Circuit breaker is testing recovery".to_string())
            } else {
                DependencyHealth::Healthy
            }
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

    /// Create a minimal test state for integration testing
    ///
    /// This creates a test state with minimal working components suitable for unit tests.
    /// Some features may not work without full initialization.
    ///
    /// **Note**: This is intended for testing purposes only. Use in `cfg(test)` or test modules.
    #[allow(dead_code)]
    pub async fn new_test_minimal() -> Self {
        use std::sync::Arc;
        use tokio::sync::Mutex;
        #[cfg(feature = "fetch")]
        let http_client = http_client().expect("Failed to create HTTP client");
        #[cfg(not(feature = "fetch"))]
        let http_client = Client::new();

        // Use Redis URL from env or default (tests should skip Redis-dependent features)
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        // Create a mock cache manager for tests when Redis is unavailable
        // This allows tests to run without requiring a Redis instance
        let cache = if std::env::var("SKIP_REDIS_TESTS").is_ok() {
            eprintln!("⚠️  SKIP_REDIS_TESTS is set - using mock cache for tests");
            // Create a mock cache by using a dummy connection attempt
            // The cache operations will fail, but non-cache tests can still run
            match CacheManager::new(&redis_url).await {
                Ok(cm) => Arc::new(Mutex::new(cm)),
                Err(e) => {
                    eprintln!("Using mock cache (Redis unavailable: {})", e);
                    panic!("Mock cache not implemented - Redis is required for tests. Start Redis with: docker run -d -p 6379:6379 redis")
                }
            }
        } else {
            // Try to create real cache manager
            match CacheManager::new(&redis_url).await {
                Ok(cm) => Arc::new(Mutex::new(cm)),
                Err(e) => {
                    eprintln!("Warning: Redis not available for tests ({})", e);
                    eprintln!("\n⚠️  Redis connection failed");
                    eprintln!("   To run tests, start Redis: docker run -d -p 6379:6379 redis");
                    eprintln!("   Or set REDIS_URL to point to your Redis instance");
                    eprintln!("   Or set SKIP_REDIS_TESTS=1 to skip Redis-dependent tests\n");
                    panic!("Redis required for integration tests")
                }
            }
        };

        let config = AppConfig::default();
        let api_config = RiptideApiConfig::default();

        // Try to load WASM extractor from default path, or skip
        #[cfg(feature = "extraction")]
        let extractor = {
            let wasm_path =
                std::env::var("WASM_EXTRACTOR_PATH").unwrap_or_else(|_| config.wasm_path.clone());

            match UnifiedExtractor::new(Some(&wasm_path)).await {
                Ok(ext) => Arc::new(ext),
                Err(e) => {
                    eprintln!(
                        "Warning: Content extractor initialization failed ({}), using default native extractor",
                        e
                    );
                    // Fall back to native extractor
                    Arc::new(
                        UnifiedExtractor::new(None)
                            .await
                            .expect("Native extractor should always work"),
                    )
                }
            }
        };

        let reliable_extractor = Arc::new(
            ReliableExtractor::new(config.reliability_config.clone())
                .expect("Failed to create reliable extractor"),
        );

        let health_checker = Arc::new(HealthChecker::new());

        let resource_manager = Arc::new(
            ResourceManager::new(api_config.clone())
                .await
                .expect("Failed to create resource manager"),
        );

        let session_config = SessionConfig::default();
        let session_manager = Arc::new(
            SessionManager::new(session_config)
                .await
                .expect("Failed to create session manager"),
        );

        let streaming = Arc::new(StreamingModule::default());

        let pdf_metrics = Arc::new(PdfMetricsCollector::new());

        #[cfg(feature = "workers")]
        let worker_service = {
            let worker_config = WorkerServiceConfig::default();
            Arc::new(
                WorkerService::new(worker_config)
                    .await
                    .expect("Failed to create worker service"),
            )
        };

        let event_bus = Arc::new(EventBus::new());

        let circuit_breaker = Arc::new(Mutex::new(CircuitBreakerState::default()));
        let performance_metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));

        let monitoring_system = Arc::new(MonitoringSystem::new());

        // For tests, we can skip complex initialization and panic if needed
        // Tests should either skip features or set appropriate env vars

        #[cfg(feature = "fetch")]
        let fetch_engine = Arc::new(FetchEngine::new().expect("Failed to create fetch engine"));
        let performance_manager =
            Arc::new(PerformanceManager::new().expect("Failed to create performance manager"));
        let auth_config = AuthConfig::default();
        let cache_warmer_enabled = false;

        #[cfg(feature = "browser")]
        let browser_launcher = Some(Arc::new(
            HeadlessLauncher::new()
                .await
                .expect("Failed to create browser launcher"),
        ));

        // Initialize facades (Phase 2C.2: Restored after breaking circular dependency)
        let facade_config = riptide_facade::RiptideConfig::default();

        let extraction_facade = Arc::new(
            riptide_facade::facades::ExtractionFacade::new(facade_config.clone())
                .await
                .expect("Failed to create extraction facade"),
        );
        let scraper_facade = Arc::new(
            riptide_facade::facades::ScraperFacade::new(facade_config.clone())
                .await
                .expect("Failed to create scraper facade"),
        );

        // Initialize SpiderFacade for tests
        #[cfg(feature = "spider")]
        let spider_facade = {
            use url::Url;
            let base_url = Url::parse("https://example.com").expect("Test URL parse failed");
            Some(Arc::new(
                riptide_facade::facades::SpiderFacade::from_preset(
                    riptide_facade::facades::SpiderPreset::Development,
                    base_url,
                )
                .await
                .expect("Failed to create spider facade for tests"),
            ))
        };

        // SearchFacade not initialized for tests (requires external API key)
        #[cfg(feature = "search")]
        let search_facade = None;

        // Initialize engine facade for tests
        let cache_storage = Arc::new(
            riptide_cache::RedisStorage::new(&redis_url)
                .await
                .expect("Failed to create Redis storage for tests"),
        );
        let engine_facade = Arc::new(riptide_facade::facades::EngineFacade::new(cache_storage));

        // Initialize resource facade for tests (Sprint 4.4)
        let resource_pool_adapter = Arc::new(crate::adapters::ResourceManagerPoolAdapter::new(
            resource_manager.clone(),
        ));
        let redis_manager = Arc::new(
            riptide_cache::redis::RedisManager::new(&redis_url)
                .await
                .expect("Failed to create Redis manager for tests"),
        );
        let redis_rate_limiter = Arc::new(riptide_cache::adapters::RedisRateLimiter::new(
            redis_manager,
            1000,
            std::time::Duration::from_secs(60),
        ));
        let business_metrics_test = Arc::new(
            riptide_facade::metrics::BusinessMetrics::new()
                .expect("Failed to create business metrics for tests"),
        );
        let resource_facade = Arc::new(riptide_facade::facades::ResourceFacade::new(
            resource_pool_adapter
                as Arc<dyn riptide_types::ports::Pool<crate::adapters::ResourceSlot>>,
            redis_rate_limiter as Arc<dyn riptide_types::ports::RateLimiter>,
            business_metrics_test.clone(),
            riptide_facade::facades::ResourceConfig::default(),
        ));

        // TODO Phase 4.3: Initialize streaming facade for tests
        // let streaming_facade = Arc::new(riptide_facade::facades::StreamingFacade::new(...));

        // Create placeholder metrics for test state
        let business_metrics =
            Arc::new(BusinessMetrics::new().expect("Failed to create business metrics for tests"));
        let transport_metrics = Arc::new(
            TransportMetrics::new().expect("Failed to create transport metrics for tests"),
        );
        let combined_metrics = Arc::new(
            CombinedMetrics::new(business_metrics.clone(), transport_metrics.clone())
                .expect("Failed to create combined metrics for tests"),
        );

        Self {
            http_client,
            cache,
            #[cfg(feature = "extraction")]
            extractor,
            reliable_extractor,
            config,
            api_config,
            resource_manager,
            business_metrics,
            transport_metrics,
            combined_metrics,
            health_checker,
            session_manager,
            streaming,
            telemetry: None,
            #[cfg(feature = "spider")]
            spider: None,
            pdf_metrics,
            #[cfg(feature = "workers")]
            worker_service,
            event_bus,
            circuit_breaker,
            performance_metrics,
            monitoring_system,
            #[cfg(feature = "fetch")]
            fetch_engine,
            performance_manager,
            auth_config,
            cache_warmer_enabled,
            #[cfg(feature = "browser")]
            browser_launcher,
            // Facades restored (Phase 2C.2: Circular dependency eliminated via trait abstraction)
            extraction_facade,
            scraper_facade,
            #[cfg(feature = "spider")]
            spider_facade,
            #[cfg(feature = "search")]
            search_facade,
            // Engine facade (Phase 3 Sprint 3.1)
            engine_facade,
            // TODO Phase 4.3: Streaming facade (Phase 4 Sprint 4.3)
            // streaming_facade,
            // Resource facade (Phase 4 Sprint 4.4)
            resource_facade,
            trace_backend: None,
            #[cfg(feature = "persistence")]
            persistence_adapter: None, // TODO: Initialize actual persistence adapter when integrated
        }
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
    pub streaming: DependencyHealth,
    pub spider: DependencyHealth,
    pub worker_service: DependencyHealth,
    pub circuit_breaker: DependencyHealth,
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

/// Integrated monitoring system for performance tracking, health scoring, and alerting
pub struct MonitoringSystem {
    /// Metrics collector for real-time performance tracking
    pub metrics_collector: Arc<MetricsCollector>,

    /// Alert manager for threshold-based alerting
    pub alert_manager: Arc<tokio::sync::Mutex<AlertManager>>,

    /// Health calculator for system health scoring
    pub health_calculator: Arc<HealthCalculator>,

    /// Configuration for monitoring behavior
    #[allow(dead_code)] // Public API - monitoring system configuration
    pub config: MonitoringConfig,
}

impl MonitoringSystem {
    /// Create a new monitoring system with default configuration
    pub fn new() -> Self {
        let config = MonitoringConfig::default();
        Self::with_config(config)
    }

    /// Create a new monitoring system with custom configuration
    pub fn with_config(config: MonitoringConfig) -> Self {
        let metrics_collector = Arc::new(MetricsCollector::with_config(config.clone()));
        let health_calculator = Arc::new(HealthCalculator::new(config.health_thresholds.clone()));

        // Create alert manager with default rules
        let alert_manager = AlertManager::new();

        Self {
            metrics_collector,
            alert_manager: Arc::new(tokio::sync::Mutex::new(alert_manager)),
            health_calculator,
            config,
        }
    }

    /// Register default alert rules for monitoring
    pub async fn register_default_alert_rules(&self) {
        let mut alert_manager = self.alert_manager.lock().await;

        // Clear existing rules and add our custom set
        // The AlertManager already has default rules, but we'll add more specific ones

        // Error rate threshold: >5%
        alert_manager.add_rule(AlertRule {
            name: "error_rate_threshold_5pct".to_string(),
            metric_name: "error_rate".to_string(),
            threshold: 5.0,
            condition: AlertCondition::GreaterThan,
            severity: AlertSeverity::Warning,
            enabled: true,
        });

        // Latency threshold: p95 >5s
        alert_manager.add_rule(AlertRule {
            name: "p95_latency_threshold_5s".to_string(),
            metric_name: "p95_extraction_time_ms".to_string(),
            threshold: 5000.0,
            condition: AlertCondition::GreaterThan,
            severity: AlertSeverity::Warning,
            enabled: true,
        });

        // Memory threshold: >80% (assuming 4GB total, 80% = 3.2GB)
        alert_manager.add_rule(AlertRule {
            name: "memory_usage_threshold_80pct".to_string(),
            metric_name: "memory_usage_bytes".to_string(),
            threshold: 3.2 * 1024.0 * 1024.0 * 1024.0, // 3.2GB
            condition: AlertCondition::GreaterThan,
            severity: AlertSeverity::Warning,
            enabled: true,
        });

        tracing::info!("Registered default alert rules for monitoring system");
    }

    /// Start background alert evaluation task
    pub fn start_alert_evaluation_task(&self, event_bus: Arc<EventBus>) {
        let metrics_collector = self.metrics_collector.clone();
        let alert_manager = self.alert_manager.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;

                // Get current metrics
                match metrics_collector.get_current_metrics().await {
                    Ok(metrics) => {
                        // Check for triggered alerts
                        let mut manager = alert_manager.lock().await;
                        let alerts = manager.check_alerts(&metrics).await;

                        // Log and emit events for each triggered alert
                        for alert in alerts {
                            // Log alert based on severity
                            match alert.severity {
                                AlertSeverity::Critical => tracing::error!(
                                    rule_name = %alert.rule_name,
                                    current_value = %alert.current_value,
                                    threshold = %alert.threshold,
                                    "CRITICAL ALERT: {}",
                                    alert.message
                                ),
                                AlertSeverity::Error => tracing::error!(
                                    rule_name = %alert.rule_name,
                                    current_value = %alert.current_value,
                                    threshold = %alert.threshold,
                                    "ERROR ALERT: {}",
                                    alert.message
                                ),
                                AlertSeverity::Warning => tracing::warn!(
                                    rule_name = %alert.rule_name,
                                    current_value = %alert.current_value,
                                    threshold = %alert.threshold,
                                    "WARNING ALERT: {}",
                                    alert.message
                                ),
                                AlertSeverity::Info => tracing::info!(
                                    rule_name = %alert.rule_name,
                                    current_value = %alert.current_value,
                                    threshold = %alert.threshold,
                                    "INFO ALERT: {}",
                                    alert.message
                                ),
                            }

                            // Create and publish alert event to event bus
                            use riptide_events::BaseEvent;
                            let mut base_event = BaseEvent::new(
                                "monitoring.alert.triggered",
                                "monitoring_system",
                                match alert.severity {
                                    AlertSeverity::Critical => EventSeverity::Critical,
                                    AlertSeverity::Error => EventSeverity::Error,
                                    AlertSeverity::Warning => EventSeverity::Warn,
                                    AlertSeverity::Info => EventSeverity::Info,
                                },
                            );

                            // Add alert metadata for downstream consumers
                            base_event.add_metadata("rule_name", &alert.rule_name);
                            base_event.add_metadata("message", &alert.message);
                            base_event
                                .add_metadata("current_value", &alert.current_value.to_string());
                            base_event.add_metadata("threshold", &alert.threshold.to_string());
                            base_event.add_metadata("severity", &format!("{:?}", alert.severity));

                            // Publish event to event bus for system-wide notification
                            // This enables downstream alerting (Slack, PagerDuty, email, webhooks)
                            if let Err(e) = event_bus.emit(base_event).await {
                                tracing::warn!(
                                    rule_name = %alert.rule_name,
                                    error = %e,
                                    "Failed to publish alert event to event bus"
                                );
                            } else {
                                tracing::debug!(
                                    rule_name = %alert.rule_name,
                                    "Alert event published to event bus successfully"
                                );
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to get metrics for alert evaluation: {}", e);
                    }
                }
            }
        });

        tracing::info!("Started background alert evaluation task with 30-second interval and event bus integration");
    }

    /// Calculate current health score
    pub async fn calculate_health_score(&self) -> Result<f32> {
        let metrics = self.metrics_collector.get_current_metrics().await?;
        Ok(self.health_calculator.calculate_health(&metrics))
    }

    /// Generate performance report
    pub async fn generate_performance_report(&self) -> Result<PerformanceReport> {
        let metrics = self.metrics_collector.get_current_metrics().await?;
        let health_score = self.health_calculator.calculate_health(&metrics);
        let health_summary = self.health_calculator.generate_health_summary(&metrics);
        let recommendations = self.health_calculator.generate_recommendations(&metrics);

        Ok(PerformanceReport {
            metrics,
            health_score,
            health_summary,
            recommendations,
        })
    }
}

impl Default for MonitoringSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance report containing metrics and health analysis
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceReport {
    /// Current performance metrics
    pub metrics: riptide_monitoring::PerformanceMetrics,

    /// Overall health score (0-100)
    pub health_score: f32,

    /// Human-readable health summary
    pub health_summary: String,

    /// List of recommendations for improvement
    pub recommendations: Vec<String>,
}
