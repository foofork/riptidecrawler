# Phase 2 Data Flow Diagrams

## 1. ResultMode Decision Flow

```
                        POST /spider/crawl?result_mode=<mode>
                                        |
                                        v
                        ┌───────────────────────────────┐
                        │  Parse result_mode parameter  │
                        └───────────────┬───────────────┘
                                        |
                ┌───────────────────────┼───────────────────────┐
                |                       |                       |
                v                       v                       v
        ┌───────────┐           ┌───────────┐         ┌───────────┐
        │   Stats   │           │    Urls   │         │   Pages   │
        │  (Phase 0)│           │ (Phase 1) │         │ (Phase 2) │
        └─────┬─────┘           └─────┬─────┘         └─────┬─────┘
              |                       |                       |
              v                       v                       v
    Return stats only     Return stats + URLs    Apply field selection
                                                          |
                                                          v
                                                  Check page count
                                                          |
                                              ┌───────────┴────────────┐
                                              |                        |
                                              v                        v
                                        <= 1000 pages              > 1000 pages
                                              |                        |
                                              v                        v
                                  Return JSON response      Error: Use Stream/Store


                ┌───────────────────────────────────────────────────────┐
                |                                                       |
                v                                                       v
        ┌───────────┐                                          ┌───────────┐
        │  Stream   │                                          │   Store   │
        │ (Phase 2) │                                          │ (Phase 2) │
        └─────┬─────┘                                          └─────┬─────┘
              |                                                       |
              v                                                       v
    Content Negotiation                                    Create job record
              |                                                       |
    ┌─────────┴──────────┐                                          v
    |                    |                                  Spawn background task
    v                    v                                           |
NDJSON              SSE                                              v
streaming         streaming                               Execute crawl → Store pages
```

## 2. Field Selection Flow

```
┌──────────────────────────────────────────────────────────────┐
│  Query: ?include=title,links&exclude=content                 │
└────────────────────────┬─────────────────────────────────────┘
                         |
                         v
            ┌────────────────────────┐
            │  FieldSelector::parse  │
            │                        │
            │  include: {title,links}│
            │  exclude: {content}    │
            └───────────┬────────────┘
                        |
                        v
        ┌───────────────────────────────┐
        │    For each CrawledPage       │
        └───────────┬───────────────────┘
                    |
        ┌───────────┴────────────┐
        |                        |
        v                        v
  Core fields              Optional fields
  (always included)        (check selector)
        |                        |
        v                        v
  - url                    should_include("title")?
  - depth                         |
  - status_code             ┌─────┴─────┐
                            |           |
                            v           v
                          YES          NO
                            |           |
                            v           v
                    Include field   Skip field


Final CrawledPage:
{
  "url": "...",              ✓ Core field
  "depth": 1,                ✓ Core field
  "status_code": 200,        ✓ Core field
  "title": "Example",        ✓ In include list
  "content": null,           ✗ In exclude list
  "links": ["..."],          ✓ In include list
  "markdown": null           ✗ Not in include list
}
```

## 3. Streaming Data Flow (NDJSON)

```
┌─────────────────────────────────────────────────────────────┐
│  Client: Accept: application/x-ndjson                       │
└────────────────────────┬────────────────────────────────────┘
                         |
                         v
            ┌────────────────────────┐
            │  Start spider crawl    │
            └───────────┬────────────┘
                        |
                        v
        ┌───────────────────────────────┐
        │  Create pages stream          │
        │  (tokio_stream::StreamExt)    │
        └───────────┬───────────────────┘
                    |
        ┌───────────┴────────────┐
        |                        |
        v                        v
  Page discovered          Apply field selection
  from crawler                   |
        |                        v
        |              Build CrawledPage
        |                        |
        |                        v
        |              Serialize to JSON
        |                        |
        └────────────┬───────────┘
                     |
                     v
    ┌────────────────────────────────────┐
    │  Stream to client                  │
    │                                    │
    │  {"type":"page","data":{...}}      │
    │  {"type":"page","data":{...}}      │
    │  {"type":"page","data":{...}}      │
    └────────────────┬───────────────────┘
                     |
                     v
            Crawl complete
                     |
                     v
    ┌────────────────────────────────────┐
    │  Send final stats event            │
    │                                    │
    │  {"type":"stats","data":{          │
    │    "pages_crawled": 142,           │
    │    "pages_failed": 3,              │
    │    "duration_seconds": 25.4        │
    │  }}                                │
    └────────────────────────────────────┘
                     |
                     v
              Close stream
```

## 4. Job Storage Lifecycle

