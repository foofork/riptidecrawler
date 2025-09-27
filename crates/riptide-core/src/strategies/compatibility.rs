//! Backward compatibility layer for trait-based strategy system
//!
//! This module provides compatibility shims and adapters to ensure that existing
//! code using the enum-based strategy system continues to work with the new trait-based approach.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use crate::strategies::{
    traits::*,
    implementations::*,
    manager::*,
    ExtractedContent, ContentChunk, ChunkingConfig,
    chunking::ChunkingMode,
    metadata::DocumentMetadata,
    performance::PerformanceMetrics,
    RegexPattern,
};

/// Compatibility wrapper for the original StrategyManager
pub struct CompatibleStrategyManager {
    enhanced_manager: EnhancedStrategyManager,
    config: StrategyConfig,
}

/// Original strategy configuration (maintained for compatibility)
#[derive(Debug, Clone)]
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

/// Original extraction strategy enum (maintained for compatibility)
#[derive(Debug, Clone)]
pub enum ExtractionStrategy {
    /// Default WASM-based extraction (fastest)
    Trek,
    /// CSS selector to JSON extraction
    CssJson {
        selectors: HashMap<String, String>,
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

/// Original processed content result (maintained for compatibility)
#[derive(Debug, Clone)]
pub struct ProcessedContent {
    pub extracted: ExtractedContent,
    pub metadata: DocumentMetadata,
    pub chunks: Vec<ContentChunk>,
    pub metrics: Option<PerformanceMetrics>,
}

impl CompatibleStrategyManager {
    /// Create a new compatible strategy manager
    pub async fn new(config: StrategyConfig) -> Self {
        let manager_config = StrategyManagerConfig {
            enable_metrics: config.enable_metrics,
            validate_schema: config.validate_schema,
            auto_strategy_selection: false, // Disable for compatibility
            fallback_enabled: true,
            max_processing_time_ms: 30_000,
        };

        let enhanced_manager = EnhancedStrategyManager::new(manager_config).await;

        Self {
            enhanced_manager,
            config,
        }
    }

    /// Extract and chunk content using the configured strategy (compatibility method)
    pub async fn extract_and_chunk(&mut self, html: &str, url: &str) -> Result<ProcessedContent> {
        // Convert enum strategy to trait strategy name
        let strategy_name = match &self.config.extraction {
            ExtractionStrategy::Trek => "trek",
            ExtractionStrategy::CssJson { .. } => "css_json",
            ExtractionStrategy::Regex { .. } => "regex",
            ExtractionStrategy::Llm { .. } => "llm",
        };

        // Register custom strategies if needed
        self.ensure_strategy_registered().await?;

        // Use the enhanced manager
        let result = self.enhanced_manager
            .extract_and_process_with_strategy(html, url, strategy_name)
            .await?;

        Ok(ProcessedContent {
            extracted: result.extracted,
            metadata: result.metadata,
            chunks: result.chunks,
            metrics: result.metrics,
        })
    }

    /// Ensure the configured strategy is registered
    async fn ensure_strategy_registered(&self) -> Result<()> {
        match &self.config.extraction {
            ExtractionStrategy::CssJson { selectors } => {
                let strategy = Arc::new(CssJsonExtractionStrategy::new(selectors.clone()));
                self.enhanced_manager.register_extraction(strategy).await?;
            }
            ExtractionStrategy::Regex { patterns } => {
                let strategy = Arc::new(RegexExtractionStrategy::new(patterns.clone()));
                self.enhanced_manager.register_extraction(strategy).await?;
            }
            ExtractionStrategy::Llm { enabled, model, prompt_template } => {
                let strategy = Arc::new(LlmExtractionStrategy::new(
                    *enabled,
                    model.clone(),
                    prompt_template.clone(),
                ));
                self.enhanced_manager.register_extraction(strategy).await?;
            }
            ExtractionStrategy::Trek => {
                // Trek is already registered by default
            }
        }
        Ok(())
    }

    /// Get metrics (compatibility method)
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.enhanced_manager.get_metrics().await
    }

    /// Extract content based on strategy (compatibility method)
    pub async fn extract_content(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        let strategy_name = match &self.config.extraction {
            ExtractionStrategy::Trek => "trek",
            ExtractionStrategy::CssJson { .. } => "css_json",
            ExtractionStrategy::Regex { .. } => "regex",
            ExtractionStrategy::Llm { .. } => "llm",
        };

        let result = self.enhanced_manager
            .extract_and_process_with_strategy(html, url, strategy_name)
            .await?;

        Ok(result.extracted)
    }

    /// Extract metadata (compatibility method)
    pub async fn extract_metadata(&self, html: &str, url: &str) -> Result<DocumentMetadata> {
        crate::strategies::metadata::extract_metadata(html, url).await
    }

    /// Chunk content (compatibility method)
    pub async fn chunk_content(&self, content: &str) -> Result<Vec<ContentChunk>> {
        let strategy_name = match &self.config.chunking.mode {
            ChunkingMode::Fixed { .. } => "fixed",
            ChunkingMode::Sliding => "sliding",
            ChunkingMode::Sentence { .. } => "sentence",
            ChunkingMode::Topic { .. } => "topic",
            ChunkingMode::Regex { .. } => "regex", // Note: this would need a regex chunking strategy
        };

        self.enhanced_manager
            .chunk_content_with_strategy(content, strategy_name)
            .await
    }
}

/// Adapter to convert enum-based extraction strategies to trait implementations
pub struct ExtractionStrategyAdapter {
    strategy: ExtractionStrategy,
}

impl ExtractionStrategyAdapter {
    pub fn new(strategy: ExtractionStrategy) -> Self {
        Self { strategy }
    }

