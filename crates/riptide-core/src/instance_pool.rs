use anyhow::{anyhow, Result};
use std::collections::VecDeque;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, SemaphorePermit};
use tokio::time::{timeout, sleep};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use wasmtime::{component::*, Config, Engine, Store};

use crate::component::{ExtractorConfig, PerformanceMetrics, WasmResourceTracker};
use crate::types::{ExtractedDoc, ExtractionMode};

/// Generate bindings from enhanced WIT file
wasmtime::component::bindgen!({
    world: "extractor",
    path: "wit",
});

/// Enhanced pooled instance with comprehensive tracking
pub struct PooledInstance {
    pub id: String,
    pub engine: Arc<Engine>,
    pub component: Arc<Component>,
    pub linker: Arc<Linker<()>>,
    pub created_at: Instant,
    pub last_used: Instant,
    pub use_count: u64,
    pub failure_count: u64,
    pub memory_usage_bytes: u64,
    pub resource_tracker: WasmResourceTracker,
}

impl PooledInstance {
    pub fn new(
        engine: Arc<Engine>,
        component: Arc<Component>,
        linker: Arc<Linker<()>>,
        max_memory_pages: usize,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let now = Instant::now();

        Self {
            id,
            engine,
            component,
            linker,
            created_at: now,
            last_used: now,
            use_count: 0,
            failure_count: 0,
            memory_usage_bytes: 0,
            resource_tracker: WasmResourceTracker::new(max_memory_pages),
        }
    }

    /// Check if instance is healthy and reusable
    pub fn is_healthy(&self, config: &ExtractorConfig) -> bool {
        self.use_count < 1000
            && self.failure_count < 5
            && self.memory_usage_bytes < config.memory_limit
            && self.resource_tracker.grow_failures() < 10
    }

    /// Update usage statistics
    pub fn record_usage(&mut self, success: bool) {
        self.last_used = Instant::now();
        self.use_count += 1;
        if !success {
            self.failure_count += 1;
        }
        self.memory_usage_bytes = self.resource_tracker.current_memory_pages() * 64 * 1024;
    }

    /// Create fresh Store with resource limits
    pub fn create_fresh_store(&mut self) -> Store<WasmResourceTracker> {
        let mut store = Store::new(&*self.engine, self.resource_tracker.clone());

        // Set resource limits
        store.limiter(|tracker| tracker);

        // Enable epoch interruption for timeouts
        store.epoch_deadline_trap();

        store
    }
}

/// Circuit breaker states for WASM error handling
#[derive(Clone, Debug)]
pub enum CircuitBreakerState {
    Closed {
        failure_count: u64,
        success_count: u64,
        last_failure: Option<Instant>,
    },
    Open {
        opened_at: Instant,
        failure_count: u64,
    },
    HalfOpen {
        test_requests: u64,
        start_time: Instant,
    },
}

/// Advanced instance pool with semaphore-based concurrency control
pub struct AdvancedInstancePool {
    /// Pool configuration
    config: ExtractorConfig,
    /// Shared engine for all instances
    engine: Arc<Engine>,
    /// Shared component for all instances
    component: Arc<Component>,
    /// Shared linker for all instances
    linker: Arc<Linker<()>>,
    /// Available instances queue
    available_instances: Arc<Mutex<VecDeque<PooledInstance>>>,
    /// Semaphore for concurrency control
    semaphore: Arc<Semaphore>,
    /// Performance metrics
    metrics: Arc<Mutex<PerformanceMetrics>>,
    /// Circuit breaker state
    circuit_state: Arc<Mutex<CircuitBreakerState>>,
    /// Component path for creation
    component_path: String,
}

