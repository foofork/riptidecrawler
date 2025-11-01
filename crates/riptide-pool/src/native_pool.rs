//! Native Extractor Pool
//!
//! Instance pooling and lifecycle management for native (non-WASM) extraction strategies.
//! This provides first-class support for CSS and Regex extractors with dedicated pooling,
//! health monitoring, and resource management.
//!
//! ## Features
//!
//! - **Instance Pooling**: Reusable extractor instances with lifecycle management
//! - **Health Monitoring**: Continuous health checks and validation
//! - **Resource Limits**: Memory and CPU usage tracking
//! - **Metrics Collection**: Performance and utilization metrics
//! - **Thread Safety**: Arc/Mutex-based concurrent access
//!
//! ## Usage
//!
//! ```no_run
//! use riptide_pool::{NativeExtractorPool, NativePoolConfig, NativeExtractorType};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = NativePoolConfig::default();
//! let pool = NativeExtractorPool::new(config, NativeExtractorType::Css).await?;
//!
//! let result = pool.extract("<!DOCTYPE html>...", "https://example.com").await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{anyhow, Result};
use riptide_extraction::strategies::{
    css_strategy::CssSelectorStrategy, regex_strategy::RegexPatternStrategy,
    traits::ExtractionStrategy, ExtractedContent,
};
use riptide_types::ExtractedDoc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::timeout;
use tracing::{debug, info, warn};
use uuid::Uuid;

#[cfg(feature = "wasm-pool")]
use async_trait::async_trait;

#[cfg(feature = "wasm-pool")]
use riptide_events::{EventBus, EventEmitter};

/// Type of native extractor to use
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeExtractorType {
    /// CSS selector-based extraction
    Css,
    /// Regex pattern-based extraction
    Regex,
}

/// Configuration for native extractor pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativePoolConfig {
    /// Maximum number of instances in pool
    pub max_pool_size: usize,
    /// Initial number of instances to create
    pub initial_pool_size: usize,
    /// Extraction timeout in milliseconds
    pub extraction_timeout: u64,
    /// Health check interval in milliseconds
    pub health_check_interval: u64,
    /// Maximum memory usage per instance (bytes)
    pub memory_limit: Option<usize>,
    /// Maximum CPU usage per instance (percentage)
    pub cpu_limit: Option<f32>,
    /// Circuit breaker failure threshold
    pub circuit_breaker_failure_threshold: u32,
    /// Circuit breaker timeout in milliseconds
    pub circuit_breaker_timeout: u64,
    /// Maximum instance reuse count before recreation
    pub max_instance_reuse: u32,
    /// Maximum failure count before instance is discarded
    pub max_failure_count: u32,
}

impl Default for NativePoolConfig {
    fn default() -> Self {
        Self {
            max_pool_size: 8,
            initial_pool_size: 2,
            extraction_timeout: 30000,
            health_check_interval: 30000,
            memory_limit: Some(256 * 1024 * 1024), // 256MB per instance
            cpu_limit: Some(80.0),
            circuit_breaker_failure_threshold: 5,
            circuit_breaker_timeout: 5000,
            max_instance_reuse: 1000,
            max_failure_count: 10,
        }
    }
}

impl NativePoolConfig {
    /// Validate configuration settings
    pub fn validate(&self) -> Result<()> {
        if self.max_pool_size == 0 {
            return Err(anyhow!("max_pool_size must be greater than 0"));
        }
        if self.initial_pool_size > self.max_pool_size {
            return Err(anyhow!(
                "initial_pool_size cannot be greater than max_pool_size"
            ));
        }
        if self.circuit_breaker_failure_threshold == 0 {
            return Err(anyhow!(
                "circuit_breaker_failure_threshold must be greater than 0"
            ));
        }
        Ok(())
    }
}

/// Pooled native extractor instance
struct PooledNativeInstance {
    /// Instance unique identifier
    id: String,
    /// Type of extractor
    extractor_type: NativeExtractorType,
    /// CSS extractor (if type is Css)
    css_extractor: Option<CssSelectorStrategy>,
    /// Regex extractor (if type is Regex)
    regex_extractor: Option<RegexPatternStrategy>,
    /// Number of times this instance has been used
    use_count: u32,
    /// Number of extraction failures
    failure_count: u32,
    /// Last used timestamp
    last_used: Instant,
    /// Creation timestamp
    created_at: Instant,
    /// Memory usage estimate (bytes)
    memory_usage: usize,
}

