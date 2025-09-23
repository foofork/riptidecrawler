# WASM Extractor API Integration Guide

## Overview

The RipTide API integrates a high-performance WebAssembly (WASM) component for content extraction, built using the Component Model specification. This guide covers integration patterns, performance optimization, and troubleshooting.

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   API Layer     │    │  WASM Runtime    │    │  Trek-rs Core   │
│                 │────▶│                  │────▶│                 │
│ - Route Handler │    │ - Component Host │    │ - HTML Parsing  │
│ - Validation    │    │ - Memory Mgmt    │    │ - Content Ext.  │
│ - Error Mapping │    │ - Stats Tracking │    │ - Readability   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## WASM Component Interface

### Component Information

```rust
// Component metadata
{
  "name": "riptide-extractor-wasm",
  "version": "1.0.0",
  "component_model_version": "0.2.0",
  "features": [
    "article-extraction",
    "full-page-extraction",
    "metadata-extraction",
    "custom-selectors",
    "trek-rs-integration"
  ]
}
```

### Core Functions

#### 1. Primary Extraction

```rust
fn extract(
    html: String,
    url: String,
    mode: ExtractionMode
) -> Result<ExtractedContent, ExtractionError>
```

**Parameters:**
- `html`: Source HTML content
- `url`: Base URL for link resolution
- `mode`: Extraction strategy (Article, Full, Metadata, Custom)

**Example Usage:**

```javascript
// JavaScript integration
const wasmModule = await loadWasmComponent();

const result = wasmModule.extract(
    htmlContent,
    "https://example.com/article",
    { Article: null }
);

if (result.tag === "ok") {
    console.log("Title:", result.val.title);
    console.log("Content:", result.val.markdown);
} else {
    console.error("Extraction failed:", result.val);
}
```

#### 2. Performance Extraction

```rust
fn extract_with_stats(
    html: String,
    url: String,
    mode: ExtractionMode
) -> Result<(ExtractedContent, ExtractionStats), ExtractionError>
```

**Returns detailed performance metrics:**

```json
{
  "content": {
    "url": "https://example.com/article",
    "title": "Article Title",
    "markdown": "# Article Title\n\nContent...",
    "text": "Article Title. Content...",
    "quality_score": 85,
    "word_count": 1200,
    "reading_time": 5
  },
  "stats": {
    "processing_time_ms": 45,
    "memory_used": 2048,
    "nodes_processed": 1250,
    "links_found": 15,
    "images_found": 8
  }
}
```

#### 3. Content Validation

```rust
fn validate_html(html: String) -> Result<bool, ExtractionError>
```

Fast pre-validation before extraction:

```javascript
const isValid = wasmModule.validate_html(htmlContent);
if (isValid.tag === "ok" && isValid.val) {
    // Proceed with extraction
    const result = wasmModule.extract(htmlContent, url, mode);
}
```

#### 4. Health Monitoring

```rust
fn health_check() -> HealthStatus
```

Component health and capabilities:

```json
{
  "status": "healthy",
  "version": "1.0.0",
  "trek_version": "1.2.3",
  "capabilities": [
    "article - Extract article content using readability algorithms",
    "full - Extract full page content including sidebars",
    "metadata - Extract only metadata and structured data",
    "custom - Custom extraction using CSS selectors"
  ],
  "memory_usage": 1048576,
  "extraction_count": 1250
}
```

## Extraction Modes

### Article Mode (Recommended)

Optimized for article content using readability algorithms:

```rust
ExtractionMode::Article
```

**Configuration:**
- Readability-based content detection
- Clean markup with article focus
- Optimized for news, blogs, documentation
- Automatic noise removal

**Output Quality:**
- High precision for main content
- Clean markdown formatting
- Proper heading hierarchy
- Link preservation

### Full Mode

Complete page extraction including navigation and sidebars:

```rust
ExtractionMode::Full
```

**Use Cases:**
- Documentation sites
- Directory pages
- Complex layouts
- When preserving page structure

### Metadata Mode

Lightweight extraction of structured data only:

```rust
ExtractionMode::Metadata
```

**Extracted Fields:**
- Title and description
- Author and publication date
- Open Graph and Twitter Card data
- Schema.org structured data
- Basic page metadata

