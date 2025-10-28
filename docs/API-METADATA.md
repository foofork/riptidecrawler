# RipTide API Metadata Reference

**Version:** 1.0.0
**Last Updated:** 2025-10-28
**API Version:** v1

---

## Table of Contents

1. [Overview](#overview)
2. [New Metadata Fields](#new-metadata-fields)
3. [Parser Information](#parser-information)
4. [Confidence Scores](#confidence-scores)
5. [Performance Metrics](#performance-metrics)
6. [Examples](#examples)
7. [Migration Guide](#migration-guide)

---

## Overview

The RipTide API now includes **enhanced metadata** in extraction responses, providing detailed information about parser selection, fallback usage, and quality scoring. This metadata enables:

- **Debugging**: Understand which parser was used and why
- **Optimization**: Identify slow extraction paths
- **Quality Assessment**: Evaluate extraction confidence and quality
- **Monitoring**: Track parser fallback rates and performance

---

## New Metadata Fields

### Complete Metadata Structure

```json
{
  "text": "Extracted content here...",
  "html": "<div>Extracted HTML...</div>",
  "metadata": {
    // Basic Information
    "url": "https://example.com",
    "title": "Example Domain",
    "timestamp": "2025-10-28T14:30:00.123Z",

    // Performance Metrics
    "extraction_time_ms": 6,
    "quality_score": 0.95,
    "word_count": 1234,

    // NEW: Parser Selection Information
    "parser_used": "native",
    "parser_path": "fast",
    "fallback_used": true,
    "primary_parser": "wasm",
    "fallback_parser": "native",

    // NEW: Confidence Scoring
    "confidence_score": 0.85,

    // Content Analysis
    "links": ["https://example.com/page1"],
    "images": ["https://example.com/image1.jpg"],
    "content_hash": "abc123def456"
  }
}
```

### Field Definitions

#### Parser Selection Fields

| Field | Type | Required | Description | Example Values |
|-------|------|----------|-------------|----------------|
| `parser_used` | string | Yes | Actual parser that succeeded | `"native"`, `"wasm"` |
| `parser_path` | string | Yes | Extraction path chosen by gate | `"fast"`, `"headless"`, `"probes_first"` |
| `fallback_used` | boolean | Yes | Whether fallback parser was invoked | `true`, `false` |
| `primary_parser` | string | Conditional | Primary parser attempted (if fallback used) | `"wasm"`, `"native"` |
| `fallback_parser` | string | Conditional | Fallback parser used (if fallback used) | `"native"`, `"wasm"` |

#### Scoring Fields

| Field | Type | Required | Description | Range |
|-------|------|----------|-------------|-------|
| `confidence_score` | float | Yes | Gate decision confidence | 0.0 - 1.0 |
| `quality_score` | float | Yes | Extraction quality assessment | 0.0 - 1.0 |

#### Performance Fields

| Field | Type | Required | Description | Unit |
|-------|------|----------|-------------|------|
| `extraction_time_ms` | integer | Yes | Total extraction time | milliseconds |
| `word_count` | integer | Yes | Extracted text word count | count |

---

## Parser Information

### `parser_used` Field

**Description:** The parser that successfully extracted content.

**Values:**

| Value | Description | Characteristics |
|-------|-------------|-----------------|
| `"native"` | Native Rust `scraper` crate | Fast, trusted HTML, full feature support |
| `"wasm"` | WASM `tl` parser | Sandboxed, secure, lightweight (currently unavailable due to Unicode issue) |

**Current Behavior:**
- **Fast path**: Always `"native"` (due to WASM Unicode error)
- **Headless path**: Always `"native"` (primary parser)

**Future Behavior (after WASM fix):**
- **Fast path**: Primarily `"wasm"` (>80%)
- **Headless path**: Primarily `"native"` (>95%)

### `parser_path` Field

**Description:** The extraction path chosen by the gate decision engine.

**Values:**

| Value | Description | Confidence Range | Flow |
|-------|-------------|------------------|------|
| `"fast"` | Direct HTTP fetch | ≥ 0.8 | HTTP fetch → WASM primary → Native fallback |
| `"probes_first"` | Try fast, fallback headless | 0.5 - 0.8 | Try fast first, if fails → headless |
| `"headless"` | Chrome headless rendering | < 0.5 | Chrome render → Native primary → WASM fallback |

**Selection Logic:**

```
confidence ≥ 0.8 → "fast"        (simple static pages)
0.5 ≤ confidence < 0.8 → "probes_first" (uncertain, try both)
confidence < 0.5 → "headless"    (JavaScript-heavy SPA)
```

### `fallback_used` Field

**Description:** Indicates whether the primary parser failed and fallback was used.

**Values:**

| Value | Meaning | Implication |
|-------|---------|-------------|
| `true` | Primary parser failed, fallback succeeded | Normal operation (WASM Unicode issue) |
| `false` | Primary parser succeeded immediately | Ideal case (after WASM fix) |

**Current Production Behavior:**
- **Fast path**: Always `true` (WASM fails → Native fallback)
- **Headless path**: Usually `false` (Native succeeds)

### `primary_parser` & `fallback_parser` Fields

**Description:** Details about parser cascade (only present when `fallback_used: true`).

**Example Scenarios:**

1. **Fast path (current):**
   ```json
   {
     "parser_used": "native",
     "fallback_used": true,
     "primary_parser": "wasm",
     "fallback_parser": "native"
   }
   ```

2. **Headless path (rare fallback):**
   ```json
   {
     "parser_used": "wasm",
     "fallback_used": true,
     "primary_parser": "native",
     "fallback_parser": "wasm"
   }
   ```

3. **No fallback (ideal):**
   ```json
   {
     "parser_used": "wasm",
     "fallback_used": false
     // primary_parser and fallback_parser fields absent
   }
   ```

---

## Confidence Scores

### `confidence_score` Field

**Description:** Numerical confidence (0.0 - 1.0) in content extraction without JavaScript rendering.

**Calculation Factors:**

```rust
Base confidence: 0.5

Confidence boosters (+):
+ 0.3 if URL contains "wikipedia.org" (known static site)
+ 0.2 if URL ends with ".html" (static HTML)
+ 0.1 if HTTPS (better-maintained sites)
+ 0.1 if URL is short (<50 chars)
+ 0.2 if no query parameters

Confidence reducers (-):
- 0.4 if URL contains "javascript:" (dynamic content)
- 0.3 if known SPA framework detected
- 0.2 if "#/" in URL (client-side routing)
- 0.2 if URL contains "ajax" or "api"
- 0.1 if complex query string

Final: clamp to 0.0 - 1.0
```

**Interpretation:**

| Range | Confidence Level | Recommended Path | Expected Parser |
|-------|------------------|------------------|-----------------|
| 0.9 - 1.0 | Very High | `fast` | WASM (after fix) |
| 0.8 - 0.9 | High | `fast` | WASM (after fix) |
| 0.7 - 0.8 | Medium-High | `probes_first` | Try fast first |
| 0.5 - 0.7 | Medium | `probes_first` | Try fast first |
| 0.3 - 0.5 | Low | `headless` | Native (Chrome) |
| 0.0 - 0.3 | Very Low | `headless` | Native (Chrome) |

**Examples:**

```json
// Static Wikipedia page (high confidence)
{
  "url": "https://en.wikipedia.org/wiki/Example",
  "confidence_score": 0.9,
  "parser_path": "fast"
}

// React SPA (low confidence)
{
  "url": "https://example.com/app#/dashboard",
  "confidence_score": 0.3,
  "parser_path": "headless"
}

// Simple blog (medium confidence)
{
  "url": "https://blog.example.com/post/123",
  "confidence_score": 0.65,
  "parser_path": "probes_first"
}
```

---

## Performance Metrics

### `extraction_time_ms` Field

**Description:** Total time from request initiation to extraction completion (milliseconds).

**Typical Values:**

| Parser Path | Typical Range | P50 | P95 | P99 |
|-------------|---------------|-----|-----|-----|
| `fast` (native fallback) | 5-15ms | 6ms | 12ms | 20ms |
| `probes_first` (fast) | 5-15ms | 6ms | 12ms | 20ms |
| `probes_first` (headless) | 300-600ms | 375ms | 550ms | 800ms |
| `headless` | 300-600ms | 375ms | 550ms | 800ms |

**Interpretation:**

```json
// Fast extraction (direct fetch)
{
  "parser_path": "fast",
  "extraction_time_ms": 6,
  "parser_used": "native"
}

// Slow extraction (headless rendering)
{
  "parser_path": "headless",
  "extraction_time_ms": 459,
  "parser_used": "native"
}
```

**Performance Alerts:**

| Condition | Severity | Action |
|-----------|----------|--------|
| `extraction_time_ms` > 1000 | Warning | Check headless service health |
| `extraction_time_ms` > 5000 | Critical | Investigate timeout issues |
| Fast path > 50ms | Warning | Check network latency |
| Headless path > 1000ms | Warning | Chrome resource pressure |

### `quality_score` Field

**Description:** Assessment of extraction quality (0.0 - 1.0) based on content analysis.

**Calculation Factors:**

```rust
Quality score = (
    0.3 * content_block_completeness +
    0.2 * structure_preservation +
    0.2 * link_extraction_success +
    0.15 * image_extraction_success +
    0.15 * metadata_richness
)
```

**Interpretation:**

| Score | Quality | Interpretation |
|-------|---------|----------------|
| 1.0 | Perfect | All content blocks extracted, full structure preserved |
| 0.9 - 0.99 | Excellent | Minor metadata missing, core content perfect |
| 0.7 - 0.89 | Good | Some metadata missing, main content complete |
| 0.5 - 0.69 | Acceptable | Partial content, may need review |
| 0.3 - 0.49 | Poor | Significant content missing, needs investigation |
| < 0.3 | Failed | Extraction largely unsuccessful |

**Examples:**

```json
// Excellent extraction
{
  "quality_score": 0.95,
  "word_count": 1234,
  "links": 15,
  "images": 5,
  "title": "Complete Article Title"
}

// Poor extraction
{
  "quality_score": 0.45,
  "word_count": 50,
  "links": 0,
  "images": 0,
  "title": ""
}
```

---

## Examples

### Example 1: Successful Fast Path (Current Production)

**Request:**
```bash
curl -X POST http://localhost:8080/api/extract \
  -H "Content-Type: application/json" \
  -H "X-API-Key: ${RIPTIDE_API_KEY}" \
  -d '{
    "url": "https://example.com"
  }'
```

**Response:**
```json
{
  "text": "Example Domain\n\nThis domain is for use in illustrative examples...",
  "html": "<div><h1>Example Domain</h1><p>This domain is...</p></div>",
  "metadata": {
    "url": "https://example.com",
    "title": "Example Domain",
    "timestamp": "2025-10-28T14:30:00.123Z",

    "extraction_time_ms": 6,
    "quality_score": 0.95,
    "confidence_score": 0.85,
    "word_count": 48,

    "parser_used": "native",
    "parser_path": "fast",
    "fallback_used": true,
    "primary_parser": "wasm",
    "fallback_parser": "native",

    "links": ["https://www.iana.org/domains/example"],
    "images": [],
    "content_hash": "a3f5c8d2e1b4"
  }
}
```

**Analysis:**
- ✅ Fast path chosen (confidence: 0.85)
- ⚠️ WASM failed (Unicode error) → Native fallback
- ✅ Extraction successful (quality: 0.95)
- ✅ Fast performance (6ms)

### Example 2: Headless Path (JavaScript-Heavy Site)

**Request:**
```bash
curl -X POST http://localhost:8080/api/extract \
  -H "Content-Type: application/json" \
  -H "X-API-Key: ${RIPTIDE_API_KEY}" \
  -d '{
    "url": "https://react.dev"
  }'
```

**Response:**
```json
{
  "text": "React\n\nThe library for web and native user interfaces...",
  "html": "<div><header><h1>React</h1></header>...</div>",
  "metadata": {
    "url": "https://react.dev",
    "title": "React",
    "timestamp": "2025-10-28T14:35:00.456Z",

    "extraction_time_ms": 459,
    "quality_score": 1.0,
    "confidence_score": 0.25,
    "word_count": 2456,

    "parser_used": "native",
    "parser_path": "headless",
    "fallback_used": false,

    "links": ["https://react.dev/learn", "https://react.dev/reference"],
    "images": ["https://react.dev/logo.png"],
    "content_hash": "b6e9d1f3c2a5"
  }
}
```

**Analysis:**
- ✅ Headless path chosen (low confidence: 0.25 for SPA)
- ✅ Native parser succeeded (no fallback needed)
- ✅ Perfect quality (1.0)
- ✅ Reasonable performance (459ms for Chrome rendering)

### Example 3: Skip Headless (Forced Fast Path)

**Request:**
```bash
curl -X POST http://localhost:8080/api/extract \
  -H "Content-Type: application/json" \
  -H "X-API-Key: ${RIPTIDE_API_KEY}" \
  -d '{
    "url": "https://example.com",
    "skip_headless": true
  }'
```

**Response:**
```json
{
  "text": "Example Domain...",
  "metadata": {
    "parser_used": "native",
    "parser_path": "fast",
    "fallback_used": true,
    "confidence_score": 0.85,
    "extraction_time_ms": 5
  }
}
```

**Analysis:**
- ✅ Fast path forced (skip_headless parameter)
- ⚠️ WASM fallback (expected in current version)
- ✅ Very fast extraction (5ms)

### Example 4: Extraction Failure (Both Parsers Fail)

**Request:**
```bash
curl -X POST http://localhost:8080/api/extract \
  -H "Content-Type: application/json" \
  -H "X-API-Key: ${RIPTIDE_API_KEY}" \
  -d '{
    "url": "https://invalid-or-blocked-site.com"
  }'
```

**Response:**
```json
{
  "error": "Extraction failed",
  "message": "Both parsers failed in fast path. WASM: unicode_data error, Native: Invalid HTML structure",
  "metadata": {
    "url": "https://invalid-or-blocked-site.com",
    "parser_path": "fast",
    "fallback_used": true,
    "primary_parser": "wasm",
    "fallback_parser": "native",
    "extraction_time_ms": 12
  }
}
```

---

## Migration Guide

### For Existing API Consumers

**Breaking Changes:** None (all new fields are additions)

**Backward Compatibility:**
- All existing fields remain unchanged
- New metadata fields are optional
- Existing integrations continue to work

**Recommended Actions:**

1. **Update Client Libraries:**
   ```typescript
   // Old
   interface Metadata {
     url: string;
     title: string;
     quality_score: number;
     // ...
   }

   // New
   interface Metadata {
     url: string;
     title: string;
     quality_score: number;

     // New parser fields
     parser_used?: 'native' | 'wasm';
     parser_path?: 'fast' | 'headless' | 'probes_first';
     fallback_used?: boolean;
     confidence_score?: number;
     // ...
   }
   ```

2. **Monitor Fallback Rates:**
   ```python
   # Track WASM fallback rate
   def track_fallback(response):
       if response['metadata'].get('fallback_used'):
           metrics.increment('parser.fallback')
       metrics.record('confidence', response['metadata']['confidence_score'])
   ```

3. **Optimize Based on Metadata:**
   ```javascript
   // Cache high-confidence results longer
   const ttl = response.metadata.confidence_score > 0.8
     ? 3600  // 1 hour
     : 600;  // 10 minutes

   cache.set(url, response, ttl);
   ```

4. **Alert on Quality Issues:**
   ```bash
   # Alert if quality drops
   if [ "$(jq '.metadata.quality_score' response.json)" < "0.7" ]; then
     echo "Low quality extraction detected!"
   fi
   ```

---

## Best Practices

1. **Always check `quality_score`** - Values <0.7 may need review
2. **Monitor `fallback_used` rate** - High rate indicates WASM issues
3. **Use `confidence_score` for caching** - Cache high-confidence results longer
4. **Track `extraction_time_ms`** - Identify performance bottlenecks
5. **Log `parser_used` for analytics** - Understand parser distribution

---

## Support

For questions or issues with API metadata:

- **Documentation:** `/docs/hybrid-parser-final-architecture.md`
- **API Reference:** `/docs/api/openapi.yaml`
- **Health Check:** `curl http://localhost:8080/healthz`

---

**API Version:** v1
**Metadata Version:** 1.0.0
**Production Status:** ✅ Deployed (2025-10-28)
