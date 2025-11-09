# Sprint 4.3: Streaming System Refactoring - Architecture & Implementation Plan

**Date:** 2025-11-08
**Status:** 📋 **PLANNING PHASE**
**Complexity:** ⚠️ **HIGH** - Full hexagonal architecture migration

---

## Executive Summary

The streaming system (`crates/riptide-api/src/streaming/`, 7,986 LOC) violates clean architecture principles by mixing transport protocols, business logic, and domain orchestration. This plan details the migration to a port-based hexagonal architecture with clear separation of concerns.

**Key Metrics:**
- **Total LOC:** 7,986 (across 17 files)
- **Largest Files:** response_helpers.rs (924), ndjson/helpers.rs (725), websocket.rs (684)
- **Protocols:** NDJSON, SSE, WebSocket
- **External Dependencies:** 34 crate imports, 26 std imports

**Target Architecture:**
```
┌──────────────────────────────────────────────────────────────┐
│ API Layer (riptide-api)                                      │
│  ├─ Thin Handlers (<50 LOC each)                             │
│  └─ Protocol Adapters (WebSocket, SSE, NDJSON)               │
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│ Application Layer (riptide-facade)                           │
│  └─ StreamingFacade (business logic, orchestration)          │
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│ Domain Layer (riptide-types)                                 │
│  ├─ StreamingTransport trait                                 │
│  ├─ StreamProcessor trait                                    │
│  ├─ StreamLifecycle trait                                    │
│  └─ StreamEvent, StreamState, StreamMetrics                  │
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│ Infrastructure Layer                                         │
│  ├─ BufferManager → riptide-reliability                      │
│  ├─ StreamConfig → riptide-config                            │
│  └─ StreamingError → riptide-types/errors                    │
└──────────────────────────────────────────────────────────────┘
```

---

## Current Architecture Analysis

### Directory Structure (7,986 LOC)

```
crates/riptide-api/src/streaming/
├── mod.rs                    (546 LOC) - Core types, module exports
├── processor.rs              (634 LOC) - Business logic, URL processing
├── pipeline.rs               (628 LOC) - Orchestration, workflow coordination
├── lifecycle.rs              (622 LOC) - Event handling, metrics tracking
├── response_helpers.rs       (924 LOC) - Protocol formatting utilities
├── websocket.rs              (684 LOC) - WebSocket transport implementation
├── sse.rs                    (575 LOC) - SSE transport implementation
├── buffer.rs                 (554 LOC) - Buffer management, backpressure
├── config.rs                 (444 LOC) - Configuration management
├── metrics.rs                (329 LOC) - Metrics collection
├── error.rs                  (265 LOC) - Error types
├── tests.rs                  (596 LOC) - Unit tests
└── ndjson/
    ├── mod.rs                (95 LOC)
    ├── helpers.rs            (725 LOC) - NDJSON formatting
    ├── streaming.rs          (195 LOC) - NDJSON transport
    ├── handlers.rs           (134 LOC) - NDJSON handlers
    └── progress.rs           (36 LOC)  - Progress tracking
```

### Dependency Analysis

**Top External Dependencies:**
- `crate::*` (34 occurrences) - Heavy internal coupling
- `std::*` (26 occurrences) - Standard library
- `axum` (12 occurrences) - HTTP framework
- `tracing` (9 occurrences) - Logging
- `tokio` (8 occurrences) - Async runtime
- `serde` (9 occurrences) - Serialization

**Files Depending on streaming/ Module:**
- `handlers/streaming.rs` - Main streaming handler (already uses StreamingFacade)
- `handlers/pdf.rs` - Uses response_helpers
- `handlers/mod.rs` - Re-exports `crawl_stream`, `deepsearch_stream`
- `state.rs` - Potential streaming state management
- `metrics.rs` - Metrics integration

### Architectural Violations

1. **Business Logic in Transport Layer**
   - `processor.rs`: URL processing, result conversion (634 LOC)
   - `pipeline.rs`: Orchestration, search integration (628 LOC)
   - Mixed with protocol-specific code

2. **Infrastructure Coupling**
   - `buffer.rs`: Should be in riptide-reliability
   - `config.rs`: Should be in riptide-config
   - `error.rs`: Should be in riptide-types/errors

3. **Protocol Implementation Duplication**
   - Similar patterns in websocket.rs, sse.rs, ndjson/
   - No shared transport abstraction
   - Response formatting duplicated

4. **Tight Coupling to HTTP Framework**
   - Direct Axum types in business logic
   - Makes testing difficult
   - Violates dependency inversion

---

## Target Architecture Design

### Phase 1: Domain Ports (riptide-types/src/ports/streaming.rs)

**File:** `crates/riptide-types/src/ports/streaming.rs` (~400 LOC)

