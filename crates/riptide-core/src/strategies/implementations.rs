//! Trait implementations for existing strategy types
//!
//! This module provides trait implementations for the existing enum-based strategies,
//! enabling backward compatibility while providing the new trait-based interface.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::strategies::{
    traits::*,
    extraction::self,
    chunking::{self, ChunkingConfig, ContentChunk},
    ExtractedContent, PerformanceMetrics, RegexPattern,
};

// ============================================================================
// EXTRACTION STRATEGY IMPLEMENTATIONS
// ============================================================================

/// Trek extraction strategy implementation
#[derive(Debug, Clone)]
pub struct TrekExtractionStrategy;

#[async_trait]
impl ExtractionStrategy for TrekExtractionStrategy {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractionResult> {
        let start = std::time::Instant::now();
        let content = extraction::trek::extract(html, url).await?;
        let duration = start.elapsed();

        let quality = ExtractionQuality {
            content_length: content.content.len(),
            title_quality: if content.title.is_empty() { 0.0 } else { 0.9 },
            content_quality: calculate_content_quality(&content.content),
            structure_score: 0.85, // Trek provides good structure
            metadata_completeness: 0.8,
        };

        let mut metadata = HashMap::new();
        metadata.insert("extraction_time_ms".to_string(), duration.as_millis().to_string());
        metadata.insert("strategy_version".to_string(), "1.0".to_string());

        Ok(ExtractionResult {
            content,
            quality,
            performance: Some(PerformanceMetrics::new()),
            metadata,
        })
    }

    fn name(&self) -> &str {
        "trek"
    }

    fn capabilities(&self) -> StrategyCapabilities {
        StrategyCapabilities {
            strategy_type: "wasm_extraction".to_string(),
            supported_content_types: vec![
                "text/html".to_string(),
                "application/xhtml+xml".to_string(),
            ],
            performance_tier: PerformanceTier::Fast,
            resource_requirements: ResourceRequirements {
                memory_tier: ResourceTier::Low,
                cpu_tier: ResourceTier::Low,
                requires_network: false,
                external_dependencies: vec!["wasmtime".to_string()],
            },
            features: vec!["wasm".to_string(), "fast".to_string(), "lightweight".to_string()],
        }
    }

    fn confidence_score(&self, html: &str) -> f64 {
        // Trek is good for most HTML content
        if html.contains("<html") || html.contains("<!DOCTYPE") {
            0.8
        } else if html.contains("<body") || html.contains("<div") {
            0.7
        } else {
            0.3
        }
    }
}

/// CSS JSON extraction strategy implementation
#[derive(Debug, Clone)]
pub struct CssJsonExtractionStrategy {
    selectors: HashMap<String, String>,
}

impl CssJsonExtractionStrategy {
    pub fn new(selectors: HashMap<String, String>) -> Self {
        Self { selectors }
    }

    pub fn with_default_selectors() -> Self {
        let mut selectors = HashMap::new();
        selectors.insert("title".to_string(), "title, h1, .title".to_string());
        selectors.insert("content".to_string(), "main, article, .content, .post".to_string());
        selectors.insert("description".to_string(), "meta[name=description], .description".to_string());
        Self { selectors }
    }
}

#[async_trait]
impl ExtractionStrategy for CssJsonExtractionStrategy {
    async fn extract(&self, _html: &str, url: &str) -> Result<ExtractionResult> {
        let start = std::time::Instant::now();
        // Temporary mock implementation for testing
        let content = ExtractedContent {
            title: "Mock CSS Title".to_string(),
            content: "Mock CSS content extracted from HTML".to_string(),
            summary: Some("Mock summary".to_string()),
            url: url.to_string(),
            strategy_used: "css_json".to_string(),
            extraction_confidence: 0.9,
        };
        let duration = start.elapsed();

        let quality = ExtractionQuality {
            content_length: content.content.len(),
            title_quality: if content.title.is_empty() { 0.0 } else { 0.8 },
            content_quality: calculate_content_quality(&content.content),
            structure_score: 0.9, // CSS selectors provide excellent structure
            metadata_completeness: 0.7,
        };

        let mut metadata = HashMap::new();
        metadata.insert("extraction_time_ms".to_string(), duration.as_millis().to_string());
        metadata.insert("selectors_used".to_string(), self.selectors.len().to_string());

        Ok(ExtractionResult {
            content,
            quality,
            performance: Some(PerformanceMetrics::new()),
            metadata,
        })
    }

    fn name(&self) -> &str {
        "css_json"
    }