```
┌─────────────────────────────────────────────────────────────┐
│  POST /spider/crawl?result_mode=store                       │
└────────────────────────┬────────────────────────────────────┘
                         |
                         v
            ┌────────────────────────┐
            │  Generate job_id (UUID)│
            └───────────┬────────────┘
                        |
                        v
        ┌───────────────────────────────┐
        │  INSERT INTO spider_jobs      │
        │  status = 'running'           │
        └───────────┬───────────────────┘
                    |
        ┌───────────┴────────────┐
        |                        |
        v                        v
  Return job_id           Spawn background task
  to client                      |
        |                        v
        |              Execute spider crawl
        |                        |
        |              ┌─────────┴─────────┐
        |              |                   |
        |              v                   v
        |        For each page       Apply field selection
        |              |                   |
        |              v                   |
        |        Build CrawledPage         |
        |              |                   |
        |              └─────────┬─────────┘
        |                        |
        |                        v
        |          ┌─────────────────────────┐
        |          │ INSERT INTO spider_pages│
        |          │ - url, title, content.. │
        |          └─────────────┬───────────┘
        |                        |
        v                        v
  Client polls          Crawl complete
  job status                   |
        |                      v
        |          ┌───────────────────────┐
        |          │ UPDATE spider_jobs    │
        |          │ status = 'completed'  │
        |          │ pages_crawled = N     │
        |          └───────────┬───────────┘
        |                      |
        v                      v
  ┌────────────────────────────────────┐
  │  GET /jobs/{id}/results?cursor=... │
  └────────────────┬───────────────────┘
                   |
                   v
      ┌────────────────────────┐
      │  SELECT FROM           │
      │  spider_pages          │
      │  WHERE job_id = ?      │
      │  AND page_id > cursor  │
      │  LIMIT 100             │
      └───────────┬────────────┘
                  |
                  v
    Return paginated results
    with next_cursor
```

## 5. Spider + Extract Workflow

```
┌─────────────────────────────────────────────────────────────┐
│  POST /spider+extract                                        │
│  {                                                           │
│    "seeds": ["https://example.com"],                        │
│    "extract_pattern": ".*\\/article\\/.*"                   │
│  }                                                           │
└────────────────────────┬────────────────────────────────────┘
                         |
                         v
            ┌────────────────────────┐
            │  Step 1: Crawl         │
            │  Discover URLs         │
            └───────────┬────────────┘
                        |
                        v
        ┌───────────────────────────────┐
        │  discovered_urls:             │
        │  - https://example.com        │
        │  - https://example.com/about  │
        │  - https://example.com/article/1  ← Match
        │  - https://example.com/article/2  ← Match
        │  - https://example.com/contact│
        └───────────┬───────────────────┘
                    |
                    v
        ┌────────────────────────┐
        │  Step 2: Filter        │
        │  Apply regex pattern   │
        └───────────┬────────────┘
                    |
                    v
        ┌───────────────────────────────┐
        │  filtered_urls:               │
        │  - https://example.com/article/1 │
        │  - https://example.com/article/2 │
        └───────────┬───────────────────┘
                    |
                    v
        ┌────────────────────────┐
        │  Step 3: Batch Extract │
        │  concurrency = 10      │
        └───────────┬────────────┘
                    |
        ┌───────────┴────────────┐
        |                        |
        v                        v
  Extract article/1       Extract article/2
        |                        |
        v                        v
  Get markdown            Get markdown
  + metadata              + metadata
        |                        |
        └───────────┬────────────┘
                    |
                    v
        ┌────────────────────────┐
        │  Step 4: Combine       │
        │  Crawl data + Extract  │
        └───────────┬────────────┘
                    |
                    v
    ┌────────────────────────────────────┐
    │  Response:                         │
    │  {                                 │
    │    "crawl_stats": {                │
    │      "pages_crawled": 5,           │
    │      "discovered_urls": 5          │
    │    },                              │
    │    "extracted_pages": [            │
    │      {                             │
    │        "url": ".../article/1",     │
    │        "markdown": "# Article...", │
    │        "crawl_depth": 1            │
    │      }                             │
    │    ]                               │
    │  }                                 │
    └────────────────────────────────────┘
```

## 6. Content Truncation Flow