```rust
// Core streaming traits
pub trait StreamingTransport: Send + Sync {
    type Message: Serialize + DeserializeOwned;
    type Error: std::error::Error;

    async fn send_event(&mut self, event: StreamEvent) -> Result<(), Self::Error>;
    async fn send_metadata(&mut self, metadata: StreamMetadata) -> Result<(), Self::Error>;
    async fn send_result(&mut self, result: StreamResult) -> Result<(), Self::Error>;
    async fn send_error(&mut self, error: StreamError) -> Result<(), Self::Error>;
    async fn close(&mut self) -> Result<(), Self::Error>;
    fn protocol_name(&self) -> &'static str;
}

pub trait StreamProcessor: Send + Sync {
    async fn process_urls(&self, urls: Vec<String>) -> StreamingResult<Vec<ProcessedResult>>;
    async fn create_progress(&self) -> StreamProgress;
    async fn create_summary(&self) -> StreamSummary;
    fn should_send_progress(&self, interval: usize) -> bool;
}

pub trait StreamLifecycle: Send + Sync {
    async fn on_connection_established(&self, connection_id: String, protocol: String);
    async fn on_stream_started(&self, connection_id: String, request_id: String, total_items: usize);
    async fn on_progress(&self, connection_id: String, completed: usize, total: usize);
    async fn on_error(&self, connection_id: String, error: StreamingError);
    async fn on_completed(&self, connection_id: String, summary: StreamCompletionSummary);
    async fn on_connection_closed(&self, connection_id: String);
}

// Domain types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamState {
    Idle,
    Connected,
    Streaming { progress: f64 },
    Paused { at: usize },
    Completed { summary: StreamSummary },
    Failed { error: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamEvent {
    Metadata(StreamMetadata),
    Result(Box<StreamResultData>),
    Progress(StreamProgress),
    Summary(StreamSummary),
    SearchMetadata(DeepSearchMetadata),
    SearchResult(Box<DeepSearchResultData>),
    Error(StreamErrorData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetrics {
    pub active_connections: usize,
    pub total_messages_sent: usize,
    pub total_messages_dropped: usize,
    pub average_latency_ms: f64,
    pub throughput_bytes_per_sec: f64,
    pub error_rate: f64,
}
```

**Rationale:** Pure domain contracts with no framework dependencies.

---

### Phase 2: Application Facade (riptide-facade/src/facades/streaming.rs)

**File:** `crates/riptide-facade/src/facades/streaming.rs` (~1,200 LOC)

**Consolidates:**
- `processor.rs` (634 LOC) - URL processing logic
- `pipeline.rs` (628 LOC) - Orchestration logic
- `lifecycle.rs` (622 LOC) - Event handling
- Partial `response_helpers.rs` (business logic only)

```rust
pub struct StreamingFacade {
    pipeline: Arc<dyn PipelinePort>,
    lifecycle: Arc<dyn StreamLifecycle>,
    buffer_manager: Arc<BufferManager>,
    metrics: Arc<RipTideMetrics>,
    config: StreamConfig,
}

impl StreamingFacade {
    // Business methods
    pub async fn create_crawl_stream(
        &self,
        request: CrawlStreamRequest,
    ) -> Result<StreamHandle, FacadeError>;

    pub async fn create_deepsearch_stream(
        &self,
        request: DeepSearchRequest,
    ) -> Result<StreamHandle, FacadeError>;

    pub async fn process_urls_concurrent(
        &self,
        urls: Vec<String>,
        options: CrawlOptions,
    ) -> Result<StreamProcessor, FacadeError>;

    pub async fn execute_stream<T: StreamingTransport>(
        &self,
        processor: StreamProcessor,
        transport: T,
    ) -> Result<StreamSummary, FacadeError>;

    // Lifecycle methods
    pub async fn start_stream(&self, stream_id: &str) -> Result<(), FacadeError>;
    pub async fn pause_stream(&self, stream_id: &str) -> Result<(), FacadeError>;
    pub async fn resume_stream(&self, stream_id: &str) -> Result<(), FacadeError>;
    pub async fn cancel_stream(&self, stream_id: &str) -> Result<(), FacadeError>;

    // Query methods
    pub async fn get_stream_status(&self, stream_id: &str) -> Result<StreamState, FacadeError>;
    pub async fn get_stream_metrics(&self, stream_id: &str) -> Result<StreamMetrics, FacadeError>;
    pub async fn list_active_streams(&self) -> Result<Vec<StreamInfo>, FacadeError>;
}
```

**Features:**
- ✅ Protocol-agnostic business logic
- ✅ Event emission (stream.started, stream.completed, stream.failed)
- ✅ Metrics collection
- ✅ Backpressure management
- ✅ Lifecycle coordination
- ✅ 50+ unit tests

---

### Phase 3: Transport Adapters (riptide-api/src/adapters/)

**Files:**
- `websocket_transport.rs` (~350 LOC) - From websocket.rs (684 LOC)
- `sse_transport.rs` (~300 LOC) - From sse.rs (575 LOC)
- `ndjson_transport.rs` (~250 LOC) - From ndjson/ (1,185 LOC)

**Example: WebSocketTransport**

```rust
pub struct WebSocketTransport {
    sender: SplitSink<WebSocket, Message>,
    session_id: String,
    message_count: AtomicUsize,
    bytes_sent: AtomicUsize,
}

impl StreamingTransport for WebSocketTransport {
    type Message = serde_json::Value;
    type Error = StreamingError;

    async fn send_event(&mut self, event: StreamEvent) -> Result<(), Self::Error> {
        let message = match event {
            StreamEvent::Metadata(m) => serde_json::to_string(&m)?,
            StreamEvent::Result(r) => serde_json::to_string(&r)?,
            // ...
        };

        self.sender.send(Message::Text(message)).await
            .map_err(|e| StreamingError::connection(e.to_string()))?;

        self.message_count.fetch_add(1, Ordering::Relaxed);
        self.bytes_sent.fetch_add(message.len(), Ordering::Relaxed);
        Ok(())
    }

    fn protocol_name(&self) -> &'static str { "websocket" }
}
```