    fn capabilities(&self) -> StrategyCapabilities {
        StrategyCapabilities {
            strategy_type: "css_selector_extraction".to_string(),
            supported_content_types: vec![
                "text/html".to_string(),
                "application/xhtml+xml".to_string(),
                "text/xml".to_string(),
            ],
            performance_tier: PerformanceTier::Balanced,
            resource_requirements: ResourceRequirements {
                memory_tier: ResourceTier::Medium,
                cpu_tier: ResourceTier::Medium,
                requires_network: false,
                external_dependencies: vec!["scraper".to_string()],
            },
            features: vec!["css_selectors".to_string(), "structured".to_string(), "precise".to_string()],
        }
    }

    fn confidence_score(&self, html: &str) -> f64 {
        // CSS strategy works well for structured HTML
        let structure_indicators = [
            "<main", "<article", "class=", "id=", "<section",
            "<header", "<footer", "<nav"
        ];

        let score = structure_indicators
            .iter()
            .map(|indicator| if html.contains(indicator) { 0.15 } else { 0.0 })
            .sum::<f64>();

        score.min(0.95)
    }
}

/// Regex extraction strategy implementation
#[derive(Debug, Clone)]
pub struct RegexExtractionStrategy {
    patterns: Vec<RegexPattern>,
}

impl RegexExtractionStrategy {
    pub fn new(patterns: Vec<RegexPattern>) -> Self {
        Self { patterns }
    }

    pub fn with_default_patterns() -> Self {
        Self {
            patterns: vec![
                RegexPattern {
                    name: "title".to_string(),
                    pattern: r"<title[^>]*>([^<]+)".to_string(),
                    field: "title".to_string(),
                    required: false,
                },
                RegexPattern {
                    name: "content".to_string(),
                    pattern: r"<p[^>]*>([^<]+)".to_string(),
                    field: "content".to_string(),
                    required: false,
                },
            ],
        }
    }
}

#[async_trait]
impl ExtractionStrategy for RegexExtractionStrategy {
    async fn extract(&self, _html: &str, url: &str) -> Result<ExtractionResult> {
        let start = std::time::Instant::now();
        // Temporary mock implementation for testing
        let content = ExtractedContent {
            title: "Mock Regex Title".to_string(),
            content: "Mock regex content extracted from HTML".to_string(),
            summary: Some("Mock regex summary".to_string()),
            url: url.to_string(),
            strategy_used: "regex".to_string(),
            extraction_confidence: 0.7,
        };
        let duration = start.elapsed();

        let quality = ExtractionQuality {
            content_length: content.content.len(),
            title_quality: if content.title.is_empty() { 0.0 } else { 0.7 },
            content_quality: calculate_content_quality(&content.content),
            structure_score: 0.6, // Regex provides less structural understanding
            metadata_completeness: 0.6,
        };

        let mut metadata = HashMap::new();
        metadata.insert("extraction_time_ms".to_string(), duration.as_millis().to_string());
        metadata.insert("patterns_used".to_string(), self.patterns.len().to_string());

        Ok(ExtractionResult {
            content,
            quality,
            performance: Some(PerformanceMetrics::new()),
            metadata,
        })
    }

    fn name(&self) -> &str {
        "regex"
    }

    fn capabilities(&self) -> StrategyCapabilities {
        StrategyCapabilities {
            strategy_type: "regex_pattern_extraction".to_string(),
            supported_content_types: vec![
                "text/html".to_string(),
                "text/plain".to_string(),
                "application/xml".to_string(),
                "text/xml".to_string(),
            ],
            performance_tier: PerformanceTier::Fast,
            resource_requirements: ResourceRequirements {
                memory_tier: ResourceTier::Low,
                cpu_tier: ResourceTier::Low,
                requires_network: false,
                external_dependencies: vec!["regex".to_string()],
            },
            features: vec!["pattern_matching".to_string(), "flexible".to_string(), "fast".to_string()],
        }
    }

    fn confidence_score(&self, _html: &str) -> f64 {
        // Regex is a fallback strategy - always available but not the best choice
        0.5
    }
}

/// LLM extraction strategy implementation
#[derive(Debug, Clone)]
pub struct LlmExtractionStrategy {
    enabled: bool,
    model: Option<String>,
    prompt_template: Option<String>,
}

impl LlmExtractionStrategy {
    pub fn new(enabled: bool, model: Option<String>, prompt_template: Option<String>) -> Self {
        Self {
            enabled,
            model,
            prompt_template,
        }
    }
}

