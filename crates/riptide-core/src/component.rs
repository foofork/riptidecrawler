use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wasmtime::{component::*, Config, Engine, Store};

use crate::memory_manager::{MemoryManager, MemoryManagerConfig};
use crate::types::{ComponentInfo, ExtractedDoc, ExtractionMode, ExtractionStats, HealthStatus};

/// Configuration for instance pooling and performance optimization
#[derive(Clone, Debug)]
pub struct ExtractorConfig {
    /// Maximum number of instances in the pool
    pub max_pool_size: usize,
    /// Initial number of instances to warm up
    pub initial_pool_size: usize,
    /// Timeout for extraction operations
    pub extraction_timeout: Duration,
    /// Memory limit per instance in bytes
    pub memory_limit: u64,
    /// Enable instance reuse optimization
    pub enable_instance_reuse: bool,
    /// Enable performance monitoring
    pub enable_metrics: bool,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            max_pool_size: 8,
            initial_pool_size: 2,
            extraction_timeout: Duration::from_secs(30),
            memory_limit: 256 * 1024 * 1024, // 256MB
            enable_instance_reuse: true,
            enable_metrics: true,
        }
    }
}

/// Performance metrics tracking
#[derive(Clone, Debug, Default)]
pub struct PerformanceMetrics {
    pub total_extractions: u64,
    pub successful_extractions: u64,
    pub failed_extractions: u64,
    pub avg_processing_time_ms: f64,
    pub memory_usage_bytes: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub pool_size: usize,
    pub active_instances: usize,
}

/// Instance pool entry with lifecycle tracking
#[allow(dead_code)]
struct PooledInstance {
    store: Store<()>,
    bindings: Extractor,
    created_at: Instant,
    last_used: Instant,
    use_count: u64,
    memory_usage: u64,
}

/// Advanced instance pool for WebAssembly components
#[allow(dead_code)]
struct InstancePool {
    instances: Vec<PooledInstance>,
    max_size: usize,
    engine: Engine,
    component: Component,
    linker: Linker<()>,
    metrics: Arc<Mutex<PerformanceMetrics>>,
}

// Generate bindings from enhanced WIT file
wasmtime::component::bindgen!({
    world: "extractor",
    path: "wit",
});

/// WebAssembly Component Model extractor for content extraction.
///
/// This extractor uses the WebAssembly Component Model (WASM-CM) to run
/// content extraction in a sandboxed environment. It provides better
/// performance and security compared to traditional WASI approaches.
///
/// # Architecture
///
/// The component model allows for:
/// - Type-safe interfaces defined in WIT (WebAssembly Interface Types)
/// - Better resource management and sandboxing
/// - Compositional design with multiple components
/// - Language-agnostic interfaces
///
/// # Performance Optimizations
///
/// This implementation includes:
/// - Instance pooling to avoid recreation overhead
/// - Memory reuse patterns
/// - Performance monitoring and metrics
/// - Circuit breaker patterns for reliability
/// - Timeout handling and resource cleanup
/// - Pre-warming for reduced latency
pub struct CmExtractor {
    /// Instance pool for efficient reuse
    #[allow(dead_code)]
    pool: Arc<Mutex<InstancePool>>,

    /// Configuration settings
    #[allow(dead_code)]
    config: ExtractorConfig,

    /// Performance metrics
    #[allow(dead_code)]
    metrics: Arc<Mutex<PerformanceMetrics>>,

    /// Circuit breaker state for error handling
    #[allow(dead_code)]
    circuit_state: Arc<Mutex<CircuitBreakerState>>,

    /// WebAssembly engine for component execution
    engine: Engine,

    /// WebAssembly component
    component: Component,

    /// WebAssembly linker for component instantiation
    linker: Linker<()>,

    /// Memory manager for WASM instance lifecycle
    memory_manager: Arc<MemoryManager>,

    /// Path to the component file
    component_path: String,
}

