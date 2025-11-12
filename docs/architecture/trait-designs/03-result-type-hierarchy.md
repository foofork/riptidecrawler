# Pipeline Result Type Hierarchy Design

**Date:** 2025-11-12
**Component:** Domain Result Types
**Priority:** HIGH
**Estimated Implementation Time:** 14 hours

---

## Overview

This document specifies **typed domain result enums** to replace `serde_json::Value` throughout the facade layer (37+ occurrences), enabling type-safe pipeline operations and compile-time guarantees.

---

## Problem Statement

### Current Violations (37+ instances)

```rust
// ❌ PROBLEM: Untyped JSON everywhere
pub struct PipelineResult {
    pub final_output: serde_json::Value,  // What is this?
}

pub struct StageResult {
    pub output: serde_json::Value,  // What shape?
}

async fn execute_fetch(...) -> RiptideResult<serde_json::Value> {
    Ok(serde_json::json!({
        "url": url,
        "content": content,
        // ... arbitrary structure
    }))
}
```

### Issues

1. **No type safety**: Can't verify structure at compile time
2. **Runtime errors**: JSON parsing failures discovered late
3. **Poor IDE support**: No autocomplete, no type hints
4. **Hard to test**: Must construct JSON manually in tests
5. **Serialization in wrong layer**: Facade does format conversion
6. **Documentation burden**: Must document JSON structure in comments

---

## Solution: Typed Result Hierarchy

### Location
`crates/riptide-types/src/domain/pipeline_results.rs` (new file)

---

## 1. Core Pipeline Result Enum

```rust
//! Pipeline stage result types
//!
//! Provides type-safe result representations for each pipeline stage,
//! eliminating the need for `serde_json::Value`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use super::operation_metadata::OperationMetadata;

/// Pipeline stage result
///
/// Represents the output of any pipeline stage with proper typing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "stage_type", content = "data")]
pub enum PipelineStageResult {
    /// Content was fetched from a source
    Fetched(FetchedContent),

    /// Content was extracted/parsed
    Extracted(ExtractedContent),

    /// Content was transformed
    Transformed(TransformedContent),

    /// Content passed validation
    Validated(ValidationResult),

    /// Content was stored
    Stored(StorageConfirmation),

    /// Cache operation completed
    Cached(CacheResult),

    /// Gate analysis completed
    Gated(GateAnalysisResult),
}

impl PipelineStageResult {
    /// Get stage name for logging
    pub fn stage_name(&self) -> &'static str {
        match self {
            Self::Fetched(_) => "Fetch",
            Self::Extracted(_) => "Extract",
            Self::Transformed(_) => "Transform",
            Self::Validated(_) => "Validate",
            Self::Stored(_) => "Store",
            Self::Cached(_) => "Cache",
            Self::Gated(_) => "Gate",
        }
    }

    /// Check if stage result indicates success
    pub fn is_successful(&self) -> bool {
        match self {
            Self::Validated(v) => v.passed,
            Self::Gated(g) => g.gate_passed,
            _ => true, // Other stages don't have pass/fail
        }
    }
}
```

---

## 2. FetchedContent (Replaces Fetch Stage JSON)

```rust
/// Content fetched from a source (URL, file, database, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchedContent {
    /// Source URL or identifier
    pub source: String,

    /// Raw content bytes
    #[serde(with = "serde_bytes")]
    pub content: Vec<u8>,

    /// Content type (MIME type or format hint)
    pub content_type: String,

    /// When content was fetched
    #[serde(with = "chrono::serde::ts_seconds")]
    pub fetched_at: DateTime<Utc>,

    /// Response metadata (e.g., HTTP status, headers)
    pub metadata: OperationMetadata,

    /// Content size in bytes
    pub size_bytes: usize,

    /// Whether content was served from cache
    pub from_cache: bool,

    /// Cache key if applicable
    pub cache_key: Option<String>,
}

impl FetchedContent {
    /// Create new fetched content
    pub fn new(
        source: impl Into<String>,
        content: Vec<u8>,
        content_type: impl Into<String>,
    ) -> Self {
        let size_bytes = content.len();
        Self {
            source: source.into(),
            content,
            content_type: content_type.into(),
            fetched_at: Utc::now(),
            metadata: OperationMetadata::new(),
            size_bytes,
            from_cache: false,
            cache_key: None,
        }
    }

    /// Get content as UTF-8 string
    pub fn as_text(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.content)
    }

    /// Check if content is HTML
    pub fn is_html(&self) -> bool {
        self.content_type.contains("html")
    }

    /// Check if content is PDF
    pub fn is_pdf(&self) -> bool {
        self.content_type.contains("pdf")
    }

    /// Check if content is JSON
    pub fn is_json(&self) -> bool {
        self.content_type.contains("json")
    }

    /// Mark as cached content
    pub fn with_cache_info(mut self, cache_key: impl Into<String>) -> Self {
        self.from_cache = true;
        self.cache_key = Some(cache_key.into());
        self
    }
}
```

