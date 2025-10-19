# Unified Confidence Scoring System

## Overview

The confidence scoring system provides a standardized way to assess and communicate extraction quality across all extraction methods in Riptide. All confidence scores are normalized to a **0.0-1.0 scale** where:

- **1.0** = Perfect confidence (100%)
- **0.8-1.0** = High confidence - reliable extraction
- **0.6-0.8** = Medium confidence - acceptable quality
- **0.4-0.6** = Low confidence - questionable quality
- **0.0-0.4** = Very low confidence - unreliable

## Core Components

### ConfidenceScore

The main struct representing a confidence score:

```rust
use riptide_core::confidence::ConfidenceScore;

// Create a simple confidence score
let score = ConfidenceScore::new(0.85, "wasm");
assert_eq!(score.value(), 0.85);
assert_eq!(score.method(), "wasm");
assert_eq!(score.quality_tier(), "high");

// Scores are automatically clamped to [0.0, 1.0]
let clamped = ConfidenceScore::new(1.5, "test");
assert_eq!(clamped.value(), 1.0);
```

### Component-Based Scoring

Build confidence from individual components:

```rust
let mut score = ConfidenceScore::builder()
    .method("wasm")
    .add_component("title_quality", 0.9)
    .add_component("content_quality", 0.8)
    .add_component("structure_score", 0.85)
    .add_component("metadata_completeness", 0.7)
    .build();

// Overall score is average of components
assert_eq!(score.value(), 0.8125); // (0.9 + 0.8 + 0.85 + 0.7) / 4
```

### Normalization Methods

Convert from different scoring systems:

```rust
// From raw score (e.g., Wasm's 0-10 scale)
let wasm_score = ConfidenceScore::from_raw_score(8.0, 10.0, "wasm");
assert_eq!(wasm_score.value(), 0.8);

// From percentage
let percent_score = ConfidenceScore::from_percentage(75.0, "css");
assert_eq!(percent_score.value(), 0.75);

// From boolean match
let regex_match = ConfidenceScore::from_boolean(true, "regex");
assert_eq!(regex_match.value(), 1.0);
```

## Aggregation Strategies

Combine multiple extraction results:

### Weighted Average

```rust
use riptide_core::confidence::AggregationStrategy;

let scores = vec![
    ConfidenceScore::new(0.8, "wasm"),
    ConfidenceScore::new(0.75, "css"),
    ConfidenceScore::new(0.9, "regex"),
];

// Wasm gets 50% weight, CSS 30%, Regex 20%
let weights = vec![0.5, 0.3, 0.2];
let aggregated = ConfidenceScore::aggregate_weighted(&scores, &weights);
assert_eq!(aggregated.value(), 0.805); // 0.8*0.5 + 0.75*0.3 + 0.9*0.2
```

### Maximum (Best Confidence)

```rust
let max_score = AggregationStrategy::Maximum.aggregate(&scores, None);
assert_eq!(max_score.value(), 0.9); // Highest confidence wins
```

### Minimum (Conservative)

```rust
let min_score = AggregationStrategy::Minimum.aggregate(&scores, None);
assert_eq!(min_score.value(), 0.75); // Most conservative estimate
```

### Harmonic Mean (Penalizes Low Scores)

```rust
let harmonic = AggregationStrategy::HarmonicMean.aggregate(&scores, None);
// Harmonic mean: n / (1/x1 + 1/x2 + ... + 1/xn)
// Useful when you want all strategies to perform well
```

## Integration with Extractors

### Wasm Extractor

```rust
use riptide_core::confidence_integration::WasmConfidenceScorer;

let scorer = WasmConfidenceScorer::new();
let score = scorer.analyze_html(html);

// Wasm analyzer checks:
// - Document structure (article, main, sections)
// - Content quality (length, paragraph density)
// - Semantic HTML5 tags
// - Metadata presence (og:tags, meta descriptions)
```

### CSS Selector Extractor

```rust
use riptide_core::confidence_integration::CssConfidenceScorer;

let scorer = CssConfidenceScorer::new();
let score = scorer.analyze_html(html);

// CSS analyzer checks:
// - Presence of content classes
// - CSS selector quality
// - DOM structure indicators
```

### Regex Extractor

```rust
use riptide_core::confidence_integration::RegexConfidenceScorer;

// Binary confidence (matched or not)
let matched = RegexConfidenceScorer::score_from_match(true);
assert_eq!(matched.value(), 1.0);

// Pattern quality (ratio of matches)
let quality = RegexConfidenceScorer::score_from_match_quality(7, 10);
assert_eq!(quality.value(), 0.7);
```

## Quality Tiers

Confidence scores are classified into quality tiers:

```rust
let score = ConfidenceScore::new(0.85, "wasm");

// Get tier classification
assert_eq!(score.quality_tier(), "high");

// Helper methods
assert!(score.is_reliable());   // >= 0.7
assert!(score.is_acceptable()); // >= 0.5
```

## Confidence Adjustments

Dynamically adjust confidence based on content indicators:

```rust
let mut score = ConfidenceScore::new(0.7, "wasm");

// Boost for positive indicators
score.boost_for_indicator("has_article_tag", 0.1);
assert_eq!(score.value(), 0.8);

// Penalize for negative indicators
score.penalize_for_indicator("content_too_short", 0.15);
assert_eq!(score.value(), 0.65);
```

## Metadata Tracking

Store additional context with confidence scores:

```rust
use serde_json::json;

let mut score = ConfidenceScore::new(0.8, "wasm");
score.set_metadata(json!({
    "extraction_time_ms": 125,
    "word_count": 1500,
    "has_images": true,
    "language": "en"
}));

// Retrieve metadata
let metadata = score.metadata().unwrap();
assert_eq!(metadata["word_count"], 1500);
```