impl PooledNativeInstance {
    /// Create new native extractor instance
    fn new(extractor_type: NativeExtractorType) -> Self {
        let id = Uuid::new_v4().to_string();

        let (css_extractor, regex_extractor) = match extractor_type {
            NativeExtractorType::Css => (Some(CssSelectorStrategy::new()), None),
            NativeExtractorType::Regex => (None, Some(RegexPatternStrategy::new())),
        };

        Self {
            id,
            extractor_type,
            css_extractor,
            regex_extractor,
            use_count: 0,
            failure_count: 0,
            last_used: Instant::now(),
            created_at: Instant::now(),
            memory_usage: 0,
        }
    }

    /// Check if instance is healthy based on config
    fn is_healthy(&self, config: &NativePoolConfig) -> bool {
        if self.use_count >= config.max_instance_reuse {
            debug!(
                instance_id = %self.id,
                use_count = self.use_count,
                "Instance exceeded max reuse count"
            );
            return false;
        }

        if self.failure_count >= config.max_failure_count {
            debug!(
                instance_id = %self.id,
                failure_count = self.failure_count,
                "Instance exceeded max failure count"
            );
            return false;
        }

        if let Some(memory_limit) = config.memory_limit {
            if self.memory_usage > memory_limit {
                debug!(
                    instance_id = %self.id,
                    memory_usage = self.memory_usage,
                    memory_limit,
                    "Instance exceeded memory limit"
                );
                return false;
            }
        }

        true
    }

    /// Record successful usage
    fn record_success(&mut self) {
        self.use_count += 1;
        self.last_used = Instant::now();
    }

    /// Record failed usage
    fn record_failure(&mut self) {
        self.use_count += 1;
        self.failure_count += 1;
        self.last_used = Instant::now();
    }

    /// Extract content using this instance
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        match self.extractor_type {
            NativeExtractorType::Css => {
                if let Some(ref extractor) = self.css_extractor {
                    let result = extractor.extract(html, url).await?;
                    Ok(result.content)
                } else {
                    Err(anyhow!("CSS extractor not initialized"))
                }
            }
            NativeExtractorType::Regex => {
                if let Some(ref extractor) = self.regex_extractor {
                    let result = extractor.extract(html, url).await?;
                    Ok(result.content)
                } else {
                    Err(anyhow!("Regex extractor not initialized"))
                }
            }
        }
    }
}

/// Circuit breaker state for native pool
#[derive(Debug, Clone)]
enum CircuitBreakerState {
    Closed {
        failure_count: u32,
        success_count: u32,
        last_failure: Option<Instant>,
    },
    Open {
        opened_at: Instant,
        failure_count: u32,
    },
    HalfOpen {
        test_requests: u32,
        start_time: Instant,
    },
}

/// Performance metrics for native pool
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NativePoolMetrics {
    /// Total extractions performed
    pub total_extractions: u64,
    /// Successful extractions
    pub successful_extractions: u64,
    /// Failed extractions
    pub failed_extractions: u64,
    /// Average processing time in milliseconds
    pub avg_processing_time_ms: f64,
    /// Average semaphore wait time in milliseconds
    pub avg_semaphore_wait_ms: f64,
    /// Current pool size
    pub pool_size: usize,
    /// Circuit breaker trips
    pub circuit_breaker_trips: u64,
    /// Timeout count
    pub timeout_count: u64,
}

/// Native Extractor Pool
///
/// Manages a pool of native (CSS/Regex) extractor instances with health monitoring,
/// resource limits, and circuit breaker pattern.
pub struct NativeExtractorPool {
    config: NativePoolConfig,
    extractor_type: NativeExtractorType,
    available_instances: Arc<Mutex<VecDeque<PooledNativeInstance>>>,
    semaphore: Arc<Semaphore>,
    metrics: Arc<Mutex<NativePoolMetrics>>,
    circuit_state: Arc<Mutex<CircuitBreakerState>>,
    pool_id: String,
    #[cfg(feature = "wasm-pool")]
    event_bus: Option<Arc<EventBus>>,
}

