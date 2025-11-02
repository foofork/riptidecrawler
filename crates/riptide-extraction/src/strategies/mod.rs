//! Extraction strategies and core infrastructure for Riptide Core
//!
//! This module provides core extraction infrastructure and trait definitions.
//! Specific extraction implementations are in dedicated crates:
//! - CSS/Regex extraction: riptide-extraction
//! - LLM extraction: riptide-intelligence
//! - Content chunking: riptide-extraction
//!
//! ## Strategy Composition
//!
//! The composition submodule (re-exported from crate::strategy_composition) enables
//! chaining multiple strategies with different execution modes:
//! - Chain: Sequential execution with fallback
//! - Parallel: Concurrent execution with result merging
//! - Fallback: Primary with secondary backup
//! - Best: Run all and pick highest confidence

// Extraction module moved to riptide-extraction
// pub mod extraction;
pub mod compatibility;
pub mod implementations;
pub mod manager;
pub mod metadata;
pub mod performance;
pub mod traits;

// New extraction strategies
pub mod css_strategy;
pub mod regex_strategy;

// Re-export composition from top-level module (commented out - module doesn't exist yet)
// pub use crate::strategy_composition as composition;

#[cfg(test)]
mod tests;

// Re-export core extraction functionality
// pub use extraction::trek; // Moved to riptide-extraction
pub use compatibility::*;
pub use implementations::*;
pub use manager::*;
pub use metadata::*;
pub use performance::*;
pub use traits::*;

// Re-export new strategies
pub use css_strategy::CssSelectorStrategy;
pub use regex_strategy::{PatternConfig, RegexPatternStrategy};

#[cfg(feature = "wasm-extractor")]
use crate::extraction_strategies::{ContentExtractor, WasmExtractor as ContentWasmExtractor};
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuration for core extraction strategies
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StrategyConfig {
    /// Extraction strategy to use (core only supports Trek)
    pub extraction: ExtractionStrategyType,
    /// Performance tracking enabled
    pub enable_metrics: bool,
    /// Schema validation enabled
    pub validate_schema: bool,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            extraction: ExtractionStrategyType::Wasm,
            enable_metrics: true,
            validate_schema: true,
        }
    }
}

/// Available core extraction strategy types (enum-based configuration)
/// Note: This is different from the ExtractionStrategy trait in traits.rs
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ExtractionStrategyType {
    /// Default WASM-based extraction (fastest, core implementation)
    Wasm,
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
        use crate::{default_patterns, regex_extract};

        match &self.config.extraction {
            ExtractionStrategyType::Wasm => {
                #[cfg(feature = "wasm-extractor")]
                {
                    // Use extraction_strategies::WasmExtractor with Option<&str> constructor
                    let extractor = ContentWasmExtractor::new(None).await?;
                    extractor.extract(html, url).await
                }
                #[cfg(not(feature = "wasm-extractor"))]
                {
                    // Fall back to regex when WASM is not available
                    tracing::warn!(
                        "WASM extractor requested but not available, falling back to regex"
                    );
                    let patterns = default_patterns();
                    let html_result = regex_extract(html, url, &patterns).await?;
                    Ok(ExtractedContent {
                        title: html_result.title,
                        content: html_result.content,
                        summary: html_result.summary,
                        url: html_result.url,
                        strategy_used: "wasm_fallback:regex".to_string(),
                        extraction_confidence: html_result.extraction_confidence,
                    })
                }
            }
            ExtractionStrategyType::Css => {
                #[cfg(feature = "wasm-extractor")]
                {
                    // Temporarily fallback to WASM until CssSelectorStrategy trait is implemented
                    let extractor = ContentWasmExtractor::new(None).await?;
                    extractor.extract(html, url).await
                }
                #[cfg(not(feature = "wasm-extractor"))]
                {
                    // Fall back to regex when WASM is not available
                    tracing::warn!("CSS extraction via WASM not available, falling back to regex");
                    let patterns = default_patterns();
                    let html_result = regex_extract(html, url, &patterns).await?;
                    Ok(ExtractedContent {
                        title: html_result.title,
                        content: html_result.content,
                        summary: html_result.summary,
                        url: html_result.url,
                        strategy_used: "css_fallback:regex".to_string(),
                        extraction_confidence: html_result.extraction_confidence,
                    })
                }
            }
            ExtractionStrategyType::Regex => {
                // Use regex extraction from this crate
                let patterns = default_patterns();
                let html_result = regex_extract(html, url, &patterns).await?;

                // Convert to ExtractedContent
                Ok(ExtractedContent {
                    title: html_result.title,
                    content: html_result.content,
                    summary: html_result.summary,
                    url: html_result.url,
                    strategy_used: html_result.strategy_used,
                    extraction_confidence: html_result.extraction_confidence,
                })
            }
            ExtractionStrategyType::Auto => {
                #[cfg(feature = "wasm-extractor")]
                {
                    // Auto-detect best strategy - temporarily simplified to use WASM only
                    // Full implementation requires CssSelectorStrategy trait implementation
                    let extractor = ContentWasmExtractor::new(None).await?;
                    let mut result = extractor.extract(html, url).await?;
                    result.strategy_used = "auto:wasm".to_string();
                    Ok(result)
                }
                #[cfg(not(feature = "wasm-extractor"))]
                {
                    // Auto-select: use regex when WASM is not available
                    let patterns = default_patterns();
                    let html_result = regex_extract(html, url, &patterns).await?;
                    Ok(ExtractedContent {
                        title: html_result.title,
                        content: html_result.content,
                        summary: html_result.summary,
                        url: html_result.url,
                        strategy_used: "auto:regex".to_string(),
                        extraction_confidence: html_result.extraction_confidence,
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

// Re-export ExtractedContent from riptide-types
pub use riptide_types::ExtractedContent;
