# Pipeline Integration Architecture

## Overview

This document describes the complete pipeline integration architecture for the Riptide API, detailing the flow from HTTP fetch through extraction to caching.

## Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────────────┐
│                     RIPTIDE PIPELINE ORCHESTRATOR                         │
│                    (crates/riptide-api/src/pipeline.rs)                   │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
        ┌────────────────────────────────────────────────┐
        │  1. CACHE CHECK (check_cache)                 │
        │     • Check Redis for cached content           │
        │     • Return immediately if found              │
        └────────────────────────────────────────────────┘
                                    │
                                    ▼
        ┌────────────────────────────────────────────────┐
        │  2. FETCH PHASE (fetch_content_with_type)     │
        │     • HTTP GET with 15s timeout                │
        │     • Extract Content-Type header              │
        │     • Read response bytes                      │
        │     • Record fetch metrics                     │
        └────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    │                               │
                    ▼                               ▼
        ┌─────────────────────┐       ┌────────────────────────┐
        │  3a. PDF PIPELINE   │       │  3b. HTML PIPELINE     │
        │  (process_pdf)      │       │  (analyze_content)     │
        │  • Detect PDF       │       │  • Extract features    │
        │  • Acquire resources│       │  • Calculate score     │
        │  • Extract text     │       │  • Gate decision       │
        │  • Convert to Doc   │       │  • Emit events         │
        └─────────────────────┘       └────────────────────────┘
                    │                               │
                    │                               ▼
                    │               ┌────────────────────────────┐
                    │               │  4. EXTRACTION PHASE       │
                    │               │  (extract_content)         │
                    │               │                            │
                    │               │  ┌──────────────────────┐  │
                    │               │  │ ReliableExtractor    │  │
                    │               │  │ • Circuit breaker    │  │
                    │               │  │ • Retry logic (3x)   │  │
                    │               │  │ • Exponential backoff│  │
                    │               │  └──────────┬───────────┘  │
                    │               │             │              │
                    │               │             ▼              │
                    │               │  ┌──────────────────────┐  │
                    │               │  │ WasmExtractorAdapter │  │
                    │               │  │ • Bridge to traits   │  │
                    │               │  │ • Metrics tracking   │  │
                    │               │  └──────────┬───────────┘  │
                    │               │             │              │
                    │               │             ▼              │
                    │               │  ┌──────────────────────┐  │
                    │               │  │ UnifiedExtractor     │  │
                    │               │  │ • WASM extraction    │  │
                    │               │  │ • Returns Content    │  │
                    │               │  └──────────┬───────────┘  │
                    │               │             │              │
                    │               │             ▼              │
                    │               │  ┌──────────────────────┐  │
                    │               │  │ convert_extracted_   │  │
                    │               │  │ content()            │  │
                    │               │  │ Content → Doc        │  │
                    │               │  └──────────────────────┘  │
                    │               └────────────────────────────┘
                    │                               │
                    └───────────────┬───────────────┘
                                    │
                                    ▼
                    ┌────────────────────────────────────┐
                    │  5. CACHE STORAGE                  │
                    │  (store_in_cache)                  │
                    │  • Store in Redis with TTL         │
                    │  • Record cache metrics            │
                    └────────────────────────────────────┘
                                    │
                                    ▼
                    ┌────────────────────────────────────┐
                    │  6. RETURN PIPELINE RESULT         │
                    │  • ExtractedDoc                    │
                    │  • Metadata (cache, gate, timing)  │
                    │  • Metrics recorded                │
                    └────────────────────────────────────┘