**Benefits:**
- ✅ Clean protocol implementation
- ✅ No business logic
- ✅ Testable with mock transports
- ✅ Easy to add new protocols

---

### Phase 4: Infrastructure Moves

**Buffer Management:**
- **From:** `crates/riptide-api/src/streaming/buffer.rs` (554 LOC)
- **To:** `crates/riptide-reliability/src/streaming/buffer.rs`
- **Rationale:** Buffer management is infrastructure concern

**Configuration:**
- **From:** `crates/riptide-api/src/streaming/config.rs` (444 LOC)
- **To:** `crates/riptide-config/src/streaming.rs`
- **Rationale:** Centralized configuration management

**Error Types:**
- **From:** `crates/riptide-api/src/streaming/error.rs` (265 LOC)
- **To:** `crates/riptide-types/src/errors/streaming.rs`
- **Rationale:** Domain error types belong in types crate

**Metrics:**
- **From:** `crates/riptide-api/src/streaming/metrics.rs` (329 LOC)
- **To:** Integrated into `crates/riptide-api/src/metrics.rs`
- **Rationale:** Unified metrics collection

---

### Phase 5: Handler Refactoring

**Target: Ultra-thin handlers (<50 LOC each)**

**Before (handlers/streaming.rs - currently ~200 LOC):**
```rust
pub async fn crawl_stream(
    State(app): State<AppState>,
    Json(body): Json<CrawlBody>,
) -> Response {
    // Complex logic mixing validation, orchestration, and transport
}
```

**After (handlers/streaming.rs - target ~40 LOC):**
```rust
pub async fn crawl_stream_ndjson(
    State(app): State<AppState>,
    Json(body): Json<CrawlBody>,
) -> Response {
    let facade = StreamingFacade::new(&app);
    let request = CrawlStreamRequest::from(body);

    match facade.create_crawl_stream(request).await {
        Ok(stream_handle) => {
            let transport = NdjsonTransport::new();
            facade.execute_stream(stream_handle.processor, transport).await
                .map(|summary| ndjson_response(summary))
                .unwrap_or_else(|e| error_response(e))
        }
        Err(e) => error_response(e),
    }
}

pub async fn crawl_stream_sse(
    State(app): State<AppState>,
    Json(body): Json<CrawlBody>,
) -> Response {
    let facade = StreamingFacade::new(&app);
    let request = CrawlStreamRequest::from(body);

    match facade.create_crawl_stream(request).await {
        Ok(stream_handle) => {
            let transport = SseTransport::new();
            facade.execute_stream(stream_handle.processor, transport).await
                .map(|summary| sse_response(summary))
                .unwrap_or_else(|e| error_response(e))
        }
        Err(e) => error_response(e),
    }
}

pub async fn crawl_stream_websocket(
    ws: WebSocketUpgrade,
    State(app): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| async move {
        let facade = StreamingFacade::new(&app);
        let (sender, receiver) = socket.split();
        let transport = WebSocketTransport::new(sender);

        // Handle WebSocket messages and execute streams
        handle_websocket_session(facade, transport, receiver).await;
    })
}
```

**Refactoring Targets:**
- `handlers/streaming.rs` - Current handler
- `handlers/pdf.rs` - Uses response_helpers (change to use facade)
- Remove re-exports from `handlers/mod.rs`

---

## File Migration Matrix

| # | Source File | Target Location | LOC | Dependencies | Complexity | Tests Affected |
|---|-------------|-----------------|-----|--------------|------------|----------------|
| 1 | streaming/error.rs | types/src/errors/streaming.rs | 265 | None | Low | 0 (new tests) |
| 2 | streaming/config.rs | config/src/streaming.rs | 444 | riptide-types | Low | config_tests.rs |
| 3 | - (new) | types/src/ports/streaming.rs | 400 | None | Medium | Unit tests |
| 4 | streaming/buffer.rs | reliability/src/streaming/buffer.rs | 554 | riptide-types | Medium | buffer_tests.rs |
| 5 | streaming/processor.rs | facade/src/facades/streaming.rs | 634 | ports, pipeline | High | processor_tests.rs |
| 6 | streaming/pipeline.rs | facade/src/facades/streaming.rs | 628 | ports, buffer | High | pipeline_tests.rs |
| 7 | streaming/lifecycle.rs | facade/src/facades/streaming.rs | 622 | ports, metrics | High | lifecycle_tests.rs |
| 8 | streaming/websocket.rs | api/src/adapters/websocket_transport.rs | 684 → 350 | ports | Medium | ws_tests.rs |
| 9 | streaming/sse.rs | api/src/adapters/sse_transport.rs | 575 → 300 | ports | Medium | sse_tests.rs |
| 10 | streaming/ndjson/ | api/src/adapters/ndjson_transport.rs | 1185 → 250 | ports | Medium | ndjson_tests.rs |
| 11 | streaming/response_helpers.rs | Split: facade + adapters | 924 → 0 | Multiple | Medium | helper_tests.rs |
| 12 | streaming/metrics.rs | Integrate into api/src/metrics.rs | 329 | riptide-metrics | Low | metrics_tests.rs |
| 13 | streaming/mod.rs | Remove (replaced by ports) | 546 → 0 | All | Medium | mod_tests.rs |
| 14 | streaming/tests.rs | Distribute to new locations | 596 | All | High | N/A |
| 15 | handlers/streaming.rs | Refactor using facade | ~200 → 50 | facade | Low | integration_tests |
| 16 | handlers/pdf.rs | Update response_helpers usage | Partial | facade | Low | pdf_tests.rs |

