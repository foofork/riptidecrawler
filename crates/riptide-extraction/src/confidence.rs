//! Unified confidence scoring system for extraction strategies
//!
//! This module provides a standardized confidence scoring system that normalizes
//! confidence scores from different extraction methods to a unified 0.0-1.0 scale.
//!
//! # Design Principles
//!
//! - **Unified Scale**: All scores normalized to 0.0-1.0 range
//! - **Composable**: Support for component-based confidence calculation
//! - **Aggregatable**: Multiple strategies can be combined
//! - **Transparent**: Track components and metadata for debugging
//! - **Serializable**: JSON serialization for API responses
//!
//! # Examples
//!
//! ```rust
//! use riptide_core::confidence::{ConfidenceScore, AggregationStrategy};
//!
//! // Create a simple confidence score
//! let score = ConfidenceScore::new(0.85, "trek");
//! assert_eq!(score.value(), 0.85);
//!
//! // Create with components
//! let mut score = ConfidenceScore::builder()
//!     .method("trek")
//!     .add_component("title", 0.9)
//!     .add_component("content", 0.8)
//!     .add_component("structure", 0.85)
//!     .build();
//!
//! // Aggregate multiple scores
//! let scores = vec![
//!     ConfidenceScore::new(0.8, "trek"),
//!     ConfidenceScore::new(0.9, "css"),
//! ];
//! let aggregated = AggregationStrategy::WeightedAverage
//!     .aggregate(&scores, Some(vec![0.7, 0.3]));
//! ```

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

/// A confidence score with value between 0.0 and 1.0
///
/// Represents the confidence level of an extraction operation.
/// Higher values indicate higher confidence in the extracted content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScore {
    /// The confidence value (0.0 - 1.0)
    value: f64,

    /// Extraction method that produced this score
    method: String,

    /// Component scores that contribute to overall confidence
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    components: HashMap<String, f64>,

    /// Optional metadata about the score calculation
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<serde_json::Value>,

    /// Timestamp when score was created
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<std::time::SystemTime>,
}

