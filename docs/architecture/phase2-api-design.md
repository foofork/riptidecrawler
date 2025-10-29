# Phase 2 API Design: Full Page Objects, Streaming & Job Storage

**Version:** 1.0
**Status:** Architecture Design
**Author:** System Architecture Designer
**Date:** 2025-10-29

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Architecture Overview](#architecture-overview)
3. [Core Data Structures](#core-data-structures)
4. [Result Mode Design](#result-mode-design)
5. [Field Selection Mechanism](#field-selection-mechanism)
6. [Streaming Endpoints](#streaming-endpoints)
7. [Job Storage & Pagination](#job-storage--pagination)
8. [Extraction Helpers](#extraction-helpers)
9. [Size Limits & Safety](#size-limits--safety)
10. [API Surface](#api-surface)
11. [Implementation Plan](#implementation-plan)

---

## Executive Summary

Phase 2 extends the spider crawling API to return full page objects with field selection, streaming delivery, and job storage capabilities. This architecture enables:

- **Scale**: Handle large crawls (1000s of pages) via streaming and pagination
- **Flexibility**: Field selection controls payload size and bandwidth
- **Usability**: Industry-standard "discover → extract" workflows
- **Performance**: NDJSON/SSE streaming for real-time consumption
- **Storage**: Async job execution with paginated result retrieval

**Key Principle**: Implement in the API layer (not just facade) for interoperability, performance, and ecosystem compatibility.

---

## Architecture Overview

### Design Principles

1. **Backward Compatibility**: Existing `result_mode=stats` remains unchanged
2. **Additive Evolution**: New modes (urls, pages, stream, store) are opt-in
3. **Performance-First**: Streaming and field selection prevent memory bloat
4. **API-First**: Core functionality in riptide-api, orchestration in facade
5. **Safety by Default**: Size limits, truncation flags, error surfaces

### System Components

```
┌─────────────────────────────────────────────────────────────┐
│                      Client Layer                            │
│  (Python SDK, REST clients, curl)                            │
└────────────┬────────────────────────────────────────────────┘
             │
             v
┌─────────────────────────────────────────────────────────────┐
│                   API Layer (riptide-api)                    │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ POST /spider?result_mode=<mode>&include=<fields>     │   │
│  │ GET  /jobs/{id}/results?cursor=...&limit=...         │   │
│  │ POST /extract/batch                                   │   │
│  │ POST /spider+extract                                  │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ ResultMode enum | CrawledPage | Field Selector       │   │
│  │ Stream Handler  | Job Storage  | Page Builder        │   │
│  └──────────────────────────────────────────────────────┘   │
└────────────┬────────────────────────────────────────────────┘
             │
             v
┌─────────────────────────────────────────────────────────────┐
│                 Facade Layer (riptide-facade)                │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ SpiderFacade                                          │   │
│  │ - crawl() → CrawlSummary                              │   │
│  │ - crawl_with_pages() → PagedCrawlSummary (NEW)        │   │
│  └──────────────────────────────────────────────────────┘   │
└────────────┬────────────────────────────────────────────────┘
             │
             v
┌─────────────────────────────────────────────────────────────┐
│               Core Engine (riptide-spider)                   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Spider Engine                                         │   │
│  │ - Frontier queue                                      │   │
│  │ - Page fetcher                                        │   │
│  │ - Link extractor                                      │   │
│  │ - SpiderResult with page data (ENHANCED)              │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Data Structures

### 1. CrawledPage (riptide-api/src/dto.rs)

The fundamental unit representing a single crawled page.

```rust
use serde::{Deserialize, Serialize};

/// A single page discovered and crawled during spider execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawledPage {
    /// Original URL requested
    pub url: String,

    /// Final URL after redirects (if different from original)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_url: Option<String>,

    /// Canonical URL from HTML metadata (if found)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_url: Option<String>,

    /// Crawl depth from seed URL (0 = seed)
    pub depth: u32,

    /// HTTP status code
    pub status_code: u16,

    /// Page title extracted from <title> or og:title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Raw HTML/text content (gated by include=content)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Normalized markdown content (gated by include=markdown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>,

    /// Outgoing links discovered on this page
    #[serde(default)]
    pub links: Vec<String>,

    /// MIME type of response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime: Option<String>,

    /// Character encoding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charset: Option<String>,

    /// Fetch time in milliseconds
    pub fetch_time_ms: u64,

    /// Whether robots.txt rules were obeyed
    #[serde(default = "default_true")]
    pub robots_obeyed: bool,

    /// Whether this URL was disallowed by robots.txt
    #[serde(default)]
    pub disallowed: bool,

    /// Fetch error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetch_error: Option<String>,

    /// Parse/extraction error (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_error: Option<String>,

    /// Whether content was truncated due to size limits
    #[serde(default)]
    pub truncated: bool,

    /// Timestamp when this page was crawled (ISO 8601)
    pub crawled_at: String,
}

fn default_true() -> bool {
    true
}
```

**Design Decisions:**

- **Optional Fields**: Heavy fields (content, markdown) are Option<String> to allow selective inclusion
- **Error Surface**: Both fetch_error and parse_error allow debugging without failing entire crawl
- **Normalization**: final_url, canonical_url provide redirect and SEO metadata
- **Compliance**: robots_obeyed and disallowed flags for transparency
- **Truncation**: truncated flag indicates when content exceeded max_content_bytes

---

### 2. SpiderResultPages (riptide-api/src/dto.rs)

Container for full page-based crawl results.

```rust
/// Spider crawl result with full page objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiderResultPages {
    /// Total pages successfully crawled
    pub pages_crawled: u64,

    /// Total pages that failed
    pub pages_failed: u64,

    /// Crawl duration in seconds
    pub duration_seconds: f64,

    /// Reason for stopping (max_pages, max_depth, budget_exhausted, manual_stop)
    pub stop_reason: String,

    /// API version for this response schema
    pub api_version: String,

    /// List of all crawled pages (subject to field selection)
    pub pages: Vec<CrawledPage>,
}

impl SpiderResultPages {
    pub fn new(
        pages_crawled: u64,
        pages_failed: u64,
        duration_seconds: f64,
        stop_reason: String,
        pages: Vec<CrawledPage>,
    ) -> Self {
        Self {
            pages_crawled,
            pages_failed,
            duration_seconds,
            stop_reason,
            api_version: "1.0".to_string(),
            pages,
        }
    }
}
```

**Design Decisions:**

- **Statistics Included**: pages_crawled/failed maintain consistency with Stats mode
- **Versioning**: api_version field enables future schema evolution
- **Stop Reason**: Transparent about why crawl ended
- **Page Array**: Full page objects for immediate consumption

---

### 3. ResultMode Enum (riptide-api/src/dto.rs)

**CURRENT STATE (Phase 1):**
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResultMode {
    Stats,  // Statistics only (default)
    Urls,   // Statistics + discovered_urls
}
```

**PHASE 2 EXTENSION:**
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResultMode {
    /// Statistics only (backward compatible default)
    Stats,

    /// Statistics + URL list
    Urls,

    /// Statistics + full page objects (subject to max_pages limit)
    Pages,

    /// NDJSON/SSE streaming of pages as they're crawled
    Stream,

    /// Store results server-side, return job_id for async retrieval
    Store,
}

impl Default for ResultMode {
    fn default() -> Self {
        Self::Stats
    }
}

impl ResultMode {
    /// Whether this mode requires page-level data collection
    pub fn requires_pages(&self) -> bool {
        matches!(self, Self::Pages | Self::Stream | Self::Store)
    }

    /// Whether this mode streams results incrementally
    pub fn is_streaming(&self) -> bool {
        matches!(self, Self::Stream)
    }

    /// Whether this mode requires job storage
    pub fn requires_storage(&self) -> bool {
        matches!(self, Self::Store)
    }
}
```

---

### 4. FieldSelector (riptide-api/src/dto.rs)

Controls which fields are included in CrawledPage to manage payload size.

```rust
use std::collections::HashSet;

/// Field selection for controlling CrawledPage payload size
#[derive(Debug, Clone)]
pub struct FieldSelector {
    /// Explicitly included fields (None = all fields except excluded)
    include: Option<HashSet<String>>,

    /// Explicitly excluded fields
    exclude: HashSet<String>,
}

impl FieldSelector {
    /// Parse from query parameters: include=title,links&exclude=content
    pub fn from_query_params(
        include: Option<&str>,
        exclude: Option<&str>,
    ) -> Result<Self, String> {
        let include_set = include.map(|s| {
            s.split(',')
                .map(|f| f.trim().to_lowercase())
                .collect::<HashSet<_>>()
        });

        let exclude_set = exclude
            .map(|s| {
                s.split(',')
                    .map(|f| f.trim().to_lowercase())
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();

        // Validate field names
        for field in include_set.iter().flatten() {
            if !Self::is_valid_field(field) {
                return Err(format!("Invalid field name: {}", field));
            }
        }

        for field in &exclude_set {
            if !Self::is_valid_field(field) {
                return Err(format!("Invalid field name: {}", field));
            }
        }

        Ok(Self {
            include: include_set,
            exclude: exclude_set,
        })
    }

    /// Check if a field should be included in the response
    pub fn should_include(&self, field: &str) -> bool {
        let field = field.to_lowercase();

        // Always exclude if explicitly excluded
        if self.exclude.contains(&field) {
            return false;
        }

        // If include list exists, only include listed fields
        if let Some(ref include) = self.include {
            include.contains(&field)
        } else {
            // No include list = include everything except excluded
            true
        }
    }

    /// Valid field names for CrawledPage
    fn is_valid_field(name: &str) -> bool {
        matches!(
            name,
            "url" | "final_url" | "canonical_url" | "depth" | "status_code" |
            "title" | "content" | "markdown" | "links" | "mime" | "charset" |
            "fetch_time_ms" | "robots_obeyed" | "disallowed" | "fetch_error" |
            "parse_error" | "truncated" | "crawled_at"
        )
    }

    /// Default selector (no filtering)
    pub fn all() -> Self {
        Self {
            include: None,
            exclude: HashSet::new(),
        }
    }

    /// Exclude heavy fields by default
    pub fn lightweight() -> Self {
        Self {
            include: None,
            exclude: ["content", "markdown"].iter().map(|s| s.to_string()).collect(),
        }
    }
}
```

**Usage Examples:**

```rust
// Include only title and links
let selector = FieldSelector::from_query_params(
    Some("title,links"),
    None
)?;

// Include everything except raw content
let selector = FieldSelector::from_query_params(
    None,
    Some("content")
)?;

// Lightweight mode: exclude heavy fields
let selector = FieldSelector::lightweight();
```

---

### 5. PageBuilder (riptide-api/src/handlers/spider_pages.rs)

Constructs CrawledPage from crawler output with field selection.

```rust
use crate::dto::{CrawledPage, FieldSelector};
use riptide_spider::PageData; // Hypothetical internal page data

/// Builds CrawledPage objects with field selection applied
pub struct PageBuilder {
    selector: FieldSelector,
    max_content_bytes: usize,
}

impl PageBuilder {
    pub fn new(selector: FieldSelector, max_content_bytes: usize) -> Self {
        Self {
            selector,
            max_content_bytes,
        }
    }

    /// Build a CrawledPage from internal page data
    pub fn build(&self, page_data: &PageData) -> CrawledPage {
        let mut page = CrawledPage {
            url: page_data.url.clone(),
            final_url: if self.selector.should_include("final_url") {
                page_data.final_url.clone()
            } else {
                None
            },
            canonical_url: if self.selector.should_include("canonical_url") {
                page_data.canonical_url.clone()
            } else {
                None
            },
            depth: page_data.depth,
            status_code: page_data.status_code,
            title: if self.selector.should_include("title") {
                page_data.title.clone()
            } else {
                None
            },
            content: if self.selector.should_include("content") {
                self.maybe_truncate_content(page_data.content.clone())
            } else {
                None
            },
            markdown: if self.selector.should_include("markdown") {
                self.maybe_truncate_content(page_data.markdown.clone())
            } else {
                None
            },
            links: if self.selector.should_include("links") {
                page_data.links.clone()
            } else {
                vec![]
            },
            mime: if self.selector.should_include("mime") {
                page_data.mime.clone()
            } else {
                None
            },
            charset: if self.selector.should_include("charset") {
                page_data.charset.clone()
            } else {
                None
            },
            fetch_time_ms: page_data.fetch_time_ms,
            robots_obeyed: page_data.robots_obeyed,
            disallowed: page_data.disallowed,
            fetch_error: if self.selector.should_include("fetch_error") {
                page_data.fetch_error.clone()
            } else {
                None
            },
            parse_error: if self.selector.should_include("parse_error") {
                page_data.parse_error.clone()
            } else {
                None
            },
            truncated: false,
            crawled_at: chrono::Utc::now().to_rfc3339(),
        };

        page
    }

    /// Truncate content if it exceeds max_content_bytes
    fn maybe_truncate_content(&self, content: Option<String>) -> Option<String> {
        content.map(|mut c| {
            if c.len() > self.max_content_bytes {
                c.truncate(self.max_content_bytes);
                c
            } else {
                c
            }
        })
    }
}
```

---

## Result Mode Design

### Mode Comparison Matrix

| Mode   | Response Type      | Latency | Memory  | Use Case                          |
|--------|--------------------|---------|---------|-----------------------------------|
| Stats  | JSON (statistics)  | Low     | Minimal | Quick metrics, monitoring         |
| Urls   | JSON (stats+URLs)  | Low     | Low     | Discover → extract workflow       |
| Pages  | JSON (full pages)  | Medium  | High    | Small crawls (<100 pages)         |
| Stream | NDJSON/SSE         | Real-time | Constant | Large crawls, real-time processing |
| Store  | JSON (job_id)      | Low     | Server  | Async large crawls, pagination    |

### Implementation Strategy

```rust
// In spider handler (riptide-api/src/handlers/spider.rs)
match query.result_mode {
    ResultMode::Stats => {
        // EXISTING: Return statistics only
        Ok(Json(SpiderResultStats::from(&crawl_summary)).into_response())
    }

    ResultMode::Urls => {
        // EXISTING (Phase 1): Return stats + URLs
        Ok(Json(SpiderResultUrls::from(&crawl_summary)).into_response())
    }

    ResultMode::Pages => {
        // NEW: Return full page objects
        let selector = FieldSelector::from_query_params(
            query.include.as_deref(),
            query.exclude.as_deref(),
        )?;

        let pages = build_pages(&crawl_summary, &selector)?;
        let result = SpiderResultPages::new(
            crawl_summary.pages_crawled,
            crawl_summary.pages_failed,
            crawl_summary.duration_secs,
            crawl_summary.stop_reason,
            pages,
        );

        Ok(Json(result).into_response())
    }

    ResultMode::Stream => {
        // NEW: NDJSON/SSE streaming
        stream_pages(crawl_summary, query.include, query.exclude).await
    }

    ResultMode::Store => {
        // NEW: Store results and return job_id
        let job_id = store_crawl_results(&crawl_summary).await?;
        Ok(Json(json!({"job_id": job_id})).into_response())
    }
}
```

---

## Field Selection Mechanism

### Query Parameter Syntax

**Include Specific Fields:**
```
GET /spider/crawl?result_mode=pages&include=title,links,markdown
```

**Exclude Heavy Fields:**
```
GET /spider/crawl?result_mode=pages&exclude=content
```

**Combined (include takes precedence):**
```
GET /spider/crawl?result_mode=pages&include=title,markdown&exclude=content
```

### Validation Rules

1. **Mutual Exclusivity**: Cannot specify both include and exclude (include wins)
2. **Valid Fields Only**: Reject unknown field names with 400 error
3. **Core Fields Always Included**: url, depth, status_code (cannot be excluded)
4. **Default Behavior**:
   - No params = all fields except content/markdown (lightweight mode)
   - Stream mode = all fields unless explicitly excluded

### Implementation in Handler

```rust
#[derive(Debug, Deserialize)]
pub struct SpiderCrawlQuery {
    #[serde(default)]
    pub result_mode: ResultMode,

    /// Comma-separated field names to include
    pub include: Option<String>,

    /// Comma-separated field names to exclude
    pub exclude: Option<String>,

    /// Maximum pages to return (default: 1000, max: 10000)
    pub max_pages: Option<usize>,
}

pub async fn spider_crawl(
    State(state): State<AppState>,
    Query(query): Query<SpiderCrawlQuery>,
    Json(body): Json<SpiderCrawlBody>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate field selection
    let selector = FieldSelector::from_query_params(
        query.include.as_deref(),
        query.exclude.as_deref(),
    )
    .map_err(|e| ApiError::validation(e))?;

    // Enforce max_pages limit for Pages mode
    if query.result_mode == ResultMode::Pages {
        let max_pages = query.max_pages.unwrap_or(1000).min(10000);
        if crawl_summary.pages_crawled as usize > max_pages {
            return Err(ApiError::validation(format!(
                "Result too large ({} pages). Use result_mode=stream or result_mode=store",
                crawl_summary.pages_crawled
            )));
        }
    }

    // ... rest of handler
}
```

---

## Streaming Endpoints

### Protocol Choice: NDJSON vs SSE

**NDJSON (Newline-Delimited JSON):**
- **Pros**: Simple, widely supported, easy to parse
- **Cons**: No native browser API, requires manual chunking
- **Format**: One JSON object per line

**SSE (Server-Sent Events):**
- **Pros**: Native browser EventSource API, automatic reconnection
- **Cons**: Text-only, requires data: prefix
- **Format**: `data: {json}\n\n`

**Decision**: Support both via `Accept` header.

### NDJSON Streaming

```rust
use axum::response::sse::{Event, Sse};
use futures::stream::{self, Stream};
use tokio_stream::StreamExt;

pub async fn stream_pages_ndjson(
    crawl_summary: CrawlSummary,
    selector: FieldSelector,
) -> Result<impl IntoResponse, ApiError> {
    let pages_stream = create_pages_stream(crawl_summary, selector);

    // NDJSON format: one JSON object per line
    let ndjson_stream = pages_stream.map(|page| {
        let json = serde_json::to_string(&page).unwrap();
        format!("{}\n", json)
    });

    Ok((
        StatusCode::OK,
        [("Content-Type", "application/x-ndjson")],
        Body::wrap_stream(ndjson_stream),
    ))
}

fn create_pages_stream(
    crawl_summary: CrawlSummary,
    selector: FieldSelector,
) -> impl Stream<Item = StreamItem> {
    stream::iter(crawl_summary.pages.into_iter().map(|page_data| {
        let builder = PageBuilder::new(selector.clone(), MAX_CONTENT_BYTES);
        StreamItem::Page(builder.build(&page_data))
    }))
    .chain(stream::once(async move {
        StreamItem::Stats(StreamStats {
            pages_crawled: crawl_summary.pages_crawled,
            pages_failed: crawl_summary.pages_failed,
            duration_seconds: crawl_summary.duration_secs,
            stop_reason: crawl_summary.stop_reason,
        })
    }))
}

#[derive(Serialize)]
#[serde(tag = "type", content = "data")]
enum StreamItem {
    #[serde(rename = "page")]
    Page(CrawledPage),

    #[serde(rename = "stats")]
    Stats(StreamStats),
}

#[derive(Serialize)]
struct StreamStats {
    pages_crawled: u64,
    pages_failed: u64,
    duration_seconds: f64,
    stop_reason: String,
}
```

**Example NDJSON Output:**
```json
{"type":"page","data":{"url":"https://example.com","depth":0,"status_code":200,"title":"Example","links":["https://example.com/page1"]}}
{"type":"page","data":{"url":"https://example.com/page1","depth":1,"status_code":200,"title":"Page 1","links":[]}}
{"type":"stats","data":{"pages_crawled":2,"pages_failed":0,"duration_seconds":1.23,"stop_reason":"max_pages"}}
```

### SSE Streaming

```rust
pub async fn stream_pages_sse(
    crawl_summary: CrawlSummary,
    selector: FieldSelector,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let pages_stream = create_pages_stream(crawl_summary, selector);

    let sse_stream = pages_stream.map(|item| {
        let json = serde_json::to_string(&item).unwrap();
        Ok(Event::default().data(json))
    });

    Ok(Sse::new(sse_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    ))
}
```

**Example SSE Output:**
```
data: {"type":"page","data":{"url":"https://example.com","depth":0,"status_code":200}}

data: {"type":"page","data":{"url":"https://example.com/page1","depth":1,"status_code":200}}

data: {"type":"stats","data":{"pages_crawled":2,"pages_failed":0,"duration_seconds":1.23}}

```

### Content Negotiation

```rust
pub async fn spider_crawl_stream(
    State(state): State<AppState>,
    Query(query): Query<SpiderCrawlQuery>,
    headers: HeaderMap,
    Json(body): Json<SpiderCrawlBody>,
) -> Result<impl IntoResponse, ApiError> {
    let accept = headers
        .get("Accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/x-ndjson");

    let selector = FieldSelector::from_query_params(
        query.include.as_deref(),
        query.exclude.as_deref(),
    )?;

    let crawl_summary = perform_crawl(&state, &body).await?;

    if accept.contains("text/event-stream") {
        stream_pages_sse(crawl_summary, selector).await.map(|s| s.into_response())
    } else {
        stream_pages_ndjson(crawl_summary, selector).await
    }
}
```

---

## Job Storage & Pagination

### Database Schema

**Table: spider_jobs**
```sql
CREATE TABLE spider_jobs (
    job_id UUID PRIMARY KEY,
    user_id VARCHAR(255),
    status VARCHAR(50) NOT NULL, -- 'running', 'completed', 'failed'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    pages_crawled BIGINT DEFAULT 0,
    pages_failed BIGINT DEFAULT 0,
    duration_seconds DOUBLE PRECISION,
    stop_reason VARCHAR(255),
    seed_urls JSONB,
    config JSONB, -- crawl configuration
    error TEXT
);

CREATE INDEX idx_spider_jobs_user_id ON spider_jobs(user_id);
CREATE INDEX idx_spider_jobs_created_at ON spider_jobs(created_at DESC);
```

**Table: spider_pages**
```sql
CREATE TABLE spider_pages (
    page_id BIGSERIAL PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES spider_jobs(job_id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    final_url TEXT,
    canonical_url TEXT,
    depth INTEGER NOT NULL,
    status_code INTEGER NOT NULL,
    title TEXT,
    content TEXT, -- nullable, only stored if requested
    markdown TEXT, -- nullable, only stored if requested
    links JSONB, -- array of strings
    mime VARCHAR(255),
    charset VARCHAR(50),
    fetch_time_ms BIGINT,
    robots_obeyed BOOLEAN DEFAULT TRUE,
    disallowed BOOLEAN DEFAULT FALSE,
    fetch_error TEXT,
    parse_error TEXT,
    truncated BOOLEAN DEFAULT FALSE,
    crawled_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_spider_pages_job_id ON spider_pages(job_id);
CREATE INDEX idx_spider_pages_page_id_job_id ON spider_pages(page_id, job_id);
```

### Job Creation (POST /spider?result_mode=store)

```rust
use uuid::Uuid;

pub async fn spider_crawl_store(
    State(state): State<AppState>,
    Query(query): Query<SpiderCrawlQuery>,
    Json(body): Json<SpiderCrawlBody>,
) -> Result<Json<JobCreatedResponse>, ApiError> {
    let job_id = Uuid::new_v4();

    // Create job record
    sqlx::query!(
        r#"
        INSERT INTO spider_jobs (job_id, user_id, status, seed_urls, config)
        VALUES ($1, $2, 'running', $3, $4)
        "#,
        job_id,
        "anonymous", // TODO: extract from auth context
        serde_json::to_value(&body.seed_urls)?,
        serde_json::to_value(&body)?
    )
    .execute(&state.db_pool)
    .await?;

    // Spawn background task to execute crawl
    tokio::spawn(async move {
        execute_crawl_job(state, job_id, body, query).await
    });

    Ok(Json(JobCreatedResponse {
        job_id: job_id.to_string(),
        status: "running".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

async fn execute_crawl_job(
    state: AppState,
    job_id: Uuid,
    body: SpiderCrawlBody,
    query: SpiderCrawlQuery,
) {
    let start_time = Instant::now();

    match perform_crawl(&state, &body).await {
        Ok(crawl_summary) => {
            let selector = FieldSelector::from_query_params(
                query.include.as_deref(),
                query.exclude.as_deref(),
            )
            .unwrap_or_else(|_| FieldSelector::lightweight());

            // Store pages in database
            for page_data in crawl_summary.pages {
                let page = PageBuilder::new(selector.clone(), MAX_CONTENT_BYTES)
                    .build(&page_data);

                let _ = sqlx::query!(
                    r#"
                    INSERT INTO spider_pages (
                        job_id, url, final_url, canonical_url, depth, status_code,
                        title, content, markdown, links, mime, charset,
                        fetch_time_ms, robots_obeyed, disallowed, fetch_error,
                        parse_error, truncated, crawled_at
                    ) VALUES (
                        $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                        $13, $14, $15, $16, $17, $18, $19
                    )
                    "#,
                    job_id,
                    page.url,
                    page.final_url,
                    page.canonical_url,
                    page.depth as i32,
                    page.status_code as i32,
                    page.title,
                    page.content,
                    page.markdown,
                    serde_json::to_value(&page.links).unwrap(),
                    page.mime,
                    page.charset,
                    page.fetch_time_ms as i64,
                    page.robots_obeyed,
                    page.disallowed,
                    page.fetch_error,
                    page.parse_error,
                    page.truncated,
                    chrono::Utc::now()
                )
                .execute(&state.db_pool)
                .await;
            }

            // Update job status
            let _ = sqlx::query!(
                r#"
                UPDATE spider_jobs
                SET status = 'completed',
                    completed_at = NOW(),
                    pages_crawled = $2,
                    pages_failed = $3,
                    duration_seconds = $4,
                    stop_reason = $5
                WHERE job_id = $1
                "#,
                job_id,
                crawl_summary.pages_crawled as i64,
                crawl_summary.pages_failed as i64,
                start_time.elapsed().as_secs_f64(),
                crawl_summary.stop_reason
            )
            .execute(&state.db_pool)
            .await;
        }
        Err(e) => {
            // Mark job as failed
            let _ = sqlx::query!(
                r#"
                UPDATE spider_jobs
                SET status = 'failed', completed_at = NOW(), error = $2
                WHERE job_id = $1
                "#,
                job_id,
                e.to_string()
            )
            .execute(&state.db_pool)
            .await;
        }
    }
}
```

### Job Results Pagination (GET /jobs/{id}/results)

```rust
#[derive(Debug, Deserialize)]
pub struct JobResultsQuery {
    /// Cursor for pagination (page_id)
    pub cursor: Option<i64>,

    /// Number of results per page (default: 100, max: 1000)
    pub limit: Option<usize>,

    /// Field selection
    pub include: Option<String>,
    pub exclude: Option<String>,
}

pub async fn get_job_results(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    Query(query): Query<JobResultsQuery>,
) -> Result<Json<JobResultsResponse>, ApiError> {
    let limit = query.limit.unwrap_or(100).min(1000);
    let cursor = query.cursor.unwrap_or(0);

    // Fetch pages with pagination
    let pages = sqlx::query_as!(
        DbCrawledPage,
        r#"
        SELECT page_id, url, final_url, canonical_url, depth, status_code,
               title, content, markdown, links, mime, charset, fetch_time_ms,
               robots_obeyed, disallowed, fetch_error, parse_error, truncated,
               crawled_at
        FROM spider_pages
        WHERE job_id = $1 AND page_id > $2
        ORDER BY page_id ASC
        LIMIT $3
        "#,
        job_id,
        cursor,
        limit as i64
    )
    .fetch_all(&state.db_pool)
    .await?;

    let selector = FieldSelector::from_query_params(
        query.include.as_deref(),
        query.exclude.as_deref(),
    )?;

    let filtered_pages: Vec<CrawledPage> = pages
        .into_iter()
        .map(|db_page| apply_field_selection(db_page, &selector))
        .collect();

    let next_cursor = filtered_pages.last().map(|p| p.page_id);
    let has_more = filtered_pages.len() == limit;

    Ok(Json(JobResultsResponse {
        pages: filtered_pages,
        cursor: next_cursor,
        has_more,
        total_pages: get_total_pages(job_id, &state.db_pool).await?,
    }))
}

#[derive(Serialize)]
pub struct JobResultsResponse {
    pub pages: Vec<CrawledPage>,
    pub cursor: Option<i64>,
    pub has_more: bool,
    pub total_pages: u64,
}
```

### Job Stats Endpoint (GET /jobs/{id}/stats)

```rust
pub async fn get_job_stats(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<JobStats>, ApiError> {
    let job = sqlx::query_as!(
        JobStats,
        r#"
        SELECT job_id, status, created_at, completed_at,
               pages_crawled, pages_failed, duration_seconds, stop_reason
        FROM spider_jobs
        WHERE job_id = $1
        "#,
        job_id
    )
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| ApiError::not_found("Job not found"))?;

    Ok(Json(job))
}

#[derive(Serialize, Deserialize)]
pub struct JobStats {
    pub job_id: Uuid,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub pages_crawled: Option<i64>,
    pub pages_failed: Option<i64>,
    pub duration_seconds: Option<f64>,
    pub stop_reason: Option<String>,
}
```

---

## Extraction Helpers

### 1. Batch Extraction (POST /extract/batch)

Extracts content from multiple URLs in parallel.

```rust
#[derive(Deserialize)]
pub struct BatchExtractRequest {
    /// URLs to extract
    pub urls: Vec<String>,

    /// Output format (markdown, text, html)
    #[serde(default = "default_markdown")]
    pub format: String,

    /// Maximum concurrent requests
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
}

fn default_markdown() -> String {
    "markdown".to_string()
}

fn default_concurrency() -> usize {
    10
}

#[derive(Serialize)]
pub struct BatchExtractResponse {
    pub total_urls: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<ExtractedPageResult>,
}

#[derive(Serialize)]
pub struct ExtractedPageResult {
    pub url: String,
    pub status: String, // "success" or "failed"
    pub markdown: Option<String>,
    pub metadata: Option<PageMetadata>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct PageMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub publish_date: Option<String>,
    pub word_count: usize,
}

pub async fn batch_extract(
    State(state): State<AppState>,
    Json(body): Json<BatchExtractRequest>,
) -> Result<Json<BatchExtractResponse>, ApiError> {
    use futures::stream::{self, StreamExt};

    let concurrency = body.concurrency.min(20);

    let results: Vec<ExtractedPageResult> = stream::iter(body.urls)
        .map(|url| async {
            match extract_single_url(&state, &url, &body.format).await {
                Ok(extracted) => ExtractedPageResult {
                    url: url.clone(),
                    status: "success".to_string(),
                    markdown: Some(extracted.content),
                    metadata: Some(PageMetadata {
                        title: extracted.title,
                        author: None, // TODO: extract from metadata
                        publish_date: None,
                        word_count: extracted.content.split_whitespace().count(),
                    }),
                    error: None,
                },
                Err(e) => ExtractedPageResult {
                    url: url.clone(),
                    status: "failed".to_string(),
                    markdown: None,
                    metadata: None,
                    error: Some(e.to_string()),
                },
            }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;

    let successful = results.iter().filter(|r| r.status == "success").count();
    let failed = results.len() - successful;

    Ok(Json(BatchExtractResponse {
        total_urls: results.len(),
        successful,
        failed,
        results,
    }))
}
```

### 2. Spider + Extract (POST /spider+extract)

Orchestrated workflow: crawl → filter → extract → deliver.

```rust
#[derive(Deserialize)]
pub struct SpiderExtractRequest {
    /// Seed URLs to start crawling
    pub seeds: Vec<String>,

    /// Crawl scope configuration
    pub scope: Option<SpiderScope>,

    /// Fields to include in results
    pub include: Vec<String>,

    /// Result delivery mode
    #[serde(default = "default_pages_mode")]
    pub result_mode: String,

    /// URL filter pattern (regex) - only extract matching URLs
    pub extract_pattern: Option<String>,
}

#[derive(Deserialize)]
pub struct SpiderScope {
    pub max_depth: Option<usize>,
    pub max_pages: Option<usize>,
    pub same_domain_only: Option<bool>,
}

fn default_pages_mode() -> String {
    "pages".to_string()
}

pub async fn spider_extract(
    State(state): State<AppState>,
    Json(body): Json<SpiderExtractRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Step 1: Crawl to discover URLs
    let crawl_body = SpiderCrawlBody {
        seed_urls: body.seeds,
        max_depth: body.scope.as_ref().and_then(|s| s.max_depth),
        max_pages: body.scope.as_ref().and_then(|s| s.max_pages),
        ..Default::default()
    };

    let crawl_summary = perform_crawl(&state, &crawl_body).await?;

    // Step 2: Filter URLs based on pattern
    let urls_to_extract: Vec<String> = if let Some(pattern) = &body.extract_pattern {
        let regex = regex::Regex::new(pattern)
            .map_err(|e| ApiError::validation(format!("Invalid regex: {}", e)))?;

        crawl_summary
            .discovered_urls
            .into_iter()
            .filter(|url| regex.is_match(url))
            .collect()
    } else {
        crawl_summary.discovered_urls
    };

    // Step 3: Extract content from filtered URLs
    let extract_request = BatchExtractRequest {
        urls: urls_to_extract,
        format: "markdown".to_string(),
        concurrency: 10,
    };

    let extract_results = batch_extract(State(state.clone()), Json(extract_request)).await?;

    // Step 4: Combine crawl metadata with extracted content
    let combined_results: Vec<CrawledPageWithExtraction> = extract_results
        .0
        .results
        .into_iter()
        .map(|extract_result| CrawledPageWithExtraction {
            url: extract_result.url.clone(),
            status: extract_result.status,
            markdown: extract_result.markdown,
            metadata: extract_result.metadata,
            crawl_depth: find_url_depth(&extract_result.url, &crawl_summary),
        })
        .collect();

    Ok(Json(SpiderExtractResponse {
        crawl_stats: SpiderResultStats {
            pages_crawled: crawl_summary.pages_crawled,
            pages_failed: crawl_summary.pages_failed,
            duration_seconds: crawl_summary.duration_secs,
            stop_reason: crawl_summary.stop_reason,
            domains: crawl_summary.domains,
        },
        extracted_pages: combined_results,
    }))
}

#[derive(Serialize)]
pub struct SpiderExtractResponse {
    pub crawl_stats: SpiderResultStats,
    pub extracted_pages: Vec<CrawledPageWithExtraction>,
}

#[derive(Serialize)]
pub struct CrawledPageWithExtraction {
    pub url: String,
    pub status: String,
    pub markdown: Option<String>,
    pub metadata: Option<PageMetadata>,
    pub crawl_depth: u32,
}
```

---

## Size Limits & Safety

### Configuration Constants

```rust
// In riptide-api/src/config.rs
pub struct SpiderLimits {
    /// Maximum pages per request (Pages mode)
    pub max_pages_per_request: usize,

    /// Maximum content bytes per page (before truncation)
    pub max_content_bytes: usize,

    /// Maximum URLs in discovered_urls list
    pub max_discovered_urls: usize,

    /// Maximum stored jobs per user
    pub max_stored_jobs_per_user: usize,

    /// Job retention period in days
    pub job_retention_days: u32,
}

impl Default for SpiderLimits {
    fn default() -> Self {
        Self {
            max_pages_per_request: 1000,
            max_content_bytes: 1_000_000, // 1MB per page
            max_discovered_urls: 10_000,
            max_stored_jobs_per_user: 100,
            job_retention_days: 30,
        }
    }
}
```

### Safety Guardrails

**1. Response Size Enforcement:**
```rust
if query.result_mode == ResultMode::Pages {
    let page_count = crawl_summary.pages.len();
    if page_count > state.config.limits.max_pages_per_request {
        return Err(ApiError::validation(format!(
            "Result too large ({} pages exceeds limit of {}). Use result_mode=stream or result_mode=store",
            page_count,
            state.config.limits.max_pages_per_request
        )));
    }
}
```

**2. Content Truncation:**
```rust
impl PageBuilder {
    fn maybe_truncate_content(&self, content: Option<String>) -> (Option<String>, bool) {
        match content {
            Some(mut c) if c.len() > self.max_content_bytes => {
                c.truncate(self.max_content_bytes);
                (Some(c), true) // truncated = true
            }
            other => (other, false),
        }
    }
}
```

**3. Job Quota Enforcement:**
```rust
pub async fn spider_crawl_store(
    State(state): State<AppState>,
    user_id: &str,
    ...
) -> Result<Json<JobCreatedResponse>, ApiError> {
    let user_job_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM spider_jobs WHERE user_id = $1 AND status = 'running'",
        user_id
    )
    .fetch_one(&state.db_pool)
    .await?;

    if user_job_count >= state.config.limits.max_stored_jobs_per_user as i64 {
        return Err(ApiError::quota_exceeded(
            "Maximum concurrent jobs reached. Please wait for existing jobs to complete."
        ));
    }

    // ... proceed with job creation
}
```

**4. Compression:**
```rust
use tower_http::compression::CompressionLayer;

// In server setup
let app = Router::new()
    .route("/spider/crawl", post(spider_crawl))
    .layer(CompressionLayer::new().gzip(true))
    .layer(DefaultBodyLimit::max(10 * 1024 * 1024)); // 10MB max request
```

### Error Handling

```rust
#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
    pub error_code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self {
            error: "ValidationError".to_string(),
            error_code: "VALIDATION_ERROR".to_string(),
            message: message.into(),
            details: None,
        }
    }

    pub fn quota_exceeded(message: impl Into<String>) -> Self {
        Self {
            error: "QuotaExceeded".to_string(),
            error_code: "QUOTA_EXCEEDED".to_string(),
            message: message.into(),
            details: None,
        }
    }

    pub fn result_too_large(page_count: usize, limit: usize) -> Self {
        Self {
            error: "ResultTooLarge".to_string(),
            error_code: "RESULT_TOO_LARGE".to_string(),
            message: format!(
                "Crawl result ({} pages) exceeds maximum response size ({})",
                page_count, limit
            ),
            details: Some(serde_json::json!({
                "page_count": page_count,
                "limit": limit,
                "suggestions": [
                    "Use result_mode=stream for NDJSON/SSE streaming",
                    "Use result_mode=store for async job execution",
                    "Reduce max_pages parameter"
                ]
            })),
        }
    }
}
```

---

## API Surface

### Complete Endpoint Specification

**1. Spider Crawl (Enhanced)**
```
POST /spider/crawl

Query Parameters:
- result_mode: stats|urls|pages|stream|store (default: stats)
- include: comma-separated field names (e.g., title,links,markdown)
- exclude: comma-separated field names (e.g., content)
- max_pages: maximum pages to return (Pages mode only, default: 1000, max: 10000)

Request Body:
{
  "seed_urls": ["https://example.com"],
  "max_depth": 2,
  "max_pages": 100,
  "strategy": "breadth_first",
  "timeout_seconds": 30,
  "concurrency": 10,
  "respect_robots": true
}

Response (result_mode=stats):
{
  "pages_crawled": 42,
  "pages_failed": 3,
  "duration_seconds": 12.5,
  "stop_reason": "max_pages",
  "domains": ["example.com"]
}

Response (result_mode=urls):
{
  "pages_crawled": 42,
  "pages_failed": 3,
  "duration_seconds": 12.5,
  "stop_reason": "max_pages",
  "domains": ["example.com"],
  "discovered_urls": ["https://example.com/page1", ...]
}

Response (result_mode=pages):
{
  "pages_crawled": 42,
  "pages_failed": 3,
  "duration_seconds": 12.5,
  "stop_reason": "max_pages",
  "api_version": "1.0",
  "pages": [
    {
      "url": "https://example.com",
      "depth": 0,
      "status_code": 200,
      "title": "Example Domain",
      "links": ["https://example.com/page1"],
      "fetch_time_ms": 245,
      "crawled_at": "2025-10-29T12:00:00Z"
    }
  ]
}

Response (result_mode=stream):
Content-Type: application/x-ndjson
{"type":"page","data":{...}}
{"type":"page","data":{...}}
{"type":"stats","data":{...}}

Response (result_mode=store):
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "running",
  "created_at": "2025-10-29T12:00:00Z"
}
```

**2. Job Results (New)**
```
GET /jobs/{job_id}/results

Query Parameters:
- cursor: pagination cursor (page_id, optional)
- limit: results per page (default: 100, max: 1000)
- include: comma-separated field names
- exclude: comma-separated field names

Response:
{
  "pages": [
    {
      "url": "https://example.com",
      "depth": 0,
      "status_code": 200,
      "title": "Example",
      ...
    }
  ],
  "cursor": 12345,
  "has_more": true,
  "total_pages": 542
}
```

**3. Job Stats (New)**
```
GET /jobs/{job_id}/stats

Response:
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "completed",
  "created_at": "2025-10-29T12:00:00Z",
  "completed_at": "2025-10-29T12:05:00Z",
  "pages_crawled": 542,
  "pages_failed": 12,
  "duration_seconds": 285.3,
  "stop_reason": "max_pages"
}
```

**4. Batch Extract (New)**
```
POST /extract/batch

Request Body:
{
  "urls": ["https://example.com/article1", "https://example.com/article2"],
  "format": "markdown",
  "concurrency": 10
}

Response:
{
  "total_urls": 2,
  "successful": 2,
  "failed": 0,
  "results": [
    {
      "url": "https://example.com/article1",
      "status": "success",
      "markdown": "# Article Title\n\nContent...",
      "metadata": {
        "title": "Article Title",
        "word_count": 542
      }
    }
  ]
}
```

**5. Spider + Extract (New)**
```
POST /spider+extract

Request Body:
{
  "seeds": ["https://example.com"],
  "scope": {
    "max_depth": 2,
    "max_pages": 100,
    "same_domain_only": true
  },
  "include": ["markdown", "title"],
  "result_mode": "pages",
  "extract_pattern": ".*\\/article\\/.*"
}

Response:
{
  "crawl_stats": {
    "pages_crawled": 42,
    "pages_failed": 3,
    "duration_seconds": 25.4,
    "stop_reason": "max_pages",
    "domains": ["example.com"]
  },
  "extracted_pages": [
    {
      "url": "https://example.com/article/123",
      "status": "success",
      "markdown": "# Article...",
      "metadata": {
        "title": "Article Title",
        "word_count": 542
      },
      "crawl_depth": 1
    }
  ]
}
```

---

## Implementation Plan

### Phase 2.1: Core Data Structures (Week 1)

**Files to Create/Modify:**
- `/workspaces/eventmesh/crates/riptide-api/src/dto.rs`
  - Add `CrawledPage` struct
  - Add `SpiderResultPages` struct
  - Extend `ResultMode` enum (Pages, Stream, Store)
  - Add `FieldSelector` struct

- `/workspaces/eventmesh/crates/riptide-api/src/handlers/spider_pages.rs` (new)
  - Implement `PageBuilder`
  - Implement `build_pages()` helper

**Tasks:**
1. Define `CrawledPage` with all fields (30 min)
2. Implement `SpiderResultPages` (15 min)
3. Extend `ResultMode` enum with new variants (15 min)
4. Implement `FieldSelector` with validation (1 hour)
5. Create `PageBuilder` with field selection logic (1 hour)
6. Write unit tests for field selection (1 hour)

**Acceptance Criteria:**
- [ ] All structs compile with proper Serialize/Deserialize
- [ ] FieldSelector correctly filters fields based on include/exclude
- [ ] PageBuilder applies field selection correctly
- [ ] Unit tests achieve 90%+ coverage

### Phase 2.2: Pages Mode (Week 1-2)

**Files to Modify:**
- `/workspaces/eventmesh/crates/riptide-api/src/handlers/spider.rs`
  - Add Pages mode case to match statement
  - Integrate `PageBuilder` and `FieldSelector`

- `/workspaces/eventmesh/crates/riptide-spider/src/lib.rs`
  - Enhance `SpiderResult` to include page-level data
  - Add page metadata collection during crawl

**Tasks:**
1. Enhance crawler to collect page data (2 hours)
2. Implement Pages mode handler logic (1 hour)
3. Add response size validation (30 min)
4. Implement content truncation (30 min)
5. Add integration tests for Pages mode (2 hours)
6. Update API documentation (1 hour)

**Acceptance Criteria:**
- [ ] `result_mode=pages` returns full page objects
- [ ] Field selection works correctly
- [ ] Content truncation enforced at 1MB per page
- [ ] Error handling for oversized responses
- [ ] Integration tests pass

### Phase 2.3: Streaming (Week 2)

**Files to Create:**
- `/workspaces/eventmesh/crates/riptide-api/src/handlers/spider_stream.rs`
  - Implement NDJSON streaming
  - Implement SSE streaming
  - Add content negotiation

**Tasks:**
1. Implement NDJSON stream handler (2 hours)
2. Implement SSE stream handler (2 hours)
3. Add content negotiation logic (1 hour)
4. Implement stream backpressure handling (1 hour)
5. Add streaming integration tests (2 hours)
6. Performance testing for large crawls (1 hour)

**Acceptance Criteria:**
- [ ] NDJSON streaming works with `Accept: application/x-ndjson`
- [ ] SSE streaming works with `Accept: text/event-stream`
- [ ] Final stats object emitted at stream end
- [ ] Backpressure prevents memory bloat
- [ ] Tests verify stream correctness

### Phase 2.4: Job Storage (Week 3)

**Files to Create:**
- `/migrations/001_spider_jobs.sql`
- `/migrations/002_spider_pages.sql`
- `/workspaces/eventmesh/crates/riptide-api/src/handlers/spider_jobs.rs`
  - Implement job creation
  - Implement background job execution
  - Implement pagination endpoint

**Tasks:**
1. Create database schema migrations (1 hour)
2. Implement job creation endpoint (2 hours)
3. Implement background crawl execution (2 hours)
4. Implement pagination with cursor (2 hours)
5. Implement job stats endpoint (1 hour)
6. Add job cleanup/retention logic (1 hour)
7. Integration tests for job lifecycle (2 hours)

**Acceptance Criteria:**
- [ ] Jobs persist to database correctly
- [ ] Background execution completes successfully
- [ ] Pagination returns correct page ranges
- [ ] Job stats endpoint accurate
- [ ] Old jobs cleaned up after retention period
- [ ] Tests cover all job states

### Phase 2.5: Extraction Helpers (Week 4)

**Files to Create:**
- `/workspaces/eventmesh/crates/riptide-api/src/handlers/extract_batch.rs`
- `/workspaces/eventmesh/crates/riptide-api/src/handlers/spider_extract.rs`

**Tasks:**
1. Implement batch extract endpoint (2 hours)
2. Add concurrent extraction logic (1 hour)
3. Implement spider+extract orchestration (2 hours)
4. Add URL filtering with regex (1 hour)
5. Error handling for extraction failures (1 hour)
6. Integration tests for workflows (2 hours)

**Acceptance Criteria:**
- [ ] Batch extract processes multiple URLs concurrently
- [ ] Spider+extract workflow completes end-to-end
- [ ] URL filtering works correctly
- [ ] Errors don't fail entire batch
- [ ] Tests verify orchestration

### Phase 2.6: Documentation & Testing (Week 4)

**Files to Create/Update:**
- `/workspaces/eventmesh/docs/02-api-reference/spider-api.md`
- `/workspaces/eventmesh/docs/01-guides/discover-extract-workflow.md`
- `/workspaces/eventmesh/examples/spider_pages_example.rs`
- `/workspaces/eventmesh/examples/spider_streaming_example.rs`
- `/workspaces/eventmesh/tests/integration/spider_phase2_tests.rs`

**Tasks:**
1. API reference documentation (2 hours)
2. Workflow guide with examples (2 hours)
3. Create example code snippets (1 hour)
4. End-to-end integration tests (3 hours)
5. Performance benchmarks (2 hours)
6. Update Python SDK (if applicable) (2 hours)

**Acceptance Criteria:**
- [ ] Complete API documentation
- [ ] Working code examples
- [ ] Integration tests for all modes
- [ ] Performance benchmarks documented
- [ ] SDK updated with new features

---

## Appendix: Architecture Decision Records

### ADR-001: ResultMode in API vs Facade

**Status:** Accepted
**Context:** Should Phase 2 features live in the API or facade layer?
**Decision:** Implement in API layer (riptide-api)
**Rationale:**
- Interoperability: External clients can use riptide directly
- Performance: Streaming requires tight integration with crawler loop
- Ecosystem: Standard contracts for libraries/SDKs
- Backward compatibility: Additive changes to existing API

### ADR-002: NDJSON vs SSE for Streaming

**Status:** Accepted
**Context:** Which streaming protocol to support?
**Decision:** Support both via content negotiation
**Rationale:**
- NDJSON: Simple, widely supported, easy parsing
- SSE: Native browser API, automatic reconnection
- Content negotiation: Let clients choose based on needs

### ADR-003: Field Selection Mechanism

**Status:** Accepted
**Context:** How to control payload size for large crawls?
**Decision:** Query parameter-based include/exclude
**Rationale:**
- Flexible: Clients specify exactly what they need
- Bandwidth optimization: Exclude heavy fields (content, markdown)
- Standard pattern: Similar to GraphQL field selection
- Backward compatible: No params = lightweight default

### ADR-004: Job Storage Schema

**Status:** Accepted
**Context:** Should we use dedicated tables or blob storage?
**Decision:** Dedicated PostgreSQL tables (spider_jobs, spider_pages)
**Rationale:**
- Query flexibility: Pagination, filtering, sorting
- Relational integrity: FK constraints between jobs and pages
- Atomic updates: Job status transitions
- Cost: Postgres storage cheaper than S3 for structured data

### ADR-005: Content Truncation Strategy

**Status:** Accepted
**Context:** How to handle pages exceeding max_content_bytes?
**Decision:** Truncate and set `truncated: true` flag
**Rationale:**
- Prevents memory bloat from enormous pages
- Transparent: Clients know content was truncated
- Fail-safe: Doesn't fail entire crawl for one large page
- Configurable: Limit can be adjusted per deployment

---

**End of Architecture Document**