**Total LOC Migration:** 7,986 → ~3,500 (56% reduction through deduplication)

---

## Implementation Phases

### Phase 1: Foundation (Estimated: 4 hours)

**Objective:** Create domain ports and error types

**Tasks:**
1. Create `riptide-types/src/ports/streaming.rs` (400 LOC)
   - Define StreamingTransport trait
   - Define StreamProcessor trait
   - Define StreamLifecycle trait
   - Define StreamEvent, StreamState, StreamMetrics types
   - Add comprehensive trait documentation

2. Move `streaming/error.rs` → `types/src/errors/streaming.rs` (265 LOC)
   - Update error types to use ports
   - Add conversion impls for ApiError
   - Update error recovery logic

3. Move `streaming/config.rs` → `config/src/streaming.rs` (444 LOC)
   - Add StreamConfig to riptide-config
   - Update environment variable loading
   - Add validation methods

**Verification:**
```bash
cargo check -p riptide-types
cargo check -p riptide-config
cargo test -p riptide-types -- streaming
```

**Quality Gates:**
- ✅ All traits compile without warnings
- ✅ Error types have proper Display/Debug impls
- ✅ Config validates correctly
- ✅ 20+ unit tests pass

---

### Phase 2: StreamingFacade (Estimated: 8 hours)

**Objective:** Create application facade with business logic

**Tasks:**
1. Create `facade/src/facades/streaming.rs` (1,200 LOC)
   - Consolidate processor.rs business logic
   - Consolidate pipeline.rs orchestration
   - Consolidate lifecycle.rs event handling
   - Implement StreamingFacade methods
   - Add comprehensive error handling

2. Write 50+ unit tests
   - Test URL processing
   - Test stream lifecycle
   - Test error scenarios
   - Test backpressure handling
   - Test metrics collection

3. Add to facade mod.rs
   - Export StreamingFacade
   - Export request/response types
   - Update Cargo.toml dependencies

**Verification:**
```bash
cargo check -p riptide-facade
cargo test -p riptide-facade -- streaming
cargo clippy -p riptide-facade -- -D warnings
```

**Quality Gates:**
- ✅ Facade compiles without warnings
- ✅ 50+ unit tests pass
- ✅ No direct HTTP framework dependencies
- ✅ Clean separation from transport layer
- ✅ Event emission works correctly

---

### Phase 3: Transport Adapters (Estimated: 6 hours)

**Objective:** Create protocol adapters implementing StreamingTransport

**Tasks:**
1. Create `api/src/adapters/websocket_transport.rs` (350 LOC)
   - Extract transport logic from websocket.rs
   - Implement StreamingTransport trait
   - Add connection management
   - Handle ping/pong keepalive

2. Create `api/src/adapters/sse_transport.rs` (300 LOC)
   - Extract transport logic from sse.rs
   - Implement StreamingTransport trait
   - Add SSE event formatting
   - Handle keep-alive comments

3. Create `api/src/adapters/ndjson_transport.rs` (250 LOC)
   - Consolidate ndjson/ directory logic
   - Implement StreamingTransport trait
   - Add NDJSON formatting
   - Handle streaming buffering

4. Write adapter tests (100 LOC)
   - Test each transport independently
   - Mock StreamingTransport trait
   - Test error scenarios

**Verification:**
```bash
cargo check -p riptide-api
cargo test -p riptide-api -- adapters::streaming
```

**Quality Gates:**
- ✅ All adapters implement StreamingTransport
- ✅ No business logic in adapters
- ✅ Proper error handling
- ✅ 30+ adapter tests pass

---

### Phase 4: Infrastructure Moves (Estimated: 4 hours)

**Objective:** Move infrastructure concerns to proper locations

**Tasks:**
1. Move `streaming/buffer.rs` → `reliability/src/streaming/buffer.rs` (554 LOC)
   - Update imports in facade
   - Add to riptide-reliability exports
   - Update Cargo.toml dependencies

2. Integrate `streaming/metrics.rs` → `api/src/metrics.rs` (329 LOC)
   - Add streaming metrics to RipTideMetrics
   - Remove duplicate metric definitions
   - Update facade to use unified metrics

3. Update all cross-crate imports
   - Fix riptide-facade imports
   - Fix riptide-api imports
   - Update re-exports

**Verification:**
```bash
cargo check --workspace
cargo test -p riptide-reliability -- buffer
cargo test -p riptide-api -- metrics
```