### Custom Mode

CSS selector-based extraction:

```rust
ExtractionMode::Custom(vec![
    ".article-content".to_string(),
    "h1, h2, h3".to_string(),
    ".author-info".to_string()
])
```

## Performance Optimization

### Memory Management

The WASM component implements efficient memory patterns:

```rust
// Automatic memory cleanup after extraction
static COMPONENT_STATE: Lazy<std::sync::Mutex<ComponentState>> =
    Lazy::new(|| std::sync::Mutex::new(ComponentState::new()));

// Memory usage tracking
fn get_memory_usage() -> u64 {
    // Platform-specific memory monitoring
    current_memory_usage()
}
```

### Extraction Caching

While the WASM component doesn't cache internally, implement caching at the API layer:

```rust
// API-level caching strategy
let cache_key = format!("extract:{}:{}", url_hash, mode_hash);

if let Some(cached) = redis_client.get(&cache_key).await? {
    return Ok(cached);
}

let result = wasm_component.extract(html, url, mode)?;
redis_client.setex(&cache_key, 3600, &result).await?;
```

### Batch Processing

For multiple extractions, consider these patterns:

```rust
// Sequential processing with shared component
for (html, url) in documents {
    let result = component.extract(html, url, mode)?;
    results.push(result);
}

// Parallel processing with component per thread
let results: Vec<_> = documents
    .par_iter()
    .map(|(html, url)| {
        let component = create_component();
        component.extract(html.clone(), url.clone(), mode)
    })
    .collect();
```

## Error Handling

### Error Types

```rust
enum ExtractionError {
    InvalidHtml(String),      // Malformed or empty HTML
    ExtractorError(String),   // Trek-rs parsing errors
    InternalError(String),    // Component internal errors
}
```

### Error Mapping

Map WASM errors to API errors:

```rust
fn map_wasm_error(wasm_error: ExtractionError) -> ApiError {
    match wasm_error {
        ExtractionError::InvalidHtml(msg) => {
            ApiError::validation(format!("Invalid HTML: {}", msg))
        }
        ExtractionError::ExtractorError(msg) => {
            ApiError::extraction(format!("Extraction failed: {}", msg))
        }
        ExtractionError::InternalError(msg) => {
            ApiError::InternalError { message: msg }
        }
    }
}
```

### Retry Strategies

```rust
async fn extract_with_retry(
    html: &str,
    url: &str,
    mode: &ExtractionMode,
    max_retries: u32
) -> Result<ExtractedContent, ApiError> {
    let mut last_error = None;

    for attempt in 0..=max_retries {
        match wasm_component.extract(html.to_string(), url.to_string(), mode.clone()) {
            Ok(content) => return Ok(content),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    // Exponential backoff
                    tokio::time::sleep(Duration::from_millis(100 * 2_u64.pow(attempt))).await;
                }
            }
        }
    }

    Err(map_wasm_error(last_error.unwrap()))
}
```

## Integration Patterns

### API Handler Integration

```rust
pub async fn crawl(
    State(state): State<AppState>,
    Json(body): Json<CrawlBody>,
) -> Result<impl IntoResponse, ApiError> {
    // ... validation and fetching ...

    // WASM extraction
    let extraction_result = state
        .wasm_extractor
        .extract_with_stats(html, url, extraction_mode)
        .map_err(map_wasm_error)?;

    let (content, stats) = extraction_result;

    // Record metrics
    state.metrics.record_extraction(
        stats.processing_time_ms,
        stats.memory_used,
        stats.nodes_processed
    );

    Ok(Json(CrawlResult {
        document: Some(content),
        processing_time_ms: stats.processing_time_ms,
        quality_score: content.quality_score.unwrap_or(0) as f32 / 100.0,
        // ... other fields ...
    }))
}
```

### Middleware Integration

```rust
// Metrics middleware for WASM operations
pub fn wasm_metrics_middleware() -> impl Fn(...) {
    move |request, next| async move {
        let start = Instant::now();
        let result = next(request).await;
        let duration = start.elapsed();

        // Record WASM operation metrics
        metrics::histogram!(
            "wasm_extraction_duration_ms",
            duration.as_millis() as f64,
            "operation" => "extract"
        );

        result
    }
}
```

