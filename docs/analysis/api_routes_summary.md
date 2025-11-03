# RipTide EventMesh API Routes Summary

**Generated**: 2025-11-03
**Analyzed by**: API Routes Researcher (Hive Mind Swarm)
**Status**: ✅ Complete

## Overview

The RipTide EventMesh codebase contains a comprehensive HTTP API with **120+ routes** spread across multiple services, plus **1 WebSocket endpoint** for real-time bidirectional communication and **5 streaming endpoints** using NDJSON format.

## Service Architecture

### 1. **riptide-api** (Main API Server)
- **Bind Address**: `0.0.0.0:8080`
- **Routes**: 100+ HTTP endpoints
- **Entry Point**: `/workspaces/eventmesh/crates/riptide-api/src/main.rs`
- **Framework**: Axum (Rust async web framework)

### 2. **riptide-streaming** (Streaming Server)
- **Bind Address**: Configurable
- **Routes**: 3 endpoints (2 streaming + 1 health)
- **Entry Point**: `/workspaces/eventmesh/crates/riptide-streaming/src/server.rs`
- **Purpose**: NDJSON streaming responses

### 3. **riptide-headless** (Browser Service)
- **Bind Address**: `0.0.0.0:9123`
- **Routes**: 2 endpoints
- **Entry Point**: `/workspaces/eventmesh/crates/riptide-headless/src/main.rs`
- **Purpose**: Headless browser rendering

### 4. **riptide-performance** (Monitoring)
- **Bind Address**: Configurable
- **Routes**: 6 memory profiling endpoints
- **Entry Point**: `/workspaces/eventmesh/crates/riptide-performance/src/monitoring/http_endpoints.rs`
- **Purpose**: Performance monitoring and memory profiling

## API Route Categories

### Core Functionality (20 routes)
- **Health & Metrics**: `/healthz`, `/metrics`, `/api/health/detailed`
- **Crawling**: `/crawl`, `/crawl/stream`, `/api/v1/crawl`
- **Extraction**: `/api/v1/extract`, `/extract`
- **Search**: `/api/v1/search`, `/search`, `/deepsearch`, `/deepsearch/stream`

### Advanced Features (40+ routes)
- **PDF Processing** (4): `/pdf/process`, `/pdf/upload`, `/pdf/process-stream`, `/pdf/healthz`
- **Stealth** (4): `/stealth/configure`, `/stealth/test`, `/stealth/capabilities`, `/stealth/healthz`
- **Table Extraction** (2): `/api/v1/tables/extract`, `/api/v1/tables/:id/export`
- **LLM Providers** (5): `/api/v1/llm/providers`, `/api/v1/llm/config`, etc.
- **Content Chunking** (1): `/api/v1/content/chunk`
- **Engine Selection** (4): `/engine/analyze`, `/engine/decide`, `/engine/stats`, `/engine/probe-first`
- **Domain Profiles** (6): `/api/v1/profiles`, `/api/v1/profiles/:domain`, etc.

### Orchestration (10+ routes)
- **Strategies** (2): `/strategies/crawl`, `/strategies/info`
- **Spider** (3): `/spider/crawl`, `/spider/status`, `/spider/control`

### Session Management (12 routes)
- **Sessions**: `/sessions`, `/sessions/:session_id`, `/sessions/:session_id/extend`
- **Cookies**: `/sessions/:session_id/cookies/:domain/:name`

### Background Processing (10 routes)
- **Workers**: `/workers/jobs`, `/workers/jobs/:job_id`, `/workers/stats/*`, `/workers/schedule`

### Browser Management (4 routes)
- `/api/v1/browser/session`, `/api/v1/browser/action`, `/api/v1/browser/pool/status`

### Monitoring & Observability (20+ routes)
- **Resources** (6): `/resources/status`, `/resources/browser-pool`, `/resources/memory`, etc.
- **Monitoring** (9): `/monitoring/health-score`, `/monitoring/performance-report`, `/monitoring/alerts/*`
- **Profiling** (6): `/api/profiling/memory`, `/api/profiling/cpu`, `/api/profiling/bottlenecks`, etc.
- **Telemetry** (3): `/api/telemetry/status`, `/api/telemetry/traces`, `/api/telemetry/traces/:trace_id`
- **Memory** (2): `/api/v1/memory/profile`, `/api/v1/memory/leaks`

### Admin (Feature-Gated) (13 routes)
**Requires**: `persistence` feature flag
- **Tenants** (7): CRUD operations for multi-tenancy
- **Cache** (3): Warm, invalidate, stats
- **State** (3): Reload, checkpoint, restore

## Middleware Stack

### Global Middleware (Applied to all routes)
1. **TraceLayer** - Request tracing and logging (OpenTelemetry support)
2. **CompressionLayer** - Response compression (gzip/brotli)
3. **TimeoutLayer** - Request timeout handling
4. **CorsLayer** - CORS support (permissive/configurable)
5. **PayloadLimitLayer** - Request size limiting

### Conditional Middleware
1. **auth_middleware** - API key authentication
2. **rate_limit_middleware** - Rate limiting per client
3. **request_validation_middleware** - Input validation
4. **SessionLayer** - Session management for stateful routes

## WebSocket Endpoint

