//! Backward compatibility layer for trait-based strategy system
//!
//! This module provides compatibility shims and adapters to ensure that existing
//! code using the enum-based strategy system continues to work with the new trait-based approach.
//!
//! NOTE: Chunking functionality has been moved to riptide-extraction crate.

use anyhow::Result;
use std::sync::Arc;

use crate::strategies::{
    implementations::*, manager::*, metadata::DocumentMetadata, performance::PerformanceMetrics,
    traits::*, ExtractedContent,
};

/// Compatibility wrapper for the original StrategyManager
pub struct CompatibleStrategyManager {
    enhanced_manager: EnhancedStrategyManager,
}

/// Legacy config structure for backward compatibility
#[derive(Debug, Clone)]
pub struct LegacyStrategyConfig {
    pub extraction: crate::strategies::ExtractionStrategyType,
    pub enable_metrics: bool,
    pub validate_schema: bool,
}

impl Default for LegacyStrategyConfig {
    fn default() -> Self {
        Self {
            extraction: crate::strategies::ExtractionStrategyType::Wasm,
            enable_metrics: true,
            validate_schema: true,
        }
    }
}

impl CompatibleStrategyManager {
    pub fn new(config: LegacyStrategyConfig) -> Self {
        let enhanced_config = StrategyManagerConfig {
            enable_metrics: config.enable_metrics,
            validate_schema: config.validate_schema,
            auto_strategy_selection: false,
            fallback_enabled: true,
            max_processing_time_ms: 30000,
        };

        // Create a registry with default strategies
        let registry = create_default_strategies();

        Self {
            enhanced_manager: futures::executor::block_on(EnhancedStrategyManager::with_registry(
                enhanced_config,
                registry,
            )),
        }
    }

    /// Extract content using legacy interface
    pub async fn extract_content(
        &mut self,
        html: &str,
        url: &str,
    ) -> Result<LegacyProcessedContent> {
        // Default to wasm strategy for compatibility
        let strategy_name = "wasm";

        // Use the enhanced manager for extraction
        let result = self
            .enhanced_manager
            .extract_and_process_with_strategy(html, url, strategy_name)
            .await?;

        Ok(LegacyProcessedContent {
            extracted: result.extracted,
            metadata: result.metadata,
            metrics: result.metrics,
        })
    }

    pub fn get_metrics(&self) -> Option<&PerformanceMetrics> {
        // Return basic metrics from the enhanced manager
        None // Simplified for core-only functionality
    }
}

/// Simplified processed content result (no chunks)
#[derive(Debug, Clone)]
pub struct LegacyProcessedContent {
    pub extracted: ExtractedContent,
    pub metadata: DocumentMetadata,
    pub metrics: Option<PerformanceMetrics>,
}

/// Create strategy implementations for backward compatibility
pub fn create_default_strategies() -> StrategyRegistry {
    let mut registry = StrategyRegistry::new();

    // Register core extraction strategies only
    registry.register_extraction(Arc::new(WasmExtractionStrategy));

    // NOTE: CSS, Regex, and LLM strategies have been moved to other crates:
    // - CSS/Regex: riptide-extraction
    // - LLM: riptide-intelligence

    registry
}

/// Migration helper for converting old enum-based configs to trait-based
pub fn migrate_extraction_strategy(
    strategy: &crate::strategies::ExtractionStrategyType,
) -> Arc<dyn ExtractionStrategy> {
    match strategy {
        crate::strategies::ExtractionStrategyType::Wasm => Arc::new(WasmExtractionStrategy),
        crate::strategies::ExtractionStrategyType::Css => Arc::new(WasmExtractionStrategy), // Fallback to WASM for now
        crate::strategies::ExtractionStrategyType::Regex => Arc::new(WasmExtractionStrategy), // Fallback to WASM for now
        crate::strategies::ExtractionStrategyType::Auto => Arc::new(WasmExtractionStrategy), // Fallback to WASM for now
    }
}

/// Provide migration guidance for deprecated features
pub mod migration {
    /// Migration guidance for features moved to other crates
    /// CSS extraction has moved to riptide-extraction
    pub const CSS_EXTRACTION_MIGRATION: &str = r#"
CSS extraction has been moved to the riptide-extraction crate.

Old:
use riptide_core::strategies::ExtractionStrategy::CssJson;

New:
use riptide_extraction::CssExtractor;
"#;

    /// Regex extraction has moved to riptide-extraction
    pub const REGEX_EXTRACTION_MIGRATION: &str = r#"
Regex extraction has been moved to the riptide-extraction crate.

Old:
use riptide_core::strategies::ExtractionStrategy::Regex;

New:
use riptide_extraction::RegexExtractor;
"#;

    /// LLM extraction has moved to riptide-intelligence
    pub const LLM_EXTRACTION_MIGRATION: &str = r#"
LLM extraction has been moved to the riptide-intelligence crate.

Old:
use riptide_core::strategies::ExtractionStrategy::Llm;

New:
use riptide_intelligence::LlmExtractor;
"#;

    /// Chunking has moved to riptide-extraction
    pub const CHUNKING_MIGRATION: &str = r#"
Content chunking has been moved to the riptide-extraction crate.

Old:
use riptide_core::strategies::chunking::*;

New:
use riptide_extraction::chunking::*;
"#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compatible_manager_basic() {
        let config = LegacyStrategyConfig::default();
        let mut manager = CompatibleStrategyManager::new(config);

        let html = "<html><body><h1>Test</h1><p>Content</p></body></html>";
        let result = manager.extract_content(html, "http://example.com").await;

        assert!(result.is_ok());
        let processed = result.unwrap();
        assert!(!processed.extracted.title.is_empty());
    }

    #[test]
    fn test_strategy_registry_creation() {
        let registry = create_default_strategies();

        // Should have wasm strategy registered
        assert!(registry.get_extraction("wasm").is_some());
    }

    #[test]
    fn test_migration_strategy() {
        let strategy = crate::strategies::ExtractionStrategyType::Wasm;
        let trait_strategy = migrate_extraction_strategy(&strategy);

        assert_eq!(trait_strategy.name(), "wasm");
    }
}
