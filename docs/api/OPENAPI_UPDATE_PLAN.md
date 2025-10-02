# OpenAPI Specification Update Plan

**Generated**: 2025-10-01
**Current Coverage**: 18% (9 of 51 endpoints)
**Target Coverage**: 100% (all 51 endpoints)

## Summary

The current OpenAPI specification (`docs/api/openapi.yaml`) is severely outdated and missing 42 endpoints across 8 major feature areas. This document outlines all required additions.

## Missing Endpoint Categories

### 1. PDF Processing (3 endpoints) - HIGH PRIORITY
- `POST /pdf/process` - Synchronous PDF processing
- `POST /pdf/process-stream` - Streaming PDF processing with progress
- `GET /pdf/health` - PDF service health check

### 2. Table Extraction (2 endpoints) - HIGH PRIORITY
- `POST /api/v1/tables/extract` - Extract tables from HTML
- `GET /api/v1/tables/{id}/export` - Export table as CSV/Markdown

### 3. LLM Provider Management (4 endpoints) - MEDIUM PRIORITY
- `GET /api/v1/llm/providers` - List available LLM providers
- `POST /api/v1/llm/providers/switch` - Switch active provider
- `GET /api/v1/llm/config` - Get LLM configuration
- `POST /api/v1/llm/config` - Update LLM configuration

### 4. Stealth (4 endpoints) - MEDIUM PRIORITY
- `POST /stealth/configure` - Configure stealth settings
- `POST /stealth/test` - Test stealth capabilities
- `GET /stealth/capabilities` - Get available stealth features
- `GET /stealth/health` - Stealth service health

### 5. Spider Crawling (3 endpoints) - MEDIUM PRIORITY
- `POST /spider/crawl` - Deep spider crawl
- `POST /spider/status` - Get spider status
- `POST /spider/control` - Control spider operations

### 6. Session Management (15 endpoints) - MEDIUM PRIORITY
- `POST /sessions` - Create new session
- `GET /sessions` - List all sessions
- `GET /sessions/stats` - Get session statistics
- `POST /sessions/cleanup` - Cleanup expired sessions
- `GET /sessions/{session_id}` - Get session info
- `DELETE /sessions/{session_id}` - Delete session
- `POST /sessions/{session_id}/extend` - Extend session
- `POST /sessions/{session_id}/cookies` - Set cookie
- `DELETE /sessions/{session_id}/cookies` - Clear all cookies
- `GET /sessions/{session_id}/cookies/{domain}` - Get domain cookies
- `GET /sessions/{session_id}/cookies/{domain}/{name}` - Get specific cookie
- `DELETE /sessions/{session_id}/cookies/{domain}/{name}` - Delete cookie

### 7. Worker Management (9 endpoints) - LOW PRIORITY (Placeholder Implementation)
- `POST /workers/jobs` - Submit job
- `GET /workers/jobs/{job_id}` - Get job status
- `GET /workers/jobs/{job_id}/result` - Get job result
- `GET /workers/stats/queue` - Get queue stats
- `GET /workers/stats/workers` - Get worker pool stats
- `GET /workers/metrics` - Get worker metrics
- `POST /workers/schedule` - Create scheduled job
- `GET /workers/schedule` - List scheduled jobs
- `DELETE /workers/schedule/{job_id}` - Delete scheduled job

### 8. Strategies Extraction (2 endpoints) - MEDIUM PRIORITY
- `POST /strategies/crawl` - Strategies-based crawling
- `GET /strategies/info` - Get available strategies

## Required Schema Additions (30+ schemas)

### PDF Schemas
- `PdfProcessRequest`
- `PdfProcessResponse`
- `ProcessingStats`
- `EnhancedProgressUpdate`

### Table Schemas
- `TableExtractionRequest`
- `TableExtractionOptions`
- `TableExtractionResponse`
- `TableSummary`
- `TableMetadata`

### LLM Schemas
- `ProvidersResponse`
- `ProviderInfo`
- `SwitchProviderRequest`
- `SwitchProviderResponse`
- `ConfigSummary`
- `ConfigUpdateRequest`
- `ConfigUpdateResponse`