impl AdvancedInstancePool {
    /// Create new instance pool with configuration
    pub async fn new(
        config: ExtractorConfig,
        engine: Engine,
        component_path: &str,
    ) -> Result<Self> {
        info!(
            max_pool_size = config.max_pool_size,
            initial_pool_size = config.initial_pool_size,
            "Initializing advanced instance pool"
        );

        // Load component
        let component = Component::from_file(&engine, component_path)?;
        let linker = Linker::new(&engine);

        let pool = Self {
            config: config.clone(),
            engine: Arc::new(engine),
            component: Arc::new(component),
            linker: Arc::new(linker),
            available_instances: Arc::new(Mutex::new(VecDeque::with_capacity(config.max_pool_size))),
            semaphore: Arc::new(Semaphore::new(config.max_pool_size)),
            metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
            circuit_state: Arc::new(Mutex::new(CircuitBreakerState::Closed {
                failure_count: 0,
                success_count: 0,
                last_failure: None,
            })),
            component_path: component_path.to_string(),
        };

        // Pre-warm the pool
        pool.warm_up().await?;

        info!("Advanced instance pool initialized successfully");
        Ok(pool)
    }

    /// Pre-warm the pool with initial instances
    async fn warm_up(&self) -> Result<()> {
        debug!(warm_count = self.config.initial_pool_size, "Warming up instance pool");

        let mut instances = self.available_instances.lock().unwrap();
        for i in 0..self.config.initial_pool_size {
            let instance = self.create_instance().await?;
            instances.push_back(instance);
            debug!(instance_index = i, "Instance pre-warmed");
        }

        // Update metrics
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.pool_size = instances.len();
        }

        info!(
            pool_size = instances.len(),
            "Pool warm-up completed"
        );

