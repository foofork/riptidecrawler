//! WASM-based HTML extraction
//!
//! This module provides WASM component-based HTML extraction capabilities
//! moved from riptide-core to provide cleaner separation of concerns.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wasmtime::{component::*, Config, Engine, ResourceLimiter, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

// Import ExtractedDoc from riptide-types instead of duplicating
use riptide_types::ExtractedDoc;

// WIT bindings - Wasmtime 37 bindgen! macro
// Generate bindings in a module to avoid namespace pollution
mod wit_bindings {
    wasmtime::component::bindgen!({
        world: "extractor",
        path: "../../wasm/riptide-extractor-wasm/wit/extractor.wit",
    });
}

impl From<ExtractedDoc> for crate::ExtractedContent {
    fn from(doc: ExtractedDoc) -> Self {
        Self {
            title: doc.title.unwrap_or_else(|| "Untitled".to_string()),
            content: doc.text,
            summary: doc.description,
            url: doc.url,
            strategy_used: "wasm_extraction".to_string(),
            extraction_confidence: doc.quality_score.map(|q| q as f64 / 100.0).unwrap_or(0.8),
        }
    }
}

/// Content extraction modes with specific behaviors (host-side)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HostExtractionMode {
    /// Extract article content using readability algorithms
    Article,

    /// Extract full page content including sidebars and navigation
    Full,

    /// Extract only metadata (title, description, structured data)
    Metadata,

    /// Custom extraction using provided CSS selectors
    Custom(Vec<String>),
}

impl HostExtractionMode {
    /// Parse mode string into HostExtractionMode
    /// Note: Named parse_mode instead of from_str to avoid confusion with FromStr trait
    pub fn parse_mode(mode: &str) -> Self {
        match mode.to_lowercase().as_str() {
            "article" => Self::Article,
            "full" => Self::Full,
            "metadata" => Self::Metadata,
            _ => Self::Article, // Default to article mode
        }
    }
}

/// Type conversions between host and WIT types
mod conversions {
    use super::*;

    // Import WIT types from the bindgen module
    use wit_bindings::{
        ExtractedContent as WitContent, ExtractionError as WitError, ExtractionMode as WitMode,
    };

    impl From<HostExtractionMode> for WitMode {
        fn from(mode: HostExtractionMode) -> Self {
            match mode {
                HostExtractionMode::Article => WitMode::Article,
                HostExtractionMode::Full => WitMode::Full,
                HostExtractionMode::Metadata => WitMode::Metadata,
                HostExtractionMode::Custom(selectors) => WitMode::Custom(selectors),
            }
        }
    }

    impl From<WitContent> for ExtractedDoc {
        fn from(wit: WitContent) -> Self {
            ExtractedDoc {
                url: wit.url,
                title: wit.title,
                byline: wit.byline,
                published_iso: wit.published_iso,
                markdown: wit.markdown,
                text: wit.text,
                links: wit.links,
                media: wit.media,
                language: wit.language,
                reading_time: wit.reading_time,
                quality_score: wit.quality_score,
                word_count: wit.word_count,
                categories: wit.categories,
                site_name: wit.site_name,
                description: wit.description,
            }
        }
    }

    impl From<WitError> for HostExtractionError {
        fn from(error: WitError) -> Self {
            match error {
                WitError::InvalidHtml(msg) => HostExtractionError::InvalidHtml(msg),
                WitError::NetworkError(msg) => HostExtractionError::NetworkError(msg),
                WitError::ParseError(msg) => HostExtractionError::ParseError(msg),
                WitError::ResourceLimit(msg) => HostExtractionError::ResourceLimit(msg),
                WitError::ExtractorError(msg) => HostExtractionError::ExtractorError(msg),
                WitError::InternalError(msg) => HostExtractionError::InternalError(msg),
                WitError::UnsupportedMode(msg) => HostExtractionError::UnsupportedMode(msg),
            }
        }
    }

    impl HostExtractionError {
        pub fn to_anyhow(self) -> anyhow::Error {
            match self {
                Self::InvalidHtml(msg) => anyhow::anyhow!("Invalid HTML: {}", msg),
                Self::NetworkError(msg) => anyhow::anyhow!("Network error: {}", msg),
                Self::ParseError(msg) => anyhow::anyhow!("Parse error: {}", msg),
                Self::ResourceLimit(msg) => anyhow::anyhow!("Resource limit: {}", msg),
                Self::ExtractorError(msg) => anyhow::anyhow!("Extractor error: {}", msg),
                Self::InternalError(msg) => anyhow::anyhow!("Internal error: {}", msg),
                Self::UnsupportedMode(msg) => anyhow::anyhow!("Unsupported mode: {}", msg),
            }
        }
    }
}