impl ConfidenceScore {
    /// Create a new confidence score with the given value and method
    ///
    /// The value will be clamped to the range [0.0, 1.0]
    pub fn new(value: f64, method: impl Into<String>) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            method: method.into(),
            components: HashMap::new(),
            metadata: None,
            timestamp: Some(std::time::SystemTime::now()),
        }
    }

    /// Create from a raw score and maximum value
    ///
    /// Normalizes raw_score/max_value to 0.0-1.0 range
    pub fn from_raw_score(raw_score: f64, max_value: f64, method: impl Into<String>) -> Self {
        let normalized = if max_value > 0.0 {
            (raw_score / max_value).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Self::new(normalized, method)
    }

    /// Create from a percentage value (0-100)
    ///
    /// Converts percentage to 0.0-1.0 scale
    pub fn from_percentage(percentage: f64, method: impl Into<String>) -> Self {
        Self::new(percentage / 100.0, method)
    }

    /// Create from a boolean match result
    ///
    /// True -> 1.0, False -> 0.0
    pub fn from_boolean(matched: bool, method: impl Into<String>) -> Self {
        Self::new(if matched { 1.0 } else { 0.0 }, method)
    }

    /// Get the confidence value (0.0 - 1.0)
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Get the extraction method name
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Get the component scores
    pub fn components(&self) -> &HashMap<String, f64> {
        &self.components
    }

    /// Get the metadata
    pub fn metadata(&self) -> Option<&serde_json::Value> {
        self.metadata.as_ref()
    }

    /// Get the timestamp
    pub fn timestamp(&self) -> Option<std::time::SystemTime> {
        self.timestamp
    }

    /// Add a component score
    ///
    /// Component scores are tracked individually and used to compute
    /// the overall confidence value as their average.
    pub fn add_component(&mut self, name: impl Into<String>, value: f64) {
        let clamped = value.clamp(0.0, 1.0);
        self.components.insert(name.into(), clamped);
        self.recompute_from_components();
    }

    /// Set metadata
    pub fn set_metadata(&mut self, metadata: serde_json::Value) {
        self.metadata = Some(metadata);
    }

    /// Set timestamp
    pub fn set_timestamp(&mut self, timestamp: std::time::SystemTime) {
        self.timestamp = Some(timestamp);
    }

    /// Boost the confidence score for a positive indicator
    pub fn boost_for_indicator(&mut self, _indicator: &str, boost: f64) {
        self.value = (self.value + boost).clamp(0.0, 1.0);
    }

    /// Penalize the confidence score for a negative indicator
    pub fn penalize_for_indicator(&mut self, _indicator: &str, penalty: f64) {
        self.value = (self.value - penalty).clamp(0.0, 1.0);
    }

    /// Get the quality tier classification
    ///
    /// - high: >= 0.8
    /// - medium: >= 0.6
    /// - low: >= 0.4
    /// - very_low: < 0.4
    pub fn quality_tier(&self) -> &'static str {
        match self.value {
            v if v >= 0.8 => "high",
            v if v >= 0.6 => "medium",
            v if v >= 0.4 => "low",
            _ => "very_low",
        }
    }

    /// Check if the score is reliable (>= 0.7)
    pub fn is_reliable(&self) -> bool {
        self.value >= 0.7
    }

    /// Check if the score is acceptable (>= 0.5)
    pub fn is_acceptable(&self) -> bool {
        self.value >= 0.5
    }

    /// Adjust confidence for age (decay over time)
    ///
    /// Applies a decay function to reduce confidence for old cached results
    pub fn adjusted_for_age(&self, age: std::time::Duration) -> f64 {
        // Safe conversion: u64 seconds will fit in f64 without precision loss up to 2^53
        #[allow(clippy::cast_precision_loss)]
        let age_seconds = age.as_secs() as f64;
        let decay_factor = (-age_seconds / 86400.0 * 0.1).exp(); // 10% decay per day
        (self.value * decay_factor).clamp(0.0, 1.0)
    }

    /// Generate a diagnostic report
    pub fn generate_report(&self) -> String {
        let mut report = format!(
            "Confidence Score Report\n\
             Method: {}\n\
             Overall Score: {:.3} ({})\n",
            self.method,
            self.value,
            self.quality_tier()
        );

        if !self.components.is_empty() {
            report.push_str("\nComponents:\n");
            for (name, value) in &self.components {
                report.push_str(&format!("  {}: {:.3}\n", name, value));
            }
        }

        if let Some(metadata) = &self.metadata {
            report.push_str(&format!("\nMetadata: {}\n", metadata));
        }

        report
    }

    /// Create a builder for constructing confidence scores
    pub fn builder() -> ConfidenceScoreBuilder {
        ConfidenceScoreBuilder::new()
    }

    /// Aggregate multiple scores using weighted average
    pub fn aggregate_weighted(scores: &[ConfidenceScore], weights: &[f64]) -> Self {
        assert_eq!(
            scores.len(),
            weights.len(),
            "Scores and weights must have same length"
        );

        let total_weight: f64 = weights.iter().sum();
        let weighted_sum: f64 = scores
            .iter()
            .zip(weights.iter())
            .map(|(s, w)| s.value * w)
            .sum();

        let value = if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        };

        let methods: Vec<_> = scores.iter().map(|s| s.method.as_str()).collect();
        Self::new(value, format!("aggregate({})", methods.join("+")))
    }

    /// Aggregate using maximum value
    pub fn aggregate_max(scores: &[ConfidenceScore]) -> Self {
        let max_score = scores.iter().map(|s| s.value).fold(0.0_f64, f64::max);

        let methods: Vec<_> = scores.iter().map(|s| s.method.as_str()).collect();
        Self::new(max_score, format!("max({})", methods.join("+")))
    }

    /// Aggregate using minimum value
    pub fn aggregate_min(scores: &[ConfidenceScore]) -> Self {
        let min_score = scores.iter().map(|s| s.value).fold(1.0_f64, f64::min);

        let methods: Vec<_> = scores.iter().map(|s| s.method.as_str()).collect();
        Self::new(min_score, format!("min({})", methods.join("+")))
    }

    /// Aggregate using harmonic mean (penalizes low scores)
    pub fn aggregate_harmonic(scores: &[ConfidenceScore]) -> Self {
        // Safe conversion: practical number of scores will fit in f64
        #[allow(clippy::cast_precision_loss)]
        let n = scores.len() as f64;
        let sum_reciprocals: f64 = scores
            .iter()
            .map(|s| if s.value > 0.0 { 1.0 / s.value } else { 0.0 })
            .sum();

        let value = if sum_reciprocals > 0.0 {
            n / sum_reciprocals
        } else {
            0.0
        };

        let methods: Vec<_> = scores.iter().map(|s| s.method.as_str()).collect();
        Self::new(value, format!("harmonic({})", methods.join("+")))
    }

    /// Recompute overall value from components
    fn recompute_from_components(&mut self) {
        if !self.components.is_empty() {
            let sum: f64 = self.components.values().sum();
            // Safe conversion: practical number of components will fit in f64
            #[allow(clippy::cast_precision_loss)]
            let count = self.components.len() as f64;
            self.value = (sum / count).clamp(0.0, 1.0);
        }
    }
}

/// Builder for constructing confidence scores
pub struct ConfidenceScoreBuilder {
    method: Option<String>,
    base_score: Option<f64>,
    components: HashMap<String, f64>,
    metadata: Option<serde_json::Value>,
}

impl ConfidenceScoreBuilder {
    pub fn new() -> Self {
        Self {
            method: None,
            base_score: None,
            components: HashMap::new(),
            metadata: None,
        }
    }

    pub fn method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    pub fn base_score(mut self, score: f64) -> Self {
        self.base_score = Some(score);
        self
    }

