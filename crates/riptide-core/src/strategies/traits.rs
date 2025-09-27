//! Unified strategy traits for extraction, chunking, and spider operations
//!
//! This module provides trait-based strategy management to replace the existing enum-based
//! approach, enabling better extensibility and composition of strategies.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::strategies::{
    ExtractedContent, ContentChunk, ChunkingConfig,
    metadata::DocumentMetadata,
    performance::PerformanceMetrics,
};

/// Core extraction strategy trait
///
/// All extraction strategies must implement this trait to be managed by the StrategyRegistry.
/// This replaces the ExtractionStrategy enum with a more flexible trait-based approach.
#[async_trait]
pub trait ExtractionStrategy: Send + Sync {
    /// Extract content from HTML
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractionResult>;

    /// Get the strategy name for identification
    fn name(&self) -> &str;

    /// Get strategy capabilities and metadata
    fn capabilities(&self) -> StrategyCapabilities;

    /// Calculate confidence score for given content (0.0 - 1.0)
    fn confidence_score(&self, html: &str) -> f64;

    /// Check if strategy is available (e.g., external dependencies)
    fn is_available(&self) -> bool {
        true
    }
}

/// Result type for extraction operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// The extracted content
    pub content: ExtractedContent,
    /// Extraction quality metrics
    pub quality: ExtractionQuality,
    /// Performance metrics
    pub performance: Option<PerformanceMetrics>,
    /// Strategy-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Extraction quality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionQuality {
    pub content_length: usize,
    pub title_quality: f64,
    pub content_quality: f64,
    pub structure_score: f64,
    pub metadata_completeness: f64,
}

impl ExtractionQuality {
    pub fn overall_score(&self) -> f64 {
        (self.title_quality + self.content_quality + self.structure_score + self.metadata_completeness) / 4.0
    }
}

/// Strategy capabilities descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyCapabilities {
    /// Strategy type identifier
    pub strategy_type: String,
    /// Supported content types
    pub supported_content_types: Vec<String>,
    /// Performance characteristics
    pub performance_tier: PerformanceTier,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// Feature flags
    pub features: Vec<String>,
}

/// Performance tier classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceTier {
    /// Ultra-fast, minimal processing
    Fast,
    /// Balanced speed and quality
    Balanced,
    /// High-quality, slower processing
    Thorough,
    /// AI-powered, variable performance
    Intelligent,
}

/// Resource requirements for strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Memory usage tier
    pub memory_tier: ResourceTier,
    /// CPU usage tier
    pub cpu_tier: ResourceTier,
    /// Requires network access
    pub requires_network: bool,
    /// Requires external dependencies
    pub external_dependencies: Vec<String>,
}

/// Resource usage tiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceTier {
    Low,
    Medium,
    High,
}

/// Chunking strategy trait
///
/// Replaces the ChunkingMode enum with a trait-based approach for better extensibility.
#[async_trait]
pub trait ChunkingStrategy: Send + Sync {
    /// Chunk content according to strategy
    async fn chunk(&self, content: &str, config: &ChunkingConfig) -> Result<Vec<ContentChunk>>;

    /// Get strategy name
    fn name(&self) -> &str;

    /// Get optimal configuration for this strategy
    fn optimal_config(&self) -> ChunkingConfig;

    /// Estimate chunk count for given content
    fn estimate_chunks(&self, content: &str, config: &ChunkingConfig) -> usize;
}

/// Spider/crawler strategy trait
///
/// Abstracts crawling behavior for different spider strategies.
#[async_trait]
pub trait SpiderStrategy: Send + Sync {
    /// Process a batch of crawl requests according to strategy
    async fn process_requests(&self, requests: Vec<CrawlRequest>) -> Result<Vec<CrawlRequest>>;

    /// Get strategy name
    fn name(&self) -> &str;

    /// Calculate priority for a request
    async fn calculate_priority(&self, request: &CrawlRequest) -> Priority;

    /// Update strategy context with crawl results
    async fn update_context(&mut self, results: &[CrawlResult]);

    /// Check if strategy should adapt based on current context
    async fn should_adapt(&self) -> bool;
}

/// Crawl request (re-exported from spider module)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlRequest {
    pub url: url::Url,
    pub depth: u32,
    pub priority: Option<f64>,
    pub score: Option<f64>,
    pub metadata: HashMap<String, String>,
    pub created_at: std::time::SystemTime,
}

impl CrawlRequest {
    pub fn new(url: url::Url) -> Self {
        Self {
            url,
            depth: 0,
            priority: None,
            score: None,
            metadata: HashMap::new(),
            created_at: std::time::SystemTime::now(),
        }
    }

    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }
}

/// Crawl result for strategy analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlResult {
    pub request: CrawlRequest,
    pub success: bool,
    pub response_time: std::time::Duration,
    pub content_length: Option<usize>,
    pub error: Option<String>,
}

/// Request priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Strategy registry for managing all strategy implementations
pub struct StrategyRegistry {
    extraction_strategies: HashMap<String, Arc<dyn ExtractionStrategy>>,
    chunking_strategies: HashMap<String, Arc<dyn ChunkingStrategy>>,
    spider_strategies: HashMap<String, Arc<dyn SpiderStrategy>>,
}

impl StrategyRegistry {
    /// Create a new strategy registry
    pub fn new() -> Self {
        Self {
            extraction_strategies: HashMap::new(),
            chunking_strategies: HashMap::new(),
            spider_strategies: HashMap::new(),
        }
    }

    /// Register an extraction strategy
    pub fn register_extraction(&mut self, strategy: Arc<dyn ExtractionStrategy>) {
        let name = strategy.name().to_string();
        self.extraction_strategies.insert(name, strategy);
    }

    /// Register a chunking strategy
    pub fn register_chunking(&mut self, strategy: Arc<dyn ChunkingStrategy>) {
        let name = strategy.name().to_string();
        self.chunking_strategies.insert(name, strategy);
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

    /// Get chunking strategy by name
    pub fn get_chunking(&self, name: &str) -> Option<&Arc<dyn ChunkingStrategy>> {
        self.chunking_strategies.get(name)
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

    /// List all available chunking strategies
    pub fn list_chunking_strategies(&self) -> Vec<String> {
        self.chunking_strategies.keys().map(|s| s.to_string()).collect()
    }

    /// List all available spider strategies
    pub fn list_spider_strategies(&self) -> Vec<String> {
        self.spider_strategies.keys().map(|s| s.to_string()).collect()
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

    pub fn with_chunking(mut self, strategy: Arc<dyn ChunkingStrategy>) -> Self {
        self.registry.register_chunking(strategy);
        self
    }

    pub fn with_spider(mut self, strategy: Arc<dyn SpiderStrategy>) -> Self {
        self.registry.register_spider(strategy);
        self
    }

    pub fn build(self) -> StrategyRegistry {
        self.registry
    }
}

impl Default for StrategyRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}