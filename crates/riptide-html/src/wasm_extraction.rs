//! WASM-based HTML extraction
//!
//! This module provides WASM component-based HTML extraction capabilities
//! moved from riptide-core to provide cleaner separation of concerns.

use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wasmtime::{component::*, Config, Engine, Store, ResourceLimiter};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use serde::{Deserialize, Serialize};

use crate::ExtractedContent;

/// Enhanced extraction result with comprehensive metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtractedDoc {
    /// Source URL for context and link resolution
    pub url: String,

    /// Extracted page title
    pub title: Option<String>,

    /// Author/byline information
    pub byline: Option<String>,

    /// Publication date in ISO 8601 format
    pub published_iso: Option<String>,

    /// Content formatted as Markdown
    pub markdown: String,

    /// Plain text content with HTML tags removed
    pub text: String,

    /// List of extracted hyperlinks
    pub links: Vec<String>,

    /// List of media URLs (images, videos, audio)
    pub media: Vec<String>,

    /// Detected content language (ISO 639-1 code)
    pub language: Option<String>,

    /// Estimated reading time in minutes
    pub reading_time: Option<u32>,

    /// Content quality score (0-100, higher = better)
    pub quality_score: Option<u8>,

    /// Word count of extracted text
    pub word_count: Option<u32>,

    /// Content categories/tags if detected
    pub categories: Vec<String>,

    /// Site name/publisher if available
    pub site_name: Option<String>,

    /// Meta description from page
    pub description: Option<String>,
}

impl From<ExtractedDoc> for ExtractedContent {
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

/// Content extraction modes with specific behaviors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtractionMode {
    /// Extract article content using readability algorithms
    Article,

    /// Extract full page content including sidebars and navigation
    Full,

    /// Extract only metadata (title, description, structured data)
    Metadata,

    /// Custom extraction using provided CSS selectors
    Custom(Vec<String>),
}

/// Structured error types for better error handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtractionError {
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

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
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

/// Component information and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<String>,
}

/// Extraction performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionStats {
    pub total_extractions: u64,
    pub successful_extractions: u64,
    pub failed_extractions: u64,
    pub avg_extraction_time: Duration,
    pub peak_memory_usage: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// Host-side memory tracking and limits for WASM instances
#[derive(Debug, Clone)]
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
}

impl WasmResourceTracker {
    pub fn new(max_pages: usize) -> Self {
        Self {
            current_pages: Arc::new(AtomicUsize::new(0)),
            max_pages,
            grow_failed_count: Arc::new(AtomicU64::new(0)),
            peak_pages: Arc::new(AtomicUsize::new(0)),
            simd_enabled: true,
            aot_cache_enabled: true,
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
        _maximum: Option<usize>,
    ) -> Result<bool, anyhow::Error> {
        let pages_needed = desired.saturating_sub(current);
        let new_total = self.current_pages.load(Ordering::Relaxed) + pages_needed;

        if new_total > self.max_pages {
            self.grow_failed_count.fetch_add(1, Ordering::Relaxed);
            Ok(false)
        } else {
            self.current_pages.fetch_add(pages_needed, Ordering::Relaxed);

            // Update peak memory
            let mut peak = self.peak_pages.load(Ordering::Relaxed);
            while new_total > peak {
                match self.peak_pages.compare_exchange(
                    peak,
                    new_total,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(x) => peak = x,
                }
            }
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
            max_memory_pages: 1024, // 64MB default
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
    config: ExtractorConfig,
    stats: Arc<Mutex<ExtractionStats>>,
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

        // Enable SIMD if configured
        if config.enable_simd {
            wasmtime_config.wasm_simd(true);
        }

        // Enable AOT cache if configured
        if config.enable_aot_cache {
            wasmtime_config.cache_config_load_default()?;
        }

        let engine = Engine::new(&wasmtime_config)?;
        let component_bytes = std::fs::read(wasm_path)?;
        let component = Component::new(&engine, component_bytes)?;

        let stats = Arc::new(Mutex::new(ExtractionStats {
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
            config,
            stats,
        })
    }

    /// Extract content from HTML using the WASM component
    pub fn extract(&self, html: &str, url: &str, _mode: &str) -> Result<ExtractedDoc> {
        let start_time = Instant::now();
        let resource_tracker = WasmResourceTracker::new(self.config.max_memory_pages);

        let mut store = Store::new(&self.engine, resource_tracker);
        store.set_fuel(1_000_000)?; // Set fuel limit for execution

        // TODO: Implement actual WASM component binding and invocation
        // This is a placeholder for the actual WASM component integration

        let extraction_time = start_time.elapsed();

        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_extractions += 1;
            stats.successful_extractions += 1;

            // Update average extraction time
            let total_time = stats.avg_extraction_time * (stats.total_extractions - 1) as u32 + extraction_time;
            stats.avg_extraction_time = total_time / stats.total_extractions as u32;
        }

        // For now, return a basic extracted document
        // In a real implementation, this would invoke the WASM component
        Ok(ExtractedDoc {
            url: url.to_string(),
            title: Some("Sample Title".to_string()),
            text: html.chars().take(1000).collect(),
            markdown: format!("# Sample Title\n\n{}", html.chars().take(500).collect::<String>()),
            quality_score: Some(80),
            word_count: Some(html.split_whitespace().count() as u32),
            ..Default::default()
        })
    }

    /// Get component information
    pub fn component_info(&self) -> ComponentInfo {
        ComponentInfo {
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
    pub fn health_status(&self) -> HealthStatus {
        let stats = self.stats.lock().unwrap();
        HealthStatus {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            last_check: chrono::Utc::now(),
            successful_extractions: stats.successful_extractions,
            failed_extractions: stats.failed_extractions,
            avg_extraction_time: stats.avg_extraction_time.as_millis() as f64,
        }
    }

    /// Get extraction statistics
    pub fn get_stats(&self) -> ExtractionStats {
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
        assert_eq!(config.max_memory_pages, 1024);
        assert!(config.enable_simd);
        assert!(config.enable_aot_cache);
        assert_eq!(config.instance_pool_size, 4);
    }

    #[test]
    fn test_extraction_mode_serialization() {
        let mode = ExtractionMode::Article;
        let serialized = serde_json::to_string(&mode).unwrap();
        let deserialized: ExtractionMode = serde_json::from_str(&serialized).unwrap();
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

        let content: ExtractedContent = doc.clone().into();
        assert_eq!(content.title, "Test Title");
        assert_eq!(content.content, "Test content");
        assert_eq!(content.url, "https://example.com");
        assert_eq!(content.strategy_used, "wasm_extraction");
        assert_eq!(content.extraction_confidence, 0.85);
    }
}