    pub fn add_component(mut self, name: impl Into<String>, value: f64) -> Self {
        self.components.insert(name.into(), value.clamp(0.0, 1.0));
        self
    }

    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn build(self) -> ConfidenceScore {
        let method = self.method.unwrap_or_else(|| "unknown".to_string());

        let value = if !self.components.is_empty() {
            let sum: f64 = self.components.values().sum();
            // Safe conversion: practical number of components will fit in f64
            #[allow(clippy::cast_precision_loss)]
            let count = self.components.len() as f64;
            sum / count
        } else {
            self.base_score.unwrap_or(0.0)
        };

        let mut score = ConfidenceScore::new(value, method);
        score.components = self.components;
        score.metadata = self.metadata;
        score
    }
}

impl Default for ConfidenceScoreBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregation strategies for combining multiple confidence scores
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationStrategy {
    /// Weighted average of scores
    WeightedAverage,
    /// Simple arithmetic mean
    Average,
    /// Take maximum score
    Maximum,
    /// Take minimum score
    Minimum,
    /// Harmonic mean (penalizes low scores)
    HarmonicMean,
}

impl AggregationStrategy {
    /// Aggregate a collection of scores using this strategy
    pub fn aggregate(
        &self,
        scores: &[ConfidenceScore],
        weights: Option<Vec<f64>>,
    ) -> ConfidenceScore {
        match self {
            AggregationStrategy::WeightedAverage => {
                let weights = weights.unwrap_or_else(|| vec![1.0; scores.len()]);
                ConfidenceScore::aggregate_weighted(scores, &weights)
            }
            AggregationStrategy::Average => {
                let weights = vec![1.0; scores.len()];
                ConfidenceScore::aggregate_weighted(scores, &weights)
            }
            AggregationStrategy::Maximum => ConfidenceScore::aggregate_max(scores),
            AggregationStrategy::Minimum => ConfidenceScore::aggregate_min(scores),
            AggregationStrategy::HarmonicMean => ConfidenceScore::aggregate_harmonic(scores),
        }
    }
}

/// Individual component of a confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceComponent {
    pub name: String,
    pub value: f64,
    pub weight: f64,
}

/// Trait for types that can compute confidence scores
pub trait ConfidenceScorer {
    /// Compute confidence score for an extracted document
    fn compute_confidence<T>(&self, doc: &T) -> ConfidenceScore;
}

// Implement comparison traits for sorting

impl PartialEq for ConfidenceScore {
    fn eq(&self, other: &Self) -> bool {
        (self.value - other.value).abs() < f64::EPSILON
    }
}

impl PartialOrd for ConfidenceScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl fmt::Display for ConfidenceScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConfidenceScore(value={:.3}, method={}, tier={})",
            self.value,
            self.method,
            self.quality_tier()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_confidence_score() {
        let score = ConfidenceScore::new(0.75, "test");
        assert_eq!(score.value(), 0.75);
        assert_eq!(score.method(), "test");
        assert_eq!(score.quality_tier(), "medium");
    }

    #[test]
    fn test_score_clamping() {
        let high = ConfidenceScore::new(1.5, "test");
        assert_eq!(high.value(), 1.0);

        let low = ConfidenceScore::new(-0.5, "test");
        assert_eq!(low.value(), 0.0);
    }

    #[test]
    fn test_from_raw_score() {
        let score = ConfidenceScore::from_raw_score(8.0, 10.0, "test");
        assert_eq!(score.value(), 0.8);
    }

    #[test]
    fn test_from_percentage() {
        let score = ConfidenceScore::from_percentage(75.0, "test");
        assert_eq!(score.value(), 0.75);
    }

    #[test]
    fn test_from_boolean() {
        let matched = ConfidenceScore::from_boolean(true, "test");
        assert_eq!(matched.value(), 1.0);

        let not_matched = ConfidenceScore::from_boolean(false, "test");
        assert_eq!(not_matched.value(), 0.0);
    }

    #[test]
    fn test_components() {
        let mut score = ConfidenceScore::new(0.0, "test");
        score.add_component("title", 0.9);
        score.add_component("content", 0.8);
        score.add_component("structure", 0.7);

        assert_eq!(score.components().len(), 3);
        assert!((score.value() - 0.8).abs() < 0.01); // Average of components
    }

    #[test]
    fn test_quality_tiers() {
        assert_eq!(ConfidenceScore::new(0.9, "test").quality_tier(), "high");
        assert_eq!(ConfidenceScore::new(0.7, "test").quality_tier(), "medium");
        assert_eq!(ConfidenceScore::new(0.5, "test").quality_tier(), "low");
        assert_eq!(ConfidenceScore::new(0.3, "test").quality_tier(), "very_low");
    }

    #[test]
    fn test_aggregation() {
        let scores = vec![
            ConfidenceScore::new(0.8, "a"),
            ConfidenceScore::new(0.9, "b"),
            ConfidenceScore::new(0.7, "c"),
        ];

        let max = ConfidenceScore::aggregate_max(&scores);
        assert_eq!(max.value(), 0.9);

        let min = ConfidenceScore::aggregate_min(&scores);
        assert_eq!(min.value(), 0.7);
    }
}
