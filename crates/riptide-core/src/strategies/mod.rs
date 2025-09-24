//! Extraction strategies and chunking modules for Riptide Core
//!
//! This module provides various extraction strategies and chunking modes
//! with ML insights for optimal content processing.

pub mod extraction;
pub mod chunking;
pub mod metadata;
pub mod performance;

pub use extraction::*;
pub use chunking::*;
pub use metadata::*;
pub use performance::*;

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use anyhow::Result;

/// Configuration for extraction and chunking strategies
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StrategyConfig {
    /// Extraction strategy to use
    pub extraction: ExtractionStrategy,
    /// Chunking mode configuration
    pub chunking: ChunkingConfig,
    /// Performance tracking enabled
    pub enable_metrics: bool,
    /// Schema validation enabled
    pub validate_schema: bool,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            extraction: ExtractionStrategy::Trek,
            chunking: ChunkingConfig::default(),
            enable_metrics: true,
            validate_schema: true,
        }
    }
}

/// Available extraction strategies
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ExtractionStrategy {
    /// Default WASM-based extraction (fastest)
    Trek,
    /// CSS selector to JSON extraction
    CssJson {
        selectors: std::collections::HashMap<String, String>,
    },
    /// Regex pattern extraction
    Regex {
        patterns: Vec<RegexPattern>,
    },
    /// LLM-based extraction (hook-based, disabled by default)
    Llm {
        enabled: bool,
        model: Option<String>,
        prompt_template: Option<String>,
    },
}

/// Regex pattern configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RegexPattern {
    pub name: String,
    pub pattern: String,
    pub field: String,
    pub required: bool,
}

/// Strategy manager for coordinating extraction and chunking
pub struct StrategyManager {
    config: StrategyConfig,
    metrics: PerformanceMetrics,
}

impl StrategyManager {
    pub fn new(config: StrategyConfig) -> Self {
        Self {
            config,
            metrics: PerformanceMetrics::new(),
        }
    }

    pub async fn extract_and_chunk(&mut self, html: &str, url: &str) -> Result<ProcessedContent> {
        let start = std::time::Instant::now();

        // Extract content based on strategy
        let extracted = self.extract_content(html, url).await?;

        // Extract metadata
        let metadata = self.extract_metadata(html, url).await?;

        // Chunk the content
        let chunks = self.chunk_content(&extracted.content).await?;

        let duration = start.elapsed();
        if self.config.enable_metrics {
            self.metrics.record_extraction(
                &self.config.extraction,
                duration,
                extracted.content.len(),
                chunks.len(),
            );
        }

        Ok(ProcessedContent {
            extracted,
            metadata,
            chunks,
            metrics: if self.config.enable_metrics {
                Some(self.metrics.clone())
            } else {
                None
            },
        })
    }

    async fn extract_content(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        match &self.config.extraction {
            ExtractionStrategy::Trek => {
                extraction::trek::extract(html, url).await
            },
            ExtractionStrategy::CssJson { selectors } => {
                extraction::css_json::extract(html, url, selectors).await
            },
            ExtractionStrategy::Regex { patterns } => {
                extraction::regex::extract(html, url, patterns).await
            },
            ExtractionStrategy::Llm { enabled, model, prompt_template } => {
                if *enabled {
                    extraction::llm::extract(html, url, model.as_deref(), prompt_template.as_deref()).await
                } else {
                    // Fallback to Trek if LLM is disabled
                    extraction::trek::extract(html, url).await
                }
            },
        }
    }

    async fn extract_metadata(&self, html: &str, url: &str) -> Result<DocumentMetadata> {
        metadata::extract_metadata(html, url).await
    }

    async fn chunk_content(&self, content: &str) -> Result<Vec<ContentChunk>> {
        chunking::chunk_content(content, &self.config.chunking).await
    }

    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }
}

/// Processed content result
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedContent {
    pub extracted: ExtractedContent,
    pub metadata: DocumentMetadata,
    pub chunks: Vec<ContentChunk>,
    pub metrics: Option<PerformanceMetrics>,
}

/// Base extracted content structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractedContent {
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub url: String,
    pub strategy_used: String,
    pub extraction_confidence: f64,
}