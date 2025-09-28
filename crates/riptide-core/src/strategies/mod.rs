//! Extraction strategies and core infrastructure for Riptide Core
//!
//! This module provides core extraction infrastructure and trait definitions.
//! Specific extraction implementations are in dedicated crates:
//! - CSS/Regex extraction: riptide-html
//! - LLM extraction: riptide-intelligence
//! - Content chunking: riptide-html

pub mod extraction;
pub mod metadata;
pub mod performance;
pub mod traits;
pub mod implementations;
// Temporarily disabled for testing trait system
// pub mod spider_implementations;
pub mod manager;
pub mod compatibility;

#[cfg(test)]
mod tests;

// Re-export core extraction functionality
pub use extraction::trek;
pub use metadata::*;
pub use performance::*;
pub use traits::*;
pub use implementations::*;
// Temporarily disabled for testing trait system
// pub use spider_implementations::*;
pub use manager::*;
pub use compatibility::*;

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use anyhow::Result;

/// Configuration for core extraction strategies
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StrategyConfig {
    /// Extraction strategy to use (core only supports Trek)
    pub extraction: ExtractionStrategy,
    /// Performance tracking enabled
    pub enable_metrics: bool,
    /// Schema validation enabled
    pub validate_schema: bool,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            extraction: ExtractionStrategy::Trek,
            enable_metrics: true,
            validate_schema: true,
        }
    }
}

/// Available core extraction strategies
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ExtractionStrategy {
    /// Default WASM-based extraction (fastest, core implementation)
    Trek,
}

/// Strategy manager for coordinating core extraction
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

    pub async fn extract_content(&mut self, html: &str, url: &str) -> Result<ProcessedContent> {
        let start = std::time::Instant::now();

        // Extract content based on strategy
        let extracted = self.perform_extraction(html, url).await?;

        // Extract metadata
        let metadata = self.extract_metadata(html, url).await?;

        let duration = start.elapsed();
        if self.config.enable_metrics {
            self.metrics.record_extraction(
                &self.config.extraction,
                duration,
                extracted.content.len(),
                0, // chunks handled by other crates
            );
        }

        Ok(ProcessedContent {
            extracted,
            metadata,
            metrics: if self.config.enable_metrics {
                Some(self.metrics.clone())
            } else {
                None
            },
        })
    }

    async fn perform_extraction(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        match &self.config.extraction {
            ExtractionStrategy::Trek => {
                extraction::trek::extract(html, url).await
            },
        }
    }

    async fn extract_metadata(&self, html: &str, url: &str) -> Result<DocumentMetadata> {
        metadata::extract_metadata(html, url).await
    }

    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }
}

/// Processed content result (core extraction only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedContent {
    pub extracted: ExtractedContent,
    pub metadata: DocumentMetadata,
    pub metrics: Option<PerformanceMetrics>,
}

/// Core extracted content structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedContent {
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub url: String,
    pub strategy_used: String,
    pub extraction_confidence: f64,
}