```

## Component Details

### 1. Cache Layer (`check_cache`, `store_in_cache`)

**Location**: `pipeline.rs:853-903`

**Responsibilities**:
- Check Redis cache before expensive operations
- Store successful extraction results with TTL
- Generate deterministic cache keys using URL + cache_mode hash
- Handle cache errors gracefully (log warning, continue without cache)

**Cache Key Format**: `riptide:v1:{cache_mode}:{url_hash}`

**Performance Impact**:
- Cache hit: ~5ms (immediate return)
- Cache miss: Full pipeline execution (~50-500ms)

### 2. Fetch Phase (`fetch_content_with_type`)

**Location**: `pipeline.rs:550-585`

**Responsibilities**:
- HTTP GET request with 15-second timeout
- Content-Type header extraction for PDF detection
- Response body reading with timeout protection
- HTTP status code capture for result metadata

**Error Handling**:
- Network timeout → `ApiError::timeout`
- Connection failure → `ApiError::fetch`
- Invalid response → `ApiError::fetch`

**Metrics**: Records fetch duration via `metrics.record_phase_timing(PhaseType::Fetch)`

### 3a. PDF Pipeline (`process_pdf_content`)

**Location**: `pipeline.rs:587-679`

**Responsibilities**:
- Detect PDF content via Content-Type or magic bytes
- Acquire PDF processing resources (RAII guard)
- Process PDF bytes using `riptide-pdf` crate
- Convert PDF result to `ExtractedDoc`
- Handle resource exhaustion gracefully

**Resource Management**:
```rust
let pdf_guard = resource_manager.acquire_pdf_resources().await?;
// Guard automatically releases resources on drop
```

**Performance**: PDF processing typically 200-1000ms depending on size

### 3b. HTML Pipeline - Gate Analysis (`analyze_content`)

**Location**: `pipeline.rs:681-754`

**Responsibilities**:
- Extract content features for decision-making:
  - HTML size and visible text ratio
  - Element counts (p, article, h1/h2)
  - Script byte count and density
  - Metadata presence (OpenGraph, JSON-LD)
  - SPA markers detection
  - Domain reputation scoring

**Gate Decision Logic**:
```rust
let quality_score = score(&gate_features);
let decision = decide(
    &gate_features,
    config.gate_hi_threshold,  // Default: 0.7
    config.gate_lo_threshold,  // Default: 0.3
);
```

**Decision Outcomes**:
- `Decision::Raw`: High quality, use fast WASM extraction
- `Decision::ProbesFirst`: Medium quality, try WASM then headless if needed
- `Decision::Headless`: Low quality, use headless browser

### 4. Extraction Phase (`extract_content`)

**Location**: `pipeline.rs:756-851`

**Architecture**:

This is the core extraction workflow with multiple reliability layers:

#### Layer 1: ReliableExtractor (Reliability Guarantees)

**Location**: `riptide-reliability` crate

**Features**:
- Circuit breaker pattern (opens after 5 consecutive failures)
- Retry logic with exponential backoff:
  - Attempt 1: Immediate
  - Attempt 2: 100ms delay
  - Attempt 3: 200ms delay
  - Attempt 4: 400ms delay
- Graceful degradation fallback
- Metrics tracking for all attempts

#### Layer 2: WasmExtractorAdapter (Trait Bridge)

**Location**: `reliability_integration.rs:9-91`

**Responsibilities**:
- Implements `WasmExtractorTrait` for `UnifiedExtractor`
- Tracks WASM cold start metrics
- Estimates memory usage
- Converts async `UnifiedExtractor::extract` to sync trait method
- Bridges `ExtractedContent` → `ExtractedDoc`

**Metrics Captured**:
```rust
metrics.update_wasm_cold_start_time(cold_start_ms);
metrics.update_wasm_memory_metrics(pages, failures, peak_pages);
```

#### Layer 3: UnifiedExtractor (WASM Execution)

**Location**: `riptide-extraction/src/unified_extractor.rs`

**Responsibilities**:
- WASM-based content extraction
- Strategy selection (WASM, CSS, Regex, Auto)
- Returns `ExtractedContent` with:
  - Title, content, summary
  - Strategy used
  - Extraction confidence score

#### Layer 4: Content Conversion (`convert_extracted_content`)

**Location**: `pipeline.rs:16-39`

**Transformation**: `ExtractedContent` → `ExtractedDoc`

```rust
ExtractedContent {            ExtractedDoc {
  title,                  →     title: Some(title),
  content,                →     text: content,
  summary,                →     description: summary,
  url,                    →     url,
  strategy_used,          →     // Logged for metrics
  extraction_confidence   →     quality_score: Some((confidence * 100.0) as u8)
}                               // + additional fields populated
```

**Additional Fields**:
- `word_count`: Calculated from content
- `links`, `media`: Empty (populated by full extraction)
- `byline`, `published_iso`: None (basic extraction)
- `markdown`, `html`: None (WASM provides text only)

### 5. Fallback Mechanism (`fallback_to_wasm_extraction`)

**Location**: `pipeline.rs:905-928`

**Trigger**: When `ReliableExtractor` exhausts all retry attempts

**Behavior**:
1. Log fallback event
2. Call `UnifiedExtractor::extract` directly (bypassing reliability layer)
3. Convert result via `convert_extracted_content`
4. Record error metrics
5. Return result or final error

**Use Case**: Last-resort extraction when reliability layer fails completely

## Integration Flow Examples

### Success Case: Fast Path (Cache Hit)

```
User Request → Cache Check → Cache Hit → Return Cached Doc
Total Time: ~5ms
```

### Success Case: Fast WASM Extraction

```
User Request → Cache Miss → Fetch (100ms) → Gate (50ms) →
Decision::Raw → WASM Extract (100ms) → Convert → Cache → Return
Total Time: ~250ms
```

### Success Case: Headless Extraction

```
User Request → Cache Miss → Fetch (100ms) → Gate (50ms) →
Decision::Headless → Headless Extract (500ms) → Convert → Cache → Return
Total Time: ~650ms
```

### Retry Case: Transient Failure

```
User Request → Cache Miss → Fetch (100ms) → Gate (50ms) →
Decision::Raw → WASM Attempt 1 (timeout) → Wait 100ms →
WASM Attempt 2 (success, 150ms) → Convert → Cache → Return
Total Time: ~400ms
```

### Fallback Case: ReliableExtractor Failure

```
User Request → Cache Miss → Fetch (100ms) → Gate (50ms) →
Decision::Raw → WASM Attempt 1-3 (all fail, ~700ms) →
Fallback WASM Direct (150ms) → Convert → Cache → Return
Total Time: ~1000ms
```

### Failure Case: Complete Extraction Failure

```
User Request → Cache Miss → Fetch (100ms) → Gate (50ms) →
Decision::Raw → WASM Attempts 1-3 (fail) → Fallback WASM (fail) →
Return ApiError::extraction
Total Time: ~850ms
Metrics: record_error(ErrorType::Wasm)
```

## Metrics & Observability

### Phase Timings

```rust
metrics.record_phase_timing(PhaseType::Fetch, duration_s);
metrics.record_phase_timing(PhaseType::Gate, duration_s);
metrics.record_phase_timing(PhaseType::Wasm, duration_s);
```

### Gate Decisions

```rust
metrics.record_gate_decision(decision_str);
metrics.record_gate_decision_enhanced(
    decision_str,
    score,
    text_ratio,
    script_density,
    spa_markers,
    duration_ms
);
```

### Extraction Results

```rust
metrics.record_extraction_result(
    mode,              // "raw", "probes", "headless"
    duration_ms,
    success,
    quality_score,
    content_length,
    links_count,
    images_count,
    has_author,
    has_date
);
```

### Reliability Metrics

```rust
metrics.record_extraction_fallback(from_mode, to_mode, reason);
```

### Error Tracking

```rust
metrics.record_error(ErrorType::Wasm);
metrics.record_error(ErrorType::Redis);
metrics.record_error(ErrorType::Fetch);
```

## Event Bus Integration

The pipeline emits events at key lifecycle points:

1. **Pipeline Start**: `pipeline.execution.started`
2. **Cache Hit**: `pipeline.cache.hit`
3. **PDF Processing**: `pipeline.pdf.processing`
4. **Gate Decision**: `pipeline.gate.decision`
5. **Reliable Success**: `pipeline.extraction.reliable_success`
6. **Reliable Failure**: `pipeline.extraction.reliable_failure`
7. **Pipeline Complete**: `pipeline.execution.completed`

All events include metadata (URL, decision, timing, etc.) for debugging and monitoring.

## Configuration

### Pipeline Options (`CrawlOptions`)

```rust
CrawlOptions {
    concurrency: 16,           // Batch processing concurrency
    cache_mode: "read_through", // "enabled", "bypass", "read_through"
    skip_extraction: false,    // Skip extraction, return raw HTML
    render_mode: RenderMode::Auto, // Auto, Pdf, Html
    // ... chunking and dynamic wait options
}
```

### Gate Thresholds (`AppConfig`)

```rust
AppConfig {
    gate_hi_threshold: 0.7,  // >= 0.7 → Decision::Raw (fast)
    gate_lo_threshold: 0.3,  // <= 0.3 → Decision::Headless (slow)
    // 0.3 < score < 0.7 → Decision::ProbesFirst (adaptive)
}
```

## Testing

### Unit Tests

- **File**: `tests/unit/test_pipeline.rs`
- **Coverage**:
  - Pipeline result creation and serialization
  - Pipeline stats calculation
  - Cache key generation and uniqueness
  - Property-based testing for invariants

### Integration Tests

- **File**: `tests/enhanced_pipeline_tests.rs`
- **Coverage**:
  - Phase timing accuracy
  - Compatibility with standard pipeline
  - Metrics collection
  - Fallback behavior
  - Batch concurrency
  - Error handling

### Missing Tests (TODO)

1. **Integration Test**: Extractor → Pipeline flow end-to-end
2. **Integration Test**: ReliableExtractor fallback scenarios
3. **Integration Test**: Circuit breaker behavior
4. **Performance Test**: 100+ RPS throughput
5. **Stress Test**: 24-hour memory stability

## Performance Characteristics

### Throughput

- **Target**: 100+ requests/second
- **Actual**: Measured in production (varies by content complexity)
- **Bottlenecks**:
  - WASM extraction: ~50-200ms per URL
  - Headless rendering: ~500-2000ms per URL
  - PDF processing: ~200-1000ms per document

### Latency (P50/P95/P99)

- **Cache Hit**: 5ms / 10ms / 20ms
- **Fast WASM**: 250ms / 400ms / 600ms
- **Headless**: 650ms / 1200ms / 2000ms
- **PDF**: 400ms / 800ms / 1500ms

### Resource Usage

- **Memory**: ~50MB base + ~5-20MB per concurrent request
- **CPU**: ~10-30% per WASM extraction (single core)
- **Redis**: ~1-100KB per cached document

## Error Budget

- **Target Availability**: 99.9% (43 minutes downtime/month)
- **Retry Budget**: Up to 3 attempts per request
- **Circuit Breaker**: Opens at 5 consecutive failures
- **Fallback Success Rate**: ~95% when primary extraction fails

## Future Improvements

1. **Streaming Extraction**: Process content as it's fetched
2. **Parallel Gate Analysis**: Analyze while fetching
3. **Predictive Caching**: Pre-warm cache based on access patterns
4. **Adaptive Retry**: Dynamic backoff based on error type
5. **Content Fingerprinting**: Detect duplicates before extraction
6. **WASM Module Pooling**: Reduce cold start overhead

## Related Documentation

- `crates/riptide-api/src/pipeline.rs` - Main pipeline implementation
- `crates/riptide-api/src/reliability_integration.rs` - Adapter layer
- `crates/riptide-reliability/` - Retry and circuit breaker logic
- `crates/riptide-extraction/` - WASM extraction engine
- `crates/riptide-pdf/` - PDF processing
