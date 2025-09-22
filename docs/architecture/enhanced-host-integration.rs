// Enhanced host-side integration for WASM Component Model
// This file shows advanced wasmtime integration with performance optimizations

use anyhow::{Context, Result};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, Semaphore};
use wasmtime::{
    component::*,
    Config, Engine, Store, AsContextMut, Caller, Linker,
    ResourceLimiter, StoreLimitsBuilder,
};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

use crate::types::{ExtractedDoc, ExtractionMode, ExtractionStats};

// Generate Component Model bindings
wasmtime::component::bindgen!({
    world: "extractor",
    path: "wit",
    async: true,
    with: {
        "riptide:extractor/extractor": generate,
    },
});

/// Configuration for the WASM Component Model extractor
#[derive(Clone, Debug)]
pub struct ExtractorConfig {
    /// Maximum number of concurrent instances
    pub max_instances: usize,

    /// Timeout for individual extractions
    pub extraction_timeout: Duration,

    /// Memory limit per instance (bytes)
    pub memory_limit: usize,

    /// Maximum fuel per extraction
    pub fuel_limit: u64,

    /// Instance idle timeout before cleanup
    pub instance_idle_timeout: Duration,

    /// Enable SIMD optimizations
    pub enable_simd: bool,

    /// Cranelift optimization level
    pub optimization_level: wasmtime::OptLevel,

    /// Enable epoch interruption for timeouts
    pub enable_epoch_interruption: bool,

    /// Pool warmup size (instances to pre-create)
    pub warmup_instances: usize,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            max_instances: 16,
            extraction_timeout: Duration::from_secs(30),
            memory_limit: 64 * 1024 * 1024, // 64MB
            fuel_limit: 10_000_000,
            instance_idle_timeout: Duration::from_secs(300), // 5 minutes
            enable_simd: true,
            optimization_level: wasmtime::OptLevel::Speed,
            enable_epoch_interruption: true,
            warmup_instances: 4,
        }
    }
}

/// Per-instance execution context
#[derive(Debug)]
pub struct ExtractorContext {
    pub wasi: WasiCtx,
    pub memory_used: usize,
    pub extractions_performed: u64,
    pub created_at: Instant,
    pub last_used: Instant,
}

impl Default for ExtractorContext {
    fn default() -> Self {
        Self {
            wasi: WasiCtxBuilder::new()
                .inherit_stdio()
                .inherit_env()
                .build(),
            memory_used: 0,
            extractions_performed: 0,
            created_at: Instant::now(),
            last_used: Instant::now(),
        }
    }
}

/// Resource limiter for WASM instances
pub struct ExtractorResourceLimiter {
    memory_limit: usize,
}

impl ExtractorResourceLimiter {
    pub fn new(memory_limit: usize) -> Self {
        Self { memory_limit }
    }
}

impl ResourceLimiter for ExtractorResourceLimiter {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        Ok(desired <= self.memory_limit)
    }

    fn table_growing(
        &mut self,
        _current: u32,
        desired: u32,
        _maximum: Option<u32>,
    ) -> anyhow::Result<bool> {
        // Allow reasonable table growth
        Ok(desired <= 10_000)
    }

    fn memory_grow_failed(&mut self, _error: &anyhow::Error) -> anyhow::Result<()> {
        tracing::warn!("Memory allocation failed - at limit");
        Ok(())
    }
}

/// Pooled WASM component instance
pub struct PooledInstance {
    store: Store<ExtractorContext>,
    instance: Extractor,
    last_used: Instant,
    id: String,
}

impl PooledInstance {
    pub fn is_stale(&self, idle_timeout: Duration) -> bool {
        self.last_used.elapsed() > idle_timeout
    }

    pub async fn reset(&mut self) -> Result<()> {
        // Reset component state if needed
        match self.instance.call_reset_state(&mut self.store).await {
            Ok(_) => {
                self.last_used = Instant::now();
                self.store.data_mut().last_used = Instant::now();
                Ok(())
            }
            Err(e) => {
                tracing::warn!("Failed to reset instance state: {}", e);
                Err(e.into())
            }
        }
    }
}

/// High-performance Component Model extractor with advanced pooling
pub struct CmExtractor {
    engine: Arc<Engine>,
    component: Arc<Component>,
    linker: Arc<Linker<ExtractorContext>>,
    instance_pool: Arc<Mutex<VecDeque<PooledInstance>>>,
    pool_semaphore: Arc<Semaphore>,
    config: ExtractorConfig,
    metrics: Arc<RwLock<ExtractorMetrics>>,
}