**Quality Gates:**
- ✅ All crates compile
- ✅ No circular dependencies
- ✅ Buffer tests pass
- ✅ Metrics integration works

---

### Phase 5: Handler Refactoring (Estimated: 3 hours)

**Objective:** Refactor handlers to ultra-thin HTTP wrappers

**Tasks:**
1. Refactor `handlers/streaming.rs` (200 → 50 LOC)
   - Use StreamingFacade for business logic
   - Create protocol-specific handlers
   - Remove direct pipeline orchestration
   - Update route registration

2. Update `handlers/pdf.rs` (partial update)
   - Replace response_helpers with facade
   - Use transport adapters
   - Maintain existing functionality

3. Update `handlers/mod.rs`
   - Remove streaming re-exports
   - Export new handlers
   - Update public API

**Verification:**
```bash
cargo check -p riptide-api
cargo test -p riptide-api -- handlers::streaming
```

**Quality Gates:**
- ✅ Handlers < 50 LOC each
- ✅ No business logic in handlers
- ✅ All routes work correctly
- ✅ Integration tests pass

---

### Phase 6: Cleanup & Testing (Estimated: 3 hours)

**Objective:** Remove old code and ensure test coverage

**Tasks:**
1. Delete old `streaming/` directory
   - Remove streaming/processor.rs
   - Remove streaming/pipeline.rs
   - Remove streaming/lifecycle.rs
   - Remove streaming/websocket.rs
   - Remove streaming/sse.rs
   - Remove streaming/ndjson/
   - Remove streaming/response_helpers.rs
   - Remove streaming/buffer.rs (moved)
   - Remove streaming/config.rs (moved)
   - Remove streaming/error.rs (moved)
   - Remove streaming/metrics.rs (integrated)
   - Remove streaming/mod.rs
   - Remove streaming/tests.rs

2. Update test files
   - Migrate tests to new locations
   - Update test imports
   - Add missing test coverage
   - Verify all tests pass

3. Update documentation
   - Update module documentation
   - Add architecture diagrams
   - Update API examples
   - Add migration guide