        Ok(())
    }

    /// Extract content with full instance pooling and fallback support
    pub async fn extract(
        &self,
        html: &str,
        url: &str,
        mode: ExtractionMode,
    ) -> Result<ExtractedDoc> {
        // Check circuit breaker
        if self.is_circuit_open() {
            return self.fallback_extract(html, url, mode).await;
        }

        let start_time = Instant::now();

        // Acquire semaphore permit with timeout
        let permit = match timeout(
            self.config.extraction_timeout,
            self.semaphore.acquire()
        ).await {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => return Err(anyhow!("Semaphore closed")),
            Err(_) => {
                self.record_timeout();
                return self.fallback_extract(html, url, mode).await;
            }
        };

        let semaphore_wait_time = start_time.elapsed();

        // Get or create instance
        let mut instance = self.get_or_create_instance().await?;

        // Perform extraction with epoch timeout
        let extraction_result = self.extract_with_instance(&mut instance, html, url, mode).await;

        // Update metrics and return instance
        let success = extraction_result.is_ok();
        instance.record_usage(success);
        self.return_instance(instance).await;

        // Update circuit breaker
        self.record_extraction_result(success, start_time.elapsed());

        // Update semaphore wait time metric
        self.update_semaphore_wait_time(semaphore_wait_time);

        // Release permit
        drop(permit);

        match extraction_result {
            Ok(doc) => Ok(doc),
            Err(e) => {
                if self.config.enable_fallback {
                    warn!(error = %e, "WASM extraction failed, falling back to native");
                    self.fallback_extract(html, url, mode).await
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Get or create instance from pool
    async fn get_or_create_instance(&self) -> Result<PooledInstance> {
        // Try to get from pool first
        {
            let mut instances = self.available_instances.lock().unwrap();
            if let Some(mut instance) = instances.pop_front() {
                if instance.is_healthy(&self.config) {
                    return Ok(instance);
                }
                // Instance is unhealthy, create new one
                debug!(instance_id = %instance.id, "Discarding unhealthy instance");
            }
        }

        // Create new instance if needed
        self.create_instance().await
    }

    /// Create new instance
    async fn create_instance(&self) -> Result<PooledInstance> {
        debug!("Creating new WASM instance");

        let instance = PooledInstance::new(
            self.engine.clone(),
            self.component.clone(),
            self.linker.clone(),
            self.config.memory_limit_pages,
        );

        debug!(instance_id = %instance.id, "New WASM instance created");
        Ok(instance)
    }

    /// Extract content using specific instance
    async fn extract_with_instance(
        &self,
        instance: &mut PooledInstance,
        html: &str,
        url: &str,
        mode: ExtractionMode,
    ) -> Result<ExtractedDoc> {
        // Create fresh store to prevent state leaks
        let mut store = instance.create_fresh_store();

        // Set epoch deadline for timeout handling
        store.set_epoch_deadline(self.config.epoch_timeout_ms);

        // Spawn epoch advancement task
        let engine_weak = Arc::downgrade(&instance.engine);
        tokio::spawn(async move {
            sleep(Duration::from_millis(30000)).await; // 30 second timeout
            if let Some(engine) = engine_weak.upgrade() {
                engine.increment_epoch();
            }
        });

        // Instantiate component with fresh bindings
        let bindings = Extractor::instantiate(&mut store, &*instance.component, &*instance.linker)
            .map_err(|e| anyhow!("Component instantiation failed: {}", e))?;

        // Convert mode to WIT format
        let wit_mode = self.convert_extraction_mode(mode);

        // Execute extraction with timeout
        let extraction_future = bindings.interface0.call_extract(&mut store, html, url, &wit_mode);

        let result = timeout(self.config.extraction_timeout, extraction_future).await;

        match result {
            Ok(Ok(Ok(content))) => {
                // Success - convert to internal format
                Ok(ExtractedDoc {
                    url: content.url,
                    title: content.title,
                    byline: content.byline,
                    published_iso: content.published_iso,
                    markdown: content.markdown,
                    text: content.text,
                    links: content.links,
                    media: content.media,
                    language: content.language,
                    reading_time: content.reading_time,
                    quality_score: content.quality_score,
                    word_count: content.word_count,
                    categories: content.categories,
                    site_name: content.site_name,
                    description: content.description,
                })
            }
            Ok(Ok(Err(extraction_error))) => {
                Err(anyhow!("WASM extraction error: {:?}", extraction_error))
            }
            Ok(Err(e)) => {
                Err(anyhow!("Component call failed: {}", e))
            }
            Err(_) => {
                // Timeout occurred
                self.record_epoch_timeout();
                Err(anyhow!("Extraction timeout after {}ms", self.config.extraction_timeout.as_millis()))
            }
        }
    }

    /// Return instance to pool
    async fn return_instance(&self, instance: PooledInstance) {
        if instance.is_healthy(&self.config) {
            let mut instances = self.available_instances.lock().unwrap();
            instances.push_back(instance);
        }
        // Otherwise, let instance drop for cleanup

        // Update pool size metric
        let pool_size = self.available_instances.lock().unwrap().len();
        let mut metrics = self.metrics.lock().unwrap();
        metrics.pool_size = pool_size;
    }

    /// Fallback to native extraction
    async fn fallback_extract(
        &self,
        html: &str,
        url: &str,
        _mode: ExtractionMode,
    ) -> Result<ExtractedDoc> {
        // Record fallback usage
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.fallback_extractions += 1;
        }

        warn!("Using fallback extraction for URL: {}", url);

        // TODO: Implement native readability-rs extraction
        // For now, return basic extraction
        Ok(ExtractedDoc {
            url: url.to_string(),
            title: Some("Fallback Extraction".to_string()),
            text: html.chars().take(1000).collect(),
            markdown: format!("# Fallback Extraction\n\n{}", html.chars().take(800).collect::<String>()),
            ..Default::default()
        })
    }

    /// Convert ExtractionMode to WIT format
    fn convert_extraction_mode(&self, mode: ExtractionMode) -> exports::riptide::extractor::extract::ExtractionMode {
        match mode {
            ExtractionMode::Article => exports::riptide::extractor::extract::ExtractionMode::Article,
            ExtractionMode::Full => exports::riptide::extractor::extract::ExtractionMode::Full,
            ExtractionMode::Metadata => exports::riptide::extractor::extract::ExtractionMode::Metadata,
            ExtractionMode::Custom(selectors) => exports::riptide::extractor::extract::ExtractionMode::Custom(selectors),
        }
    }

    /// Check if circuit breaker is open
    fn is_circuit_open(&self) -> bool {
        let state = self.circuit_state.lock().unwrap();
        match *state {
            CircuitBreakerState::Open { opened_at, .. } => {
                opened_at.elapsed() < self.config.circuit_breaker_timeout
            }
            _ => false,
        }
    }

    /// Record extraction result for circuit breaker
    fn record_extraction_result(&self, success: bool, duration: Duration) {
        let mut state = self.circuit_state.lock().unwrap();
        let mut metrics = self.metrics.lock().unwrap();

        // Update basic metrics
        metrics.total_extractions += 1;
        if success {
            metrics.successful_extractions += 1;
        } else {
            metrics.failed_extractions += 1;
        }

        // Update average processing time
        let new_time = duration.as_millis() as f64;
        metrics.avg_processing_time_ms = if metrics.total_extractions == 1 {
            new_time
        } else {
            (metrics.avg_processing_time_ms + new_time) / 2.0
        };

        // Update circuit breaker state
        let new_state = match &*state {
            CircuitBreakerState::Closed { failure_count, success_count, .. } => {
                let new_failure_count = if success { 0 } else { failure_count + 1 };
                let new_success_count = if success { success_count + 1 } else { *success_count };
                let total_requests = new_failure_count + new_success_count;

                if total_requests >= 10 {
                    let failure_rate = (new_failure_count as f64 / total_requests as f64) * 100.0;
                    if failure_rate >= self.config.circuit_breaker_failure_threshold {
                        metrics.circuit_breaker_trips += 1;
                        warn!(
                            failure_rate = failure_rate,
                            threshold = self.config.circuit_breaker_failure_threshold,
                            "Circuit breaker opened due to high failure rate"
                        );
                        CircuitBreakerState::Open {
                            opened_at: Instant::now(),
                            failure_count: new_failure_count,
                        }
                    } else {
                        CircuitBreakerState::Closed {
                            failure_count: new_failure_count,
                            success_count: new_success_count,
                            last_failure: if success { None } else { Some(Instant::now()) },
                        }
                    }
                } else {
                    CircuitBreakerState::Closed {
                        failure_count: new_failure_count,
                        success_count: new_success_count,
                        last_failure: if success { None } else { Some(Instant::now()) },
                    }
                }
            }
            CircuitBreakerState::Open { opened_at, failure_count } => {
                if opened_at.elapsed() >= self.config.circuit_breaker_timeout {
                    info!("Circuit breaker transitioning to half-open");
                    CircuitBreakerState::HalfOpen {
                        test_requests: 0,
                        start_time: Instant::now(),
                    }
                } else {
                    CircuitBreakerState::Open {
                        opened_at: *opened_at,
                        failure_count: *failure_count,
                    }
                }
            }
            CircuitBreakerState::HalfOpen { test_requests, start_time } => {
                if success {
                    info!("Circuit breaker closing after successful test request");
                    CircuitBreakerState::Closed {
                        failure_count: 0,
                        success_count: 1,
                        last_failure: None,
                    }
                } else if *test_requests >= 3 {
                    warn!("Circuit breaker reopening after failed test requests");
                    CircuitBreakerState::Open {
                        opened_at: Instant::now(),
                        failure_count: 1,
                    }
                } else {
                    CircuitBreakerState::HalfOpen {
                        test_requests: test_requests + 1,
                        start_time: *start_time,
                    }
                }
            }
        };

        *state = new_state;
    }

    /// Record timeout occurrence
    fn record_timeout(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.failed_extractions += 1;
        metrics.total_extractions += 1;
    }

    /// Record epoch timeout
    fn record_epoch_timeout(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.epoch_timeouts += 1;
    }

    /// Update semaphore wait time metric
    fn update_semaphore_wait_time(&self, wait_time: Duration) {
        let mut metrics = self.metrics.lock().unwrap();
        let wait_ms = wait_time.as_millis() as f64;
        metrics.semaphore_wait_time_ms = if metrics.total_extractions == 1 {
            wait_ms
        } else {
            (metrics.semaphore_wait_time_ms + wait_ms) / 2.0
        };
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Get pool status for health checks
    pub fn get_pool_status(&self) -> (usize, usize, usize) {
        let available = self.available_instances.lock().unwrap().len();
        let max_size = self.config.max_pool_size;
        let active = max_size - available;
        (available, active, max_size)
    }
}

/// Configuration for RIPTIDE_WASM_INSTANCES_PER_WORKER environment variable
pub fn get_instances_per_worker() -> usize {
    env::var("RIPTIDE_WASM_INSTANCES_PER_WORKER")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8)
}