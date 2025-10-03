//! Trait implementations for existing strategy types
//!
//! This module provides trait implementations for the existing enum-based strategies,
//! enabling backward compatibility while providing the new trait-based interface.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::strategies::{traits::*, ExtractedContent, PerformanceMetrics};

// ============================================================================
// EXTRACTION STRATEGY IMPLEMENTATIONS
// ============================================================================

/// Trek extraction strategy implementation
#[derive(Debug, Clone)]
pub struct TrekExtractionStrategy;

#[async_trait]
impl ExtractionStrategy for TrekExtractionStrategy {
    async fn extract(&self, html: &str, _url: &str) -> Result<ExtractionResult> {
        let start = std::time::Instant::now();
        // Trek extraction moved to riptide-html, returning mock result
        let content = ExtractedContent {
            title: "Mock Title".to_string(),
            content: html.chars().take(1000).collect(),
            summary: Some("Mock summary for testing".to_string()),
            url: _url.to_string(),
            strategy_used: "trek".to_string(),
            extraction_confidence: 0.85,
        };
        let duration = start.elapsed();

        let quality = ExtractionQuality {
            content_length: content.content.len(),
            title_quality: if content.title.is_empty() { 0.0 } else { 0.9 },
            content_quality: calculate_content_quality(&content.content),
            structure_score: 0.85, // Trek provides good structure
            metadata_completeness: 0.8,
        };

        let mut metadata = HashMap::new();
        metadata.insert(
            "extraction_time_ms".to_string(),
            duration.as_millis().to_string(),
        );
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
            features: vec![
                "wasm".to_string(),
                "fast".to_string(),
                "lightweight".to_string(),
            ],
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

// CSS and Regex extraction strategies have been moved to riptide-html crate

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