/// Performance and usage metrics
#[derive(Default, Debug)]
pub struct ExtractorMetrics {
    pub total_extractions: u64,
    pub successful_extractions: u64,
    pub failed_extractions: u64,
    pub total_processing_time: Duration,
    pub average_processing_time: Duration,
    pub peak_memory_usage: usize,
    pub instances_created: u64,
    pub instances_destroyed: u64,
    pub pool_hits: u64,
    pub pool_misses: u64,
}

impl CmExtractor {
    /// Create a new extractor with advanced configuration
    pub async fn new(wasm_path: &str, config: ExtractorConfig) -> Result<Self> {
        let engine = Arc::new(Self::create_engine(&config)?);
        let component = Arc::new(Component::from_file(&engine, wasm_path)?);
        let linker = Arc::new(Self::create_linker(&engine).await?);

        let instance_pool = Arc::new(Mutex::new(VecDeque::new()));
        let pool_semaphore = Arc::new(Semaphore::new(config.max_instances));
        let metrics = Arc::new(RwLock::new(ExtractorMetrics::default()));

        let extractor = Self {
            engine,
            component,
            linker,
            instance_pool,
            pool_semaphore,
            config,
            metrics,
        };

        // Warm up the pool
        extractor.warmup_pool().await?;

        Ok(extractor)
    }

    /// Create optimized Wasmtime engine
    fn create_engine(config: &ExtractorConfig) -> Result<Engine> {
        let mut wasmtime_config = Config::new();

        // Component Model support
        wasmtime_config.wasm_component_model(true);
        wasmtime_config.async_support(true);

        // Performance optimizations
        wasmtime_config.cranelift_opt_level(config.optimization_level);
        wasmtime_config.wasm_simd(config.enable_simd);
        wasmtime_config.wasm_bulk_memory(true);
        wasmtime_config.wasm_multi_memory(true);
        wasmtime_config.wasm_memory64(false); // Usually not needed
        wasmtime_config.wasm_threads(false); // Disable for security

        // Resource management
        wasmtime_config.consume_fuel(true);
        if config.enable_epoch_interruption {
            wasmtime_config.epoch_interruption(true);
        }

        // Security features
        wasmtime_config.cranelift_debug_verifier(false); // Disable in production
        wasmtime_config.generate_address_map(false);

        // Memory configuration
        wasmtime_config.dynamic_memory_guard_size(0x10000); // 64KB guard
        wasmtime_config.static_memory_guard_size(0x10000);
        wasmtime_config.static_memory_maximum_size(config.memory_limit as u64);

        // Pooling allocator for better performance
        wasmtime_config.allocation_strategy(wasmtime::InstanceAllocationStrategy::Pooling {
            strategy: wasmtime::PoolingAllocationStrategy::default(),
            module_limits: wasmtime::ModuleLimits {
                imported_functions: 100,
                imported_tables: 10,
                imported_memories: 10,
                imported_globals: 100,
                types: 1000,
                functions: 10000,
                tables: 10,
                memories: 10,
                globals: 100,
                table_elements: 10000,
                memory_pages: (config.memory_limit / 65536) as u32,
            },
            instance_limits: wasmtime::InstanceLimits {
                count: config.max_instances as u32,
                host_stacks: config.max_instances as u32,
            },
        });

        Engine::new(&wasmtime_config).context("Failed to create Wasmtime engine")
    }

    /// Create component linker with host functions
    async fn create_linker(engine: &Engine) -> Result<Linker<ExtractorContext>> {
        let mut linker = Linker::new(engine);

        // Add WASI support
        wasmtime_wasi::add_to_linker_async(&mut linker)
            .context("Failed to add WASI to linker")?;

        // Add custom host functions if needed
        // linker.func_wrap_async("env", "host_log", |mut caller: Caller<'_, ExtractorContext>, ptr: u32, len: u32| {
        //     Box::new(async move {
        //         // Custom logging function
        //         Ok(())
        //     })
        // })?;

        Ok(linker)
    }

    /// Pre-create instances to warm up the pool
    async fn warmup_pool(&self) -> Result<()> {
        for i in 0..self.config.warmup_instances {
            match self.create_instance().await {
                Ok(instance) => {
                    let mut pool = self.instance_pool.lock().await;
                    pool.push_back(instance);
                    tracing::debug!("Warmed up instance {}/{}", i + 1, self.config.warmup_instances);
                }
                Err(e) => {
                    tracing::warn!("Failed to create warmup instance {}: {}", i, e);
                }
            }
        }
        Ok(())
    }