## Quality Scoring

The WASM component provides content quality scoring:

```rust
fn calculate_quality_score(response: &TrekResponse) -> u32 {
    let mut score = 50; // Base score

    // Content length scoring
    let content_length = response.content.len();
    if content_length > 1000 { score += 20; }
    if content_length > 5000 { score += 10; }

    // Metadata completeness
    if !response.metadata.title.is_empty() { score += 10; }
    if !response.metadata.author.is_empty() { score += 5; }
    if !response.metadata.published.is_empty() { score += 5; }

    // Word count considerations
    let word_count = response.metadata.word_count;
    if word_count > 300 { score += 10; }
    if word_count > 1000 { score += 5; }

    score.min(100)
}
```

## Monitoring and Debugging

### Component State Monitoring

```rust
// Get component information
let info = wasm_component.get_info();
println!("Component: {} v{}", info.name, info.version);
println!("Features: {:?}", info.features);

// Health check
let health = wasm_component.health_check();
println!("Status: {}", health.status);
println!("Memory Usage: {} bytes", health.memory_usage.unwrap_or(0));
println!("Extractions: {}", health.extraction_count.unwrap_or(0));
```

### Performance Metrics

```rust
// Track extraction performance
#[derive(Serialize)]
struct ExtractionMetrics {
    processing_time_ms: u64,
    memory_used: u64,
    quality_score: u32,
    word_count: u32,
    nodes_processed: u32,
}

let (content, stats) = wasm_component.extract_with_stats(html, url, mode)?;

let metrics = ExtractionMetrics {
    processing_time_ms: stats.processing_time_ms,
    memory_used: stats.memory_used,
    quality_score: content.quality_score.unwrap_or(0),
    word_count: content.word_count.unwrap_or(0),
    nodes_processed: stats.nodes_processed.unwrap_or(0),
};

// Send to monitoring system
send_metrics("wasm.extraction", &metrics).await;
```

### State Management

```rust
// Reset component state when needed
if memory_usage > threshold || error_rate > limit {
    match wasm_component.reset_state() {
        Ok(message) => {
            tracing::info!("Component state reset: {}", message);
        }
        Err(e) => {
            tracing::error!("Failed to reset component state: {:?}", e);
        }
    }
}
```

## Best Practices

### 1. Resource Management

- Monitor memory usage and reset state periodically
- Implement circuit breakers for high error rates
- Use connection pooling for multiple component instances

### 2. Error Handling

- Always validate HTML before extraction
- Implement proper retry logic with exponential backoff
- Log detailed error information for debugging

### 3. Performance

- Choose appropriate extraction modes for content type
- Cache extraction results to avoid redundant processing
- Monitor processing times and optimize based on metrics

### 4. Security

- Validate and sanitize all input HTML
- Implement rate limiting to prevent abuse
- Monitor for malicious content patterns

### 5. Observability

- Track extraction success rates by mode
- Monitor memory usage and processing times
- Implement alerts for component health issues

## Troubleshooting

### Common Issues

**High Memory Usage:**
```rust
// Check component memory
let health = wasm_component.health_check();
if let Some(memory) = health.memory_usage {
    if memory > MAX_MEMORY_THRESHOLD {
        wasm_component.reset_state()?;
    }
}
```

**Extraction Failures:**
```rust
// Debug extraction issues
if let Err(e) = wasm_component.validate_html(&html) {
    tracing::warn!("HTML validation failed: {:?}", e);
    return Err(ApiError::validation("Invalid HTML content"));
}
```

**Performance Degradation:**
```rust
// Monitor processing times
let start = Instant::now();
let result = wasm_component.extract(html, url, mode)?;
let duration = start.elapsed();

if duration > Duration::from_millis(SLOW_EXTRACTION_THRESHOLD_MS) {
    tracing::warn!(
        "Slow extraction detected: {}ms for URL: {}",
        duration.as_millis(),
        url
    );
}
```

This integration guide provides comprehensive coverage of WASM extractor usage patterns, performance optimization, and production deployment considerations.