**Verification:**
```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

**Quality Gates:**
- ✅ 0 warnings in build
- ✅ All tests pass (200+ tests)
- ✅ Test coverage > 80%
- ✅ Documentation complete

---

## Risk Assessment & Mitigation

### Critical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **Breaking WebSocket connections** | High | Medium | Comprehensive integration tests, gradual rollout |
| **Performance regression** | High | Low | Benchmark before/after, load testing |
| **Test infrastructure dependencies** | High | Medium | Mock all external dependencies |
| **Circular dependencies** | Medium | Low | Careful crate organization, dependency graph analysis |
| **API breaking changes** | Medium | Medium | Maintain backward compatibility layer |
| **Buffer backpressure issues** | High | Low | Extensive backpressure testing |
| **Metrics collection gaps** | Medium | Low | Verify all lifecycle events emit metrics |

### Dependencies & Blockers

**External Dependencies:**
- ✅ **riptide-streaming** crate exists (7 test files)
- ✅ **StreamingFacade stub** exists (2,582 LOC)
- ⚠️ **Integration tests** may need Redis/external services
- ✅ **Phase 3.1 complete** - Handler refactoring patterns established

**Potential Blockers:**
1. **Test environment setup** - Redis, mock HTTP servers
2. **WebSocket integration testing** - Requires real WebSocket connections
3. **Performance benchmarking** - Need load testing infrastructure
4. **Documentation updates** - API docs need comprehensive examples

**Mitigation:**
- Use test helpers from Phase 3.1 (AppStateBuilder)
- Create mock transport implementations
- Add benchmark suite in Phase 6
- Generate docs with examples in Phase 6

---

## Testing Strategy

### Unit Tests (200+ tests)

**StreamingFacade (50+ tests):**
- ✅ URL processing with various options
- ✅ Stream lifecycle events
- ✅ Error handling and recovery
- ✅ Backpressure scenarios
- ✅ Metrics collection accuracy
- ✅ Event emission verification

**Transport Adapters (30+ tests per protocol):**
- ✅ Protocol-specific message formatting
- ✅ Connection management
- ✅ Keep-alive mechanisms
- ✅ Error scenarios
- ✅ Disconnection handling

**Ports & Domain Types (20+ tests):**
- ✅ Trait contract verification
- ✅ State transitions
- ✅ Event serialization
- ✅ Metrics calculation

**Infrastructure (30+ tests):**
- ✅ Buffer management
- ✅ Configuration validation
- ✅ Error type conversions

### Integration Tests (50+ tests)

**End-to-End Streaming (20+ tests):**
```rust
#[tokio::test]
async fn test_ndjson_crawl_stream_end_to_end() {
    let app = test_app().await;
    let body = CrawlBody {
        urls: vec!["https://example.com".to_string()],
        options: Some(CrawlOptions::default()),
    };

    let response = crawl_stream_ndjson(State(app), Json(body)).await;

    assert_eq!(response.status(), StatusCode::OK);
    // Verify NDJSON format
    // Verify all events received
    // Verify metrics collected
}
```

**WebSocket Sessions (15+ tests):**
- Full WebSocket handshake
- Bidirectional messaging
- Ping/pong keepalive
- Graceful disconnection
- Error recovery

**SSE Streaming (15+ tests):**
- SSE event stream formatting
- Keep-alive comments
- Reconnection handling
- Event ID tracking

### Performance Tests (10+ tests)

**Throughput Benchmarks:**
- 1,000 URLs/stream
- 10,000 URLs/stream
- Multiple concurrent streams

**Latency Benchmarks:**
- Event emission latency
- End-to-end streaming latency
- Backpressure response time

**Memory Benchmarks:**
- Buffer memory usage
- Connection memory overhead
- Leak detection

---

## Quality Gates & Acceptance Criteria

### Phase Completion Criteria

**Phase 1 (Foundation):**
- [ ] StreamingTransport trait compiles
- [ ] StreamProcessor trait compiles
- [ ] StreamLifecycle trait compiles
- [ ] Error types moved to riptide-types
- [ ] Config moved to riptide-config
- [ ] 20+ unit tests pass
- [ ] 0 clippy warnings

**Phase 2 (Facade):**
- [ ] StreamingFacade implements all business methods
- [ ] 50+ unit tests pass with >80% coverage
- [ ] No direct HTTP framework dependencies
- [ ] Event emission works correctly
- [ ] Metrics collection integrated
- [ ] 0 clippy warnings

**Phase 3 (Adapters):**
- [ ] WebSocketTransport implements StreamingTransport
- [ ] SseTransport implements StreamingTransport
- [ ] NdjsonTransport implements StreamingTransport
- [ ] 30+ adapter tests pass
- [ ] No business logic in adapters
- [ ] 0 clippy warnings

**Phase 4 (Infrastructure):**
- [ ] Buffer moved to riptide-reliability
- [ ] Metrics integrated into RipTideMetrics
- [ ] All cross-crate imports work
- [ ] No circular dependencies
- [ ] 0 clippy warnings

**Phase 5 (Handlers):**
- [ ] handlers/streaming.rs < 50 LOC
- [ ] handlers/pdf.rs updated
- [ ] All routes work correctly
- [ ] Integration tests pass
- [ ] 0 clippy warnings

**Phase 6 (Cleanup):**
- [ ] Old streaming/ directory deleted
- [ ] 200+ tests pass
- [ ] Test coverage > 80%
- [ ] Documentation complete
- [ ] 0 clippy warnings
- [ ] Cargo build --workspace succeeds

### Final Acceptance Criteria

**Functional:**
- ✅ All streaming protocols work (NDJSON, SSE, WebSocket)
- ✅ Backpressure handling maintains stability
- ✅ Metrics collection is comprehensive
- ✅ Error handling is robust
- ✅ Lifecycle events are tracked

**Non-Functional:**
- ✅ Performance: < 5% regression from baseline
- ✅ Memory: No memory leaks detected
- ✅ Latency: P99 < 100ms for event emission
- ✅ Throughput: > 10,000 messages/sec per connection

**Code Quality:**
- ✅ 0 clippy warnings
- ✅ 0 compiler warnings
- ✅ Test coverage > 80%
- ✅ Documentation coverage > 90%
- ✅ All handlers < 50 LOC

**Architecture:**
- ✅ Clean hexagonal architecture
- ✅ No circular dependencies
- ✅ Proper separation of concerns
- ✅ Protocol-agnostic business logic
- ✅ Infrastructure in correct layers

---

## Effort Estimation

### Time Breakdown (28 hours total)

| Phase | Tasks | Estimated Hours | Complexity |
|-------|-------|-----------------|------------|
| **Phase 1: Foundation** | Ports, errors, config | 4 | Medium |
| **Phase 2: Facade** | Business logic consolidation | 8 | High |
| **Phase 3: Adapters** | Transport implementations | 6 | Medium |
| **Phase 4: Infrastructure** | File moves, integration | 4 | Low-Medium |
| **Phase 5: Handlers** | Handler refactoring | 3 | Low |
| **Phase 6: Cleanup** | Testing, docs, cleanup | 3 | Medium |
| **Total** | | **28 hours** | |

### Critical Path

```
Phase 1 (4h) → Phase 2 (8h) → Phase 3 (6h) → Phase 5 (3h) → Phase 6 (3h)
                    ↓
              Phase 4 (4h) ────────────────────┘