    /// Extract content with full performance monitoring
    pub async fn extract_with_metrics(
        &self,
        html: &str,
        url: &str,
        mode: ExtractionMode,
    ) -> Result<(ExtractedDoc, ExtractionStats)> {
        let start_time = Instant::now();
        let _permit = self.pool_semaphore.acquire().await?;

        // Get instance from pool
        let mut instance = self.get_or_create_instance().await?;

        // Set up resource limits for this extraction
        {
            let limiter = ExtractorResourceLimiter::new(self.config.memory_limit);
            instance.store.limiter(|_| limiter);
            instance.store.add_fuel(self.config.fuel_limit)?;
        }

        // Perform extraction with timeout
        let extraction_result = tokio::time::timeout(
            self.config.extraction_timeout,
            self.perform_extraction(&mut instance, html, url, mode),
        ).await;

        let processing_time = start_time.elapsed();

        // Update metrics
        self.update_metrics(processing_time, extraction_result.is_ok()).await;

        // Return instance to pool
        self.return_instance(instance).await;

        match extraction_result {
            Ok(Ok((content, stats))) => Ok((content, stats)),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow::anyhow!("Extraction timed out after {:?}", self.config.extraction_timeout)),
        }
    }

    /// Simplified extraction interface
    pub async fn extract(
        &self,
        html: &str,
        url: &str,
        mode: ExtractionMode,
    ) -> Result<ExtractedDoc> {
        let (content, _stats) = self.extract_with_metrics(html, url, mode).await?;
        Ok(content)
    }

    /// Get instance from pool or create new one
    async fn get_or_create_instance(&self) -> Result<PooledInstance> {
        let mut pool = self.instance_pool.lock().await;

        // Try to reuse existing instance
        while let Some(mut instance) = pool.pop_front() {
            if !instance.is_stale(self.config.instance_idle_timeout) {
                // Update metrics
                {
                    let mut metrics = self.metrics.write().await;
                    metrics.pool_hits += 1;
                }
                return Ok(instance);
            } else {
                // Instance is stale, destroy it
                tracing::debug!("Destroying stale instance: {}", instance.id);
                let mut metrics = self.metrics.write().await;
                metrics.instances_destroyed += 1;
            }
        }

        // No suitable instance found, create new one
        {
            let mut metrics = self.metrics.write().await;
            metrics.pool_misses += 1;
        }

        self.create_instance().await
    }

    /// Create new component instance
    async fn create_instance(&self) -> Result<PooledInstance> {
        let mut store = Store::new(&self.engine, ExtractorContext::default());

        // Configure store limits
        let limits = StoreLimitsBuilder::new()
            .memory_size(self.config.memory_limit)
            .table_elements(10000)
            .instances(1)
            .tables(10)
            .memories(10)
            .build();
        store.limiter(|_| limits);

        // Enable epoch interruption if configured
        if self.config.enable_epoch_interruption {
            store.set_epoch_deadline(1);
        }

        // Instantiate component
        let instance = Extractor::instantiate(&mut store, &self.component, &self.linker)
            .await
            .context("Failed to instantiate WASM component")?;

        let id = format!("instance-{}", uuid::Uuid::new_v4());
        tracing::debug!("Created new instance: {}", id);

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.instances_created += 1;
        }

        Ok(PooledInstance {
            store,
            instance,
            last_used: Instant::now(),
            id,
        })
    }

    /// Perform the actual extraction
    async fn perform_extraction(
        &self,
        instance: &mut PooledInstance,
        html: &str,
        url: &str,
        mode: ExtractionMode,
    ) -> Result<(ExtractedDoc, ExtractionStats)> {
        // Convert mode to WIT format
        let wit_mode = match mode {
            ExtractionMode::Article => exports::riptide::extractor::extractor::ExtractionMode::Article,
            ExtractionMode::Full => exports::riptide::extractor::extractor::ExtractionMode::Full,
            ExtractionMode::Metadata => exports::riptide::extractor::extractor::ExtractionMode::Metadata,
            ExtractionMode::Custom(selectors) => {
                exports::riptide::extractor::extractor::ExtractionMode::Custom(selectors)
            }
        };

        // Call the component function
        let result = instance
            .instance
            .call_extract_with_stats(&mut instance.store, html, url, &wit_mode)
            .await?;

        match result {
            Ok((wit_content, wit_stats)) => {
                // Convert from WIT types to Rust types
                let content = convert_wit_content_to_extracted_doc(wit_content);
                let stats = convert_wit_stats_to_extraction_stats(wit_stats);

                // Update instance metrics
                instance.last_used = Instant::now();
                instance.store.data_mut().extractions_performed += 1;
                instance.store.data_mut().last_used = Instant::now();

                Ok((content, stats))
            }
            Err(extraction_error) => {
                Err(anyhow::anyhow!("Extraction failed: {:?}", extraction_error))
            }
        }
    }

    /// Return instance to pool for reuse
    async fn return_instance(&self, instance: PooledInstance) {
        let mut pool = self.instance_pool.lock().await;

        // Only return if pool isn't full
        if pool.len() < self.config.max_instances {
            pool.push_back(instance);
        } else {
            // Pool is full, let instance be destroyed
            tracing::debug!("Pool full, destroying instance: {}", instance.id);
            let mut metrics = self.metrics.write().await;
            metrics.instances_destroyed += 1;
        }
    }

    /// Update performance metrics
    async fn update_metrics(&self, processing_time: Duration, success: bool) {
        let mut metrics = self.metrics.write().await;

        metrics.total_extractions += 1;
        if success {
            metrics.successful_extractions += 1;
        } else {
            metrics.failed_extractions += 1;
        }

        metrics.total_processing_time += processing_time;
        metrics.average_processing_time =
            metrics.total_processing_time / metrics.total_extractions as u32;
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> ExtractorMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Health check for the extractor
    pub async fn health_check(&self) -> Result<String> {
        // Try to get a temporary instance
        let _permit = self.pool_semaphore.try_acquire()
            .map_err(|_| anyhow::anyhow!("All instances busy"))?;

        let mut instance = self.get_or_create_instance().await?;

        let health_result = instance
            .instance
            .call_health_check(&mut instance.store)
            .await?;

        self.return_instance(instance).await;

        Ok(format!("Extractor healthy: {:?}", health_result))
    }

    /// Clean up stale instances and perform maintenance
    pub async fn maintenance(&self) -> Result<()> {
        let mut pool = self.instance_pool.lock().await;
        let mut removed_count = 0;

        // Remove stale instances
        pool.retain(|instance| {
            if instance.is_stale(self.config.instance_idle_timeout) {
                removed_count += 1;
                false
            } else {
                true
            }
        });

        if removed_count > 0 {
            tracing::info!("Cleaned up {} stale instances", removed_count);
            let mut metrics = self.metrics.write().await;
            metrics.instances_destroyed += removed_count;
        }

        Ok(())
    }
}