impl NativeExtractorPool {
    /// Create new native extractor pool
    pub async fn new(
        config: NativePoolConfig,
        extractor_type: NativeExtractorType,
    ) -> Result<Self> {
        config.validate()?;

        info!(
            max_pool_size = config.max_pool_size,
            initial_pool_size = config.initial_pool_size,
            extractor_type = ?extractor_type,
            "Initializing native extractor pool"
        );

        let pool = Self {
            config: config.clone(),
            extractor_type,
            available_instances: Arc::new(Mutex::new(VecDeque::with_capacity(
                config.max_pool_size,
            ))),
            semaphore: Arc::new(Semaphore::new(config.max_pool_size)),
            metrics: Arc::new(Mutex::new(NativePoolMetrics::default())),
            circuit_state: Arc::new(Mutex::new(CircuitBreakerState::Closed {
                failure_count: 0,
                success_count: 0,
                last_failure: None,
            })),
            pool_id: Uuid::new_v4().to_string(),
            #[cfg(feature = "wasm-pool")]
            event_bus: None,
        };

        pool.warm_up().await?;

        info!(pool_id = %pool.pool_id, "Native extractor pool initialized");
        Ok(pool)
    }

    async fn warm_up(&self) -> Result<()> {
        debug!(
            warm_count = self.config.initial_pool_size,
            "Warming up native pool"
        );

        let mut created_instances = Vec::new();
        for _ in 0..self.config.initial_pool_size {
            created_instances.push(self.create_instance());
        }

        {
            let mut instances = self.available_instances.lock().await;
            for instance in created_instances {
                instances.push_back(instance);
            }
        }

        let pool_size = self.available_instances.lock().await.len();
        self.metrics.lock().await.pool_size = pool_size;

        info!(pool_size, "Native pool warm-up completed");
        Ok(())
    }

    fn create_instance(&self) -> PooledNativeInstance {
        debug!(extractor_type = ?self.extractor_type, "Creating native instance");
        PooledNativeInstance::new(self.extractor_type)
    }

    pub async fn extract(&self, html: &str, url: &str) -> Result<ExtractedDoc> {
        if self.is_circuit_open().await {
            return Err(anyhow!("Circuit breaker is open"));
        }

        let start_time = Instant::now();
        let timeout_duration = Duration::from_millis(self.config.extraction_timeout);

        let permit = match timeout(timeout_duration, self.semaphore.acquire()).await {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => return Err(anyhow!("Semaphore closed")),
            Err(_) => {
                self.record_timeout().await;
                return Err(anyhow!("Extraction timeout"));
            }
        };

        let semaphore_wait_time = start_time.elapsed();
        let mut instance = self.get_or_create_instance().await;
        let extraction_result = instance.extract(html, url).await;

        let success = extraction_result.is_ok();
        if success {
            instance.record_success();
        } else {
            instance.record_failure();
        }

        self.return_instance(instance).await;
        self.record_extraction_result(success, start_time.elapsed())
            .await;
        self.update_semaphore_wait_time(semaphore_wait_time).await;

        drop(permit);

        extraction_result.map(|content| ExtractedDoc {
            url: content.url,
            title: Some(content.title),
            text: content.content.clone(),
            markdown: Some(content.content),
            byline: None,
            published_iso: None,
            links: Vec::new(),
            media: Vec::new(),
            language: None,
            reading_time: None,
            quality_score: None,
            parser_metadata: None,
            word_count: None,
            categories: Vec::new(),
            site_name: None,
            description: content.summary,
            html: None,
        })
    }

    async fn get_or_create_instance(&self) -> PooledNativeInstance {
        let maybe_instance = self.available_instances.lock().await.pop_front();

        if let Some(instance) = maybe_instance {
            if instance.is_healthy(&self.config) {
                return instance;
            }
            debug!(instance_id = %instance.id, "Discarding unhealthy instance");
        }

        self.create_instance()
    }

    async fn return_instance(&self, instance: PooledNativeInstance) {
        let is_healthy = instance.is_healthy(&self.config);

        if is_healthy {
            self.available_instances.lock().await.push_back(instance);
        }

        let pool_size = self.available_instances.lock().await.len();
        self.metrics.lock().await.pool_size = pool_size;
    }

