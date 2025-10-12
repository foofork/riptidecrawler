//! Extraction strategies and core infrastructure for Riptide Core
//!
//! This module provides core extraction infrastructure and trait definitions.
//! Specific extraction implementations are in dedicated crates:
//! - CSS/Regex extraction: riptide-html
//! - LLM extraction: riptide-intelligence
//! - Content chunking: riptide-html
//!
//! ## Strategy Composition
//!
//! The composition submodule (re-exported from crate::strategy_composition) enables
//! chaining multiple strategies with different execution modes:
//! - Chain: Sequential execution with fallback
//! - Parallel: Concurrent execution with result merging
//! - Fallback: Primary with secondary backup
//! - Best: Run all and pick highest confidence

// Extraction module moved to riptide-html
// pub mod extraction;
pub mod implementations;
pub mod metadata;
pub mod performance;
pub mod traits;
// Temporarily disabled for testing trait system
// pub mod spider_implementations;
pub mod compatibility;
pub mod manager;

// New extraction strategies
pub mod css_strategy;
pub mod regex_strategy;

// Re-export composition from top-level module
pub use crate::strategy_composition as composition;

#[cfg(test)]
mod tests;

// Re-export core extraction functionality
// pub use extraction::trek; // Moved to riptide-html
pub use implementations::*;
pub use metadata::*;
pub use performance::*;
pub use traits::*;
// Temporarily disabled for testing trait system
// pub use spider_implementations::*;
pub use compatibility::*;
pub use manager::*;

// Re-export new strategies
pub use css_strategy::CssSelectorStrategy;
pub use regex_strategy::{PatternConfig, RegexPatternStrategy};

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    /// CSS selector-based extraction
    Css,
    /// Regular expression-based extraction
    Regex,
    /// Automatic strategy selection based on content analysis
    Auto,
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
        use riptide_html::{ContentExtractor, CssExtractorStrategy, TrekExtractor};

        match &self.config.extraction {
            ExtractionStrategy::Trek => {
                // Use real Trek extractor from riptide-html
                let extractor = TrekExtractor::new(None).await?;
                let html_result = extractor.extract(html, url).await?;
                let confidence = extractor.confidence_score(html);

                // Convert riptide_html::ExtractedContent to riptide_core::ExtractedContent
                Ok(ExtractedContent {
                    title: html_result.title,
                    content: html_result.content,
                    summary: html_result.summary,
                    url: html_result.url,
                    strategy_used: html_result.strategy_used,
                    extraction_confidence: confidence,
                })
            }
            ExtractionStrategy::Css => {
                // Use real CSS extraction strategy from riptide-html
                let extractor = CssExtractorStrategy::new();
                let html_result = extractor.extract(html, url).await?;
                let confidence = extractor.confidence_score(html);

                // Convert riptide_html::ExtractedContent to riptide_core::ExtractedContent
                Ok(ExtractedContent {
                    title: html_result.title,
                    content: html_result.content,
                    summary: html_result.summary,
                    url: html_result.url,
                    strategy_used: html_result.strategy_used,
                    extraction_confidence: confidence,
                })
            }
            ExtractionStrategy::Regex => {
                // Use regex extraction from riptide-html
                let patterns = riptide_html::default_patterns();
                let html_result = riptide_html::regex_extract(html, url, &patterns).await?;

                // Convert riptide_html::ExtractedContent to riptide_core::ExtractedContent
                Ok(ExtractedContent {
                    title: html_result.title,
                    content: html_result.content,
                    summary: html_result.summary,
                    url: html_result.url,
                    strategy_used: html_result.strategy_used,
                    extraction_confidence: html_result.extraction_confidence,
                })
            }
            ExtractionStrategy::Auto => {
                // Auto-detect best strategy based on content structure
                // Priority: CSS (if article tags) > Trek (fallback) > Regex (last resort)

                // Check for semantic HTML structure
                let has_article = html.contains("<article");
                let has_main = html.contains("<main");
                let has_content_classes = html.contains("class=\"content\"")
                    || html.contains("class=\"post\"")
                    || html.contains("class=\"article\"");

                if has_article || has_main || has_content_classes {
                    // Use CSS extraction for semantic HTML
                    let extractor = CssExtractorStrategy::new();
                    let html_result = extractor.extract(html, url).await?;
                    let confidence = extractor.confidence_score(html);

                    // Convert and set auto:css strategy
                    Ok(ExtractedContent {
                        title: html_result.title,
                        content: html_result.content,
                        summary: html_result.summary,
                        url: html_result.url,
                        strategy_used: "auto:css".to_string(),
                        extraction_confidence: confidence,
                    })
                } else {
                    // Fallback to Trek extraction
                    let extractor = TrekExtractor::new(None).await?;
                    let html_result = extractor.extract(html, url).await?;
                    let confidence = extractor.confidence_score(html);

                    // Convert and set auto:trek strategy
                    Ok(ExtractedContent {
                        title: html_result.title,
                        content: html_result.content,
                        summary: html_result.summary,
                        url: html_result.url,
                        strategy_used: "auto:trek".to_string(),
                        extraction_confidence: confidence,
                    })
                }
            }
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
