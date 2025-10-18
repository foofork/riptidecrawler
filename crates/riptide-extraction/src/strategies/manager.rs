//! Enhanced strategy manager with trait-based approach
//!
//! This module provides an enhanced StrategyManager that uses the new trait-based
//! strategy system while maintaining backward compatibility with the existing enum-based approach.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::strategies::{
    metadata::DocumentMetadata,
    performance::PerformanceMetrics,
    traits::*,
    // Temporarily disabled for testing trait system
    // spider_implementations::*,
    ExtractedContent,
};

/// Enhanced strategy manager with trait support
pub struct EnhancedStrategyManager {
    /// Strategy registry for all trait implementations
    registry: Arc<RwLock<StrategyRegistry>>,
    /// Default extraction strategy name
    default_extraction: String,
    /// Default spider strategy name
    default_spider: String,
    /// Performance metrics collection
    metrics: Arc<RwLock<PerformanceMetrics>>,
    /// Configuration
    config: StrategyManagerConfig,
}

/// Configuration for the enhanced strategy manager
#[derive(Debug, Clone)]
pub struct StrategyManagerConfig {
    /// Enable performance metrics collection
    pub enable_metrics: bool,
    /// Enable schema validation
    pub validate_schema: bool,
    /// Enable auto-strategy selection based on content
    pub auto_strategy_selection: bool,
    /// Fallback strategy if primary fails
    pub fallback_enabled: bool,
    /// Maximum processing time per operation (milliseconds)
    pub max_processing_time_ms: u64,
}

impl Default for StrategyManagerConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            validate_schema: true,
            auto_strategy_selection: true,
            fallback_enabled: true,
            max_processing_time_ms: 30_000,
        }
    }
}

/// Result of processing with comprehensive metadata
#[derive(Debug, Clone)]
pub struct EnhancedProcessingResult {
    /// Extracted content
    pub extracted: ExtractedContent,
    /// Document metadata
    pub metadata: DocumentMetadata,
    /// Extraction result details
    pub extraction_result: ExtractionResult,
    /// Performance metrics
    pub metrics: Option<PerformanceMetrics>,
    /// Strategy used for extraction
    pub strategy_used: String,
    /// Processing time
    pub processing_time: std::time::Duration,
}

impl EnhancedStrategyManager {
    /// Create a new enhanced strategy manager
    pub async fn new(config: StrategyManagerConfig) -> Self {
        let registry = Arc::new(RwLock::new(StrategyRegistry::new()));

        Self {
            registry,
            default_extraction: "wasm".to_string(),
            default_spider: "breadth_first".to_string(),
            metrics: Arc::new(RwLock::new(PerformanceMetrics::new())),
            config,
        }
    }

    /// Create with custom registry
    pub async fn with_registry(config: StrategyManagerConfig, registry: StrategyRegistry) -> Self {
        let registry = Arc::new(RwLock::new(registry));

        Self {
            registry,
            default_extraction: "wasm".to_string(),
            default_spider: "breadth_first".to_string(),
            metrics: Arc::new(RwLock::new(PerformanceMetrics::new())),
            config,
        }
    }

    /// Register a new extraction strategy
    pub async fn register_extraction(&self, strategy: Arc<dyn ExtractionStrategy>) -> Result<()> {
        let mut registry = self.registry.write().await;
        registry.register_extraction(strategy);
        Ok(())
    }

    /// Register a new spider strategy
    pub async fn register_spider(&self, strategy: Arc<dyn SpiderStrategy>) -> Result<()> {
        let mut registry = self.registry.write().await;
        registry.register_spider(strategy);
        Ok(())
    }

    /// Set default extraction strategy
    pub fn set_default_extraction(&mut self, strategy_name: String) {
        self.default_extraction = strategy_name;
    }

    /// Set default spider strategy
    pub fn set_default_spider(&mut self, strategy_name: String) {
        self.default_spider = strategy_name;
    }

    /// Extract and process content with automatic strategy selection
    pub async fn extract_and_process(
        &self,
        html: &str,
        url: &str,
    ) -> Result<EnhancedProcessingResult> {
        // Select best extraction strategy
        let strategy_name = if self.config.auto_strategy_selection {
            self.select_best_extraction_strategy(html).await?
        } else {
            self.default_extraction.clone()
        };

        self.extract_and_process_with_strategy(html, url, &strategy_name)
            .await
    }

    /// Extract and process content with specific strategy
    pub async fn extract_and_process_with_strategy(
        &self,
        html: &str,
        url: &str,
        strategy_name: &str,
    ) -> Result<EnhancedProcessingResult> {
        let start = std::time::Instant::now();

        // Get extraction strategy
        let registry = self.registry.read().await;
        let strategy = registry
            .get_extraction(strategy_name)
            .ok_or_else(|| anyhow::anyhow!("Extraction strategy '{}' not found", strategy_name))?;

        // Perform extraction
        let extraction_result = strategy.extract(html, url).await?;

        // Extract metadata
        let metadata = crate::strategies::metadata::extract_metadata(html, url).await?;

        // Note: Chunking has been moved to riptide-extraction crate

        let processing_time = start.elapsed();

        // Update metrics if enabled
        let metrics = if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.record_extraction(
                &crate::strategies::ExtractionStrategy::Wasm, // Convert to enum for compatibility
                processing_time,
                extraction_result.content.content.len(),
                0, // No chunks in core
            );
            Some(metrics.clone())
        } else {
            None
        };