---

## 3. ExtractedContent (Replaces Extract Stage JSON)

```rust
/// Content extracted and parsed from raw source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedContent {
    /// Source URL or identifier
    pub source: String,

    /// Document title (if available)
    pub title: Option<String>,

    /// Main extracted text content
    pub text: String,

    /// Markdown representation (if generated)
    pub markdown: Option<String>,

    /// HTML representation (if preserved)
    pub html: Option<String>,

    /// Extracted links with anchor text
    pub links: Vec<ExtractedLink>,

    /// Extracted images with metadata
    pub images: Vec<ExtractedImage>,

    /// Structured data (schema.org, JSON-LD, etc.)
    pub structured_data: Vec<StructuredData>,

    /// Document metadata
    pub metadata: HashMap<String, String>,

    /// Extraction strategy used
    pub strategy: String,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,

    /// Word count
    pub word_count: usize,

    /// Character count
    pub char_count: usize,

    /// Extraction timestamp
    #[serde(with = "chrono::serde::ts_seconds")]
    pub extracted_at: DateTime<Utc>,
}

impl ExtractedContent {
    /// Create new extracted content
    pub fn new(source: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        let word_count = text.split_whitespace().count();
        let char_count = text.chars().count();

        Self {
            source: source.into(),
            title: None,
            text,
            markdown: None,
            html: None,
            links: Vec::new(),
            images: Vec::new(),
            structured_data: Vec::new(),
            metadata: HashMap::new(),
            strategy: "default".to_string(),
            confidence: 1.0,
            word_count,
            char_count,
            extracted_at: Utc::now(),
        }
    }

    /// Check if extraction has high confidence
    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 0.7
    }

    /// Check if content is substantial
    pub fn is_substantial(&self) -> bool {
        self.word_count >= 100
    }

    /// Get all URLs from links
    pub fn all_urls(&self) -> Vec<&str> {
        self.links.iter().map(|l| l.url.as_str()).collect()
    }
}

/// Extracted link with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedLink {
    /// Link URL (absolute)
    pub url: String,

    /// Anchor text
    pub text: String,

    /// Link title attribute
    pub title: Option<String>,

    /// Relationship (rel attribute)
    pub rel: Option<String>,
}

/// Extracted image with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedImage {
    /// Image URL (absolute)
    pub url: String,

    /// Alt text
    pub alt: Option<String>,

    /// Image width in pixels
    pub width: Option<u32>,

    /// Image height in pixels
    pub height: Option<u32>,
}

/// Structured data from page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredData {
    /// Schema type (e.g., "Article", "Product")
    pub schema_type: String,

    /// Structured data fields
    pub fields: HashMap<String, serde_json::Value>,
}
```

---

## 4. ValidationResult (Replaces Validation Stage JSON)

```rust
/// Result of content validation/quality gates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed overall
    pub passed: bool,

    /// Overall quality score (0.0 - 1.0)
    pub quality_score: f64,

    /// Individual gate results
    pub gates: Vec<GateResult>,

    /// Warnings (non-fatal issues)
    pub warnings: Vec<String>,

    /// Errors (fatal issues)
    pub errors: Vec<String>,

    /// Validation timestamp
    #[serde(with = "chrono::serde::ts_seconds")]
    pub validated_at: DateTime<Utc>,
}

impl ValidationResult {
    /// Create passed validation
    pub fn passed(quality_score: f64) -> Self {
        Self {
            passed: true,
            quality_score,
            gates: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            validated_at: Utc::now(),
        }
    }

    /// Create failed validation
    pub fn failed(quality_score: f64, errors: Vec<String>) -> Self {
        Self {
            passed: false,
            quality_score,
            gates: Vec::new(),
            warnings: Vec::new(),
            errors,
            validated_at: Utc::now(),
        }
    }

    /// Add gate result
    pub fn with_gate(mut self, gate: GateResult) -> Self {
        self.gates.push(gate);
        self
    }

    /// Add warning
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Get number of passed gates
    pub fn gates_passed_count(&self) -> usize {
        self.gates.iter().filter(|g| g.passed).count()
    }

    /// Get number of failed gates
    pub fn gates_failed_count(&self) -> usize {
        self.gates.iter().filter(|g| !g.passed).count()
    }
}

/// Individual gate validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    /// Gate name
    pub name: String,

    /// Whether gate passed
    pub passed: bool,

    /// Gate score
    pub score: f64,

    /// Gate threshold
    pub threshold: f64,

    /// Reason for pass/fail
    pub reason: Option<String>,
}
```

