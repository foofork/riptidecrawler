//! Comprehensive test suite for unified confidence scoring system
//!
//! This test suite follows TDD principles and defines expected behavior
//! for the confidence scoring system across all extraction methods.

use riptide_core::confidence::{
    AggregationStrategy, ConfidenceComponent, ConfidenceScore, ConfidenceScorer,
};
use serde_json::json;
use std::collections::HashMap;

/// Test that confidence scores are always in the 0.0-1.0 range
#[test]
fn test_confidence_score_range_0_to_1() {
    // Test valid scores
    let score = ConfidenceScore::new(0.75, "trek");
    assert!(score.value() >= 0.0 && score.value() <= 1.0);

    // Test boundary values
    let min_score = ConfidenceScore::new(0.0, "test");
    assert_eq!(min_score.value(), 0.0);

    let max_score = ConfidenceScore::new(1.0, "test");
    assert_eq!(max_score.value(), 1.0);

    // Test clamping of out-of-range values
    let clamped_high = ConfidenceScore::new(1.5, "test");
    assert_eq!(clamped_high.value(), 1.0);

    let clamped_low = ConfidenceScore::new(-0.5, "test");
    assert_eq!(clamped_low.value(), 0.0);
}

/// Test confidence normalization across different extraction methods
#[test]
fn test_confidence_normalization_across_methods() {
    // Trek extractor: raw score 8/10 (80%) -> normalized 0.8
    let trek_score = ConfidenceScore::from_raw_score(8.0, 10.0, "trek");
    assert!((trek_score.value() - 0.8).abs() < 0.001);

    // CSS extractor: percentage 75% -> normalized 0.75
    let css_score = ConfidenceScore::from_percentage(75.0, "css");
    assert!((css_score.value() - 0.75).abs() < 0.001);

    // Regex extractor: binary match -> normalized 1.0 or 0.0
    let regex_match = ConfidenceScore::from_boolean(true, "regex");
    assert_eq!(regex_match.value(), 1.0);

    let regex_no_match = ConfidenceScore::from_boolean(false, "regex");
    assert_eq!(regex_no_match.value(), 0.0);

    // Headless browser: probabilistic score 0.85 -> normalized 0.85
    let headless_score = ConfidenceScore::new(0.85, "headless");
    assert_eq!(headless_score.value(), 0.85);
}

/// Test confidence component tracking
#[test]
fn test_confidence_components() {
    let mut score = ConfidenceScore::new(0.0, "multi");

    // Add individual components
    score.add_component("title_quality", 0.9);
    score.add_component("content_length", 0.8);
    score.add_component("structure_score", 0.75);
    score.add_component("metadata_completeness", 0.6);

    // Verify components are tracked
    let components = score.components();
    assert_eq!(components.len(), 4);
    assert_eq!(components["title_quality"], 0.9);
    assert_eq!(components["content_length"], 0.8);
    assert_eq!(components["structure_score"], 0.75);
    assert_eq!(components["metadata_completeness"], 0.6);

    // Verify overall score is computed from components
    let expected_avg = (0.9 + 0.8 + 0.75 + 0.6) / 4.0;
    assert!((score.value() - expected_avg).abs() < 0.001);
}

/// Test confidence aggregation for multi-strategy extraction
#[test]
fn test_confidence_aggregation_multi_strategy() {
    let scores = vec![
        ConfidenceScore::new(0.8, "trek"),
        ConfidenceScore::new(0.75, "css"),
        ConfidenceScore::new(0.9, "regex"),
    ];

    // Test weighted average aggregation
    let weights = vec![0.5, 0.3, 0.2]; // Trek has highest weight
    let aggregated = ConfidenceScore::aggregate_weighted(&scores, &weights);
    let expected = 0.8 * 0.5 + 0.75 * 0.3 + 0.9 * 0.2;
    assert!((aggregated.value() - expected).abs() < 0.001);

    // Test maximum aggregation (most confident strategy wins)
    let max_aggregated = ConfidenceScore::aggregate_max(&scores);
    assert_eq!(max_aggregated.value(), 0.9);

    // Test minimum aggregation (least confident strategy)
    let min_aggregated = ConfidenceScore::aggregate_min(&scores);
    assert_eq!(min_aggregated.value(), 0.75);

    // Test harmonic mean aggregation (penalizes low scores)
    let harmonic = ConfidenceScore::aggregate_harmonic(&scores);
    let expected_harmonic = 3.0 / (1.0 / 0.8 + 1.0 / 0.75 + 1.0 / 0.9);
    assert!((harmonic.value() - expected_harmonic).abs() < 0.001);
}

