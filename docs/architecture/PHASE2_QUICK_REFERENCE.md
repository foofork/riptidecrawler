# Phase 2 Quick Reference Card

## 🎯 5 Result Modes

```rust
Stats   → JSON stats only             (existing, backward compatible)
Urls    → JSON stats + URL list       (existing, Phase 1)
Pages   → JSON stats + full pages     (NEW, max 1000 pages)
Stream  → NDJSON/SSE real-time        (NEW, unlimited pages)
Store   → Async job + pagination      (NEW, server-side storage)
```

## 📊 CrawledPage Fields (18 total)

**Core (always included):**
```rust
url: String
depth: u32
status_code: u16
```

**Heavy (gated by field selection):**
```rust
content: Option<String>     // Raw HTML (1MB limit)
markdown: Option<String>    // Normalized markdown
```

**Metadata:**
```rust
title: Option<String>
links: Vec<String>
final_url: Option<String>
canonical_url: Option<String>
mime: Option<String>
charset: Option<String>
fetch_time_ms: u64
crawled_at: String (ISO 8601)
```

**Compliance:**
```rust
robots_obeyed: bool
disallowed: bool
```

**Debugging:**
```rust
fetch_error: Option<String>
parse_error: Option<String>
truncated: bool
```

## 🔧 API Endpoints

### Enhanced Spider Crawl
```bash
POST /spider/crawl?result_mode=pages&include=title,links&exclude=content
```

**Query Parameters:**
- `result_mode`: stats|urls|pages|stream|store (default: stats)
- `include`: comma-separated field names
- `exclude`: comma-separated field names
- `max_pages`: max pages for Pages mode (default: 1000, max: 10000)

### Job Storage (NEW)
```bash
# Create async job
POST /spider/crawl?result_mode=store
→ {"job_id": "uuid", "status": "running"}

# Get results with pagination
GET /jobs/{id}/results?cursor=12345&limit=100&include=title,links
→ {"pages": [...], "cursor": 12346, "has_more": true}

# Get job stats
GET /jobs/{id}/stats
→ {"status": "completed", "pages_crawled": 542, ...}
```

### Extraction Helpers (NEW)
```bash
# Batch extract
POST /extract/batch
{"urls": ["url1", "url2"], "format": "markdown", "concurrency": 10}

# Spider + Extract
POST /spider+extract
{"seeds": ["..."], "extract_pattern": ".*\\/article\\/.*"}
```

## 🎨 Field Selection Examples

```bash
# Lightweight (exclude heavy fields)
?include=title,links

# Full page (include everything)
?include=url,depth,status_code,title,content,markdown,links

# Exclude only content
?exclude=content

# Default (no params)
# → includes all except content,markdown
```

## 📡 Streaming Protocols

**NDJSON:**
```bash
curl -H "Accept: application/x-ndjson" /spider/crawl?result_mode=stream
```
```json
{"type":"page","data":{"url":"...","title":"..."}}
{"type":"page","data":{"url":"...","title":"..."}}
{"type":"stats","data":{"pages_crawled":2}}
```

**SSE:**
```bash
curl -H "Accept: text/event-stream" /spider/crawl?result_mode=stream
```
```
data: {"type":"page","data":{"url":"..."}}

data: {"type":"stats","data":{"pages_crawled":2}}

```

## 🛡️ Safety Limits

```rust
max_pages_per_request: 1000     // Pages mode limit
max_content_bytes: 1MB          // Per-page truncation
max_discovered_urls: 10,000     // URL list limit
max_stored_jobs_per_user: 100   // Concurrent jobs
job_retention_days: 30          // Storage duration
```

## 🚨 Error Codes

```
VALIDATION_ERROR     → Invalid field names or parameters
QUOTA_EXCEEDED       → Too many concurrent jobs
RESULT_TOO_LARGE     → Use Stream or Store mode instead
```

## 📝 Response Examples

**Pages Mode:**
```json
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
      "truncated": false,
      "crawled_at": "2025-10-29T12:00:00Z"
    }
  ]
}
```

**Stream Mode (NDJSON):**
```json
{"type":"page","data":{"url":"https://example.com","depth":0}}
{"type":"page","data":{"url":"https://example.com/page1","depth":1}}
{"type":"stats","data":{"pages_crawled":2,"pages_failed":0}}
```

**Store Mode:**
```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "running",
  "created_at": "2025-10-29T12:00:00Z"
}
```

## 🗂️ Database Schema

**spider_jobs:**
```sql
job_id (UUID), status, created_at, completed_at,
pages_crawled, pages_failed, duration_seconds, stop_reason
```

**spider_pages:**
```sql
page_id (BIGSERIAL), job_id (FK), url, title, content,
markdown, links (JSONB), depth, status_code, ...
```

## 🔄 Workflow: Spider + Extract

```
1. Crawl → Discover URLs
2. Filter → Apply regex pattern
3. Extract → Batch markdown extraction
4. Combine → Crawl metadata + extracted content
```

```bash
POST /spider+extract
{
  "seeds": ["https://example.com"],
  "scope": {"max_depth": 2},
  "extract_pattern": ".*\\/article\\/.*",
  "include": ["markdown", "title"]
}
```

## 📚 Implementation Files

**Core Structs:**
- `riptide-api/src/dto.rs` → CrawledPage, SpiderResultPages, ResultMode

**Handlers:**
- `riptide-api/src/handlers/spider_pages.rs` → PageBuilder
- `riptide-api/src/handlers/spider_stream.rs` → NDJSON/SSE
- `riptide-api/src/handlers/spider_jobs.rs` → Job storage
- `riptide-api/src/handlers/extract_batch.rs` → Batch extract
- `riptide-api/src/handlers/spider_extract.rs` → Spider+extract

**Migrations:**
- `migrations/001_spider_jobs.sql`
- `migrations/002_spider_pages.sql`

## 🎯 Decision Flow

```
result_mode=pages → Check page count
  ├─ <= 1000 pages → Return JSON
  └─ > 1000 pages → Error: Use Stream/Store

result_mode=stream → Content-Type negotiation
  ├─ Accept: application/x-ndjson → NDJSON
  └─ Accept: text/event-stream → SSE

result_mode=store → Create job
  ├─ Spawn background task
  ├─ Store to database
  └─ Return job_id
```

## ⚙️ Configuration

```rust
// In config
pub struct SpiderLimits {
    pub max_pages_per_request: usize,
    pub max_content_bytes: usize,
    pub max_discovered_urls: usize,
    pub max_stored_jobs_per_user: usize,
    pub job_retention_days: u32,
}
```

## 🧪 Testing Checklist

- [ ] Field selection filters correctly
- [ ] Content truncation sets `truncated: true`
- [ ] NDJSON streaming emits final stats
- [ ] SSE streaming handles disconnects
- [ ] Job pagination cursor works
- [ ] Per-page errors don't fail crawl
- [ ] Quota enforcement blocks excess jobs
- [ ] Response size limits trigger errors

## 📖 Full Documentation

**Main Architecture:** `/workspaces/eventmesh/docs/architecture/phase2-api-design.md`
**Data Flows:** `/workspaces/eventmesh/docs/architecture/phase2-data-flow.md`
**Summary:** `/workspaces/eventmesh/docs/architecture/PHASE2_ARCHITECTURE_SUMMARY.md`
**Completion Report:** `/workspaces/eventmesh/docs/architecture/PHASE2_COMPLETION_REPORT.md`

---

**Quick Start:** Read phase2-api-design.md Section 3 (Core Data Structures) → Section 4 (Result Mode Design) → Section 11 (Implementation Plan)