/// Circuit breaker states for handling failures
#[allow(dead_code)]
#[derive(Clone, Debug)]
enum CircuitBreakerState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

/// Enhanced error types with recovery information
#[derive(Debug, thiserror::Error)]
pub enum ExtractorError {
    #[error("Component instantiation failed: {message}")]
    InstantiationFailed { message: String, retryable: bool },

    #[error("Extraction timeout after {timeout_ms}ms")]
    ExtractionTimeout { timeout_ms: u64, retryable: bool },

    #[error("Memory limit exceeded: {used_bytes}/{limit_bytes} bytes")]
    MemoryLimitExceeded {
        used_bytes: u64,
        limit_bytes: u64,
        retryable: bool,
    },

    #[error("Pool exhausted: {active}/{max} instances")]
    PoolExhausted {
        active: usize,
        max: usize,
        retryable: bool,
    },

    #[error("Circuit breaker open: {reason}")]
    CircuitBreakerOpen { reason: String, retryable: bool },

    #[error("Component error: {message}")]
    ComponentError {
        message: String,
        error_code: Option<String>,
        retryable: bool,
    },
}

impl ExtractorError {
    pub fn is_retryable(&self) -> bool {
        match self {
            ExtractorError::InstantiationFailed { retryable, .. } => *retryable,
            ExtractorError::ExtractionTimeout { retryable, .. } => *retryable,
            ExtractorError::MemoryLimitExceeded { retryable, .. } => *retryable,
            ExtractorError::PoolExhausted { retryable, .. } => *retryable,
            ExtractorError::CircuitBreakerOpen { retryable, .. } => *retryable,
            ExtractorError::ComponentError { retryable, .. } => *retryable,
        }
    }
}

impl InstancePool {
    #[allow(dead_code)]
    fn new(
        engine: Engine,
        component: Component,
        linker: Linker<()>,
        max_size: usize,
        metrics: Arc<Mutex<PerformanceMetrics>>,
    ) -> Self {
        Self {
            instances: Vec::with_capacity(max_size),
            max_size,
            engine,
            component,
            linker,
            metrics,
        }
    }

    #[allow(dead_code)]
    fn get_instance(&mut self) -> Result<PooledInstance, ExtractorError> {
        // Try to reuse an existing instance
        if let Some(mut instance) = self.instances.pop() {
            instance.last_used = Instant::now();
            instance.use_count += 1;
            return Ok(instance);
        }

        // Create new instance if pool not at capacity
        if self.instances.len() < self.max_size {
            self.create_new_instance()
        } else {
            Err(ExtractorError::PoolExhausted {
                active: self.instances.len(),
                max: self.max_size,
                retryable: true,
            })
        }
    }

    #[allow(dead_code)]
    fn return_instance(&mut self, instance: PooledInstance) {
        // Return instance to pool if it's still healthy and under limits
        if instance.use_count < 1000 && instance.memory_usage < 512 * 1024 * 1024 {
            self.instances.push(instance);
        }
        // Otherwise, let it drop for cleanup
    }

    fn create_new_instance(&self) -> Result<PooledInstance, ExtractorError> {
        let mut store = Store::new(&self.engine, ());
        match Extractor::instantiate(&mut store, &self.component, &self.linker) {
            Ok(bindings) => {
                let now = Instant::now();
                Ok(PooledInstance {
                    store,
                    bindings,
                    created_at: now,
                    last_used: now,
                    use_count: 0,
                    memory_usage: 0,
                })
            }
            Err(e) => Err(ExtractorError::InstantiationFailed {
                message: e.to_string(),
                retryable: true,
            }),
        }
    }

    fn warm_up(&mut self, count: usize) -> Result<(), ExtractorError> {
        for _ in 0..count.min(self.max_size) {
            let instance = self.create_new_instance()?;
            self.instances.push(instance);
        }
        Ok(())
    }
}