/// Test confidence score serialization
#[test]
fn test_confidence_score_serialization() {
    let mut score = ConfidenceScore::new(0.85, "trek");
    score.add_component("title", 0.9);
    score.add_component("content", 0.8);
    score.set_metadata(json!({"extraction_time_ms": 150}));

    // Serialize to JSON
    let json = serde_json::to_string(&score).expect("Failed to serialize");
    assert!(json.contains("0.85"));
    assert!(json.contains("trek"));
    assert!(json.contains("title"));
    assert!(json.contains("extraction_time_ms"));

    // Deserialize from JSON
    let deserialized: ConfidenceScore =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.value(), score.value());
    assert_eq!(deserialized.method(), score.method());
    assert_eq!(deserialized.components().len(), 2);
}

/// Test confidence quality tiers classification
#[test]
fn test_confidence_quality_tiers() {
    let high_confidence = ConfidenceScore::new(0.9, "trek");
    assert_eq!(high_confidence.quality_tier(), "high");
    assert!(high_confidence.is_reliable());

    let medium_confidence = ConfidenceScore::new(0.7, "css");
    assert_eq!(medium_confidence.quality_tier(), "medium");
    assert!(medium_confidence.is_acceptable());

    let low_confidence = ConfidenceScore::new(0.4, "fallback");
    assert_eq!(low_confidence.quality_tier(), "low");
    assert!(!low_confidence.is_reliable());

    let very_low = ConfidenceScore::new(0.2, "error");
    assert_eq!(very_low.quality_tier(), "very_low");
    assert!(!very_low.is_acceptable());
}

/// Test confidence adjustment based on content indicators
#[test]
fn test_confidence_adjustment_content_indicators() {
    let mut score = ConfidenceScore::new(0.7, "trek");

    // Boost for article tags
    score.boost_for_indicator("has_article_tag", 0.1);
    assert_eq!(score.value(), 0.8);

    // Boost for structured content
    score.boost_for_indicator("has_main_content", 0.05);
    assert_eq!(score.value(), 0.85);

    // Penalty for short content
    score.penalize_for_indicator("content_too_short", 0.1);
    assert_eq!(score.value(), 0.75);

    // Ensure scores don't exceed bounds after adjustments
    score.boost_for_indicator("high_quality", 0.5);
    assert_eq!(score.value(), 1.0); // Clamped at 1.0
}

/// Test confidence metadata tracking
#[test]
fn test_confidence_metadata() {
    let mut score = ConfidenceScore::new(0.8, "trek");

    // Add metadata
    let metadata = json!({
        "extraction_time_ms": 125,
        "word_count": 1500,
        "has_images": true,
        "language": "en"
    });
    score.set_metadata(metadata.clone());

    // Verify metadata retrieval
    let retrieved = score.metadata().expect("Metadata should be present");
    assert_eq!(retrieved["extraction_time_ms"], 125);
    assert_eq!(retrieved["word_count"], 1500);
    assert_eq!(retrieved["has_images"], true);
    assert_eq!(retrieved["language"], "en");
}

/// Test confidence scorer trait implementation
#[test]
fn test_confidence_scorer_trait() {
    // Create mock extracted document
    let doc = create_mock_document(
        "<html><head><title>Test Article</title></head><body><article><p>Content here</p></article></body></html>",
        "https://example.com/article"
    );

    // Test Trek scorer
    let trek_scorer = TrekConfidenceScorer::new();
    let score = trek_scorer.compute_confidence(&doc);
    assert!(score.value() > 0.7); // Should have good confidence for article tag

    // Test CSS scorer
    let css_scorer = CssConfidenceScorer::new();
    let score = css_scorer.compute_confidence(&doc);
    assert!(score.value() > 0.6);

    // Test Regex scorer
    let regex_scorer = RegexConfidenceScorer::new();
    let score = regex_scorer.compute_confidence(&doc);
    assert!(score.value() >= 0.0 && score.value() <= 1.0);
}

/// Test confidence threshold filtering
#[test]
fn test_confidence_threshold_filtering() {
    let scores = vec![
        ConfidenceScore::new(0.9, "trek"),
        ConfidenceScore::new(0.5, "css"),
        ConfidenceScore::new(0.3, "fallback"),
        ConfidenceScore::new(0.8, "regex"),
    ];

    // Filter by minimum threshold
    let high_confidence: Vec<_> = scores
        .iter()
        .filter(|s| s.value() >= 0.7)
        .collect();
    assert_eq!(high_confidence.len(), 2);

    // Filter by quality tier
    let reliable: Vec<_> = scores
        .iter()
        .filter(|s| s.is_reliable())
        .collect();
    assert!(reliable.len() >= 2);
}

/// Test confidence comparison and ordering
#[test]
fn test_confidence_comparison() {
    let score_a = ConfidenceScore::new(0.8, "trek");
    let score_b = ConfidenceScore::new(0.7, "css");
    let score_c = ConfidenceScore::new(0.8, "regex");

    // Test comparison operators
    assert!(score_a > score_b);
    assert!(score_b < score_a);
    assert!(score_a == score_c); // Same value, different method

    // Test sorting
    let mut scores = vec![score_b.clone(), score_a.clone(), score_c.clone()];
    scores.sort_by(|a, b| b.value().partial_cmp(&a.value()).unwrap());
    assert_eq!(scores[0].value(), 0.8);
    assert_eq!(scores[2].value(), 0.7);
}

