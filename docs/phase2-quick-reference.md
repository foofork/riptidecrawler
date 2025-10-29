# Phase 2 Quick Reference

## Implementation Files

### 1. Core Data Structures
**File:** `/workspaces/eventmesh/crates/riptide-api/src/dto.rs`

- `ResultMode` enum (lines 3-23): Stats, Urls, Pages, Stream, Store
- `CrawledPage` struct (lines 92-244): Full page object with all metadata
- `SpiderResultPages` struct (lines 266-310): Pages result wrapper
- `FieldFilter` struct (lines 246-264): Field selection support
- Unit tests (lines 312-478): 12 comprehensive tests

### 2. API Handler
**File:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/spider.rs`

- `SpiderCrawlQuery` (lines 27-41): Query parameters with include/exclude/max_content_bytes
- Pages mode handler (lines 188-221): Implementation of result_mode=pages

### 3. Test Suite
**File:** `/workspaces/eventmesh/tests/unit/phase2_result_mode_tests.rs`

- 24 comprehensive tests for all Phase 2 features

## API Endpoints

### Spider Crawl with Pages Mode
```bash
POST /spider/crawl?result_mode=pages&include=title,links,markdown&max_content_bytes=500000
```

### Query Parameters
- `result_mode`: stats | urls | pages | stream | store
- `include`: Comma-separated list of fields to include
- `exclude`: Comma-separated list of fields to exclude
- `max_content_bytes`: Maximum size for content/markdown fields (default: 1MB)

## Available Fields in CrawledPage

### Always Present
- `url`: String
- `depth`: u32
- `status_code`: u16
- `links`: Vec<String>

### Optional (use include/exclude to control)
- `title`: Option<String>
- `content`: Option<String> - Raw HTML/text
- `markdown`: Option<String> - Normalized markdown
- `truncated`: Option<bool> - Set if content was truncated
- `final_url`: Option<String> - URL after redirects
- `mime`: Option<String> - MIME type
- `fetch_time_ms`: Option<u64> - Fetch duration
- `robots_obeyed`: Option<bool> - Robots.txt compliance
- `fetch_error`: Option<String> - Fetch error message
- `parse_error`: Option<String> - Parse error message

## Testing

### Run all Phase 2 tests
```bash
cargo test --package riptide-api --lib dto
```

### Run comprehensive test suite
```bash
cargo test phase2_result_mode
```

## Backward Compatibility

✅ `result_mode=stats` - Returns statistics only (existing behavior)
✅ `result_mode=urls` - Returns statistics + discovered_urls (existing)
✅ Default mode is `stats` if not specified

## Build Status

```bash
cargo build --package riptide-api
cargo test --package riptide-api --lib dto
```

All tests passing ✅
