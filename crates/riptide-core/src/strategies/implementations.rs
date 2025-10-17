//! Trait implementations for existing strategy types
//!
//! This module provides trait implementations for the existing enum-based strategies,
//! enabling backward compatibility while providing the new trait-based interface.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::html_parser::{EnhancedHtmlExtractor, Metadata as HtmlMetadata};
use crate::strategies::{traits::*, ExtractedContent};
use riptide_extraction::enhanced_extractor::StructuredExtractor;

// ============================================================================
// EXTRACTION STRATEGY IMPLEMENTATIONS
// ============================================================================

/// WASM extraction strategy implementation (formerly Trek)
#[derive(Debug, Clone)]
pub struct WasmExtractionStrategy;

#[async_trait]
impl ExtractionStrategy for WasmExtractionStrategy {
    async fn extract(&self, html: &str, _url: &str) -> Result<ExtractionResult> {
        let start = std::time::Instant::now();

        // First try site-specific extraction
        let structured_content =
            if let Some(site_content) = StructuredExtractor::extract_site_specific(html, _url) {
                site_content
            } else {
                // Use structured extraction for better content preservation
                StructuredExtractor::extract_structured_content(html, Some(_url))?
            };

        // Also extract metadata using the existing extractor
        let extractor = EnhancedHtmlExtractor::new(Some(_url))?;
        let metadata_result = extractor.extract(html, _url)?;
        let html_metadata = metadata_result.metadata.clone();

        // Convert to ExtractedContent format
        let title = html_metadata
            .title
            .clone()
            .or_else(|| html_metadata.og_title.clone())
            .unwrap_or_else(|| "Untitled".to_string());

        let summary = html_metadata
            .description
            .clone()
            .or_else(|| html_metadata.og_description.clone())
            .or_else(|| Some(create_summary(&structured_content)));

        let content = ExtractedContent {
            title: title.clone(),
            content: structured_content.clone(),
            summary,
            url: _url.to_string(),
            strategy_used: "wasm".to_string(),
            extraction_confidence: metadata_result.quality_score,
        };
        let duration = start.elapsed();

        // Calculate quality metrics
        let title_quality = if title.is_empty() {
            0.0
        } else if html_metadata.og_title.is_some() {
            1.0 // High quality if we have OG title
        } else {
            0.8
        };

        let metadata_score = calculate_metadata_completeness(&html_metadata);

        let quality = ExtractionQuality {
            content_length: content.content.len(),
            title_quality,
            content_quality: calculate_content_quality(&content.content),
            structure_score: if metadata_result.is_article {
                0.95
            } else {
                0.75
            },
            metadata_completeness: metadata_score,
        };

        // Build metadata map with extraction details
        let mut metadata = HashMap::new();
        metadata.insert(
            "extraction_time_ms".to_string(),
            duration.as_millis().to_string(),
        );
        metadata.insert("strategy_version".to_string(), "3.0".to_string());
        metadata.insert(
            "is_article".to_string(),
            metadata_result.is_article.to_string(),
        );
        metadata.insert(
            "link_count".to_string(),
            metadata_result.links.len().to_string(),
        );
        metadata.insert(
            "media_count".to_string(),
            metadata_result.media.len().to_string(),
        );
        metadata.insert(
            "quality_score".to_string(),
            metadata_result.quality_score.to_string(),
        );
        metadata.insert(
            "word_count".to_string(),
            structured_content.split_whitespace().count().to_string(),
        );

        if let Some(author) = html_metadata.author {
            metadata.insert("author".to_string(), author);
        }
        if let Some(published) = html_metadata.published_date {
            metadata.insert("published_date".to_string(), published);
        }
        if !html_metadata.keywords.is_empty() {
            metadata.insert("keywords".to_string(), html_metadata.keywords.join(", "));
        }

        Ok(ExtractionResult {
            content,
            quality,
            performance: Some(riptide_types::traits::PerformanceMetrics::new()),
            metadata,
        })
    }

    fn name(&self) -> &str {
        "wasm"
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
            features: vec![
                "wasm".to_string(),
                "fast".to_string(),
                "lightweight".to_string(),
            ],
        }
    }

    fn confidence_score(&self, html: &str) -> f64 {
        // WASM extraction is good for most HTML content
        if html.contains("<html") || html.contains("<!DOCTYPE") {
            0.8
        } else if html.contains("<body") || html.contains("<div") {
            0.7
        } else {
            0.3
        }
    }
}

// CSS and Regex extraction strategies have been moved to riptide-extraction crate

/// Calculate content quality score based on various heuristics
fn calculate_content_quality(content: &str) -> f64 {
    if content.is_empty() {
        return 0.0;
    }

    let mut score = 0.5; // Base score

    // Length bonus (not too short, not too long)
    let ideal_length = 2000.0;
    let length_ratio = (content.len() as f64 / ideal_length).min(1.0);
    score += length_ratio * 0.2;

    // Word density
    let words = content.split_whitespace().count();
    if words > 50 {
        score += 0.1;
    }

    // Sentence structure
    let sentences =
        content.matches('.').count() + content.matches('!').count() + content.matches('?').count();
    if sentences > 5 {
        score += 0.1;
    }

    // Capitalization (proper sentences)
    if content.chars().next().is_some_and(|c| c.is_uppercase()) {
        score += 0.1;
    }

    score.min(1.0)
}

/// Create a summary from content
fn create_summary(content: &str) -> String {
    let words: Vec<&str> = content.split_whitespace().collect();
    let summary_length = words.len().min(50);
    words[..summary_length].join(" ") + if words.len() > 50 { "..." } else { "" }
}

/// Calculate metadata completeness score
fn calculate_metadata_completeness(metadata: &HtmlMetadata) -> f64 {
    let mut score = 0.0;
    let max_score = 7.0;

    if metadata.title.is_some() {
        score += 1.0;
    }
    if metadata.description.is_some() {
        score += 1.0;
    }
    if metadata.og_title.is_some() {
        score += 1.0;
    }
    if metadata.og_description.is_some() {
        score += 1.0;
    }
    if metadata.author.is_some() {
        score += 1.0;
    }
    if !metadata.keywords.is_empty() {
        score += 1.0;
    }
    if metadata.published_date.is_some() {
        score += 1.0;
    }

    score / max_score
}