    /// Convert to trait implementation
    pub fn to_trait(&self) -> Arc<dyn crate::strategies::traits::ExtractionStrategy> {
        match &self.strategy {
            ExtractionStrategy::Trek => Arc::new(TrekExtractionStrategy),
            ExtractionStrategy::CssJson { selectors } => {
                Arc::new(CssJsonExtractionStrategy::new(selectors.clone()))
            }
            ExtractionStrategy::Regex { patterns } => {
                Arc::new(RegexExtractionStrategy::new(patterns.clone()))
            }
            ExtractionStrategy::Llm { enabled, model, prompt_template } => {
                Arc::new(LlmExtractionStrategy::new(
                    *enabled,
                    model.clone(),
                    prompt_template.clone(),
                ))
            }
        }
    }
}

/// Factory for creating strategies from enum configurations
pub struct StrategyFactory;

impl StrategyFactory {
    /// Create extraction strategy from enum
    pub fn create_extraction(strategy: ExtractionStrategy) -> Arc<dyn crate::strategies::traits::ExtractionStrategy> {
        ExtractionStrategyAdapter::new(strategy).to_trait()
    }

    /// Create chunking strategy from config
    pub fn create_chunking(config: &ChunkingConfig) -> Arc<dyn ChunkingStrategy> {
        match &config.mode {
            ChunkingMode::Fixed { size, by_tokens } => {
                Arc::new(FixedChunkingStrategy::new(*size, *by_tokens))
            }
            ChunkingMode::Sliding => Arc::new(SlidingChunkingStrategy),
            ChunkingMode::Sentence { max_sentences } => {
                Arc::new(SentenceChunkingStrategy::new(*max_sentences))
            }
            ChunkingMode::Topic { similarity_threshold } => {
                Arc::new(TopicChunkingStrategy::new(*similarity_threshold))
            }
            ChunkingMode::Regex { .. } => {
                // For regex chunking, we'd need to implement a regex chunking strategy
                // For now, fallback to sliding
                Arc::new(SlidingChunkingStrategy)
            }
        }
    }

    /// Create a complete registry from legacy configuration
    pub fn create_registry_from_config(config: &StrategyConfig) -> StrategyRegistry {
        let mut registry = create_default_registry();

        // Add custom extraction strategy if needed
        let extraction_strategy = Self::create_extraction(config.extraction.clone());
        registry.register_extraction(extraction_strategy);

        // Add custom chunking strategy
        let chunking_strategy = Self::create_chunking(&config.chunking);
        registry.register_chunking(chunking_strategy);

        registry
    }
}

/// Migration utilities for upgrading from enum-based to trait-based strategies
pub struct MigrationUtils;

impl MigrationUtils {
    /// Convert old StrategyConfig to new StrategyManagerConfig
    pub fn convert_config(old_config: &StrategyConfig) -> StrategyManagerConfig {
        StrategyManagerConfig {
            enable_metrics: old_config.enable_metrics,
            validate_schema: old_config.validate_schema,
            auto_strategy_selection: false, // Maintain exact behavior
            fallback_enabled: true,
            max_processing_time_ms: 30_000,
        }
    }

    /// Create enhanced manager from old configuration
    pub async fn upgrade_manager(old_config: StrategyConfig) -> Result<EnhancedStrategyManager> {
        let manager_config = Self::convert_config(&old_config);
        let registry = StrategyFactory::create_registry_from_config(&old_config);

        Ok(EnhancedStrategyManager::with_registry(manager_config, registry).await)
    }

    /// Extract strategy name from enum for use with trait-based system
    pub fn extract_strategy_name(strategy: &ExtractionStrategy) -> &'static str {
        match strategy {
            ExtractionStrategy::Trek => "trek",
            ExtractionStrategy::CssJson { .. } => "css_json",
            ExtractionStrategy::Regex { .. } => "regex",
            ExtractionStrategy::Llm { .. } => "llm",
        }
    }
}

/// Macro for backward compatibility that maps old function calls to new trait-based calls
#[macro_export]
macro_rules! strategy_compat {
    (extract_content($manager:expr, $html:expr, $url:expr)) => {
        $manager.extract_content($html, $url).await
    };
    (chunk_content($manager:expr, $content:expr)) => {
        $manager.chunk_content($content).await
    };
    (extract_and_chunk($manager:expr, $html:expr, $url:expr)) => {
        $manager.extract_and_chunk($html, $url).await
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compatibility_manager() {
        let config = StrategyConfig::default();
        let mut manager = CompatibleStrategyManager::new(config).await;

        let html = r#"<html><head><title>Test</title></head><body><p>Content</p></body></html>"#;
        let url = "https://example.com";

        let result = manager.extract_and_chunk(html, url).await;
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert!(!processed.extracted.title.is_empty());
        assert!(!processed.extracted.content.is_empty());
    }

    #[tokio::test]
    async fn test_strategy_factory() {
        let strategy = ExtractionStrategy::Trek;
        let trait_strategy = StrategyFactory::create_extraction(strategy);

        assert_eq!(trait_strategy.name(), "trek");
    }

    #[tokio::test]
    async fn test_migration_utils() {
        let old_config = StrategyConfig::default();
        let new_config = MigrationUtils::convert_config(&old_config);

        assert_eq!(new_config.enable_metrics, old_config.enable_metrics);
        assert_eq!(new_config.validate_schema, old_config.validate_schema);

        let manager = MigrationUtils::upgrade_manager(old_config).await;
        assert!(manager.is_ok());
    }
}