```

**Parallel Execution Opportunities:**
- Phase 4 can start after Phase 2 complete
- Phase 3 and Phase 4 can run in parallel
- Testing can begin incrementally after each phase

**Resource Requirements:**
- 1 senior developer (full-time)
- OR 2 developers working in parallel on Phase 3+4
- Code reviewer for each phase
- QA for integration testing

---

## Migration Checklist

### Pre-Migration

- [ ] Review current streaming architecture
- [ ] Document all streaming endpoints
- [ ] Identify all external dependencies
- [ ] Create baseline performance benchmarks
- [ ] Set up test environment
- [ ] Review Phase 3.1 patterns for consistency

### Phase 1: Foundation

- [ ] Create riptide-types/src/ports/streaming.rs
- [ ] Move streaming/error.rs → types/src/errors/streaming.rs
- [ ] Move streaming/config.rs → config/src/streaming.rs
- [ ] Update Cargo.toml dependencies
- [ ] Write 20+ unit tests
- [ ] Run `cargo check --workspace`
- [ ] Run `cargo clippy --workspace -- -D warnings`
- [ ] Commit: "feat(ports): add streaming domain ports"

### Phase 2: StreamingFacade

- [ ] Create facade/src/facades/streaming.rs
- [ ] Migrate processor.rs business logic
- [ ] Migrate pipeline.rs orchestration
- [ ] Migrate lifecycle.rs event handling
- [ ] Implement facade methods
- [ ] Write 50+ unit tests
- [ ] Update facade/src/facades/mod.rs
- [ ] Update riptide-facade/Cargo.toml
- [ ] Run `cargo test -p riptide-facade`
- [ ] Commit: "feat(facade): add StreamingFacade with business logic"

### Phase 3: Transport Adapters

- [ ] Create api/src/adapters/websocket_transport.rs
- [ ] Create api/src/adapters/sse_transport.rs
- [ ] Create api/src/adapters/ndjson_transport.rs
- [ ] Create api/src/adapters/mod.rs
- [ ] Write 30+ adapter tests
- [ ] Run `cargo test -p riptide-api -- adapters`
- [ ] Commit: "feat(adapters): add streaming transport adapters"

### Phase 4: Infrastructure Moves

- [ ] Move buffer.rs → reliability/src/streaming/buffer.rs
- [ ] Integrate metrics.rs → api/src/metrics.rs
- [ ] Update all cross-crate imports
- [ ] Update Cargo.toml dependencies
- [ ] Run `cargo check --workspace`
- [ ] Commit: "refactor(infra): move streaming infrastructure"

### Phase 5: Handler Refactoring

- [ ] Refactor handlers/streaming.rs
- [ ] Update handlers/pdf.rs
- [ ] Update handlers/mod.rs
- [ ] Test all streaming routes
- [ ] Run integration tests
- [ ] Commit: "refactor(handlers): use StreamingFacade"

### Phase 6: Cleanup & Testing

- [ ] Delete streaming/processor.rs
- [ ] Delete streaming/pipeline.rs
- [ ] Delete streaming/lifecycle.rs
- [ ] Delete streaming/websocket.rs
- [ ] Delete streaming/sse.rs
- [ ] Delete streaming/ndjson/
- [ ] Delete streaming/response_helpers.rs
- [ ] Delete streaming/buffer.rs (moved)
- [ ] Delete streaming/config.rs (moved)
- [ ] Delete streaming/error.rs (moved)
- [ ] Delete streaming/metrics.rs (integrated)
- [ ] Delete streaming/mod.rs
- [ ] Delete streaming/tests.rs
- [ ] Delete streaming/ directory
- [ ] Migrate all tests to new locations
- [ ] Run full test suite (200+ tests)
- [ ] Update documentation
- [ ] Run benchmarks
- [ ] Commit: "refactor(streaming): complete hexagonal architecture migration"

### Post-Migration

- [ ] Run full test suite
- [ ] Run performance benchmarks
- [ ] Compare metrics with baseline
- [ ] Update API documentation
- [ ] Create migration guide
- [ ] Review with team
- [ ] Deploy to staging
- [ ] Monitor metrics
- [ ] Deploy to production

---

## Success Metrics

### Code Quality Metrics

| Metric | Baseline | Target | Actual |
|--------|----------|--------|--------|
| **Total LOC** | 7,986 | ~3,500 | TBD |
| **Handler LOC** | ~200 | <50 | TBD |
| **Clippy Warnings** | 0 | 0 | TBD |
| **Test Coverage** | ~70% | >80% | TBD |
| **Cyclomatic Complexity** | Medium | Low | TBD |

### Performance Metrics

| Metric | Baseline | Target | Actual |
|--------|----------|--------|--------|
| **P99 Event Latency** | TBD | <100ms | TBD |
| **Throughput (msg/sec)** | TBD | >10,000 | TBD |
| **Memory per Connection** | TBD | <5MB | TBD |
| **Concurrent Connections** | TBD | >1,000 | TBD |
| **CPU Usage** | TBD | <50% | TBD |

### Architecture Metrics

| Metric | Baseline | Target | Actual |
|--------|----------|--------|--------|
| **Circular Dependencies** | 0 | 0 | TBD |
| **Port Implementations** | 0 | 3 | TBD |
| **Facade Methods** | 0 | 15+ | TBD |
| **Protocol Adapters** | 0 | 3 | TBD |
| **Infrastructure Violations** | High | 0 | TBD |

---

## Rollout Plan

### Phase 1: Development (Week 1)

- **Day 1-2:** Foundation (Phases 1-2)
- **Day 3:** Adapters (Phase 3)
- **Day 4:** Infrastructure & Handlers (Phases 4-5)
- **Day 5:** Testing & Cleanup (Phase 6)

### Phase 2: Testing (Week 2)

- **Day 1-2:** Integration testing
- **Day 3:** Performance testing
- **Day 4:** Load testing
- **Day 5:** Security testing

### Phase 3: Deployment (Week 3)

- **Day 1:** Deploy to staging
- **Day 2-3:** Monitor staging metrics
- **Day 4:** Deploy to production (canary)
- **Day 5:** Full production rollout

### Rollback Strategy

**Triggers:**
- P99 latency > 200ms
- Error rate > 5%
- Memory usage > 2x baseline
- Connection failures > 1%

**Rollback Steps:**
1. Revert git commit
2. Redeploy previous version
3. Monitor metrics
4. Investigate root cause
5. Fix and re-deploy

---

## Appendix

### A. Current File Dependencies

```
streaming/mod.rs (546 LOC)
├── buffer.rs (554 LOC)
├── config.rs (444 LOC)
├── error.rs (265 LOC)
├── lifecycle.rs (622 LOC)
│   ├── metrics.rs (329 LOC)
│   └── processor.rs (634 LOC)
├── pipeline.rs (628 LOC)
│   ├── processor.rs (634 LOC)
│   └── buffer.rs (554 LOC)
├── processor.rs (634 LOC)
│   ├── pipeline::PipelineOrchestrator
│   └── models::*
├── response_helpers.rs (924 LOC)
│   └── Used by: pdf.rs, streaming.rs
├── websocket.rs (684 LOC)
│   ├── buffer.rs
│   ├── config.rs
│   └── lifecycle.rs
├── sse.rs (575 LOC)
│   ├── buffer.rs
│   ├── config.rs
│   └── response_helpers.rs
└── ndjson/ (1,185 LOC)
    ├── helpers.rs (725 LOC)
    ├── streaming.rs (195 LOC)
    ├── handlers.rs (134 LOC)
    └── progress.rs (36 LOC)