### Stealth Schemas
- `StealthConfigRequest`
- `StealthConfigResponse`
- `StealthTestRequest`
- `StealthTestResponse`
- `StealthMetrics`

### Spider Schemas
- `SpiderCrawlBody`
- `SpiderCrawlResponse`
- `SpiderStatusRequest`
- `SpiderStatusResponse`
- `SpiderControlRequest`

### Session Schemas
- `CreateSessionResponse`
- `SessionInfoResponse`
- `CookieResponse`
- `SetCookieRequest`
- `ExtendSessionRequest`

### Worker Schemas
- `SubmitJobRequest`
- `SubmitJobResponse`
- `JobStatusResponse`
- `JobResultResponse`
- `QueueStatsResponse`
- `WorkerPoolStatsResponse`
- `ScheduledJobResponse`
- `CreateScheduledJobRequest`

### Strategies Schemas
- `StrategiesCrawlRequest`
- `StrategiesCrawlResponse`
- `StrategiesInfo`
- `ChunkingConfig`
- `TopicChunkingConfig`

## Required Tag Additions

```yaml
tags:
  - name: PDF
    description: PDF processing and extraction
  - name: Tables
    description: Table extraction from HTML
  - name: LLM
    description: LLM provider management
  - name: Stealth
    description: Stealth configuration and anti-detection
  - name: Spider
    description: Deep web crawling with spider engine
  - name: Sessions
    description: Browser session and cookie management
  - name: Workers
    description: Background job processing
  - name: Strategies
    description: Advanced extraction strategies
```

## Implementation Priority

### Phase 1 (Week 1) - Critical Documentation
1. Add PDF endpoints (3)
2. Add Table extraction endpoints (2)
3. Add LLM management endpoints (4)
4. Add all related schemas

### Phase 2 (Week 2) - Extended Features
1. Add Spider endpoints (3)
2. Add Session management (15)
3. Add Worker management (9)
4. Add Strategies endpoints (2)
5. Add all related schemas

### Phase 3 (Week 3) - Enhancements
1. Add Stealth endpoints (4)
2. Enhance /crawl with chunking/spider options
3. Fix /render PDF confusion
4. Add comprehensive examples

### Phase 4 (Week 4) - Validation
1. Generate client SDKs from OpenAPI
2. Test all endpoints against spec
3. Validate all schemas
4. Add integration tests

## Fixes Required

### Remove PDF from /render
The current spec incorrectly documents PDF processing as part of `/render` endpoint. PDF processing should be exclusively via `/pdf` endpoints.

**Changes needed:**
1. Remove `pdf` from `RenderMode` enum
2. Remove `pdf_config` from `RenderRequest`
3. Remove `pdf_result` from `RenderResponse`
4. Add note in `/render` description redirecting to `/pdf` endpoints

### Enhance /crawl Options
Add missing fields to `CrawlOptions`:
- `chunking_config`
- `use_spider`
- `spider_max_depth`
- `spider_strategy`

## Expected File Size
- Current: 1,292 lines
- Estimated after update: ~3,500-4,000 lines
- Increase: ~2,700 lines (~270% growth)

## Validation Commands

```bash
# Validate OpenAPI spec
npx @apidevtools/swagger-cli validate docs/api/openapi.yaml

# Generate client
npx @openapitools/openapi-generator-cli generate \
  -i docs/api/openapi.yaml \
  -g typescript-axios \
  -o generated/typescript-client

# Test against live API
newman run docs/api/openapi-tests.json \
  --environment prod.postman_environment.json
```

## Next Steps

1. Create full OpenAPI 3.0 spec with all endpoints
2. Add all 30+ schemas with complete field definitions
3. Include request/response examples for each endpoint
4. Add security schemes where applicable
5. Document all query parameters and headers
6. Add comprehensive error responses
7. Validate spec with tooling
8. Generate and test client SDKs

## References

- Handler implementation: `crates/riptide-api/src/handlers/`
- Current OpenAPI: `docs/api/openapi.yaml`
- API documentation: `docs/api/README.md`
- Crosswalk analysis: Agent analysis output from 2025-10-01