```
┌─────────────────────────────────────────────────────────────┐
│  Page fetched with content_length = 5MB                     │
└────────────────────────┬────────────────────────────────────┘
                         |
                         v
            ┌────────────────────────┐
            │  Check size limits     │
            │  max_content_bytes=1MB │
            └───────────┬────────────┘
                        |
        ┌───────────────┴────────────┐
        |                            |
        v                            v
  content.len() <= 1MB      content.len() > 1MB
        |                            |
        v                            v
  Keep full content         Truncate to 1MB
  truncated = false         truncated = true
        |                            |
        └────────────┬───────────────┘
                     |
                     v
        ┌────────────────────────┐
        │  Build CrawledPage     │
        │  {                     │
        │    "content": "...",   │
        │    "truncated": bool   │
        │  }                     │
        └────────────────────────┘


Client receives:
{
  "url": "https://example.com/huge-page",
  "content": "First 1MB of content...",
  "truncated": true  ← Client knows content is incomplete
}
```

## 7. Error Handling Flow

```
┌─────────────────────────────────────────────────────────────┐
│  Spider crawling page: https://example.com/broken           │
└────────────────────────┬────────────────────────────────────┘
                         |
                         v
            ┌────────────────────────┐
            │  Attempt HTTP fetch    │
            └───────────┬────────────┘
                        |
        ┌───────────────┴────────────┐
        |                            |
        v                            v
  Fetch succeeds            Fetch fails (timeout)
        |                            |
        v                            v
  Parse HTML               Set fetch_error
        |                            |
┌───────┴────────┐                  v
|                |          Build CrawledPage with error
v                v                  |
Parse OK    Parse fails             |
    |            |                  |
    v            v                  |
Return page  Set parse_error        |
             |                      |
             v                      |
     Build CrawledPage              |
     with parse_error               |
             |                      |
             └──────────┬───────────┘
                        |
                        v
            ┌────────────────────────┐
            │  Include in response   │
            │  Don't fail entire     │
            │  crawl for one error   │
            └────────────────────────┘


Example error page:
{
  "url": "https://example.com/broken",
  "depth": 1,
  "status_code": 0,
  "fetch_error": "Connection timeout after 30s",
  "title": null,
  "content": null
}

Crawl continues with remaining pages
```

## 8. Pagination Cursor Flow

```
┌─────────────────────────────────────────────────────────────┐
│  Client: GET /jobs/{id}/results?limit=100                   │
└────────────────────────┬────────────────────────────────────┘
                         |
                         v
            ┌────────────────────────┐
            │  Request 1: No cursor  │
            │  cursor = 0 (start)    │
            └───────────┬────────────┘
                        |
                        v
        ┌───────────────────────────────┐
        │  SELECT * FROM spider_pages   │
        │  WHERE job_id = ?             │
        │  AND page_id > 0              │
        │  ORDER BY page_id ASC         │
        │  LIMIT 100                    │
        └───────────┬───────────────────┘
                    |
                    v
    ┌────────────────────────────────────┐
    │  Response 1:                       │
    │  {                                 │
    │    "pages": [                      │
    │      {page_id: 1, ...},            │
    │      {page_id: 2, ...},            │
    │      ...                           │
    │      {page_id: 100, ...}           │
    │    ],                              │
    │    "cursor": 100,    ← Next page   │
    │    "has_more": true                │
    │  }                                 │
    └────────────────┬───────────────────┘
                     |
                     v
            ┌────────────────────────┐
            │  Request 2:            │
            │  cursor = 100          │
            └───────────┬────────────┘
                        |
                        v
        ┌───────────────────────────────┐
        │  SELECT * FROM spider_pages   │
        │  WHERE job_id = ?             │
        │  AND page_id > 100            │
        │  ORDER BY page_id ASC         │
        │  LIMIT 100                    │
        └───────────┬───────────────────┘
                    |
                    v
    ┌────────────────────────────────────┐
    │  Response 2:                       │
    │  {                                 │
    │    "pages": [                      │
    │      {page_id: 101, ...},          │
    │      {page_id: 102, ...},          │
    │      ...                           │
    │      {page_id: 200, ...}           │
    │    ],                              │
    │    "cursor": 200,                  │
    │    "has_more": true                │
    │  }                                 │
    └────────────────────────────────────┘

Continue until has_more = false
```

---

## Summary

These data flow diagrams illustrate:

1. **ResultMode Decision Flow**: How the API routes requests to different modes
2. **Field Selection Flow**: How FieldSelector filters CrawledPage fields
3. **Streaming Data Flow**: Real-time page delivery via NDJSON/SSE
4. **Job Storage Lifecycle**: Async crawl execution with pagination
5. **Spider + Extract Workflow**: Orchestrated discover→filter→extract
6. **Content Truncation**: Size limit enforcement with transparency
7. **Error Handling**: Per-page error isolation
8. **Pagination Cursor**: Efficient result traversal

All flows maintain backward compatibility while enabling scalable, production-ready crawling.