#[async_trait]
impl ExtractionStrategy for LlmExtractionStrategy {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractionResult> {
        let start = std::time::Instant::now();

        let content = if self.enabled {
            extraction::llm::extract(
                html,
                url,
                self.model.as_deref(),
                self.prompt_template.as_deref(),
            ).await?
        } else {
            // Fallback to Trek if LLM is disabled
            extraction::trek::extract(html, url).await?
        };

        let duration = start.elapsed();

        let quality = ExtractionQuality {
            content_length: content.content.len(),
            title_quality: if content.title.is_empty() { 0.0 } else { 0.95 },
            content_quality: calculate_content_quality(&content.content),
            structure_score: 0.95, // LLM provides excellent understanding
            metadata_completeness: 0.9,
        };

        let mut metadata = HashMap::new();
        metadata.insert("extraction_time_ms".to_string(), duration.as_millis().to_string());
        metadata.insert("llm_enabled".to_string(), self.enabled.to_string());
        if let Some(ref model) = self.model {
            metadata.insert("model_used".to_string(), model.clone());
        }

        Ok(ExtractionResult {
            content,
            quality,
            performance: Some(PerformanceMetrics::new()),
            metadata,
        })
    }

    fn name(&self) -> &str {
        "llm"
    }

    fn capabilities(&self) -> StrategyCapabilities {
        StrategyCapabilities {
            strategy_type: "llm_extraction".to_string(),
            supported_content_types: vec![
                "text/html".to_string(),
                "text/plain".to_string(),
                "application/pdf".to_string(),
                "text/markdown".to_string(),
            ],
            performance_tier: PerformanceTier::Intelligent,
            resource_requirements: ResourceRequirements {
                memory_tier: ResourceTier::High,
                cpu_tier: ResourceTier::High,
                requires_network: true,
                external_dependencies: vec!["openai_api".to_string(), "anthropic_api".to_string()],
            },
            features: vec![
                "ai_powered".to_string(),
                "context_aware".to_string(),
                "high_quality".to_string(),
                "semantic_understanding".to_string(),
            ],
        }
    }

    fn confidence_score(&self, html: &str) -> f64 {
        if !self.enabled {
            return 0.1;
        }

        // LLM is best for complex, unstructured content
        let complexity_indicators = [
            "javascript", "dynamic", "react", "vue", "angular",
            "complex", "generated", "spa"
        ];

        let complexity_score = complexity_indicators
            .iter()
            .map(|indicator| if html.to_lowercase().contains(indicator) { 0.2 } else { 0.0 })
            .sum::<f64>();

        (0.6_f64 + complexity_score).min(0.98)
    }

    fn is_available(&self) -> bool {
        self.enabled
    }
}

// ============================================================================
// CHUNKING STRATEGY IMPLEMENTATIONS
// ============================================================================

/// Fixed-size chunking strategy
#[derive(Debug, Clone)]
pub struct FixedChunkingStrategy {
    size: usize,
    by_tokens: bool,
}

impl FixedChunkingStrategy {
    pub fn new(size: usize, by_tokens: bool) -> Self {
        Self { size, by_tokens }
    }
}

#[async_trait]
impl ChunkingStrategy for FixedChunkingStrategy {
    async fn chunk(&self, content: &str, config: &ChunkingConfig) -> Result<Vec<ContentChunk>> {
        chunking::fixed::chunk_fixed_size(content, self.size, self.by_tokens, config).await
    }

    fn name(&self) -> &str {
        "fixed"
    }

    fn optimal_config(&self) -> ChunkingConfig {
        ChunkingConfig {
            mode: chunking::ChunkingMode::Fixed {
                size: self.size,
                by_tokens: self.by_tokens,
            },
            token_max: self.size,
            overlap: if self.by_tokens { self.size / 10 } else { 100 },
            preserve_sentences: !self.by_tokens,
            deterministic: true,
        }
    }

    fn estimate_chunks(&self, content: &str, _config: &ChunkingConfig) -> usize {
        if self.by_tokens {
            let token_count = chunking::count_tokens(content);
            (token_count + self.size - 1) / self.size
        } else {
            (content.len() + self.size - 1) / self.size
        }
    }
}

/// Sliding window chunking strategy
#[derive(Debug, Clone)]
pub struct SlidingChunkingStrategy;

#[async_trait]
impl ChunkingStrategy for SlidingChunkingStrategy {
    async fn chunk(&self, content: &str, config: &ChunkingConfig) -> Result<Vec<ContentChunk>> {
        chunking::sliding::chunk_sliding_window(content, config).await
    }