**Path**: `/ws/crawl` (WebSocket upgrade)
**Handler**: `crawl_websocket -> handle_websocket`
**Location**: `/workspaces/eventmesh/crates/riptide-api/src/streaming/websocket.rs`

### Features
- ✅ Bidirectional real-time communication
- ✅ Ping/pong keepalive (30-second interval)
- ✅ Backpressure handling
- ✅ Connection health monitoring
- ✅ Progress tracking
- ✅ Real-time result streaming

### Message Types (Client → Server)
- `crawl` - Submit crawl request
- `ping` - Ping request
- `status` - Connection status request

### Message Types (Server → Client)
- `welcome` - Initial welcome message
- `metadata` - Crawl metadata
- `result` - Individual crawl result
- `summary` - Final summary
- `pong` - Pong response
- `status` - Status response
- `error` - Error message

## Streaming Endpoints

### NDJSON Streaming (5 endpoints)
1. `/crawl/stream` - Streaming crawl with progress
2. `/api/v1/crawl/stream` - Streaming crawl (v1 alias)
3. `/deepsearch/stream` - Streaming deep search
4. `/api/v1/deepsearch/stream` - Streaming deep search (v1 alias)
5. `/pdf/process-stream` - PDF processing with progress

**Response Format**: `application/x-ndjson` (newline-delimited JSON)

## Authentication & Security

### Authentication
- **Method**: API key authentication via `auth_middleware`
- **Applied to**: Most routes except health checks and metrics
- **Header**: `Authorization: Bearer <api_key>` (assumed)

### Rate Limiting
- **Applied to**: All mutation operations (POST, PUT, DELETE)
- **Implementation**: `rate_limit_middleware`
- **Scope**: Per client/API key

### Input Validation
- **Applied to**: All user input
- **Implementation**: `request_validation_middleware`
- **Validation**: Request size, content type, required fields

## Request/Response Patterns

### Standard Request Types
- **CrawlBody**: URLs + options
- **ExtractRequest**: Single URL extraction
- **DeepSearchRequest**: Query-based search
- **PDFProcessingRequest**: Multipart file upload
- **SessionRequest**: Session creation/management
- **WorkerJobRequest**: Background job submission

### Standard Response Types
- **CrawlResponse**: Results array + metadata
- **ExtractionResult**: Extracted content + metadata
- **StreamingResponse**: NDJSON lines
- **ErrorResponse**: Error type + message + retryable flag
- **HealthResponse**: Component status + metrics

## Notable Features

### Phase-Based Development
- **Phase 10**: Engine selection and optimization
- **Phase 10.4**: Domain warm-start caching with profiles

### Performance Monitoring
- **jemalloc Integration**: Memory profiling in production
- **Real-time Metrics**: Prometheus endpoint at `/metrics`
- **Memory Leak Detection**: Automated leak detection and reporting
- **WASM Health**: WASM instance monitoring

### Distributed Tracing
- **OpenTelemetry**: Full distributed tracing support
- **Trace Visualization**: `/api/telemetry/traces/:trace_id` endpoint
- **Span Collection**: Automatic span collection across services

### Background Processing
- **Redis-backed Queue**: Job queue with Redis persistence
- **Scheduled Jobs**: Cron-like job scheduling
- **Worker Pool**: Configurable worker concurrency
- **Job States**: Pending, running, completed, failed

## File Locations

### Route Definitions
- Main router: `/workspaces/eventmesh/crates/riptide-api/src/main.rs`
- Route modules: `/workspaces/eventmesh/crates/riptide-api/src/routes/*.rs`
- Handlers: `/workspaces/eventmesh/crates/riptide-api/src/handlers/*.rs`

### Key Files
- **PDF routes**: `crates/riptide-api/src/routes/pdf.rs`
- **Stealth routes**: `crates/riptide-api/src/routes/stealth.rs`
- **LLM routes**: `crates/riptide-api/src/routes/llm.rs`
- **Profile routes**: `crates/riptide-api/src/routes/profiles.rs`
- **Engine routes**: `crates/riptide-api/src/routes/engine.rs`
- **WebSocket**: `crates/riptide-api/src/streaming/websocket.rs`

## Complete Catalog

**Full JSON catalog**: `/workspaces/eventmesh/docs/analysis/api_routes_catalog.json`

The complete catalog includes:
- All route paths and methods
- Handler function names
- Request/response type definitions
- Authentication requirements
- Rate limiting status
- Streaming capabilities
- Feature flags
- Middleware stack details

## Summary Statistics

| Metric | Count |
|--------|-------|
| **Total HTTP Routes** | 120+ |
| **WebSocket Endpoints** | 1 |
| **Streaming Endpoints** | 5 |
| **Servers** | 4 |
| **Route Categories** | 15 |
| **Handler Files** | 35+ |
| **Feature-Gated Routes** | 13 |
| **Deprecated Routes** | 3 |

## Next Steps

This catalog provides a complete map of all public interfaces in the RipTide EventMesh system. Use this for:
- API documentation generation
- Client SDK development
- Integration testing
- Security auditing
- Performance optimization
- Load balancing configuration

---

**Coordination**: Findings stored in memory at key `hive/analysis/api-routes`
**Status**: Ready for integration with other Hive Mind agents