/// Structured error types for better error handling (host-side)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostExtractionError {
    /// Invalid or malformed HTML input
    InvalidHtml(String),

    /// Network-related errors during processing
    NetworkError(String),

    /// HTML parsing failures
    ParseError(String),

    /// Resource limits exceeded (memory, time, etc.)
    ResourceLimit(String),

    /// Trek-rs library errors
    ExtractorError(String),

    /// Component internal processing errors
    InternalError(String),

    /// Unsupported extraction mode
    UnsupportedMode(String),
}

/// Component health status (host-side)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostHealthStatus {
    /// Overall component health
    pub status: String,

    /// Component version
    pub version: String,

    /// Last health check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,

    /// Number of successful extractions
    pub successful_extractions: u64,

    /// Number of failed extractions
    pub failed_extractions: u64,

    /// Average extraction time in milliseconds
    pub avg_extraction_time: f64,
}

/// Component information and metadata (host-side)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostComponentInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<String>,
}

/// Extraction performance statistics (host-side)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostExtractionStats {
    pub total_extractions: u64,
    pub successful_extractions: u64,
    pub failed_extractions: u64,
    pub avg_extraction_time: Duration,
    pub peak_memory_usage: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// Simple host context with WASI support (no resource limiting for now)
pub struct WasmHostContext {
    /// WASI context
    pub wasi: WasiCtx,
    /// Resource table for WASI
    pub table: ResourceTable,
}

impl Default for WasmHostContext {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmHostContext {
    pub fn new() -> Self {
        let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_env().build();
        Self {
            wasi,
            table: ResourceTable::new(),
        }
    }
}

impl WasiView for WasmHostContext {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

/// Host-side memory tracking and limits for WASM instances with WASI support
pub struct WasmResourceTracker {
    /// Current memory pages allocated
    pub current_pages: Arc<AtomicUsize>,
    /// Maximum memory pages allowed
    pub max_pages: usize,
    /// Memory growth failures count
    pub grow_failed_count: Arc<AtomicU64>,
    /// Peak memory usage in pages
    pub peak_pages: Arc<AtomicUsize>,
    /// Enable SIMD optimizations
    pub simd_enabled: bool,
    /// AOT cache enabled
    pub aot_cache_enabled: bool,
    /// WASI context
    pub wasi: WasiCtx,
    /// Resource table for WASI
    pub table: ResourceTable,
}

impl WasmResourceTracker {
    pub fn new(max_pages: usize) -> Self {
        let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_env().build();

        Self {
            current_pages: Arc::new(AtomicUsize::new(0)),
            max_pages,
            grow_failed_count: Arc::new(AtomicU64::new(0)),
            peak_pages: Arc::new(AtomicUsize::new(0)),
            simd_enabled: true,
            aot_cache_enabled: true,
            wasi,
            table: ResourceTable::new(),
        }
    }

    /// Get current memory usage in pages
    pub fn current_memory_pages(&self) -> usize {
        self.current_pages.load(Ordering::Relaxed)
    }

    /// Get total grow failures
    pub fn grow_failures(&self) -> u64 {
        self.grow_failed_count.load(Ordering::Relaxed)
    }

    /// Get peak memory usage in pages
    pub fn peak_memory_pages(&self) -> usize {
        self.peak_pages.load(Ordering::Relaxed)
    }
}

impl ResourceLimiter for WasmResourceTracker {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> Result<bool, anyhow::Error> {
        const WASM_PAGE_SIZE: usize = 65536; // 64KB per page

        // Convert bytes to pages - desired is the NEW total memory size
        let desired_pages = desired / WASM_PAGE_SIZE;
        let current_pages = current / WASM_PAGE_SIZE;

        // Debug logging to understand memory growth patterns
        eprintln!("WASM Memory Growth Request:");
        eprintln!("  Current: {} bytes ({} pages)", current, current_pages);
        eprintln!("  Desired: {} bytes ({} pages)", desired, desired_pages);
        eprintln!("  Maximum: {:?}", maximum);
        eprintln!(
            "  Our limit: {} pages ({} MB)",
            self.max_pages,
            self.max_pages * 64 / 1024
        );

        if desired_pages > self.max_pages {
            self.grow_failed_count.fetch_add(1, Ordering::Relaxed);
            eprintln!("  DENIED: Exceeds limit!");
            Ok(false)
        } else {
            // Store the new total (not a delta)
            self.current_pages.store(desired_pages, Ordering::Relaxed);

            // Update peak memory
            let mut peak = self.peak_pages.load(Ordering::Relaxed);
            while desired_pages > peak {
                match self.peak_pages.compare_exchange(
                    peak,
                    desired_pages,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(x) => peak = x,
                }
            }
            eprintln!("  ALLOWED: Within limit");
            Ok(true)
        }
    }