```

### B. Target File Structure

```
riptide-types/
└── src/
    ├── ports/
    │   └── streaming.rs (400 LOC) - NEW
    └── errors/
        └── streaming.rs (265 LOC) - MOVED

riptide-config/
└── src/
    └── streaming.rs (444 LOC) - MOVED

riptide-facade/
└── src/
    └── facades/
        └── streaming.rs (1,200 LOC) - NEW
            - Consolidates: processor.rs, pipeline.rs, lifecycle.rs

riptide-reliability/
└── src/
    └── streaming/
        └── buffer.rs (554 LOC) - MOVED

riptide-api/
└── src/
    ├── adapters/
    │   ├── websocket_transport.rs (350 LOC) - NEW
    │   ├── sse_transport.rs (300 LOC) - NEW
    │   └── ndjson_transport.rs (250 LOC) - NEW
    ├── handlers/
    │   └── streaming.rs (50 LOC) - REFACTORED
    └── metrics.rs (integrates streaming/metrics.rs)
```

### C. Test File Organization

```
riptide-types/
└── src/
    └── ports/
        └── streaming.rs
            - 20+ unit tests inline

riptide-facade/
└── tests/
    └── streaming_facade_tests.rs (50+ tests)

riptide-api/
└── tests/
    ├── adapters/
    │   ├── websocket_transport_tests.rs (10+ tests)
    │   ├── sse_transport_tests.rs (10+ tests)
    │   └── ndjson_transport_tests.rs (10+ tests)
    └── integration/
        └── streaming_integration_tests.rs (50+ tests)

riptide-reliability/
└── tests/
    └── buffer_tests.rs (30+ tests)
```

### D. Performance Baseline Benchmarks

**To be measured before Phase 1:**

```bash
# Throughput benchmark
cargo bench --bench streaming_throughput

# Latency benchmark
cargo bench --bench streaming_latency

# Memory benchmark
cargo bench --bench streaming_memory

# Concurrent connections benchmark
cargo bench --bench streaming_connections
```

**Expected Baselines (to be measured):**
- Event emission latency: TBD
- Throughput: TBD messages/sec
- Memory per connection: TBD MB
- Max concurrent connections: TBD

### E. API Breaking Changes

**None expected** - Maintaining backward compatibility:

1. **Handler endpoints unchanged:**
   - `/crawl/stream` (NDJSON)
   - `/crawl/sse` (SSE)
   - `/crawl/ws` (WebSocket)

2. **Response formats unchanged:**
   - NDJSON event structure maintained
   - SSE event structure maintained
   - WebSocket message structure maintained

3. **Request validation unchanged:**
   - Same validation rules
   - Same error responses

4. **Internal API changes only:**
   - StreamingFacade is internal
   - Transport adapters are internal
   - Ports are internal contracts

---

## Conclusion

This plan provides a comprehensive roadmap for migrating the streaming system to hexagonal architecture. The migration is divided into 6 manageable phases with clear acceptance criteria, quality gates, and rollback strategies.

**Key Success Factors:**
1. ✅ **Incremental migration** - Each phase is independently verifiable
2. ✅ **Comprehensive testing** - 200+ tests ensure correctness
3. ✅ **Clear separation** - Ports, facades, and adapters are well-defined
4. ✅ **Performance monitoring** - Benchmarks prevent regressions
5. ✅ **Rollback safety** - Clear rollback triggers and procedures

**Next Steps:**
1. Review and approve this plan
2. Set up performance baseline benchmarks
3. Begin Phase 1: Foundation
4. Execute phases sequentially with quality gates
5. Monitor metrics throughout migration

**Estimated Completion:** 28 hours (1 week with focused effort)

---

**Created:** 2025-11-08
**Author:** System Architecture Designer
**Sprint:** 4.3 - Streaming System Refactoring