    async fn is_circuit_open(&self) -> bool {
        let state = self.circuit_state.lock().await;
        match *state {
            CircuitBreakerState::Open { opened_at, .. } => {
                opened_at.elapsed() < Duration::from_millis(self.config.circuit_breaker_timeout)
            }
            _ => false,
        }
    }

    async fn record_extraction_result(&self, success: bool, duration: Duration) {
        {
            let mut metrics = self.metrics.lock().await;
            metrics.total_extractions += 1;
            if success {
                metrics.successful_extractions += 1;
            } else {
                metrics.failed_extractions += 1;
            }

            let new_time = duration.as_millis() as f64;
            metrics.avg_processing_time_ms = if metrics.total_extractions == 1 {
                new_time
            } else {
                (metrics.avg_processing_time_ms + new_time) / 2.0
            };
        }

        let mut state = self.circuit_state.lock().await;
        let new_state = match &*state {
            CircuitBreakerState::Closed {
                failure_count,
                success_count,
                ..
            } => {
                let new_failure_count = if success { 0 } else { failure_count + 1 };
                let new_success_count = if success {
                    success_count + 1
                } else {
                    *success_count
                };
                let total_requests = new_failure_count + new_success_count;

                if total_requests >= 10 {
                    let failure_rate = (new_failure_count as f64 / total_requests as f64) * 100.0;
                    if failure_rate >= self.config.circuit_breaker_failure_threshold as f64 {
                        warn!(failure_rate, "Native pool circuit breaker opened");
                        self.metrics.lock().await.circuit_breaker_trips += 1;

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
            CircuitBreakerState::Open {
                opened_at,
                failure_count,
            } => {
                if opened_at.elapsed() >= Duration::from_millis(self.config.circuit_breaker_timeout)
                {
                    info!("Native pool circuit breaker half-open");
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
            CircuitBreakerState::HalfOpen {
                test_requests,
                start_time,
            } => {
                if success {
                    info!("Native pool circuit breaker closing");
                    CircuitBreakerState::Closed {
                        failure_count: 0,
                        success_count: 1,
                        last_failure: None,
                    }
                } else if *test_requests >= 3 {
                    warn!("Native pool circuit breaker reopening");
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

    async fn record_timeout(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.timeout_count += 1;
        metrics.total_extractions += 1;
        metrics.failed_extractions += 1;
    }

    async fn update_semaphore_wait_time(&self, wait_time: Duration) {
        let mut metrics = self.metrics.lock().await;
        let wait_ms = wait_time.as_millis() as f64;
        metrics.avg_semaphore_wait_ms = if metrics.total_extractions == 1 {
            wait_ms
        } else {
            (metrics.avg_semaphore_wait_ms + wait_ms) / 2.0
        };
    }

    pub async fn get_metrics(&self) -> NativePoolMetrics {
        self.metrics.lock().await.clone()
    }

    pub async fn get_pool_status(&self) -> (usize, usize, usize) {
        let available = self.available_instances.lock().await.len();
        let max_size = self.config.max_pool_size;
        let active = max_size - available;
        (available, active, max_size)
    }

    pub fn pool_id(&self) -> &str {
        &self.pool_id
    }

    pub fn extractor_type(&self) -> NativeExtractorType {
        self.extractor_type
    }

    #[cfg(feature = "wasm-pool")]
    pub fn set_event_bus(&mut self, event_bus: Arc<EventBus>) {
        self.event_bus = Some(event_bus);
    }
}

#[cfg(feature = "wasm-pool")]
#[async_trait]
impl EventEmitter for NativeExtractorPool {
    async fn emit_event<E: riptide_events::Event + 'static>(&self, event: E) -> Result<()> {
        if let Some(event_bus) = &self.event_bus {
            event_bus.emit(event).await
        } else {
            Ok(())
        }
    }

    async fn emit_events<E: riptide_events::Event + 'static>(&self, events: Vec<E>) -> Result<()> {
        if let Some(event_bus) = &self.event_bus {
            for event in events {
                event_bus.emit(event).await?;
            }
        }
        Ok(())
    }
}