---

## 5. GateAnalysisResult (Specific to Gate Stage)

```rust
/// Result of gate analysis (content quality assessment)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateAnalysisResult {
    /// Overall gate decision
    pub gate_passed: bool,

    /// Content quality score (0.0 - 1.0)
    pub quality_score: f64,

    /// High threshold value
    pub high_threshold: f64,

    /// Low threshold value
    pub low_threshold: f64,

    /// Quality metrics breakdown
    pub metrics: QualityMetrics,

    /// Recommendation for next action
    pub recommendation: GateRecommendation,

    /// Analysis timestamp
    #[serde(with = "chrono::serde::ts_seconds")]
    pub analyzed_at: DateTime<Utc>,
}

/// Quality metrics breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Text/content ratio
    pub text_ratio: f64,

    /// Link density
    pub link_density: f64,

    /// Word count
    pub word_count: usize,

    /// Sentence count
    pub sentence_count: usize,

    /// Average sentence length
    pub avg_sentence_length: f64,

    /// Readability score
    pub readability: Option<f64>,
}

/// Gate recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateRecommendation {
    /// Content is high quality, proceed
    Accept,

    /// Content quality uncertain, may need retry
    Retry,

    /// Content is low quality, reject
    Reject,

    /// Fallback to headless rendering
    FallbackHeadless,
}
```

---

## 6. StorageConfirmation (Replaces Store Stage JSON)

```rust
/// Confirmation of successful storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfirmation {
    /// Unique resource identifier
    pub resource_id: String,

    /// Storage location/key
    pub storage_key: String,

    /// Storage backend used
    pub storage_backend: String,

    /// When stored
    #[serde(with = "chrono::serde::ts_seconds")]
    pub stored_at: DateTime<Utc>,

    /// Size stored (bytes)
    pub size_bytes: usize,

    /// TTL if applicable
    pub ttl: Option<Duration>,

    /// Storage metadata
    pub metadata: HashMap<String, String>,
}
```

---

## 7. CacheResult

```rust
/// Result of cache operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheResult {
    /// Cache operation type
    pub operation: CacheOperation,

    /// Whether operation was successful
    pub success: bool,

    /// Cache key
    pub key: String,

    /// Cache hit/miss (for get operations)
    pub cache_hit: Option<bool>,

    /// Size of cached data (bytes)
    pub size_bytes: Option<usize>,

    /// TTL set (for set operations)
    pub ttl: Option<Duration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheOperation {
    Get,
    Set,
    Delete,
    Exists,
}
```

---

## 8. Complete Pipeline Result

```rust
/// Complete pipeline execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineExecutionResult {
    /// Pipeline execution ID
    pub execution_id: String,

    /// Source URL or identifier
    pub source: String,

    /// Pipeline configuration used
    pub pipeline_config: String,

    /// Individual stage results in order
    pub stages: Vec<PipelineStageResult>,

    /// Final output (last successful stage)
    pub final_output: Option<Box<PipelineStageResult>>,

    /// Overall success
    pub success: bool,

    /// Total execution time
    pub duration: Duration,

    /// Errors encountered
    pub errors: Vec<String>,

    /// Started at
    #[serde(with = "chrono::serde::ts_seconds")]
    pub started_at: DateTime<Utc>,

    /// Completed at
    #[serde(with = "chrono::serde::ts_seconds")]
    pub completed_at: DateTime<Utc>,
}

impl PipelineExecutionResult {
    /// Create new pipeline result
    pub fn new(execution_id: impl Into<String>, source: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            execution_id: execution_id.into(),
            source: source.into(),
            pipeline_config: "default".to_string(),
            stages: Vec::new(),
            final_output: None,
            success: false,
            duration: Duration::from_secs(0),
            errors: Vec::new(),
            started_at: now,
            completed_at: now,
        }
    }

    /// Add stage result
    pub fn add_stage(&mut self, result: PipelineStageResult) {
        self.stages.push(result);
    }

    /// Mark as completed
    pub fn complete(mut self, success: bool) -> Self {
        self.success = success;
        self.completed_at = Utc::now();
        self.duration = (self.completed_at - self.started_at)
            .to_std()
            .unwrap_or_default();

        // Set final output to last successful stage
        if success && !self.stages.is_empty() {
            self.final_output = self.stages.last().cloned().map(Box::new);
        }

        self
    }

    /// Get stage result by type
    pub fn get_stage<F>(&self, predicate: F) -> Option<&PipelineStageResult>
    where
        F: Fn(&PipelineStageResult) -> bool,
    {
        self.stages.iter().find(|stage| predicate(stage))
    }

    /// Get extracted content if available
    pub fn extracted_content(&self) -> Option<&ExtractedContent> {
        self.get_stage(|s| matches!(s, PipelineStageResult::Extracted(_)))
            .and_then(|s| match s {
                PipelineStageResult::Extracted(content) => Some(content),
                _ => None,
            })
    }
}
```

