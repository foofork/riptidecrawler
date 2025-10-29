# Result Mode Architecture Design

**Version:** 1.0.0
**Status:** Architecture Design
**Author:** System Architect
**Date:** 2025-10-29

## Executive Summary

This document specifies the architecture for the `result_mode` parameter system that allows API consumers to choose between statistics-only (`stats`) and URL-list (`urls`) response modes while maintaining full backward compatibility with existing APIs.

## Table of Contents

1. [System Overview](#system-overview)
2. [Type System Design](#type-system-design)
3. [Data Flow Architecture](#data-flow-architecture)
4. [API Layer Design](#api-layer-design)
5. [Backward Compatibility](#backward-compatibility)
6. [Implementation Specifications](#implementation-specifications)

---

## 1. System Overview

### 1.1 Objectives

- **Flexible Response Modes**: Allow clients to choose between minimal statistics or full URL lists
- **Backward Compatibility**: Existing API consumers continue working without changes (default: `stats`)
- **Performance**: Minimize memory overhead when URLs aren't needed
- **Extensibility**: Design supports future result modes (e.g., `pages`, `content`)

### 1.2 Architecture Principles

1. **Default Behavior Preservation**: No `result_mode` parameter = statistics only (current behavior)
2. **Type Safety**: Use Rust enums and strong typing to prevent invalid states
3. **Layered Architecture**: Changes propagate through `riptide-spider` → `riptide-facade` → `riptide-api`
4. **Memory Efficiency**: Only collect URLs when explicitly requested

---

## 2. Type System Design

### 2.1 Core Enum: ResultMode

```rust
/// Spider crawl result mode
///
/// Determines what data is returned from a spider crawl operation.
/// This controls memory usage and response payload size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResultMode {
    /// Return statistics only (default, backward compatible)
    ///
    /// Response includes:
    /// - pages_crawled count
    /// - pages_failed count
    /// - duration
    /// - stop_reason
    /// - domains list
    /// - performance metrics
    Stats,

    /// Return statistics plus discovered URLs
    ///
    /// Response includes everything from Stats plus:
    /// - discovered_urls: Vec<String> (all URLs found during crawl)
    Urls,
}

impl Default for ResultMode {
    fn default() -> Self {
        // Backward compatibility: existing API calls get stats only
        Self::Stats
    }
}

impl ResultMode {
    /// Check if this mode requires URL collection
    pub fn requires_url_collection(&self) -> bool {
        matches!(self, ResultMode::Urls)
    }
}
```

### 2.2 Result Structs

#### 2.2.1 SpiderResultStats (Existing, Renamed)

```rust
/// Spider crawl result with statistics only
///
/// This is the default result type, providing lightweight
/// statistics about the crawl operation without storing URLs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiderResultStats {
    /// Total pages successfully crawled
    pub pages_crawled: u64,

    /// Total pages that failed to crawl
    pub pages_failed: u64,

    /// Duration of the crawl operation
    pub duration: Duration,

    /// Reason for stopping the crawl
    pub stop_reason: String,

    /// Final performance metrics
    pub performance: PerformanceMetrics,

    /// List of unique domains crawled
    pub domains: Vec<String>,
}
```

#### 2.2.2 SpiderResultUrls (New)

```rust
/// Spider crawl result with discovered URLs
///
/// Extends statistics with a complete list of all URLs
/// discovered during the crawl operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiderResultUrls {
    /// Total pages successfully crawled
    pub pages_crawled: u64,

    /// Total pages that failed to crawl
    pub pages_failed: u64,

    /// Duration of the crawl operation
    pub duration: Duration,

    /// Reason for stopping the crawl
    pub stop_reason: String,

    /// Final performance metrics
    pub performance: PerformanceMetrics,

    /// List of unique domains crawled
    pub domains: Vec<String>,

    /// All URLs discovered during the crawl
    ///
    /// This includes:
    /// - Successfully crawled URLs
    /// - URLs that failed to crawl
    /// - URLs found but not crawled (due to depth/budget limits)
    pub discovered_urls: Vec<String>,
}
```

#### 2.2.3 SpiderResult Enum (Replaces Current Struct)

```rust
/// Spider crawl result
///
/// This enum allows returning different result types based
/// on the requested result mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", content = "data")]
pub enum SpiderResult {
    /// Statistics-only result
    Stats(SpiderResultStats),

    /// Statistics with discovered URLs
    Urls(SpiderResultUrls),
}

impl SpiderResult {
    /// Get pages crawled count (available in all modes)
    pub fn pages_crawled(&self) -> u64 {
        match self {
            SpiderResult::Stats(s) => s.pages_crawled,
            SpiderResult::Urls(u) => u.pages_crawled,
        }
    }

    /// Get pages failed count (available in all modes)
    pub fn pages_failed(&self) -> u64 {
        match self {
            SpiderResult::Stats(s) => s.pages_failed,
            SpiderResult::Urls(u) => u.pages_failed,
        }
    }

    /// Get stop reason (available in all modes)
    pub fn stop_reason(&self) -> &str {
        match self {
            SpiderResult::Stats(s) => &s.stop_reason,
            SpiderResult::Urls(u) => &u.stop_reason,
        }
    }

    /// Get discovered URLs if available
    pub fn discovered_urls(&self) -> Option<&Vec<String>> {
        match self {
            SpiderResult::Stats(_) => None,
            SpiderResult::Urls(u) => Some(&u.discovered_urls),
        }
    }
}
```

### 2.3 Serde Serialization Strategy

The enum uses `tag = "mode", content = "data"` for JSON representation:

**Stats Mode:**
```json
{
  "mode": "stats",
  "data": {
    "pages_crawled": 42,
    "pages_failed": 3,
    "duration": { "secs": 120, "nanos": 500000000 },
    "stop_reason": "MaxPagesReached",
    "performance": { ... },
    "domains": ["example.com", "test.com"]
  }
}
```

**Urls Mode:**
```json
{
  "mode": "urls",
  "data": {
    "pages_crawled": 42,
    "pages_failed": 3,
    "duration": { "secs": 120, "nanos": 500000000 },
    "stop_reason": "MaxPagesReached",
    "performance": { ... },
    "domains": ["example.com", "test.com"],
    "discovered_urls": [
      "https://example.com/",
      "https://example.com/about",
      "https://test.com/products",
      ...
    ]
  }
}
```

### 2.4 Query Parameter Parsing

```rust
/// Parse result_mode from query string
impl std::str::FromStr for ResultMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "stats" => Ok(ResultMode::Stats),
            "urls" => Ok(ResultMode::Urls),
            _ => Err(format!(
                "Invalid result_mode: '{}'. Valid options: 'stats', 'urls'",
                s
            )),
        }
    }
}

/// Axum query extractor usage:
///
/// #[derive(Deserialize)]
/// struct SpiderQuery {
///     #[serde(default)]
///     result_mode: ResultMode,  // Defaults to Stats if not provided
/// }
```

---

## 3. Data Flow Architecture

### 3.1 High-Level Data Flow Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                        API Request                                │
│  POST /spider/crawl?result_mode=urls                             │
│  {                                                               │
│    "seed_urls": ["https://example.com"],                        │
│    "max_pages": 100                                             │
│  }                                                              │
└────────────────┬─────────────────────────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────────────────────────────┐
│                   riptide-api Handler                            │
│  - Parse query parameter result_mode                             │
│  - Default to ResultMode::Stats if not provided                  │
│  - Pass to SpiderFacade                                          │
└────────────────┬─────────────────────────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────────────────────────────┐
│                   riptide-facade Layer                           │
│  SpiderFacade::crawl_with_mode(seeds, result_mode)              │
│  - Pass result_mode to Spider::crawl()                          │
│  - Return SpiderResult enum                                      │
└────────────────┬─────────────────────────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────────────────────────────┐
│                   riptide-spider Engine                          │
│                                                                  │
│  ┌────────────────────────────────────────┐                     │
│  │  Spider::crawl(seeds, result_mode)     │                     │
│  │                                        │                     │
│  │  if result_mode.requires_url_collection() {                 │
│  │      discovered_urls = Vec::new()      │                     │
│  │  }                                     │                     │
│  └────────────┬───────────────────────────┘                     │
│               │                                                  │
│               ▼                                                  │
│  ┌─────────────────────────────────────────────┐               │
│  │  Crawl Loop (Frontier Processing)           │               │
│  │                                             │               │
│  │  for url in frontier.pop() {                │               │
│  │      let response = fetch(url).await;       │               │
│  │                                             │               │
│  │      if result_mode == Urls {              │               │
│  │          discovered_urls.push(url.clone()); │               │
│  │      }                                      │               │
│  │                                             │               │
│  │      extract_links(response);               │               │
│  │      frontier.push(new_links);              │               │
│  │  }                                          │               │
│  └────────────┬────────────────────────────────┘               │
│               │                                                  │
│               ▼                                                  │
│  ┌─────────────────────────────────────────────┐               │
│  │  Build Result                                │               │
│  │                                             │               │
│  │  match result_mode {                        │               │
│  │      Stats => SpiderResult::Stats(...)      │               │
│  │      Urls => SpiderResult::Urls(           │               │
│  │          discovered_urls: discovered_urls   │               │
│  │      )                                      │               │
│  │  }                                          │               │
│  └────────────┬────────────────────────────────┘               │
└───────────────┼──────────────────────────────────────────────────┘
                │
                ▼
┌──────────────────────────────────────────────────────────────────┐
│                   API Response                                   │
│  {                                                              │
│    "mode": "urls",                                             │
│    "data": {                                                   │
│      "pages_crawled": 42,                                      │
│      "discovered_urls": [ ... ]                                │
│    }                                                           │
│  }                                                             │
└──────────────────────────────────────────────────────────────────┘
```

### 3.2 URL Collection Flow in riptide-spider

```
Spider::crawl(seeds, result_mode)
│
├─ Initialize
│  ├─ discovered_urls: Option<Vec<String>> = if result_mode == Urls {
│  │      Some(Vec::with_capacity(1024))
│  │  } else {
│  │      None
│  │  }
│  └─ frontier: FrontierManager
│
├─ Crawl Loop
│  │
│  ├─ url = frontier.pop()
│  │
│  ├─ Record URL if tracking
│  │  if let Some(ref mut urls) = discovered_urls {
│  │      urls.push(url.to_string());
│  │  }
│  │
│  ├─ Fetch page
│  │  response = http_client.fetch(url).await
│  │
│  ├─ Extract links
│  │  links = extract_links(&response.html)
│  │
│  ├─ Add to frontier
│  │  for link in links {
│  │      frontier.push(link)
│  │  }
│  │
│  └─ Loop until stop condition
│
└─ Build Result
   │
   ├─ Common stats
   │  pages_crawled = state.pages_crawled
   │  pages_failed = state.pages_failed
   │  duration = start_time.elapsed()
   │
   └─ Mode-specific result
      match result_mode {
          Stats => SpiderResult::Stats(SpiderResultStats { ... }),
          Urls => SpiderResult::Urls(SpiderResultUrls {
              ...,
              discovered_urls: discovered_urls.unwrap_or_default(),
          }),
      }
```

### 3.3 Type Hierarchy Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         ResultMode                               │
│                       (enum, Copy)                               │
│                                                                  │
│  ┌──────────────┐                  ┌──────────────┐            │
│  │    Stats     │                  │     Urls     │            │
│  │  (default)   │                  │              │            │
│  └──────────────┘                  └──────────────┘            │
└──────────┬──────────────────────────────┬────────────────────────┘
           │                              │
           │                              │
           ▼                              ▼
┌────────────────────────┐    ┌────────────────────────────────┐
│  SpiderResultStats     │    │    SpiderResultUrls            │
│                        │    │                                │
│  + pages_crawled       │    │  + pages_crawled               │
│  + pages_failed        │    │  + pages_failed                │
│  + duration            │    │  + duration                    │
│  + stop_reason         │    │  + stop_reason                 │
│  + performance         │    │  + performance                 │
│  + domains             │    │  + domains                     │
│                        │    │  + discovered_urls: Vec<String>│
└────────────────────────┘    └────────────────────────────────┘
           │                              │
           │                              │
           └──────────┬───────────────────┘
                      │
                      ▼
           ┌──────────────────────────┐
           │    SpiderResult          │
           │       (enum)             │
           │                          │
           │  Stats(SpiderResultStats)│
           │  Urls(SpiderResultUrls)  │
           └──────────────────────────┘
```

---

## 4. API Layer Design

### 4.1 Request Schema

#### 4.1.1 Query Parameter

```
GET/POST /spider/crawl?result_mode={mode}

Parameters:
  result_mode (optional, string):
    - "stats" (default): Return statistics only
    - "urls": Return statistics + discovered URLs

  If omitted: Defaults to "stats" (backward compatible)
```

#### 4.1.2 Request Body (POST)

```json
{
  "seed_urls": [
    "https://example.com",
    "https://test.com"
  ],
  "max_depth": 3,
  "max_pages": 100,
  "strategy": "breadth_first"
}
```

### 4.2 Response Schema

#### 4.2.1 Stats Mode Response (Default)

```json
{
  "result": {
    "mode": "stats",
    "data": {
      "pages_crawled": 42,
      "pages_failed": 3,
      "duration": {
        "secs": 120,
        "nanos": 500000000
      },
      "stop_reason": "MaxPagesReached",
      "performance": {
        "pages_per_second": 0.35,
        "avg_page_time_ms": 2857,
        "total_bytes": 1048576,
        "cache_hit_rate": 0.15
      },
      "domains": [
        "example.com",
        "cdn.example.com",
        "test.com"
      ]
    }
  },
  "state": {
    "active": false,
    "pages_crawled": 42,
    "pages_failed": 3,
    "frontier_size": 0,
    "current_depth": 3
  },
  "performance": { ... }
}
```

#### 4.2.2 Urls Mode Response

```json
{
  "result": {
    "mode": "urls",
    "data": {
      "pages_crawled": 42,
      "pages_failed": 3,
      "duration": {
        "secs": 120,
        "nanos": 500000000
      },
      "stop_reason": "MaxPagesReached",
      "performance": { ... },
      "domains": [
        "example.com",
        "test.com"
      ],
      "discovered_urls": [
        "https://example.com/",
        "https://example.com/about",
        "https://example.com/contact",
        "https://example.com/blog/post-1",
        "https://test.com/",
        "https://test.com/products",
        ... (up to pages_crawled count)
      ]
    }
  },
  "state": { ... },
  "performance": { ... }
}
```

### 4.3 Handler Implementation Pattern

```rust
// In riptide-api/src/handlers/spider.rs

#[derive(Deserialize)]
struct SpiderCrawlQuery {
    /// Result mode (defaults to Stats if not provided)
    #[serde(default)]
    result_mode: ResultMode,
}

pub async fn spider_crawl(
    State(state): State<AppState>,
    Query(query): Query<SpiderCrawlQuery>,
    Json(body): Json<SpiderCrawlBody>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();

    info!(
        result_mode = ?query.result_mode,
        seed_count = body.seed_urls.len(),
        "Received spider crawl request"
    );

    // Get spider facade
    let spider_facade = state.spider_facade.as_ref()
        .ok_or_else(|| ApiError::ConfigError {
            message: "SpiderFacade is not enabled".to_string(),
        })?;

    // Parse seed URLs
    let seed_urls = parse_seed_urls(&body.seed_urls)?;

    // Perform crawl with specified result mode
    let result = spider_facade
        .crawl_with_mode(seed_urls, query.result_mode)
        .await
        .map_err(|e| {
            ApiError::internal(format!("Spider crawl failed: {}", e))
        })?;

    // Build response (result is already in the correct format)
    let response = SpiderCrawlResponse {
        result,
        state: spider_facade.get_state().await?,
        performance: spider_facade.get_metrics().await?,
    };

    Ok(Json(response))
}
```

---

## 5. Backward Compatibility

### 5.1 Compatibility Matrix

| API Version | result_mode Parameter | Response Format | Behavior |
|-------------|----------------------|-----------------|----------|
| **Existing (Pre-implementation)** | N/A | Stats only (implicit) | Current behavior |
| **v1.0 (Post-implementation)** | Omitted | `{"mode": "stats", "data": {...}}` | **Same as existing** |
| **v1.0** | `?result_mode=stats` | `{"mode": "stats", "data": {...}}` | Explicit stats |
| **v1.0** | `?result_mode=urls` | `{"mode": "urls", "data": {...}}` | New URLs mode |

### 5.2 Breaking Change Analysis

#### ✅ **NON-BREAKING** (Safe for existing clients)

1. **No parameter provided**: Defaults to `Stats` mode
2. **Response structure**: Wrapped in `{"mode": "...", "data": {...}}`
3. **All existing fields preserved**: No field removal or renaming

#### ⚠️ **MINOR BREAKING CHANGE** (JSON structure)

**Old Response Format:**
```json
{
  "result": {
    "pages_crawled": 42,
    "pages_failed": 3,
    ...
  }
}
```

**New Response Format:**
```json
{
  "result": {
    "mode": "stats",
    "data": {
      "pages_crawled": 42,
      "pages_failed": 3,
      ...
    }
  }
}
```

**Impact**: Clients accessing `response.result.pages_crawled` must update to `response.result.data.pages_crawled`

### 5.3 Migration Strategy

**Option A: Strict Versioning (Recommended)**
- Deploy as v2.0.0 with clear migration guide
- Maintain v1.x with old response format for 6 months
- API versioning: `/v1/spider/crawl` vs `/v2/spider/crawl`

**Option B: Gradual Migration**
- Add `api_version` query parameter
- `?api_version=1` returns old format (flat result)
- `?api_version=2` returns new format (tagged enum)
- Default to v2 after deprecation period

**Option C: Transparent Compatibility Layer** *(Chosen for this design)*
- Serde's `#[serde(flatten)]` can expose both formats
- Add compatibility shim in API response serialization
- Gradual client migration without breaking existing integrations

### 5.4 SDK Compatibility

**Python SDK:**
```python
# Old code (still works)
result = await client.spider.crawl(seed_urls=["..."])
print(result.pages_crawled)  # Accesses result.data.pages_crawled internally

# New code (explicit mode)
result = await client.spider.crawl(
    seed_urls=["..."],
    result_mode="urls"
)
for url in result.discovered_urls:
    print(url)
```

**Rust SDK:**
```rust
// Type-safe enum matching
let result = client.spider().crawl(seeds).result_mode(ResultMode::Urls).await?;

match result {
    SpiderResult::Stats(stats) => {
        println!("Pages: {}", stats.pages_crawled);
    }
    SpiderResult::Urls(urls) => {
        println!("Pages: {}", urls.pages_crawled);
        println!("URLs: {:?}", urls.discovered_urls);
    }
}
```

---

## 6. Implementation Specifications

### 6.1 Phase 1: Core Type System (riptide-spider)

**Location:** `crates/riptide-spider/src/result.rs`

**Tasks:**
1. Define `ResultMode` enum
2. Define `SpiderResultStats` struct (rename from `SpiderResult`)
3. Define `SpiderResultUrls` struct (new)
4. Define `SpiderResult` enum (replaces existing struct)
5. Add helper methods for result access
6. Implement `FromStr` for query parsing
7. Add comprehensive unit tests

**Acceptance Criteria:**
- All types compile with proper Serde derives
- `ResultMode::default()` returns `Stats`
- JSON serialization matches specification
- Tests verify backward compatibility

### 6.2 Phase 2: Spider Engine Integration

**Location:** `crates/riptide-spider/src/core.rs`

**Tasks:**
1. Add `result_mode: ResultMode` parameter to `Spider::crawl()`
2. Add `discovered_urls: Option<Vec<String>>` field to crawl state
3. Modify crawl loop to track URLs when `result_mode == Urls`
4. Update result building logic to return correct enum variant
5. Add benchmarks to verify minimal overhead in `Stats` mode

**Data Collection Strategy:**
```rust
// Inside Spider struct
struct CrawlContext {
    // Existing fields...
    discovered_urls: Option<Vec<String>>,
}

// In crawl initialization
let mut context = CrawlContext {
    discovered_urls: if result_mode.requires_url_collection() {
        Some(Vec::with_capacity(1024)) // Pre-allocate for efficiency
    } else {
        None
    },
    // ...
};

// During URL processing
if let Some(ref mut urls) = context.discovered_urls {
    urls.push(url.to_string());
}
```

**Acceptance Criteria:**
- URLs only collected when `result_mode == Urls`
- Memory overhead in `Stats` mode < 1%
- Performance tests show negligible latency difference

### 6.3 Phase 3: Facade Layer Update

**Location:** `crates/riptide-facade/src/facades/spider.rs`

**Tasks:**
1. Add `crawl_with_mode()` method to `SpiderFacade`
2. Update `CrawlSummary` conversion to handle both result types
3. Maintain backward-compatible `crawl()` method (defaults to Stats)
4. Add integration tests with both modes

**API Design:**
```rust
impl SpiderFacade {
    /// Crawl with default result mode (stats only)
    pub async fn crawl(&self, seeds: Vec<Url>) -> Result<CrawlSummary> {
        self.crawl_with_mode(seeds, ResultMode::default()).await
    }

    /// Crawl with specified result mode
    pub async fn crawl_with_mode(
        &self,
        seeds: Vec<Url>,
        result_mode: ResultMode,
    ) -> Result<SpiderResult> {
        let spider = self.spider.lock().await;
        spider.crawl(seeds, result_mode).await
    }
}
```

### 6.4 Phase 4: API Handler Implementation

**Location:** `crates/riptide-api/src/handlers/spider.rs`

**Tasks:**
1. Add `SpiderCrawlQuery` struct with `result_mode` field
2. Update `spider_crawl()` handler to parse query parameter
3. Update `SpiderCrawlResponse` to use new `SpiderResult` enum
4. Add API integration tests
5. Update OpenAPI documentation

**Query Extraction:**
```rust
#[derive(Deserialize)]
struct SpiderCrawlQuery {
    #[serde(default)]
    result_mode: ResultMode,
}

// Axum automatically deserializes from ?result_mode=urls
```

### 6.5 Phase 5: Testing & Documentation

**Testing Requirements:**
1. **Unit Tests**: Each type system component
2. **Integration Tests**: Full flow from API to Spider
3. **Backward Compatibility Tests**: Verify existing clients work
4. **Performance Tests**: Benchmark memory/latency overhead
5. **SDK Tests**: Python and Rust SDK compatibility

**Documentation Updates:**
1. API documentation (`/docs/api/spider-crawl.md`)
2. Migration guide for existing users
3. SDK examples for both modes
4. OpenAPI spec updates
5. Changelog entry

### 6.6 Performance Targets

| Metric | Stats Mode | Urls Mode | Notes |
|--------|-----------|-----------|-------|
| Memory overhead | Baseline | +2MB per 10k URLs | Vec<String> storage |
| Latency | Baseline | +1-2% | String cloning overhead |
| Throughput | Baseline | -1-2% | URL collection overhead |

### 6.7 Future Extensibility

The design supports future result modes:

```rust
pub enum ResultMode {
    Stats,
    Urls,
    // Future modes:
    Pages,      // Full CrawledPage objects
    Content,    // With extracted content
    Links,      // With link graph
}

pub enum SpiderResult {
    Stats(SpiderResultStats),
    Urls(SpiderResultUrls),
    // Future variants:
    Pages(SpiderResultPages),
    Content(SpiderResultContent),
    Links(SpiderResultLinks),
}
```

---

## 7. Implementation Checklist

### 7.1 Development Tasks

- [ ] **riptide-spider**
  - [ ] Define `ResultMode` enum
  - [ ] Define `SpiderResultStats` struct
  - [ ] Define `SpiderResultUrls` struct
  - [ ] Define `SpiderResult` enum
  - [ ] Update `Spider::crawl()` signature
  - [ ] Implement URL collection logic
  - [ ] Add unit tests
  - [ ] Add benchmarks

- [ ] **riptide-facade**
  - [ ] Add `crawl_with_mode()` method
  - [ ] Update `CrawlSummary` conversion
  - [ ] Add integration tests

- [ ] **riptide-api**
  - [ ] Add `SpiderCrawlQuery` struct
  - [ ] Update `spider_crawl()` handler
  - [ ] Update response models
  - [ ] Add API integration tests

- [ ] **Testing**
  - [ ] Unit tests for all components
  - [ ] Integration tests (API → Spider)
  - [ ] Backward compatibility tests
  - [ ] Performance benchmarks
  - [ ] SDK compatibility tests

- [ ] **Documentation**
  - [ ] API documentation
  - [ ] Migration guide
  - [ ] SDK examples
  - [ ] OpenAPI spec
  - [ ] Changelog

### 7.2 Acceptance Criteria

✅ **Functional Requirements:**
- [ ] `?result_mode=stats` returns statistics only
- [ ] `?result_mode=urls` returns statistics + discovered URLs
- [ ] No parameter defaults to `stats` mode
- [ ] All existing API clients continue working

✅ **Performance Requirements:**
- [ ] Stats mode has <1% memory overhead vs baseline
- [ ] Stats mode has <1% latency overhead vs baseline
- [ ] Urls mode memory scales linearly with URL count

✅ **Quality Requirements:**
- [ ] 100% test coverage for new types
- [ ] All tests pass
- [ ] No compiler warnings
- [ ] Clippy lints pass
- [ ] Documentation complete

---

## 8. Architecture Decision Records

### ADR-001: Use Enum for Result Mode Instead of Boolean

**Decision:** Use `ResultMode` enum instead of `include_urls: bool`

**Rationale:**
- Extensible to future modes (`pages`, `content`, etc.)
- Self-documenting API (`?result_mode=urls` vs `?include_urls=true`)
- Type-safe pattern matching in Rust

**Alternatives Considered:**
- Boolean flag: Not extensible, unclear semantics
- String parameter without enum: No type safety

### ADR-002: Default to Stats Mode

**Decision:** `ResultMode::default()` returns `Stats`

**Rationale:**
- Preserves existing behavior (backward compatible)
- Minimal memory overhead for clients that don't need URLs
- Explicit opt-in for larger payloads

### ADR-003: Tagged Enum JSON Representation

**Decision:** Use `#[serde(tag = "mode", content = "data")]`

**Rationale:**
- Clear discrimination between modes in JSON
- Type-safe deserialization
- Forward-compatible with new modes

**JSON Structure:**
```json
{
  "mode": "urls",
  "data": { ... }
}
```

### ADR-004: Optional URL Collection in Spider

**Decision:** Only allocate `Vec<String>` when `result_mode == Urls`

**Rationale:**
- Zero overhead in default mode
- Pay-for-what-you-use memory model
- Simple Option<Vec<T>> pattern

---

## 9. Security & Privacy Considerations

### 9.1 Data Exposure

**Risk:** Discovered URLs may contain sensitive information (session tokens, API keys)

**Mitigation:**
- Document security implications in API docs
- Add option to sanitize URLs (remove query params)
- Rate-limit `urls` mode requests (higher resource cost)

### 9.2 Memory DOS

**Risk:** Large crawls with `urls` mode could exhaust memory

**Mitigation:**
- Enforce `max_pages` limit strictly
- Add `max_discovered_urls` cap (default: 10,000)
- Monitor memory usage per request

---

## 10. Conclusion

This architecture provides a clean, extensible solution for result mode selection while maintaining full backward compatibility. The type-safe enum approach ensures correctness, and the optional URL collection minimizes overhead for existing use cases.

**Next Steps:**
1. Review this document with the development team
2. Assign implementation tasks to coder agents
3. Create test specifications for QA
4. Update API documentation

**Coordination:**
- Store this design in memory for researcher/coder access
- Notify team via hooks when implementation begins
- Track progress with TodoWrite tool

---

**Document Version History:**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-10-29 | System Architect | Initial architecture design |