/// Convert WIT types to internal types (implement based on actual WIT definitions)
fn convert_wit_content_to_extracted_doc(
    wit_content: exports::riptide::extractor::extractor::ExtractedContent,
) -> ExtractedDoc {
    ExtractedDoc {
        url: wit_content.url,
        title: wit_content.title,
        byline: wit_content.byline,
        published_iso: wit_content.published_iso,
        markdown: wit_content.markdown,
        text: wit_content.text,
        links: wit_content.links,
        media: wit_content.media,
        // Add other fields as needed
    }
}

fn convert_wit_stats_to_extraction_stats(
    wit_stats: exports::riptide::extractor::extractor::ExtractionStats,
) -> ExtractionStats {
    ExtractionStats {
        processing_time_ms: wit_stats.processing_time_ms,
        memory_used: wit_stats.memory_used,
        nodes_processed: wit_stats.nodes_processed,
        links_found: wit_stats.links_found,
        images_found: wit_stats.images_found,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extractor_creation() {
        let config = ExtractorConfig::default();
        let wasm_path = "../../target/wasm32-wasip2/release/riptide_extractor_wasm.wasm";

        let result = CmExtractor::new(wasm_path, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pool_reuse() {
        let config = ExtractorConfig {
            warmup_instances: 2,
            ..Default::default()
        };
        let wasm_path = "../../target/wasm32-wasip2/release/riptide_extractor_wasm.wasm";

        let extractor = CmExtractor::new(wasm_path, config).await.unwrap();

        // Perform multiple extractions to test pool reuse
        let html = "<html><head><title>Test</title></head><body><p>Content</p></body></html>";

        for i in 0..5 {
            let result = extractor
                .extract(html, &format!("https://test{}.com", i), ExtractionMode::Article)
                .await;
            assert!(result.is_ok());
        }

        let metrics = extractor.get_metrics().await;
        assert!(metrics.pool_hits > 0);
    }
}