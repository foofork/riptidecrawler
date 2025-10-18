//! Unified strategy traits for extraction, chunking, and spider operations
//!
//! This module provides trait-based strategy management to replace the existing enum-based
//! approach, enabling better extensibility and composition of strategies.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// Re-export traits and types
pub use riptide_types::ExtractionResult as RiptideExtractionResult;

// Import spider types from riptide-spider crate
pub use riptide_spider::{CrawlRequest, CrawlResult, Priority};

// Re-export ExtractionQuality and ExtractedContent for local use
pub use riptide_types::{ExtractedContent, ExtractionQuality};

// ============================================================================
// TRAIT DEFINITIONS
// ============================================================================

/// Trait for extraction strategy implementations
#[async_trait]
pub trait ExtractionStrategy: Send + Sync {
    /// Extract content from HTML
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractionResult>;

    /// Get strategy name
    fn name(&self) -> &str;

    /// Get strategy capabilities
    fn capabilities(&self) -> StrategyCapabilities;

    /// Check if strategy is available in current environment
    fn is_available(&self) -> bool {
        true
    }

    /// Calculate confidence score for given HTML (0.0 - 1.0)
    fn confidence_score(&self, html: &str) -> f64 {
        // Default implementation based on content structure
        let has_article = html.contains("<article");
        let has_main = html.contains("<main");
        let has_semantic = has_article || has_main;

        if has_semantic {
            0.8
        } else {
            0.5
        }
    }
}

/// Trait for spider/crawling strategy implementations
#[async_trait]
pub trait SpiderStrategy: Send + Sync {
    /// Get strategy name
    fn name(&self) -> &str;

    /// Initialize spider strategy
    async fn initialize(&mut self) -> Result<()>;

    /// Get next URL to crawl
    async fn next_url(&mut self) -> Option<CrawlRequest>;

    /// Add discovered URLs to the queue
    async fn add_urls(&mut self, requests: Vec<CrawlRequest>) -> Result<()>;

    /// Process a batch of crawl requests (prioritize, filter, etc.)
    async fn process_requests(&self, requests: Vec<CrawlRequest>) -> Result<Vec<CrawlRequest>> {
        // Default implementation: return requests as-is
        Ok(requests)
    }

    /// Mark URL as completed
    async fn mark_completed(&mut self, url: &str, result: CrawlResult) -> Result<()>;

    /// Check if crawling is complete
    fn is_complete(&self) -> bool;

    /// Get crawl statistics
    fn stats(&self) -> CrawlStats;
}

// ============================================================================
// SUPPORTING TYPES
// ============================================================================

/// Extraction result with quality and performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// Extracted content
    pub content: ExtractedContent,
    /// Quality metrics
    pub quality: ExtractionQuality,
    /// Performance metrics
    pub performance: Option<crate::strategies::PerformanceMetrics>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Strategy capabilities and requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyCapabilities {
    /// Strategy type identifier
    pub strategy_type: String,
    /// Supported content types
    pub supported_content_types: Vec<String>,
    /// Performance tier
    pub performance_tier: PerformanceTier,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
}

/// Performance tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceTier {
    /// Very fast, minimal processing
    Fast,
    /// Balanced speed and quality
    Balanced,
    /// High quality, slower processing
    Thorough,
    /// Best quality, slowest processing
    Comprehensive,
}

/// Resource requirements for a strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Memory tier requirement
    pub memory_tier: ResourceTier,
    /// CPU tier requirement
    pub cpu_tier: ResourceTier,
    /// Requires network access
    pub requires_network: bool,
    /// External dependencies
    pub external_dependencies: Vec<String>,
}

/// Resource tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceTier {
    /// Minimal resources (< 10MB, < 10% CPU)
    Low,
    /// Moderate resources (10-50MB, 10-30% CPU)
    Medium,
    /// High resources (50-200MB, 30-70% CPU)
    High,
    /// Very high resources (> 200MB, > 70% CPU)
    VeryHigh,
}

/// Crawl statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrawlStats {
    /// Total URLs discovered
    pub total_discovered: usize,
    /// URLs completed
    pub completed: usize,
    /// URLs pending
    pub pending: usize,
    /// URLs failed
    pub failed: usize,
    /// Average crawl time per URL
    pub average_time_ms: u64,
}

/// Strategy registry for managing all strategy implementations
pub struct StrategyRegistry {
    extraction_strategies: HashMap<String, Arc<dyn ExtractionStrategy>>,
    spider_strategies: HashMap<String, Arc<dyn SpiderStrategy>>,
}

impl StrategyRegistry {
    /// Create a new strategy registry
    pub fn new() -> Self {
        Self {
            extraction_strategies: HashMap::new(),
            spider_strategies: HashMap::new(),
        }
    }

    /// Register an extraction strategy
    pub fn register_extraction(&mut self, strategy: Arc<dyn ExtractionStrategy>) {
        let name = strategy.name().to_string();
        self.extraction_strategies.insert(name, strategy);
    }

    /// Register a spider strategy
    pub fn register_spider(&mut self, strategy: Arc<dyn SpiderStrategy>) {
        let name = strategy.name().to_string();
        self.spider_strategies.insert(name, strategy);
    }

    /// Get extraction strategy by name
    pub fn get_extraction(&self, name: &str) -> Option<&Arc<dyn ExtractionStrategy>> {
        self.extraction_strategies.get(name)
    }

    /// Get spider strategy by name
    pub fn get_spider(&self, name: &str) -> Option<&Arc<dyn SpiderStrategy>> {
        self.spider_strategies.get(name)
    }

    /// List all available extraction strategies
    pub fn list_extraction_strategies(&self) -> Vec<(String, StrategyCapabilities)> {
        self.extraction_strategies
            .values()
            .map(|s| (s.name().to_string(), s.capabilities()))
            .collect()
    }

    /// List all available chunking strategies (moved to riptide-extraction)
    pub fn list_chunking_strategies(&self) -> Vec<String> {
        vec![] // Empty - chunking moved to riptide-extraction crate
    }

    /// List all available spider strategies
    pub fn list_spider_strategies(&self) -> Vec<String> {
        self.spider_strategies
            .keys()
            .map(|s| s.to_string())
            .collect()
    }

    /// Find best extraction strategy for given content
    pub fn find_best_extraction(&self, html: &str) -> Option<&Arc<dyn ExtractionStrategy>> {
        self.extraction_strategies
            .values()
            .filter(|s| s.is_available())
            .max_by(|a, b| {
                a.confidence_score(html)
                    .partial_cmp(&b.confidence_score(html))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating configured strategy registry
pub struct StrategyRegistryBuilder {
    registry: StrategyRegistry,
}

impl StrategyRegistryBuilder {
    pub fn new() -> Self {
        Self {
            registry: StrategyRegistry::new(),
        }
    }

    pub fn with_extraction(mut self, strategy: Arc<dyn ExtractionStrategy>) -> Self {
        self.registry.register_extraction(strategy);
        self
    }

    // pub fn with_spider(mut self, strategy: Arc<dyn SpiderStrategy>) -> Self {
    //     self.registry.register_spider(strategy);
    //     self
    // }

    pub fn build(self) -> StrategyRegistry {
        self.registry
    }
}

impl Default for StrategyRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