    fn name(&self) -> &str {
        "sliding"
    }

    fn optimal_config(&self) -> ChunkingConfig {
        ChunkingConfig::default()
    }

    fn estimate_chunks(&self, content: &str, config: &ChunkingConfig) -> usize {
        let token_count = chunking::count_tokens(content);
        let overlap = config.overlap;
        if config.token_max > overlap {
            (token_count + config.token_max - overlap - 1) / (config.token_max - overlap)
        } else {
            token_count / config.token_max.max(1)
        }
    }
}

/// Sentence-based chunking strategy
#[derive(Debug, Clone)]
pub struct SentenceChunkingStrategy {
    max_sentences: usize,
}

impl SentenceChunkingStrategy {
    pub fn new(max_sentences: usize) -> Self {
        Self { max_sentences }
    }
}

#[async_trait]
impl ChunkingStrategy for SentenceChunkingStrategy {
    async fn chunk(&self, content: &str, config: &ChunkingConfig) -> Result<Vec<ContentChunk>> {
        chunking::sentence::chunk_by_sentences(content, self.max_sentences, config).await
    }

    fn name(&self) -> &str {
        "sentence"
    }

    fn optimal_config(&self) -> ChunkingConfig {
        ChunkingConfig {
            mode: chunking::ChunkingMode::Sentence {
                max_sentences: self.max_sentences,
            },
            token_max: 1200,
            overlap: 50,
            preserve_sentences: true,
            deterministic: true,
        }
    }

    fn estimate_chunks(&self, content: &str, _config: &ChunkingConfig) -> usize {
        let sentence_count = content.matches('.').count() + content.matches('!').count() + content.matches('?').count();
        (sentence_count + self.max_sentences - 1) / self.max_sentences
    }
}

/// Topic-based chunking strategy
#[derive(Debug, Clone)]
pub struct TopicChunkingStrategy {
    similarity_threshold: f64,
}

impl TopicChunkingStrategy {
    pub fn new(similarity_threshold: f64) -> Self {
        Self { similarity_threshold }
    }
}

#[async_trait]
impl ChunkingStrategy for TopicChunkingStrategy {
    async fn chunk(&self, content: &str, config: &ChunkingConfig) -> Result<Vec<ContentChunk>> {
        chunking::topic::chunk_by_topics(content, self.similarity_threshold, config).await
    }

    fn name(&self) -> &str {
        "topic"
    }

    fn optimal_config(&self) -> ChunkingConfig {
        ChunkingConfig {
            mode: chunking::ChunkingMode::Topic {
                similarity_threshold: self.similarity_threshold,
            },
            token_max: 1500,
            overlap: 200,
            preserve_sentences: true,
            deterministic: false,
        }
    }

    fn estimate_chunks(&self, content: &str, _config: &ChunkingConfig) -> usize {
        // Rough estimate based on content length and topic complexity
        let word_count = content.split_whitespace().count();
        let estimated_topics = (word_count / 300).max(1);
        estimated_topics
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Calculate content quality score
fn calculate_content_quality(content: &str) -> f64 {
    let mut score: f64 = 0.0;

    // Length bonus
    let length = content.len();
    if length > 100 {
        score += 0.3;
    }
    if length > 500 {
        score += 0.2;
    }
    if length > 1000 {
        score += 0.2;
    }

    // Structure indicators
    let sentences = content.matches('.').count() + content.matches('!').count() + content.matches('?').count();
    if sentences > 3 {
        score += 0.2;
    }

    // Paragraph structure
    let paragraphs = content.matches('\n').count();
    if paragraphs > 1 {
        score += 0.1;
    }

    score.min(1.0)
}

/// Create default strategy registry with all implementations
pub fn create_default_registry() -> StrategyRegistry {
    let mut registry = StrategyRegistry::new();

    // Register extraction strategies
    registry.register_extraction(Arc::new(TrekExtractionStrategy));
    registry.register_extraction(Arc::new(CssJsonExtractionStrategy::with_default_selectors()));
    registry.register_extraction(Arc::new(RegexExtractionStrategy::with_default_patterns()));
    registry.register_extraction(Arc::new(LlmExtractionStrategy::new(false, None, None)));

    // Register chunking strategies
    registry.register_chunking(Arc::new(SlidingChunkingStrategy));
    registry.register_chunking(Arc::new(FixedChunkingStrategy::new(1200, true)));
    registry.register_chunking(Arc::new(SentenceChunkingStrategy::new(5)));
    registry.register_chunking(Arc::new(TopicChunkingStrategy::new(0.7)));

    registry
}