## Time-Based Decay

Adjust confidence for cached results:

```rust
let mut score = ConfidenceScore::new(0.9, "wasm");
score.set_timestamp(std::time::SystemTime::now());

// Fresh content maintains confidence
let fresh = score.adjusted_for_age(std::time::Duration::from_secs(60));
assert!((fresh - 0.9).abs() < 0.01);

// Old content decays (10% per day)
let old = score.adjusted_for_age(std::time::Duration::from_secs(86400));
assert!(old < 0.9);
```

## Serialization

Confidence scores are fully serializable:

```rust
let score = ConfidenceScore::new(0.85, "wasm");
let json = serde_json::to_string(&score)?;

// JSON output:
// {
//   "value": 0.85,
//   "method": "wasm",
//   "components": {},
//   "timestamp": "2025-10-11T..."
// }

let deserialized: ConfidenceScore = serde_json::from_str(&json)?;
assert_eq!(deserialized.value(), 0.85);
```

## Migration from Legacy Quality Score

Convert old `quality_score` (u8, 0-10) to new confidence system:

```rust
use riptide_core::confidence_integration::quality_score_to_confidence;

// Old system: quality_score = 8 (out of 10)
let old_quality = Some(8_u8);
let confidence = quality_score_to_confidence(old_quality);
assert_eq!(confidence.value(), 0.8); // Normalized to 0-1 scale

// Unknown quality defaults to 0.5 (medium confidence)
let unknown = quality_score_to_confidence(None);
assert_eq!(unknown.value(), 0.5);
```

## Best Practices

### 1. Always Normalize to 0.0-1.0

```rust
// ✅ Good: Normalized
let score = ConfidenceScore::from_percentage(75.0, "method");

// ❌ Bad: Raw percentage
let raw = 75.0; // Ambiguous - 75% or 0.75?
```

### 2. Use Component Scores for Transparency

```rust
// ✅ Good: Transparent scoring
let score = ConfidenceScore::builder()
    .method("wasm")
    .add_component("title", 0.9)
    .add_component("content", 0.8)
    .add_component("structure", 0.85)
    .build();

// ❌ Bad: Opaque magic number
let score = ConfidenceScore::new(0.817, "wasm");
```

### 3. Choose Appropriate Aggregation

```rust
// For combining complementary strategies: Weighted Average
let combined = AggregationStrategy::WeightedAverage
    .aggregate(&scores, Some(weights));

// For conservative estimates: Minimum
let conservative = AggregationStrategy::Minimum
    .aggregate(&scores, None);

// For best-effort extraction: Maximum
let best_effort = AggregationStrategy::Maximum
    .aggregate(&scores, None);
```

### 4. Document Scoring Criteria

```rust
// ✅ Good: Clear criteria
/// Wasm confidence scoring:
/// - Base: 0.8 (high-quality extractor)
/// - +0.1 for article tag
/// - +0.05 for good structure
/// - +0.05 for metadata
fn compute_wasm_confidence(html: &str) -> ConfidenceScore {
    // Implementation...
}
```

## API Reference

### ConfidenceScore Methods

- `new(value: f64, method: impl Into<String>) -> Self`
- `from_raw_score(raw: f64, max: f64, method) -> Self`
- `from_percentage(percent: f64, method) -> Self`
- `from_boolean(matched: bool, method) -> Self`
- `value() -> f64` - Get the confidence value
- `method() -> &str` - Get the extraction method name
- `quality_tier() -> &'static str` - Get tier: high/medium/low/very_low
- `is_reliable() -> bool` - Check if >= 0.7
- `is_acceptable() -> bool` - Check if >= 0.5
- `add_component(&mut self, name, value)`
- `boost_for_indicator(&mut self, indicator, boost)`
- `penalize_for_indicator(&mut self, indicator, penalty)`
- `adjusted_for_age(&self, age: Duration) -> f64`
- `generate_report() -> String` - Diagnostic report

### Aggregation Methods

- `aggregate_weighted(&[ConfidenceScore], &[f64]) -> ConfidenceScore`
- `aggregate_max(&[ConfidenceScore]) -> ConfidenceScore`
- `aggregate_min(&[ConfidenceScore]) -> ConfidenceScore`
- `aggregate_harmonic(&[ConfidenceScore]) -> ConfidenceScore`

### AggregationStrategy Enum

- `WeightedAverage` - Weighted average of scores
- `Average` - Simple arithmetic mean
- `Maximum` - Take highest confidence
- `Minimum` - Take lowest confidence
- `HarmonicMean` - Harmonic mean (penalizes low scores)

## Examples

See `/tests/confidence-scoring/` for comprehensive examples.

## Architecture

```
┌─────────────────────────────────────────┐
│        ConfidenceScore (0.0-1.0)        │
├─────────────────────────────────────────┤
│  • value: f64                           │
│  • method: String                       │
│  • components: HashMap<String, f64>     │
│  • metadata: Option<Value>              │
└─────────────────────────────────────────┘
                    │
        ┌───────────┴───────────┐
        │                       │
┌───────▼────────┐    ┌────────▼────────┐
│   Wasm Scorer  │    │   CSS Scorer    │
│   (0.8 base)   │    │   (0.7 base)    │
└────────────────┘    └─────────────────┘
        │                       │
        └───────────┬───────────┘
                    │
        ┌───────────▼──────────┐
        │  Aggregation Logic   │
        │  • Weighted Average  │
        │  • Maximum           │
        │  • Minimum           │
        │  • Harmonic Mean     │
        └──────────────────────┘
```

## Testing

Run confidence scoring tests:

```bash
cargo test --package riptide-core confidence
```

Expected coverage: >90%