        Ok(EnhancedProcessingResult {
            extracted: extraction_result.content.clone(),
            metadata,
            extraction_result,
            metrics,
            strategy_used: strategy_name.to_string(),
            processing_time,
        })
    }

    /// Select the best extraction strategy for given content
    async fn select_best_extraction_strategy(&self, html: &str) -> Result<String> {
        let registry = self.registry.read().await;

        if let Some(best_strategy) = registry.find_best_extraction(html) {
            Ok(best_strategy.name().to_string())
        } else {
            Ok(self.default_extraction.clone())
        }
    }

    /// Process crawl requests using spider strategy
    pub async fn process_crawl_requests(
        &self,
        requests: Vec<CrawlRequest>,
    ) -> Result<Vec<CrawlRequest>> {
        self.process_crawl_requests_with_strategy(requests, &self.default_spider)
            .await
    }

    /// Process crawl requests with specific spider strategy
    pub async fn process_crawl_requests_with_strategy(
        &self,
        requests: Vec<CrawlRequest>,
        strategy_name: &str,
    ) -> Result<Vec<CrawlRequest>> {
        let registry = self.registry.read().await;
        let strategy = registry
            .get_spider(strategy_name)
            .ok_or_else(|| anyhow::anyhow!("Spider strategy '{}' not found", strategy_name))?;

        strategy.process_requests(requests).await
    }

    /// List all available strategies
    pub async fn list_strategies(&self) -> StrategyListing {
        let registry = self.registry.read().await;

        StrategyListing {
            extraction: registry
                .list_extraction_strategies()
                .into_iter()
                .map(|(name, caps)| (name.to_string(), caps.clone()))
                .collect(),
            chunking: registry
                .list_chunking_strategies()
                .into_iter()
                .map(|name| name.to_string())
                .collect(),
            spider: registry
                .list_spider_strategies()
                .into_iter()
                .map(|name| name.to_string())
                .collect(),
        }
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Reset performance metrics
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = PerformanceMetrics::new();
    }

    /// Get strategy capabilities
    pub async fn get_strategy_capabilities(
        &self,
        strategy_name: &str,
    ) -> Result<StrategyCapabilities> {
        let registry = self.registry.read().await;

        if let Some(strategy) = registry.get_extraction(strategy_name) {
            Ok(strategy.capabilities())
        } else {
            Err(anyhow::anyhow!("Strategy '{}' not found", strategy_name))
        }
    }

    /// Validate strategy configuration
    pub async fn validate_strategy(&self, strategy_name: &str) -> Result<ValidationResult> {
        let registry = self.registry.read().await;

        if let Some(strategy) = registry.get_extraction(strategy_name) {
            let available = strategy.is_available();
            let capabilities = strategy.capabilities();

            Ok(ValidationResult {
                strategy_name: strategy_name.to_string(),
                available,
                missing_dependencies: if available {
                    vec![]
                } else {
                    capabilities.resource_requirements.external_dependencies
                },
                performance_tier: capabilities.performance_tier,
            })
        } else {
            Err(anyhow::anyhow!("Strategy '{}' not found", strategy_name))
        }
    }
}

/// Listing of all available strategies
#[derive(Debug, Clone)]
pub struct StrategyListing {
    pub extraction: Vec<(String, StrategyCapabilities)>,
    pub chunking: Vec<String>,
    pub spider: Vec<String>,
}

/// Strategy validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub strategy_name: String,
    pub available: bool,
    pub missing_dependencies: Vec<String>,
    pub performance_tier: PerformanceTier,
}

/// Builder for creating configured strategy manager
pub struct EnhancedStrategyManagerBuilder {
    config: StrategyManagerConfig,
    registry: Option<StrategyRegistry>,
    default_extraction: Option<String>,
    default_spider: Option<String>,
}

impl EnhancedStrategyManagerBuilder {
    pub fn new() -> Self {
        Self {
            config: StrategyManagerConfig::default(),
            registry: None,
            default_extraction: None,
            default_spider: None,
        }
    }

    pub fn with_config(mut self, config: StrategyManagerConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_registry(mut self, registry: StrategyRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    pub fn with_default_extraction(mut self, strategy: String) -> Self {
        self.default_extraction = Some(strategy);
        self
    }

    pub fn with_default_spider(mut self, strategy: String) -> Self {
        self.default_spider = Some(strategy);
        self
    }

    pub async fn build(self) -> EnhancedStrategyManager {
        let mut manager = if let Some(registry) = self.registry {
            EnhancedStrategyManager::with_registry(self.config, registry).await
        } else {
            EnhancedStrategyManager::new(self.config).await
        };

        if let Some(extraction) = self.default_extraction {
            manager.set_default_extraction(extraction);
        }

        if let Some(spider) = self.default_spider {
            manager.set_default_spider(spider);
        }

        manager
    }
}

impl Default for EnhancedStrategyManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