    fn table_growing(
        &mut self,
        _current: usize,
        _desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool, anyhow::Error> {
        Ok(true) // Allow table growth
    }
}

impl WasiView for WasmResourceTracker {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

/// Configuration for instance pooling and performance optimization
#[derive(Clone, Debug)]
pub struct ExtractorConfig {
    /// Maximum memory pages for WASM instances (64KB per page)
    pub max_memory_pages: usize,
    /// Timeout for extraction operations
    pub extraction_timeout: Duration,
    /// Enable WASM SIMD optimizations
    pub enable_simd: bool,
    /// Enable AOT compilation caching
    pub enable_aot_cache: bool,
    /// Pool size for instance reuse
    pub instance_pool_size: usize,
    /// Maximum idle time for pooled instances
    pub max_idle_time: Duration,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            max_memory_pages: 8192, // 512MB default (increased to handle HTML parsing with all dependencies)
            extraction_timeout: Duration::from_secs(30),
            enable_simd: true,
            enable_aot_cache: true,
            instance_pool_size: 4,
            max_idle_time: Duration::from_secs(300),
        }
    }
}

/// WASM Component-based extractor
#[allow(dead_code)]
pub struct CmExtractor {
    engine: Engine,
    component: Component,
    linker: Linker<WasmResourceTracker>,
    config: ExtractorConfig,
    stats: Arc<Mutex<HostExtractionStats>>,
}

impl CmExtractor {
    /// Create a new WASM component extractor
    pub async fn new(wasm_path: &str) -> Result<Self> {
        let config = ExtractorConfig::default();
        Self::with_config(wasm_path, config).await
    }

    /// Create a new WASM component extractor with custom configuration
    pub async fn with_config(wasm_path: &str, config: ExtractorConfig) -> Result<Self> {
        let mut wasmtime_config = Config::new();
        wasmtime_config.wasm_component_model(true);

        // Enable fuel consumption for execution limits
        wasmtime_config.consume_fuel(true);

        // Configure memory limits - convert pages to bytes (64KB per page)
        // This prevents "error while executing at wasm backtrace" memory allocation failures
        let max_memory_bytes = config.max_memory_pages * 65536; // 64KB per page
        wasmtime_config.max_wasm_stack(2 * 1024 * 1024); // 2MB stack

        // CRITICAL: Reserve virtual address space for linear memory growth
        // This allows WASM linear memory to grow dynamically during execution
        wasmtime_config.memory_reservation_for_growth(max_memory_bytes as u64);

        // Set memory guard size for bounds checking (prevents buffer overruns)
        wasmtime_config.memory_guard_size(2 * 1024 * 1024); // 2MB guard

        // Memory init COW (copy-on-write) can help with memory initialization
        wasmtime_config.memory_init_cow(true);

        // Enable SIMD if configured
        if config.enable_simd {
            wasmtime_config.wasm_simd(true);
        }

        // Enable AOT cache if configured (Wasmtime 37+)
        // Note: Wasmtime 37's cache API changed. The Config type doesn't have
        // cache_config_load_default() as a method. Instead, caching is configured
        // via the `cache` feature flag (which is enabled in Cargo.toml).
        // For explicit cache control, use environment variables:
        // - WASMTIME_CACHE_DIR: Set cache directory
        // - Or rely on default cache at $HOME/.cache/wasmtime
        if config.enable_aot_cache {
            // Wasmtime 37 automatically uses disk caching when the `cache` feature is enabled
            // First run: ~60s compile time, subsequent runs: <1s load from cache
            // Cache location: $HOME/.cache/wasmtime or $WASMTIME_CACHE_DIR
            eprintln!(
                "Wasmtime AOT caching enabled via feature flag - compiled modules will be cached"
            );
        }

        let engine = Engine::new(&wasmtime_config)?;
        let component_bytes = std::fs::read(wasm_path)?;
        let component = Component::new(&engine, component_bytes)?;

        // Create linker with WASI Preview 2 support
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

        let stats = Arc::new(Mutex::new(HostExtractionStats {
            total_extractions: 0,
            successful_extractions: 0,
            failed_extractions: 0,
            avg_extraction_time: Duration::from_millis(0),
            peak_memory_usage: 0,
            cache_hits: 0,
            cache_misses: 0,
        }));

        Ok(Self {
            engine,
            component,
            linker,
            config,
            stats,
        })
    }

