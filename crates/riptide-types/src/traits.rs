//! Core trait definitions for extraction, spider, and chunking strategies

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::extracted::{ExtractedContent, ExtractionQuality};

/// Performance metrics for strategy execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub extraction_time_ms: u64,
    pub content_length: usize,
    pub memory_used_bytes: Option<u64>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            extraction_time_ms: 0,
            content_length: 0,
            memory_used_bytes: None,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Core extraction strategy trait
///
/// All extraction strategies must implement this trait to be managed by strategy registries.
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

/// Crawl request
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