---

## 9. Migration Example: Before & After

### Before (Untyped JSON)
```rust
// ❌ BEFORE: Untyped, error-prone
async fn execute_fetch(&self, url: &str) -> RiptideResult<serde_json::Value> {
    let response = self.http_client.get(url).await?;
    Ok(serde_json::json!({
        "url": url,
        "content": String::from_utf8_lossy(&response.body),
        "timestamp": SystemTime::now(),
    }))
}

// Usage - must parse JSON
let result = facade.execute_fetch(url).await?;
let content = result["content"].as_str().unwrap(); // Runtime error if wrong type!
```

### After (Typed)
```rust
// ✅ AFTER: Type-safe, self-documenting
async fn execute_fetch(&self, url: &str) -> RiptideResult<FetchedContent> {
    let response = self.http_client.get(url).await?;
    Ok(FetchedContent::new(url, response.body, response.content_type))
}

// Usage - compile-time safety
let result = facade.execute_fetch(url).await?;
let content = result.as_text()?; // Type-safe, clear error handling
```

---

## 10. Serialization Strategy

**Rule**: Serialization happens **only in handler layer**, never in facade.

### Handler Layer (API)
```rust
// Handler converts domain type to JSON
async fn extract_handler(
    State(facade): State<Arc<ExtractionFacade>>,
    Json(req): Json<ExtractRequest>,
) -> Result<Json<ExtractResponse>, ApiError> {
    // Facade returns typed result
    let extracted = facade.extract(&req.url).await?;

    // Handler serializes to JSON for HTTP response
    Ok(Json(ExtractResponse::from(extracted)))
}
```

---

## 11. Testing Benefits

### Type-Safe Test Construction
```rust
#[test]
fn test_extracted_content_confidence() {
    let content = ExtractedContent::new("http://example.com", "Test content")
        .with_confidence(0.95);

    assert!(content.is_high_confidence());
}
```

### Mock-Friendly
```rust
#[tokio::test]
async fn test_facade_with_mock() {
    let mock_http = MockHttpClient::new();
    mock_http.expect_response(HttpResponse {
        status: 200,
        body: b"<html>Test</html>".to_vec(),
        ...
    });

    let facade = ExtractionFacade::new(Arc::new(mock_http));
    let result = facade.extract("http://example.com").await.unwrap();

    // Type-safe assertions
    assert!(result.is_html());
    assert_eq!(result.source, "http://example.com");
}
```

---

## 12. Implementation Checklist

### Phase 1: Core Types (6 hours)
- [ ] Implement `PipelineStageResult` enum
- [ ] Implement `FetchedContent`
- [ ] Implement `ExtractedContent` with supporting types
- [ ] Implement `ValidationResult` and `GateAnalysisResult`
- [ ] Implement `StorageConfirmation` and `CacheResult`
- [ ] Add comprehensive unit tests

### Phase 2: Pipeline Integration (4 hours)
- [ ] Implement `PipelineExecutionResult`
- [ ] Update pipeline facade methods to return typed results
- [ ] Update internal pipeline stages
- [ ] Add integration tests

### Phase 3: Facade Migration (4 hours)
- [ ] Replace `serde_json::Value` in all facade return types
- [ ] Update facade method implementations
- [ ] Update facade tests

---

## Conclusion

This typed result hierarchy eliminates all 37+ `serde_json::Value` usages in the facade layer, providing:

✅ **Compile-time type safety**
✅ **Self-documenting code**
✅ **Better IDE support**
✅ **Easier testing**
✅ **Clear separation of concerns** (serialization in handlers only)

**Implementation time**: 14 hours with comprehensive tests.