    /// Extract content from HTML using the WASM component
    pub fn extract(&self, html: &str, url: &str, mode: &str) -> Result<ExtractedDoc> {
        use wit_bindings::Extractor;

        let start_time = Instant::now();

        // Use WasmResourceTracker which implements ResourceLimiter for memory control
        let resource_tracker = WasmResourceTracker::new(self.config.max_memory_pages);

        let mut store = Store::new(&self.engine, resource_tracker);
        store.set_fuel(1_000_000)?; // Set fuel limit for execution

        // CRITICAL: Set the resource limiter on the store to enable memory growth control
        store.limiter(|state| state);

        // Parse mode and convert to WIT type
        let host_mode = HostExtractionMode::parse_mode(mode);
        let wit_mode: wit_bindings::ExtractionMode = host_mode.into();

        // Instantiate component using the linker
        let instance = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        // Call WASM extract function
        let result = instance.call_extract(&mut store, html, url, &wit_mode);

        let extraction_time = start_time.elapsed();

        // Handle result and convert types
        let extracted_doc = match result {
            Ok(Ok(wit_content)) => {
                // Success: convert WIT type to host type
                let doc: ExtractedDoc = wit_content.into();

                // Update success statistics
                if let Ok(mut stats) = self.stats.lock() {
                    stats.total_extractions += 1;
                    stats.successful_extractions += 1;

                    // Update average extraction time
                    let total_time = stats.avg_extraction_time
                        * (stats.total_extractions - 1) as u32
                        + extraction_time;
                    stats.avg_extraction_time = total_time / stats.total_extractions as u32;
                }

                Ok(doc)
            }
            Ok(Err(wit_error)) => {
                // Extraction error from WASM component
                let host_error: HostExtractionError = wit_error.into();

                // Update failure statistics
                if let Ok(mut stats) = self.stats.lock() {
                    stats.total_extractions += 1;
                    stats.failed_extractions += 1;
                }

                Err(host_error.to_anyhow())
            }
            Err(e) => {
                // Runtime error (trap, out of fuel, etc.)
                if let Ok(mut stats) = self.stats.lock() {
                    stats.total_extractions += 1;
                    stats.failed_extractions += 1;
                }

                Err(anyhow::anyhow!("WASM runtime error: {}", e))
            }
        };

        extracted_doc
    }

    /// Get component information
    pub fn component_info(&self) -> HostComponentInfo {
        HostComponentInfo {
            name: "WASM Content Extractor".to_string(),
            version: "1.0.0".to_string(),
            description: "WASM-based HTML content extraction component".to_string(),
            author: "RipTide Team".to_string(),
            capabilities: vec![
                "html_extraction".to_string(),
                "content_analysis".to_string(),
                "metadata_extraction".to_string(),
            ],
        }
    }

    /// Get health status
    pub fn health_status(&self) -> HostHealthStatus {
        HostHealthStatus {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            last_check: chrono::Utc::now(),
            successful_extractions: self.stats.lock().unwrap().successful_extractions,
            failed_extractions: self.stats.lock().unwrap().failed_extractions,
            avg_extraction_time: self.stats.lock().unwrap().avg_extraction_time.as_millis() as f64,
        }
    }

    /// Get extraction statistics
    pub fn get_stats(&self) -> HostExtractionStats {
        self.stats.lock().unwrap().clone()
    }
}

/// Simple WASM extractor wrapper
pub struct WasmExtractor {
    cm_extractor: CmExtractor,
}

impl WasmExtractor {
    pub async fn new(wasm_path: &str) -> Result<Self> {
        let cm_extractor = CmExtractor::new(wasm_path).await?;
        Ok(Self { cm_extractor })
    }

    pub fn extract(&self, html: &[u8], url: &str, mode: &str) -> Result<ExtractedDoc> {
        let html_str = String::from_utf8_lossy(html);
        self.cm_extractor.extract(&html_str, url, mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_resource_tracker() {
        let tracker = WasmResourceTracker::new(100);
        assert_eq!(tracker.current_memory_pages(), 0);
        assert_eq!(tracker.grow_failures(), 0);
        assert_eq!(tracker.peak_memory_pages(), 0);
    }

    #[test]
    fn test_extractor_config_default() {
        let config = ExtractorConfig::default();
        assert_eq!(config.max_memory_pages, 8192); // Updated: 512MB = 8192 pages
        assert!(config.enable_simd);
        assert!(config.enable_aot_cache);
        assert_eq!(config.instance_pool_size, 4);
    }

    #[test]
    fn test_extraction_mode_serialization() {
        let mode = HostExtractionMode::Article;
        let serialized = serde_json::to_string(&mode).unwrap();
        let deserialized: HostExtractionMode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(mode, deserialized);
    }

    #[test]
    fn test_extracted_doc_conversion() {
        let doc = ExtractedDoc {
            url: "https://example.com".to_string(),
            title: Some("Test Title".to_string()),
            text: "Test content".to_string(),
            quality_score: Some(85),
            ..Default::default()
        };

        let content: crate::ExtractedContent = doc.clone().into();
        assert_eq!(content.title, "Test Title");
        assert_eq!(content.content, "Test content");
        assert_eq!(content.url, "https://example.com");
        assert_eq!(content.strategy_used, "wasm_extraction");
        assert_eq!(content.extraction_confidence, 0.85);
    }
}