/// Test confidence decay over time for cached results
#[test]
fn test_confidence_decay_over_time() {
    let mut score = ConfidenceScore::new(0.9, "trek");
    score.set_timestamp(std::time::SystemTime::now());

    // Fresh score should maintain high confidence
    let fresh_adjusted = score.adjusted_for_age(std::time::Duration::from_secs(60));
    assert!((fresh_adjusted - 0.9).abs() < 0.01);

    // Old score should decay
    let old_adjusted = score.adjusted_for_age(std::time::Duration::from_secs(86400)); // 1 day
    assert!(old_adjusted < 0.9);
    assert!(old_adjusted > 0.5); // But not too much
}

/// Test confidence builder pattern
#[test]
fn test_confidence_builder() {
    let score = ConfidenceScore::builder()
        .method("trek")
        .base_score(0.75)
        .add_component("title", 0.9)
        .add_component("content", 0.8)
        .add_component("structure", 0.7)
        .metadata(json!({"html_size": 50000}))
        .build();

    assert!(score.value() > 0.7);
    assert_eq!(score.method(), "trek");
    assert_eq!(score.components().len(), 3);
    assert!(score.metadata().is_some());
}

// Helper functions and mock implementations

fn create_mock_document(html: &str, url: &str) -> MockExtractedDoc {
    MockExtractedDoc {
        html: html.to_string(),
        url: url.to_string(),
        title: Some("Test Article".to_string()),
        content: "Content here".to_string(),
    }
}

struct MockExtractedDoc {
    html: String,
    url: String,
    title: Option<String>,
    content: String,
}

// Mock scorer implementations for testing

struct TrekConfidenceScorer;
impl TrekConfidenceScorer {
    fn new() -> Self {
        Self
    }
}

impl ConfidenceScorer for TrekConfidenceScorer {
    fn compute_confidence(&self, doc: &MockExtractedDoc) -> ConfidenceScore {
        let mut score = ConfidenceScore::new(0.8, "trek");

        if doc.html.contains("<article") {
            score.boost_for_indicator("article_tag", 0.1);
        }
        if doc.content.len() > 100 {
            score.boost_for_indicator("content_length", 0.05);
        }

        score
    }
}

struct CssConfidenceScorer;
impl CssConfidenceScorer {
    fn new() -> Self {
        Self
    }
}

impl ConfidenceScorer for CssConfidenceScorer {
    fn compute_confidence(&self, doc: &MockExtractedDoc) -> ConfidenceScore {
        let mut score = ConfidenceScore::new(0.7, "css");

        if doc.html.contains("class=\"content\"") || doc.html.contains("<main") {
            score.boost_for_indicator("structured_content", 0.1);
        }

        score
    }
}

struct RegexConfidenceScorer;
impl RegexConfidenceScorer {
    fn new() -> Self {
        Self
    }
}

impl ConfidenceScorer for RegexConfidenceScorer {
    fn compute_confidence(&self, doc: &MockExtractedDoc) -> ConfidenceScore {
        // Regex provides binary confidence based on pattern matching
        let has_match = doc.html.contains("<article") || doc.html.contains("<main");
        ConfidenceScore::from_boolean(has_match, "regex")
    }
}

/// Test aggregation strategies enum
#[test]
fn test_aggregation_strategies() {
    let scores = vec![
        ConfidenceScore::new(0.8, "trek"),
        ConfidenceScore::new(0.9, "css"),
        ConfidenceScore::new(0.7, "regex"),
    ];

    // Test each aggregation strategy
    let weighted = AggregationStrategy::WeightedAverage.aggregate(&scores, Some(vec![0.5, 0.3, 0.2]));
    assert!(weighted.value() > 0.7 && weighted.value() < 0.9);

    let max = AggregationStrategy::Maximum.aggregate(&scores, None);
    assert_eq!(max.value(), 0.9);

    let min = AggregationStrategy::Minimum.aggregate(&scores, None);
    assert_eq!(min.value(), 0.7);

    let avg = AggregationStrategy::Average.aggregate(&scores, None);
    let expected = (0.8 + 0.9 + 0.7) / 3.0;
    assert!((avg.value() - expected).abs() < 0.001);

    let harmonic = AggregationStrategy::HarmonicMean.aggregate(&scores, None);
    assert!(harmonic.value() > 0.7 && harmonic.value() < 0.85);
}

/// Test confidence reporting and diagnostics
#[test]
fn test_confidence_reporting() {
    let mut score = ConfidenceScore::new(0.85, "trek");
    score.add_component("title_quality", 0.9);
    score.add_component("content_quality", 0.8);
    score.add_component("structure_score", 0.85);

    let report = score.generate_report();

    // Verify report contains key information
    assert!(report.contains("0.85"));
    assert!(report.contains("trek"));
    assert!(report.contains("title_quality"));
    assert!(report.contains("high")); // Quality tier
}
