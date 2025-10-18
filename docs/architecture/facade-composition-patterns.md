# Advanced Facade Composition Patterns for Riptide-API

**Document Version:** 1.0
**Date:** 2025-10-18
**Status:** Design & Implementation Guide
**Phase:** P1-A4 Phase 2 - Advanced Composition

---

## Executive Summary

This document describes advanced composition patterns for the riptide-facade layer, enabling complex multi-step workflows that coordinate Browser, Extraction, Scraper, and Pipeline facades. These patterns support the full P1 completion requirements and provide the foundation for production-ready web scraping workflows.

### Key Achievements
- **Phase 1 Complete**: ScraperFacade with 24 passing tests
- **Foundation Ready**: Builder pattern, error handling, configuration
- **Phase 2 Target**: Browser + Extraction + Pipeline composition patterns

---

## Table of Contents

1. [Multi-Step Workflow Requirements](#1-multi-step-workflow-requirements)
2. [Composition Architecture](#2-composition-architecture)
3. [Pipeline Templates](#3-pipeline-templates)
4. [Error Recovery Strategies](#4-error-recovery-strategies)
5. [Caching Integration](#5-caching-integration)
6. [Resource Management](#6-resource-management)
7. [Metrics & Observability](#7-metrics--observability)
8. [Example Handlers](#8-example-handlers)
9. [Missing Capabilities Analysis](#9-missing-capabilities-analysis)
10. [Implementation Roadmap](#10-implementation-roadmap)

---

## 1. Multi-Step Workflow Requirements

### 1.1 Common Workflow Patterns

#### Pattern A: Browser → Extract → Transform
**Use Case**: Dynamic content extraction from JavaScript-heavy sites

```rust
// Workflow Steps:
// 1. Launch browser with stealth features
// 2. Navigate to URL and wait for dynamic content
// 3. Extract rendered HTML
// 4. Parse with extraction strategies
// 5. Transform to desired format
// 6. Cache results
```

**Requirements**:
- Browser session management with automatic cleanup
- Wait for dynamic content rendering
- Multiple extraction strategy fallback
- Result transformation and validation
- Caching with TTL

**Error Scenarios**:
- Browser launch failure → Retry with backoff
- Navigation timeout → Fallback to HTTP fetch
- Extraction failure → Try alternative strategies
- Transform validation failure → Return partial results

---

#### Pattern B: Scrape → Extract → Cache
**Use Case**: High-throughput content scraping with caching

```rust
// Workflow Steps:
// 1. HTTP fetch with retry logic
// 2. Content-type detection
// 3. Multi-strategy extraction (CSS → WASM → Fallback)
// 4. Quality scoring and validation
// 5. Cache with intelligent TTL
// 6. Return with metadata
```

**Requirements**:
- Efficient HTTP client with connection pooling
- Automatic content-type detection (HTML/PDF/JSON)
- Strategy selection based on content characteristics
- Quality thresholds and validation
- Multi-level caching (memory + Redis)

**Error Scenarios**:
- Network failure → Exponential backoff retry
- Invalid content → Return error with diagnostics
- Low quality extraction → Fallback to alternative strategy
- Cache write failure → Log warning, continue

---

#### Pattern C: Browser → Screenshot → Extract → Store
**Use Case**: Visual verification with content extraction

```rust
// Workflow Steps:
// 1. Launch browser with viewport configuration
// 2. Navigate and wait for page load
// 3. Capture full-page screenshot
// 4. Extract visible text content
// 5. Store screenshot and metadata
// 6. Return combined results
```

**Requirements**:
- Browser viewport configuration
- Screenshot capture with format options (PNG/JPEG)
- OCR integration for image text extraction (optional)
- Storage abstraction (local/S3/database)
- Atomic operation guarantee (all-or-nothing)

**Error Scenarios**:
- Screenshot failure → Continue with HTML extraction
- Storage failure → Retry with different storage backend
- Partial completion → Cleanup and rollback

---

#### Pattern D: Batch-Scrape → Pipeline → Aggregate
**Use Case**: Bulk URL processing with aggregation

```rust
// Workflow Steps:
// 1. Accept list of URLs
// 2. Parallel fetch with concurrency control
// 3. Per-URL extraction pipeline
// 4. Aggregate results
// 5. Generate summary statistics
// 6. Store batch results
```

**Requirements**:
- Parallel execution with configurable concurrency
- Per-URL timeout and error isolation
- Progress tracking and partial results
- Result aggregation and deduplication
- Batch metrics and reporting

**Error Scenarios**:
- Individual URL failure → Continue with others
- Concurrency exhaustion → Queue and rate limit
- Aggregation failure → Return partial results with errors

---

### 1.2 Workflow Complexity Matrix

| Workflow | Facades | Async Steps | Error Points | Cache Layers | Priority |
|----------|---------|-------------|--------------|--------------|----------|
| Browser → Extract | Browser, Extractor | 5 | 4 | 1 | HIGH |
| Scrape → Cache | Scraper, Cache | 3 | 2 | 2 | HIGH |
| Browser → Screenshot → Store | Browser, Storage | 6 | 5 | 1 | MEDIUM |
| Batch Pipeline | Pipeline, Scraper, Cache | 8+ | 10+ | 3 | HIGH |
| Crawl → Extract → Transform | Spider, Extractor, Pipeline | 10+ | 12+ | 2 | LOW |

---

## 2. Composition Architecture

### 2.1 Facade Coordination Model

```rust
/// Composition coordinator for multi-facade workflows
pub struct WorkflowComposer {
    /// Shared runtime for all facades
    runtime: Arc<RiptideRuntime>,

    /// Facade instances
    browser: Arc<BrowserFacade>,
    scraper: Arc<ScraperFacade>,
    extractor: Arc<ExtractionFacade>,
    pipeline: Arc<PipelineFacade>,
    cache: Arc<CacheFacade>,

    /// Shared configuration
    config: Arc<RiptideConfig>,

    /// Metrics collector
    metrics: Arc<MetricsCollector>,
}

impl WorkflowComposer {
    /// Create a new workflow composer with shared resources
    pub async fn new(config: RiptideConfig) -> Result<Self> {
        let runtime = Arc::new(RiptideRuntime::new());

        // Initialize facades with shared runtime
        let browser = Arc::new(BrowserFacade::new(config.clone()).await?);
        let scraper = Arc::new(ScraperFacade::new(config.clone()).await?);
        let extractor = Arc::new(ExtractionFacade::new(config.clone()).await?);
        let pipeline = Arc::new(PipelineFacade::new(config.clone()).await?);
        let cache = Arc::new(CacheFacade::new(config.clone()).await?);

        Ok(Self {
            runtime,
            browser,
            scraper,
            extractor,
            pipeline,
            cache,
            config: Arc::new(config),
            metrics: Arc::new(MetricsCollector::new()),
        })
    }

    /// Execute a composed workflow
    pub async fn execute_workflow(
        &self,
        workflow: WorkflowDefinition,
    ) -> Result<WorkflowResult> {
        // Workflow execution with coordination
        todo!()
    }
}
```

---

### 2.2 Shared Resource Management

#### Resource Pooling Strategy

```rust
/// Shared resource pool for all facades
pub struct ResourcePool {
    /// Browser instance pool (max 20)
    browser_pool: Arc<BrowserPool>,

    /// HTTP connection pool (max 100)
    http_pool: Arc<HttpConnectionPool>,

    /// WASM instance pool (max 10)
    wasm_pool: Arc<WasmInstancePool>,

    /// Cache connection pool (max 5)
    cache_pool: Arc<CacheConnectionPool>,
}

impl ResourcePool {
    /// Acquire resources for a workflow
    pub async fn acquire(&self, needs: ResourceNeeds) -> Result<AcquiredResources> {
        let mut acquired = AcquiredResources::new();

        // Acquire in order to prevent deadlock
        if needs.browser {
            acquired.browser = Some(self.browser_pool.acquire().await?);
        }
        if needs.http {
            acquired.http = Some(self.http_pool.acquire().await?);
        }
        if needs.wasm {
            acquired.wasm = Some(self.wasm_pool.acquire().await?);
        }

        Ok(acquired)
    }

    /// Release resources back to pools
    pub async fn release(&self, resources: AcquiredResources) {
        // Resources automatically released via Drop
    }
}

/// Scope-based resource acquisition with automatic cleanup
pub struct WorkflowScope<'a> {
    pool: &'a ResourcePool,
    resources: Option<AcquiredResources>,
}

impl<'a> WorkflowScope<'a> {
    pub async fn new(pool: &'a ResourcePool, needs: ResourceNeeds) -> Result<Self> {
        let resources = pool.acquire(needs).await?;
        Ok(Self {
            pool,
            resources: Some(resources),
        })
    }

    pub fn resources(&self) -> &AcquiredResources {
        self.resources.as_ref().unwrap()
    }
}

impl<'a> Drop for WorkflowScope<'a> {
    fn drop(&mut self) {
        if let Some(resources) = self.resources.take() {
            // Async drop pattern (resources implement Drop)
            drop(resources);
        }
    }
}
```

---

### 2.3 Error Propagation Patterns

#### Error Context Chain

```rust
/// Rich error context for composed workflows
#[derive(Debug)]
pub struct WorkflowError {
    /// Error kind
    kind: ErrorKind,

    /// Stage where error occurred
    stage: WorkflowStage,

    /// Original error
    source: Option<Box<dyn std::error::Error + Send + Sync>>,

    /// Error context chain
    context: Vec<ErrorContext>,

    /// Retry information
    retry_info: Option<RetryInfo>,
}

impl WorkflowError {
    /// Create error with context
    pub fn new(kind: ErrorKind, stage: WorkflowStage) -> Self {
        Self {
            kind,
            stage,
            source: None,
            context: Vec::new(),
            retry_info: None,
        }
    }

    /// Add context to error
    pub fn with_context(mut self, ctx: impl Into<ErrorContext>) -> Self {
        self.context.push(ctx.into());
        self
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind,
            ErrorKind::Timeout | ErrorKind::NetworkError | ErrorKind::ResourceExhausted
        )
    }
}

/// Workflow stages for error tracking
#[derive(Debug, Clone, Copy)]
pub enum WorkflowStage {
    Initialization,
    ResourceAcquisition,
    BrowserLaunch,
    Navigation,
    Extraction,
    Transformation,
    Validation,
    Caching,
    Storage,
    Cleanup,
}

/// Error context for debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub message: String,
    pub timestamp: Instant,
    pub metadata: HashMap<String, String>,
}
```

---

## 3. Pipeline Templates

### 3.1 Pre-Built Pipeline Templates

#### Template 1: Crawl-and-Extract

```rust
impl PipelineFacade {
    /// Browser-based crawl and extract pipeline
    pub async fn crawl_and_extract_pipeline(
        &self,
        url: &str,
        options: CrawlExtractOptions,
    ) -> Result<Pipeline> {
        self.builder()
            // Stage 1: Launch browser
            .add_stage(PipelineStage::BrowserLaunch {
                config: BrowserConfig {
                    headless: true,
                    stealth: options.enable_stealth,
                    viewport: options.viewport,
                },
            })
            // Stage 2: Navigate with wait
            .add_stage(PipelineStage::Navigate {
                url: url.to_string(),
                wait_for: options.wait_for.clone(),
                timeout: Duration::from_secs(30),
            })
            // Stage 3: Execute actions (optional)
            .add_stage_if(options.actions.is_some(), || {
                PipelineStage::BrowserActions {
                    actions: options.actions.clone().unwrap(),
                }
            })
            // Stage 4: Extract content
            .add_stage(PipelineStage::Extract {
                strategy: ExtractionStrategy::Auto,
                options: ExtractionOptions {
                    include_metadata: true,
                    extract_links: options.extract_links,
                    extract_images: options.extract_images,
                },
            })
            // Stage 5: Validate quality
            .add_stage(PipelineStage::Validate {
                validator: Arc::new(QualityValidator {
                    min_length: options.min_content_length,
                    min_confidence: options.min_confidence,
                }),
            })
            // Stage 6: Transform (optional)
            .add_stage_if(options.transform.is_some(), || {
                PipelineStage::Transform {
                    transformer: options.transform.clone().unwrap(),
                }
            })
            // Stage 7: Cache result
            .add_stage(PipelineStage::Cache {
                ttl: options.cache_ttl,
                key_generator: Arc::new(UrlKeyGenerator),
            })
            // Configuration
            .with_retry(3)
            .with_timeout(Duration::from_secs(60))
            .with_error_recovery(ErrorRecoveryStrategy::Fallback)
            .build()
            .await
    }
}

/// Options for crawl-and-extract pipeline
pub struct CrawlExtractOptions {
    pub enable_stealth: bool,
    pub viewport: Option<Viewport>,
    pub wait_for: WaitCondition,
    pub actions: Option<Vec<BrowserAction>>,
    pub extract_links: bool,
    pub extract_images: bool,
    pub min_content_length: usize,
    pub min_confidence: f64,
    pub transform: Option<Arc<dyn Transformer>>,
    pub cache_ttl: Duration,
}
```

---

#### Template 2: Batch-Scrape

```rust
impl PipelineFacade {
    /// Parallel batch scraping pipeline
    pub async fn batch_scrape_pipeline(
        &self,
        urls: Vec<String>,
        options: BatchScrapeOptions,
    ) -> Result<Pipeline> {
        self.builder()
            // Stage 1: Parallel fetch
            .add_stage(PipelineStage::ParallelFetch {
                urls: urls.clone(),
                concurrency: options.concurrency,
                timeout_per_url: Duration::from_secs(30),
                retry_failed: true,
            })
            // Stage 2: Content-type routing
            .add_stage(PipelineStage::Route {
                router: Arc::new(ContentTypeRouter::new()),
            })
            // Stage 3: Parallel extraction
            .add_stage(PipelineStage::ParallelExtract {
                strategy: ExtractionStrategy::Multi,
                concurrency: options.concurrency,
            })
            // Stage 4: Aggregate results
            .add_stage(PipelineStage::Aggregate {
                aggregator: Arc::new(ResultAggregator {
                    deduplicate: options.deduplicate,
                    merge_similar: options.merge_similar_threshold,
                }),
            })
            // Stage 5: Batch cache
            .add_stage(PipelineStage::BatchCache {
                ttl: options.cache_ttl,
                batch_size: 100,
            })
            // Configuration
            .with_parallelism(options.concurrency)
            .with_retry(2)
            .with_timeout(Duration::from_secs(120))
            .with_progress_tracking(true)
            .build()
            .await
    }
}

pub struct BatchScrapeOptions {
    pub concurrency: usize,
    pub deduplicate: bool,
    pub merge_similar_threshold: Option<f64>,
    pub cache_ttl: Duration,
}
```

---

#### Template 3: Render-and-Extract

```rust
impl PipelineFacade {
    /// Full-page render with screenshot and extraction
    pub async fn render_and_extract_pipeline(
        &self,
        url: &str,
        options: RenderExtractOptions,
    ) -> Result<Pipeline> {
        self.builder()
            // Stage 1: Launch browser
            .add_stage(PipelineStage::BrowserLaunch {
                config: BrowserConfig {
                    headless: true,
                    stealth: true,
                    viewport: options.viewport.clone(),
                },
            })
            // Stage 2: Navigate
            .add_stage(PipelineStage::Navigate {
                url: url.to_string(),
                wait_for: WaitCondition::NetworkIdle,
                timeout: Duration::from_secs(30),
            })
            // Stage 3: Screenshot
            .add_stage(PipelineStage::Screenshot {
                options: ScreenshotOptions {
                    full_page: options.full_page_screenshot,
                    format: options.screenshot_format,
                    quality: Some(90),
                    ..Default::default()
                },
            })
            // Stage 4: Extract content
            .add_stage(PipelineStage::Extract {
                strategy: ExtractionStrategy::Wasm,
                options: ExtractionOptions {
                    include_metadata: true,
                    extract_links: true,
                    extract_images: true,
                },
            })
            // Stage 5: OCR (optional for screenshot text)
            .add_stage_if(options.enable_ocr, || {
                PipelineStage::Ocr {
                    language: options.ocr_language.clone(),
                }
            })
            // Stage 6: Store screenshot
            .add_stage(PipelineStage::Store {
                destination: options.storage_destination.clone(),
                format: StorageFormat::Screenshot,
            })
            // Stage 7: Store extracted content
            .add_stage(PipelineStage::Store {
                destination: options.storage_destination.clone(),
                format: StorageFormat::Json,
            })
            // Configuration
            .with_retry(2)
            .with_timeout(Duration::from_secs(90))
            .with_atomic_storage(true)
            .build()
            .await
    }
}

pub struct RenderExtractOptions {
    pub viewport: Option<Viewport>,
    pub full_page_screenshot: bool,
    pub screenshot_format: ImageFormat,
    pub enable_ocr: bool,
    pub ocr_language: Option<String>,
    pub storage_destination: StoreDestination,
}
```

---

### 3.2 Pipeline Stage Registry

```rust
/// Registry of available pipeline stages
pub struct StageRegistry {
    stages: HashMap<String, Box<dyn StageFactory>>,
}

impl StageRegistry {
    /// Register default stages
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Fetch stages
        registry.register("fetch", Box::new(FetchStageFactory));
        registry.register("parallel_fetch", Box::new(ParallelFetchStageFactory));

        // Browser stages
        registry.register("browser_launch", Box::new(BrowserLaunchStageFactory));
        registry.register("navigate", Box::new(NavigateStageFactory));
        registry.register("browser_actions", Box::new(BrowserActionsStageFactory));
        registry.register("screenshot", Box::new(ScreenshotStageFactory));

        // Extraction stages
        registry.register("extract", Box::new(ExtractStageFactory));
        registry.register("parallel_extract", Box::new(ParallelExtractStageFactory));
        registry.register("schema_extract", Box::new(SchemaExtractStageFactory));

        // Transform stages
        registry.register("transform", Box::new(TransformStageFactory));
        registry.register("validate", Box::new(ValidateStageFactory));
        registry.register("aggregate", Box::new(AggregateStageFactory));

        // Storage stages
        registry.register("cache", Box::new(CacheStageFactory));
        registry.register("store", Box::new(StoreStageFactory));
        registry.register("batch_cache", Box::new(BatchCacheStageFactory));

        // Routing stages
        registry.register("route", Box::new(RouteStageFactory));
        registry.register("conditional", Box::new(ConditionalStageFactory));

        registry
    }
}
```

---

## 4. Error Recovery Strategies

### 4.1 Retry Strategies

#### Exponential Backoff with Jitter

```rust
/// Retry configuration for pipeline stages
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,

    /// Initial backoff duration
    pub initial_backoff: Duration,

    /// Backoff multiplier
    pub backoff_multiplier: f64,

    /// Maximum backoff duration
    pub max_backoff: Duration,

    /// Add random jitter to prevent thundering herd
    pub jitter: bool,
}

impl RetryConfig {
    pub fn exponential() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            max_backoff: Duration::from_secs(10),
            jitter: true,
        }
    }

    /// Calculate backoff for attempt
    pub fn backoff_for_attempt(&self, attempt: usize) -> Duration {
        let base_backoff = self.initial_backoff.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);

        let backoff = Duration::from_millis(
            base_backoff.min(self.max_backoff.as_millis() as f64) as u64
        );

        if self.jitter {
            // Add up to 25% random jitter
            let jitter_factor = 1.0 + (rand::random::<f64>() * 0.25 - 0.125);
            Duration::from_millis((backoff.as_millis() as f64 * jitter_factor) as u64)
        } else {
            backoff
        }
    }
}
```

---

### 4.2 Fallback Strategies

#### Multi-Strategy Fallback Chain

```rust
/// Fallback strategy for extraction failures
pub enum FallbackStrategy {
    /// Try alternative extraction strategies
    AlternativeStrategy {
        strategies: Vec<ExtractionStrategy>,
    },

    /// Fallback to simpler method (e.g., Browser → HTTP)
    SimplerMethod {
        fallback_method: ExecutionMethod,
    },

    /// Use cached result if available
    CachedResult {
        max_age: Duration,
    },

    /// Return partial results with errors
    PartialResults {
        include_errors: bool,
    },

    /// Custom fallback handler
    Custom {
        handler: Arc<dyn FallbackHandler>,
    },
}

impl PipelineExecutor {
    async fn execute_stage_with_fallback(
        &self,
        stage: &PipelineStage,
        context: &mut PipelineContext,
        fallback: &FallbackStrategy,
    ) -> Result<StageResult> {
        match self.execute_stage(stage, context).await {
            Ok(result) => Ok(result),
            Err(error) => {
                tracing::warn!(
                    stage = %stage.name(),
                    error = %error,
                    "Stage failed, attempting fallback"
                );

                match fallback {
                    FallbackStrategy::AlternativeStrategy { strategies } => {
                        self.try_alternative_strategies(stage, context, strategies).await
                    }
                    FallbackStrategy::SimplerMethod { fallback_method } => {
                        self.try_simpler_method(stage, context, fallback_method).await
                    }
                    FallbackStrategy::CachedResult { max_age } => {
                        self.try_cached_result(stage, context, *max_age).await
                    }
                    FallbackStrategy::PartialResults { include_errors } => {
                        Ok(StageResult::partial(context, *include_errors))
                    }
                    FallbackStrategy::Custom { handler } => {
                        handler.handle_fallback(stage, context, error).await
                    }
                }
            }
        }
    }
}
```

---

### 4.3 Circuit Breaker Pattern

```rust
/// Circuit breaker for external services
pub struct CircuitBreaker {
    /// Current state
    state: Arc<RwLock<CircuitState>>,

    /// Configuration
    config: CircuitBreakerConfig,

    /// Metrics
    metrics: Arc<CircuitMetrics>,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: usize,

    /// Success threshold to close circuit
    pub success_threshold: usize,

    /// Timeout duration in open state
    pub timeout: Duration,

    /// Window duration for failure counting
    pub window: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        let state = self.state.read().await.clone();

        match state {
            CircuitState::Open => {
                // Check if timeout expired
                if self.should_attempt_reset().await {
                    self.transition_to_half_open().await;
                    self.execute_and_track(f).await
                } else {
                    Err(RiptideError::CircuitOpen)
                }
            }
            CircuitState::HalfOpen => {
                // Trial request
                match self.execute_and_track(f).await {
                    Ok(result) => {
                        self.on_success().await;
                        Ok(result)
                    }
                    Err(err) => {
                        self.on_failure().await;
                        Err(err)
                    }
                }
            }
            CircuitState::Closed => {
                self.execute_and_track(f).await
            }
        }
    }

    async fn execute_and_track<F, T>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        match f.await {
            Ok(result) => {
                self.metrics.record_success().await;
                Ok(result)
            }
            Err(err) => {
                self.metrics.record_failure().await;

                // Check if should open circuit
                if self.should_open_circuit().await {
                    self.transition_to_open().await;
                }

                Err(err)
            }
        }
    }
}
```

---

## 5. Caching Integration

### 5.1 Multi-Level Caching Strategy

```rust
/// Multi-level cache for pipeline results
pub struct CacheStrategy {
    /// L1: In-memory LRU cache
    memory_cache: Arc<RwLock<LruCache<String, CachedValue>>>,

    /// L2: Redis distributed cache
    redis_cache: Option<Arc<RedisCache>>,

    /// L3: Disk cache for large payloads
    disk_cache: Option<Arc<DiskCache>>,

    /// Configuration
    config: CacheConfig,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// L1 cache size (number of entries)
    pub memory_cache_size: usize,

    /// L2 cache TTL
    pub redis_ttl: Duration,

    /// L3 cache directory
    pub disk_cache_dir: Option<PathBuf>,

    /// Maximum payload size for memory cache (bytes)
    pub max_memory_payload_size: usize,

    /// Enable compression for disk cache
    pub compress_disk_cache: bool,
}

impl CacheStrategy {
    /// Get from cache with multi-level fallthrough
    pub async fn get(&self, key: &str) -> Option<CachedValue> {
        // Try L1 (memory)
        if let Some(value) = self.memory_cache.read().await.get(key) {
            self.record_hit(CacheLevel::Memory);
            return Some(value.clone());
        }

        // Try L2 (Redis)
        if let Some(redis) = &self.redis_cache {
            if let Ok(Some(value)) = redis.get(key).await {
                // Promote to L1
                if value.size() <= self.config.max_memory_payload_size {
                    self.memory_cache.write().await.put(key.to_string(), value.clone());
                }
                self.record_hit(CacheLevel::Redis);
                return Some(value);
            }
        }

        // Try L3 (Disk)
        if let Some(disk) = &self.disk_cache {
            if let Ok(Some(value)) = disk.get(key).await {
                // Promote to L2 and L1
                if let Some(redis) = &self.redis_cache {
                    let _ = redis.set(key, &value, self.config.redis_ttl).await;
                }
                if value.size() <= self.config.max_memory_payload_size {
                    self.memory_cache.write().await.put(key.to_string(), value.clone());
                }
                self.record_hit(CacheLevel::Disk);
                return Some(value);
            }
        }

        self.record_miss();
        None
    }

    /// Set value in appropriate cache levels
    pub async fn set(&self, key: String, value: CachedValue, ttl: Duration) -> Result<()> {
        let size = value.size();

        // L1: Memory (if small enough)
        if size <= self.config.max_memory_payload_size {
            self.memory_cache.write().await.put(key.clone(), value.clone());
        }

        // L2: Redis (always)
        if let Some(redis) = &self.redis_cache {
            redis.set(&key, &value, ttl).await?;
        }

        // L3: Disk (for large payloads or long TTL)
        if size > self.config.max_memory_payload_size || ttl > Duration::from_secs(3600) {
            if let Some(disk) = &self.disk_cache {
                disk.set(&key, &value, ttl).await?;
            }
        }

        Ok(())
    }
}
```

---

### 5.2 Cache Key Generation

```rust
/// Intelligent cache key generation
pub trait CacheKeyGenerator: Send + Sync {
    fn generate_key(&self, context: &PipelineContext) -> String;
}

/// URL-based cache key generator
pub struct UrlKeyGenerator;

impl CacheKeyGenerator for UrlKeyGenerator {
    fn generate_key(&self, context: &PipelineContext) -> String {
        let url = context.get_url();
        let strategy = context.get_strategy();

        // Include relevant parameters in key
        format!(
            "extract:{}:{}:v1",
            strategy,
            hash_url(url)
        )
    }
}

/// Content-based cache key generator
pub struct ContentHashKeyGenerator;

impl CacheKeyGenerator for ContentHashKeyGenerator {
    fn generate_key(&self, context: &PipelineContext) -> String {
        let content = context.get_content();
        let hash = xxhash::xxh3_64(content.as_bytes());

        format!("content:{:x}:v1", hash)
    }
}

/// Composite cache key generator
pub struct CompositeKeyGenerator {
    generators: Vec<Box<dyn CacheKeyGenerator>>,
}

impl CacheKeyGenerator for CompositeKeyGenerator {
    fn generate_key(&self, context: &PipelineContext) -> String {
        let parts: Vec<String> = self
            .generators
            .iter()
            .map(|g| g.generate_key(context))
            .collect();

        parts.join(":")
    }
}
```

---

### 5.3 Cache Invalidation Strategies

```rust
/// Cache invalidation policy
pub enum InvalidationPolicy {
    /// Time-based expiration
    TTL { duration: Duration },

    /// Invalidate on condition
    Conditional { predicate: Arc<dyn Fn(&CachedValue) -> bool + Send + Sync> },

    /// Least Recently Used eviction
    LRU { max_entries: usize },

    /// Least Frequently Used eviction
    LFU { max_entries: usize },

    /// Manual invalidation only
    Manual,
}

/// Cache invalidation manager
pub struct InvalidationManager {
    policy: InvalidationPolicy,
    stats: Arc<RwLock<InvalidationStats>>,
}

impl InvalidationManager {
    /// Check if cache entry should be invalidated
    pub async fn should_invalidate(&self, entry: &CacheEntry) -> bool {
        match &self.policy {
            InvalidationPolicy::TTL { duration } => {
                entry.age() > *duration
            }
            InvalidationPolicy::Conditional { predicate } => {
                predicate(&entry.value)
            }
            InvalidationPolicy::LRU { .. } | InvalidationPolicy::LFU { .. } => {
                // Handled by cache implementation
                false
            }
            InvalidationPolicy::Manual => false,
        }
    }

    /// Perform cache cleanup
    pub async fn cleanup(&self, cache: &mut dyn Cache) -> Result<CleanupStats> {
        let start = Instant::now();
        let mut removed = 0;
        let mut bytes_freed = 0;

        // Scan cache for entries to remove
        for entry in cache.iter_mut() {
            if self.should_invalidate(&entry).await {
                bytes_freed += entry.size();
                cache.remove(&entry.key)?;
                removed += 1;
            }
        }

        Ok(CleanupStats {
            duration: start.elapsed(),
            entries_removed: removed,
            bytes_freed,
        })
    }
}
```

---

## 6. Resource Management

### 6.1 Resource Lifecycle

```rust
/// Resource lifecycle manager for workflows
pub struct ResourceLifecycle {
    /// Active resources
    active: Arc<RwLock<HashMap<ResourceId, Box<dyn Resource>>>>,

    /// Resource limits
    limits: ResourceLimits,

    /// Cleanup scheduler
    cleanup: Arc<CleanupScheduler>,
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_browsers: usize,
    pub max_http_connections: usize,
    pub max_wasm_instances: usize,
    pub max_memory_mb: usize,
}

impl ResourceLifecycle {
    /// Allocate resources for workflow
    pub async fn allocate(&self, needs: ResourceNeeds) -> Result<ResourceHandle> {
        // Check limits
        self.check_limits(&needs).await?;

        // Acquire resources
        let resources = self.acquire_resources(needs).await?;

        // Register for cleanup
        let handle = self.register_resources(resources).await?;

        Ok(handle)
    }

    /// Release resources
    pub async fn release(&self, handle: ResourceHandle) -> Result<()> {
        let resources = self.active.write().await.remove(&handle.id);

        if let Some(resources) = resources {
            // Graceful shutdown
            resources.shutdown().await?;
        }

        Ok(())
    }

    /// Automatic cleanup of expired resources
    pub async fn auto_cleanup(&self) {
        let mut expired = Vec::new();

        {
            let active = self.active.read().await;
            for (id, resource) in active.iter() {
                if resource.is_expired() {
                    expired.push(*id);
                }
            }
        }

        for id in expired {
            if let Some(resource) = self.active.write().await.remove(&id) {
                let _ = resource.shutdown().await;
            }
        }
    }
}

/// RAII resource handle with automatic cleanup
pub struct ResourceHandle {
    id: ResourceId,
    lifecycle: Arc<ResourceLifecycle>,
    released: Arc<AtomicBool>,
}

impl Drop for ResourceHandle {
    fn drop(&mut self) {
        if !self.released.load(Ordering::SeqCst) {
            // Spawn cleanup task
            let id = self.id;
            let lifecycle = Arc::clone(&self.lifecycle);
            tokio::spawn(async move {
                let _ = lifecycle.release(ResourceHandle {
                    id,
                    lifecycle,
                    released: Arc::new(AtomicBool::new(true)),
                }).await;
            });
        }
    }
}
```

---

### 6.2 Memory Management

```rust
/// Memory pressure monitor for workflows
pub struct MemoryMonitor {
    /// Current usage
    current_usage: Arc<AtomicUsize>,

    /// Soft limit (start cleanup)
    soft_limit: usize,

    /// Hard limit (reject new allocations)
    hard_limit: usize,

    /// Pressure callbacks
    callbacks: Arc<RwLock<Vec<Box<dyn PressureCallback>>>>,
}

impl MemoryMonitor {
    /// Check if allocation is allowed
    pub async fn check_allocation(&self, size: usize) -> Result<()> {
        let current = self.current_usage.load(Ordering::SeqCst);
        let new_usage = current + size;

        if new_usage > self.hard_limit {
            return Err(RiptideError::MemoryExhausted {
                requested: size,
                available: self.hard_limit - current,
            });
        }

        if new_usage > self.soft_limit {
            // Trigger cleanup
            self.trigger_pressure_callbacks(MemoryPressure::High).await;
        }

        Ok(())
    }

    /// Track allocation
    pub fn track_allocation(&self, size: usize) {
        self.current_usage.fetch_add(size, Ordering::SeqCst);
    }

    /// Track deallocation
    pub fn track_deallocation(&self, size: usize) {
        self.current_usage.fetch_sub(size, Ordering::SeqCst);
    }

    /// Get current memory pressure
    pub fn pressure(&self) -> MemoryPressure {
        let current = self.current_usage.load(Ordering::SeqCst);
        let usage_percent = (current as f64 / self.hard_limit as f64) * 100.0;

        match usage_percent {
            p if p < 70.0 => MemoryPressure::Low,
            p if p < 85.0 => MemoryPressure::Medium,
            p if p < 95.0 => MemoryPressure::High,
            _ => MemoryPressure::Critical,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryPressure {
    Low,
    Medium,
    High,
    Critical,
}

pub trait PressureCallback: Send + Sync {
    fn on_pressure(&self, pressure: MemoryPressure) -> impl Future<Output = ()> + Send;
}
```

---

## 7. Metrics & Observability

### 7.1 Metrics Collection

```rust
/// Comprehensive metrics collector for workflows
pub struct MetricsCollector {
    /// Stage metrics
    stage_metrics: Arc<RwLock<HashMap<String, StageMetrics>>>,

    /// Workflow metrics
    workflow_metrics: Arc<RwLock<WorkflowMetrics>>,

    /// Resource metrics
    resource_metrics: Arc<RwLock<ResourceMetrics>>,

    /// Error metrics
    error_metrics: Arc<RwLock<ErrorMetrics>>,
}

#[derive(Debug, Clone, Default)]
pub struct StageMetrics {
    pub executions: u64,
    pub successes: u64,
    pub failures: u64,
    pub retries: u64,
    pub total_duration: Duration,
    pub min_duration: Option<Duration>,
    pub max_duration: Option<Duration>,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

#[derive(Debug, Clone, Default)]
pub struct WorkflowMetrics {
    pub total_workflows: u64,
    pub completed_workflows: u64,
    pub failed_workflows: u64,
    pub avg_duration: Duration,
    pub p50_duration: Duration,
    pub p95_duration: Duration,
    pub p99_duration: Duration,
}

impl MetricsCollector {
    /// Record stage execution
    pub async fn record_stage(
        &self,
        stage: &str,
        duration: Duration,
        result: &Result<StageResult>,
    ) {
        let mut metrics = self.stage_metrics.write().await;
        let stage_metrics = metrics.entry(stage.to_string()).or_default();

        stage_metrics.executions += 1;
        stage_metrics.total_duration += duration;

        match result {
            Ok(r) => {
                stage_metrics.successes += 1;
                if matches!(r.status, StageStatus::CachedSuccess) {
                    stage_metrics.cache_hits += 1;
                }
            }
            Err(_) => {
                stage_metrics.failures += 1;
            }
        }

        // Update min/max
        if stage_metrics.min_duration.is_none() || duration < stage_metrics.min_duration.unwrap() {
            stage_metrics.min_duration = Some(duration);
        }
        if stage_metrics.max_duration.is_none() || duration > stage_metrics.max_duration.unwrap() {
            stage_metrics.max_duration = Some(duration);
        }
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let mut output = String::new();

        // Stage metrics
        let stage_metrics = self.stage_metrics.read().await;
        for (stage, metrics) in stage_metrics.iter() {
            output.push_str(&format!(
                "riptide_stage_executions_total{{stage=\"{}\"}} {}\n",
                stage, metrics.executions
            ));
            output.push_str(&format!(
                "riptide_stage_duration_seconds{{stage=\"{}\"}} {:.3}\n",
                stage,
                metrics.total_duration.as_secs_f64() / metrics.executions as f64
            ));
            output.push_str(&format!(
                "riptide_stage_cache_hit_ratio{{stage=\"{}\"}} {:.3}\n",
                stage,
                metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64
            ));
        }

        output
    }
}
```

---

### 7.2 Distributed Tracing

```rust
/// OpenTelemetry integration for distributed tracing
pub struct TracingIntegration {
    tracer: Tracer,
    config: TracingConfig,
}

#[derive(Debug, Clone)]
pub struct TracingConfig {
    pub service_name: String,
    pub endpoint: String,
    pub sample_rate: f64,
}

impl TracingIntegration {
    /// Create span for workflow
    pub fn workflow_span(&self, workflow_id: &str) -> Span {
        self.tracer
            .span_builder("workflow.execute")
            .with_attributes(vec![
                KeyValue::new("workflow.id", workflow_id.to_string()),
                KeyValue::new("service.name", self.config.service_name.clone()),
            ])
            .start(&self.tracer)
    }

    /// Create span for stage
    pub fn stage_span(&self, stage: &str, parent: &SpanContext) -> Span {
        self.tracer
            .span_builder(format!("stage.{}", stage))
            .with_parent_context(parent.clone())
            .with_attributes(vec![
                KeyValue::new("stage.name", stage.to_string()),
            ])
            .start(&self.tracer)
    }

    /// Record error in span
    pub fn record_error(&self, span: &Span, error: &RiptideError) {
        span.set_status(Status::error(error.to_string()));
        span.set_attribute(KeyValue::new("error.type", error.kind()));
        span.set_attribute(KeyValue::new("error.message", error.to_string()));
    }
}
```

---

## 8. Example Handlers

### 8.1 /api/crawl-and-extract Endpoint

```rust
/// Handler for browser-based crawl and extract workflow
#[axum::debug_handler]
pub async fn crawl_and_extract_handler(
    State(composer): State<Arc<WorkflowComposer>>,
    Json(req): Json<CrawlExtractRequest>,
) -> Result<Json<CrawlExtractResponse>, AppError> {
    let span = info_span!("crawl_and_extract", url = %req.url);

    async move {
        // Create pipeline
        let pipeline = composer
            .pipeline
            .crawl_and_extract_pipeline(&req.url, req.into_options())
            .await?;

        // Execute with timeout
        let result = tokio::time::timeout(
            Duration::from_secs(req.timeout_secs.unwrap_or(60)),
            composer.pipeline.execute(pipeline),
        )
        .await
        .map_err(|_| RiptideError::Timeout)??;

        // Extract final result
        let data = result.final_output;

        Ok(Json(CrawlExtractResponse {
            url: req.url,
            title: data["title"].as_str().map(String::from),
            content: data["text"].as_str().unwrap_or("").to_string(),
            links: extract_links(&data),
            images: extract_images(&data),
            metadata: extract_metadata(&data),
            extraction_time_ms: result.total_duration.as_millis() as u64,
            stages_completed: result.stages_completed,
            cache_hits: count_cache_hits(&result),
        }))
    }
    .instrument(span)
    .await
}

#[derive(Debug, Deserialize)]
pub struct CrawlExtractRequest {
    pub url: String,
    pub enable_stealth: Option<bool>,
    pub wait_for_selector: Option<String>,
    pub extract_links: Option<bool>,
    pub extract_images: Option<bool>,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct CrawlExtractResponse {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub links: Vec<String>,
    pub images: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub extraction_time_ms: u64,
    pub stages_completed: usize,
    pub cache_hits: usize,
}
```

---

### 8.2 /api/batch-scrape Endpoint

```rust
/// Handler for batch scraping workflow
#[axum::debug_handler]
pub async fn batch_scrape_handler(
    State(composer): State<Arc<WorkflowComposer>>,
    Json(req): Json<BatchScrapeRequest>,
) -> Result<Json<BatchScrapeResponse>, AppError> {
    let span = info_span!("batch_scrape", urls = req.urls.len());

    async move {
        // Validate request
        if req.urls.len() > 100 {
            return Err(AppError::BadRequest("Maximum 100 URLs per batch".into()));
        }

        // Create pipeline
        let options = BatchScrapeOptions {
            concurrency: req.concurrency.unwrap_or(5).min(10),
            deduplicate: req.deduplicate.unwrap_or(true),
            merge_similar_threshold: req.merge_similar_threshold,
            cache_ttl: Duration::from_secs(req.cache_ttl_secs.unwrap_or(3600)),
        };

        let pipeline = composer
            .pipeline
            .batch_scrape_pipeline(req.urls.clone(), options)
            .await?;

        // Execute with progress tracking
        let result = composer.pipeline.execute(pipeline).await?;

        // Parse results
        let results = parse_batch_results(&result.final_output, &req.urls)?;

        Ok(Json(BatchScrapeResponse {
            total_urls: req.urls.len(),
            successful: results.len(),
            failed: req.urls.len() - results.len(),
            results,
            execution_time_ms: result.total_duration.as_millis() as u64,
            cache_hit_ratio: calculate_cache_ratio(&result),
        }))
    }
    .instrument(span)
    .await
}

#[derive(Debug, Deserialize)]
pub struct BatchScrapeRequest {
    pub urls: Vec<String>,
    pub concurrency: Option<usize>,
    pub deduplicate: Option<bool>,
    pub merge_similar_threshold: Option<f64>,
    pub cache_ttl_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct BatchScrapeResponse {
    pub total_urls: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<ScrapedItem>,
    pub execution_time_ms: u64,
    pub cache_hit_ratio: f64,
}

#[derive(Debug, Serialize)]
pub struct ScrapedItem {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub word_count: usize,
    pub extracted_at: i64,
}
```

---

### 8.3 /api/render-and-extract Endpoint

```rust
/// Handler for render and extract workflow with screenshot
#[axum::debug_handler]
pub async fn render_and_extract_handler(
    State(composer): State<Arc<WorkflowComposer>>,
    Json(req): Json<RenderExtractRequest>,
) -> Result<Json<RenderExtractResponse>, AppError> {
    let span = info_span!("render_and_extract", url = %req.url);

    async move {
        // Create pipeline
        let options = RenderExtractOptions {
            viewport: req.viewport.map(|v| Viewport {
                width: v.width,
                height: v.height,
            }),
            full_page_screenshot: req.full_page.unwrap_or(true),
            screenshot_format: req.format.unwrap_or(ImageFormat::Png),
            enable_ocr: req.enable_ocr.unwrap_or(false),
            ocr_language: req.ocr_language.clone(),
            storage_destination: StoreDestination::Memory,
        };

        let pipeline = composer
            .pipeline
            .render_and_extract_pipeline(&req.url, options)
            .await?;

        // Execute
        let result = composer.pipeline.execute(pipeline).await?;

        // Extract results
        let data = result.final_output;
        let screenshot_data = data["screenshot"].as_str()
            .ok_or_else(|| AppError::Internal("Screenshot not found".into()))?;

        Ok(Json(RenderExtractResponse {
            url: req.url,
            title: data["title"].as_str().map(String::from),
            content: data["text"].as_str().unwrap_or("").to_string(),
            screenshot: ScreenshotData {
                data: screenshot_data.to_string(),
                format: req.format.unwrap_or(ImageFormat::Png),
                width: data["screenshot_width"].as_u64().unwrap_or(1920) as u32,
                height: data["screenshot_height"].as_u64().unwrap_or(1080) as u32,
            },
            ocr_text: data["ocr_text"].as_str().map(String::from),
            execution_time_ms: result.total_duration.as_millis() as u64,
        }))
    }
    .instrument(span)
    .await
}

#[derive(Debug, Deserialize)]
pub struct RenderExtractRequest {
    pub url: String,
    pub viewport: Option<ViewportSpec>,
    pub full_page: Option<bool>,
    pub format: Option<ImageFormat>,
    pub enable_ocr: Option<bool>,
    pub ocr_language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ViewportSpec {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize)]
pub struct RenderExtractResponse {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub screenshot: ScreenshotData,
    pub ocr_text: Option<String>,
    pub execution_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct ScreenshotData {
    pub data: String, // Base64 encoded
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
}
```

---

## 9. Missing Capabilities Analysis

### 9.1 Current Gaps for P1 Completion

| Capability | Status | Priority | Effort | Blocker |
|------------|--------|----------|--------|---------|
| **BrowserFacade Implementation** | ⚙️ Partial | HIGH | 3 days | Phase 2 |
| **ExtractionFacade Implementation** | ⚙️ Partial | HIGH | 2 days | Phase 2 |
| **PipelineFacade Stage Execution** | 🔴 TODO | HIGH | 4 days | Phase 2 |
| **CacheFacade Integration** | 🔴 TODO | MEDIUM | 2 days | Phase 2 |
| **StorageFacade for Screenshots** | 🔴 TODO | LOW | 2 days | Phase 3 |
| **Workflow Orchestrator** | 🔴 TODO | HIGH | 3 days | Phase 2 |
| **Resource Pool Manager** | 🔴 TODO | MEDIUM | 2 days | Phase 2 |
| **Metrics Collector** | 🔴 TODO | MEDIUM | 2 days | Phase 3 |
| **Distributed Tracing** | 🔴 TODO | LOW | 1 day | Phase 3 |

**Total Phase 2 Effort**: ~16 days (3 weeks with testing)

---

### 9.2 Implementation Priority

#### Phase 2A: Core Facades (Week 1)
- ✅ ScraperFacade (DONE)
- 🔴 BrowserFacade (browser.rs partial → full implementation)
- 🔴 ExtractionFacade (extractor.rs partial → full implementation)

#### Phase 2B: Composition Layer (Week 2)
- 🔴 PipelineFacade stage execution
- 🔴 Workflow orchestrator
- 🔴 Resource pool manager

#### Phase 2C: Integration (Week 3)
- 🔴 API handlers (/crawl-and-extract, /batch-scrape, /render-and-extract)
- 🔴 Integration tests
- 🔴 Performance benchmarks

---

### 9.3 Missing Features per Facade

#### BrowserFacade
- ✅ Basic structure defined
- ✅ Screenshot method implemented
- ✅ Navigation implemented
- 🔴 Browser actions execution (Click, Type, Wait, Scroll)
- 🔴 Cookie management
- 🔴 Local storage access
- 🔴 Wait conditions (NetworkIdle, Selector)
- 🔴 Error recovery for navigation failures

#### ExtractionFacade
- ✅ Basic structure defined
- ✅ HTML extraction implemented
- ✅ PDF extraction implemented
- ✅ Strategy fallback chain
- 🔴 Schema-based extraction
- 🔴 Confidence scoring refinement
- 🔴 Custom transformer support
- 🔴 Validation pipeline

#### PipelineFacade
- ✅ Basic structure defined
- ✅ Builder pattern implemented
- ✅ Sequential execution
- ✅ Parallel execution
- 🔴 Pre-built templates (crawl-and-extract, batch-scrape, render-and-extract)
- 🔴 Stage registry
- 🔴 Error recovery strategies
- 🔴 Resource management integration
- 🔴 Progress tracking

#### CacheFacade
- 🔴 Not yet implemented
- 🔴 Multi-level caching (L1/L2/L3)
- 🔴 Cache key generation
- 🔴 Invalidation policies
- 🔴 Cleanup scheduler

---

## 10. Implementation Roadmap

### Week 1: Core Facades
**Target**: Complete BrowserFacade and ExtractionFacade

#### Day 1-2: BrowserFacade
- Implement browser actions (click, type, scroll, etc.)
- Add wait conditions (NetworkIdle, Selector)
- Cookie management
- Local storage access
- Error recovery

#### Day 3-4: ExtractionFacade
- Schema-based extraction
- Confidence scoring refinement
- Custom transformer support
- Validation pipeline integration

#### Day 5: Testing
- Unit tests for browser actions
- Unit tests for extraction strategies
- Integration tests with mock browser

---

### Week 2: Composition Layer
**Target**: Complete PipelineFacade and WorkflowComposer

#### Day 6-7: PipelineFacade Templates
- Implement crawl-and-extract template
- Implement batch-scrape template
- Implement render-and-extract template
- Stage registry

#### Day 8-9: Workflow Orchestrator
- Implement WorkflowComposer
- Resource pool manager
- Error recovery strategies
- Progress tracking

#### Day 10: Testing
- Pipeline execution tests
- Workflow orchestration tests
- Resource management tests

---

### Week 3: Integration & Polish
**Target**: Complete API handlers and documentation

#### Day 11-12: API Handlers
- /api/crawl-and-extract endpoint
- /api/batch-scrape endpoint
- /api/render-and-extract endpoint
- Request validation and error handling

#### Day 13-14: Integration Testing
- End-to-end workflow tests
- Performance benchmarks
- Load testing
- Error scenario testing

#### Day 15: Documentation & Polish
- API documentation
- Architecture diagrams
- Usage examples
- Performance tuning

---

### Success Criteria

#### Phase 2 Complete When:
- [x] ScraperFacade implemented (24 tests passing)
- [ ] BrowserFacade fully implemented (15+ tests)
- [ ] ExtractionFacade fully implemented (20+ tests)
- [ ] PipelineFacade with 3 templates (10+ tests)
- [ ] WorkflowComposer implemented (8+ tests)
- [ ] 3 API handlers working (crawl-and-extract, batch-scrape, render-and-extract)
- [ ] Integration tests passing (10+ scenarios)
- [ ] Performance benchmarks documented
- [ ] Architecture documentation complete

---

## Conclusion

This document provides a comprehensive design for advanced facade composition patterns in riptide-api. The architecture supports:

1. **Multi-step workflows** with coordinated facade interactions
2. **Error recovery** with retry, fallback, and circuit breaker patterns
3. **Resource management** with pooling and automatic cleanup
4. **Caching** with multi-level strategies
5. **Observability** with metrics and distributed tracing

### Next Steps
1. Review and approve architecture design
2. Begin Week 1 implementation (BrowserFacade + ExtractionFacade)
3. Implement PipelineFacade templates (Week 2)
4. Create API handlers and integration tests (Week 3)

**Estimated P1-A4 Completion**: 3 weeks (15 days)
**P1 Total Completion**: 82% → 95% (+13%)

---

**Document Status**: ✅ COMPLETE
**Ready For**: Implementation Phase
**Dependencies**: riptide-engine, riptide-extraction, riptide-pdf, riptide-cache