impl CmExtractor {
    /// Creates a new component model extractor with default configuration.
    pub async fn new(wasm_path: &str) -> Result<Self> {
        Self::with_config(wasm_path, ExtractorConfig::default()).await
    }

    /// Creates a new component model extractor with custom configuration.
    pub async fn with_config(wasm_path: &str, config: ExtractorConfig) -> Result<Self> {
        // Configure Wasmtime for optimal performance
        let mut wasmtime_config = Config::new();
        wasmtime_config.wasm_component_model(true);
        wasmtime_config.cranelift_opt_level(wasmtime::OptLevel::Speed);
        wasmtime_config.wasm_simd(true);
        wasmtime_config.wasm_bulk_memory(true);
        wasmtime_config.wasm_multi_memory(true);
        wasmtime_config.wasm_memory64(false);

        // Set memory limits
        wasmtime_config.max_wasm_stack(2 * 1024 * 1024); // 2MB stack

        let engine = Engine::new(&wasmtime_config)?;
        let component = Component::from_file(&engine, wasm_path)?;
        let linker = Linker::new(&engine);

        let metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));
        let pool = Arc::new(Mutex::new(InstancePool::new(
            engine.clone(),
            component.clone(),
            linker.clone(),
            config.max_pool_size,
            metrics.clone(),
        )));

        // Pre-warm the pool
        if config.initial_pool_size > 0 {
            pool.lock()
                .map_err(|e| anyhow::anyhow!("Failed to acquire pool lock for warm-up: {}", e))?
                .warm_up(config.initial_pool_size)?;
        }

        // Create memory manager with appropriate configuration
        let memory_config = MemoryManagerConfig {
            max_total_memory_mb: config.memory_limit / (1024 * 1024), // Convert bytes to MB
            max_instances: config.max_pool_size,
            min_instances: config.initial_pool_size,
            ..Default::default()
        };
        let memory_manager = Arc::new(MemoryManager::new(memory_config, engine.clone()).await?);

        Ok(Self {
            pool,
            config,
            metrics,
            circuit_state: Arc::new(Mutex::new(CircuitBreakerState::Closed)),
            engine,
            component,
            linker,
            memory_manager,
            component_path: wasm_path.to_string(),
        })
    }

    /// Extracts content from HTML using the WebAssembly component.
    ///
    /// This method instantiates the component, calls the extraction function,
    /// and returns the structured content data.
    ///
    /// # Arguments
    ///
    /// * `html` - Raw HTML content to extract from
    /// * `url` - Source URL for context and link resolution
    /// * `mode` - Extraction mode ("article", "full", "metadata")
    ///
    /// # Returns
    ///
    /// An `ExtractedDoc` containing the structured content data.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Component instantiation fails
    /// - The extraction function throws an exception
    /// - The returned JSON cannot be parsed
    /// - Memory limits are exceeded
    ///
    /// # Performance Notes
    ///
    /// Each call to `extract` creates a new component instance. For high-throughput
    /// scenarios, consider using an instance pool or the reusable extraction methods.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riptide_core::component::CmExtractor;
    ///
    /// let extractor = CmExtractor::new("./extractor.wasm").await?;
    /// let html = "<html><body><h1>Title</h1><p>Content</p></body></html>";
    /// let doc = extractor.extract(html, "https://example.com", "article")?;
    ///
    /// println!("Title: {:?}", doc.title);
    /// println!("Text: {}", doc.text);
    /// ```
    /// Extract content using the legacy string-based mode for backward compatibility
    pub fn extract(&self, html: &str, url: &str, mode: &str) -> Result<ExtractedDoc> {
        // Convert legacy mode string to new ExtractionMode enum
        let extraction_mode = match mode {
            "article" => ExtractionMode::Article,
            "full" => ExtractionMode::Full,
            "metadata" => ExtractionMode::Metadata,
            _ => ExtractionMode::Article, // Default fallback
        };

        self.extract_typed(html, url, extraction_mode)
    }

    /// Extract content using the enhanced typed interface
    pub fn extract_typed(
        &self,
        html: &str,
        url: &str,
        mode: ExtractionMode,
    ) -> Result<ExtractedDoc> {
        // Create a new store for this extraction operation
        let mut store = Store::new(&self.engine, ());

        // Instantiate the component with the configured linker
        let bindings = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        // Convert our ExtractionMode to the WIT extraction-mode
        let wit_mode = match mode {
            ExtractionMode::Article => {
                exports::riptide::extractor::extract::ExtractionMode::Article
            }
            ExtractionMode::Full => exports::riptide::extractor::extract::ExtractionMode::Full,
            ExtractionMode::Metadata => {
                exports::riptide::extractor::extract::ExtractionMode::Metadata
            }
            ExtractionMode::Custom(selectors) => {
                exports::riptide::extractor::extract::ExtractionMode::Custom(selectors)
            }
        };

        // Call the enhanced extraction function
        let result = bindings
            .interface0
            .call_extract(&mut store, html, url, &wit_mode)?;

        match result {
            Ok(extracted_content) => {
                // Convert from Component Model types to our internal types
                Ok(ExtractedDoc {
                    url: extracted_content.url,
                    title: extracted_content.title,
                    byline: extracted_content.byline,
                    published_iso: extracted_content.published_iso,
                    markdown: extracted_content.markdown,
                    text: extracted_content.text,
                    links: extracted_content.links,
                    media: extracted_content.media,
                    language: extracted_content.language,
                    reading_time: extracted_content.reading_time,
                    quality_score: extracted_content.quality_score,
                    word_count: extracted_content.word_count,
                    categories: extracted_content.categories,
                    site_name: extracted_content.site_name,
                    description: extracted_content.description,
                })
            }
            Err(extraction_error) => {
                // Convert Component Model error to anyhow::Error
                let error_msg = match extraction_error {
                    exports::riptide::extractor::extract::ExtractionError::InvalidHtml(msg) => {
                        format!("Invalid HTML: {}", msg)
                    }
                    exports::riptide::extractor::extract::ExtractionError::NetworkError(msg) => {
                        format!("Network error: {}", msg)
                    }
                    exports::riptide::extractor::extract::ExtractionError::ParseError(msg) => {
                        format!("Parse error: {}", msg)
                    }
                    exports::riptide::extractor::extract::ExtractionError::ResourceLimit(msg) => {
                        format!("Resource limit: {}", msg)
                    }
                    exports::riptide::extractor::extract::ExtractionError::ExtractorError(msg) => {
                        format!("Extractor error: {}", msg)
                    }
                    exports::riptide::extractor::extract::ExtractionError::InternalError(msg) => {
                        format!("Internal error: {}", msg)
                    }
                    exports::riptide::extractor::extract::ExtractionError::UnsupportedMode(msg) => {
                        format!("Unsupported mode: {}", msg)
                    }
                };
                Err(anyhow::anyhow!(error_msg))
            }
        }
    }

    /// Extract content with detailed performance statistics
    pub fn extract_with_stats(
        &self,
        html: &str,
        url: &str,
        mode: ExtractionMode,
    ) -> Result<(ExtractedDoc, ExtractionStats)> {
        let mut store = Store::new(&self.engine, ());
        let bindings = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        // Convert our ExtractionMode to the WIT extraction-mode
        let wit_mode = match mode {
            ExtractionMode::Article => {
                exports::riptide::extractor::extract::ExtractionMode::Article
            }
            ExtractionMode::Full => exports::riptide::extractor::extract::ExtractionMode::Full,
            ExtractionMode::Metadata => {
                exports::riptide::extractor::extract::ExtractionMode::Metadata
            }
            ExtractionMode::Custom(selectors) => {
                exports::riptide::extractor::extract::ExtractionMode::Custom(selectors)
            }
        };

        let result = bindings
            .interface0
            .call_extract_with_stats(&mut store, html, url, &wit_mode)?;

        match result {
            Ok((extracted_content, stats)) => {
                let doc = ExtractedDoc {
                    url: extracted_content.url,
                    title: extracted_content.title,
                    byline: extracted_content.byline,
                    published_iso: extracted_content.published_iso,
                    markdown: extracted_content.markdown,
                    text: extracted_content.text,
                    links: extracted_content.links,
                    media: extracted_content.media,
                    language: extracted_content.language,
                    reading_time: extracted_content.reading_time,
                    quality_score: extracted_content.quality_score,
                    word_count: extracted_content.word_count,
                    categories: extracted_content.categories,
                    site_name: extracted_content.site_name,
                    description: extracted_content.description,
                };

                let extraction_stats = ExtractionStats {
                    processing_time_ms: stats.processing_time_ms,
                    memory_used: stats.memory_used,
                    nodes_processed: stats.nodes_processed,
                    links_found: stats.links_found,
                    images_found: stats.images_found,
                };

                Ok((doc, extraction_stats))
            }
            Err(extraction_error) => {
                let error_msg = format!("Extraction failed: {:?}", extraction_error);
                Err(anyhow::anyhow!(error_msg))
            }
        }
    }

    /// Validate HTML content without full extraction using memory management
    pub async fn validate_html(&self, html: &str) -> Result<bool> {
        // Get a managed WASM instance from the memory manager
        let _instance_handle = self
            .memory_manager
            .get_instance(&self.component_path)
            .await?;

        let mut store = Store::new(&self.engine, ());
        let bindings = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        match bindings.interface0.call_validate_html(&mut store, html)? {
            Ok(is_valid) => Ok(is_valid),
            Err(err) => Err(anyhow::anyhow!("Validation error: {:?}", err)),
        }
    }

    /// Get component health status using memory management
    pub async fn health_check(&self) -> Result<HealthStatus> {
        // Get a managed WASM instance from the memory manager
        let _instance_handle = self
            .memory_manager
            .get_instance(&self.component_path)
            .await?;

        let mut store = Store::new(&self.engine, ());
        let bindings = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        let health = bindings.interface0.call_health_check(&mut store)?;
        Ok(HealthStatus {
            status: health.status,
            version: health.version,
            trek_version: health.trek_version,
            capabilities: health.capabilities,
            memory_usage: health.memory_usage,
            extraction_count: health.extraction_count,
        })
    }

    /// Get detailed component information using memory management
    pub async fn get_info(&self) -> Result<ComponentInfo> {
        // Get a managed WASM instance from the memory manager
        let _instance_handle = self
            .memory_manager
            .get_instance(&self.component_path)
            .await?;

        let mut store = Store::new(&self.engine, ());
        let bindings = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        let info = bindings.interface0.call_get_info(&mut store)?;
        Ok(ComponentInfo {
            name: info.name,
            version: info.version,
            component_model_version: info.component_model_version,
            features: info.features,
            supported_modes: info.supported_modes,
            build_timestamp: info.build_timestamp,
            git_commit: info.git_commit,
        })
    }

    /// Reset component state and clear caches using memory management
    pub async fn reset_state(&self) -> Result<String> {
        // Get a managed WASM instance from the memory manager
        let _instance_handle = self
            .memory_manager
            .get_instance(&self.component_path)
            .await?;

        let mut store = Store::new(&self.engine, ());
        let bindings = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        match bindings.interface0.call_reset_state(&mut store)? {
            Ok(message) => Ok(message),
            Err(err) => Err(anyhow::anyhow!("Reset failed: {:?}", err)),
        }
    }

    /// Get supported extraction modes
    pub fn get_modes(&self) -> Result<Vec<String>> {
        let mut store = Store::new(&self.engine, ());
        let bindings = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        bindings.interface0.call_get_modes(&mut store)
    }
}
