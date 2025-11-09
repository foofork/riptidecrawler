# Sprint 4.3: Streaming Architecture Diagrams

## Current Architecture (Violations)

```
┌─────────────────────────────────────────────────────────────────┐
│                    HTTP Layer (Axum Framework)                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ /crawl/stream│  │  /crawl/sse  │  │  /crawl/ws   │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                  │                  │                   │
└─────────┼──────────────────┼──────────────────┼──────────────────┘
          │                  │                  │
          ▼                  ▼                  ▼
┌─────────────────────────────────────────────────────────────────┐
│        streaming/ Module (7,986 LOC) - MONOLITHIC               │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  ❌ VIOLATION: Mixed Concerns                            │  │
│  │                                                            │  │
│  │  processor.rs (634 LOC)    ← Business Logic              │  │
│  │  pipeline.rs (628 LOC)     ← Orchestration               │  │
│  │  lifecycle.rs (622 LOC)    ← Event Handling              │  │
│  │  ────────────────────────────────────────────────────    │  │
│  │  websocket.rs (684 LOC)    ← Transport + Logic           │  │
│  │  sse.rs (575 LOC)          ← Transport + Logic           │  │
│  │  ndjson/ (1,185 LOC)       ← Transport + Logic           │  │
│  │  ────────────────────────────────────────────────────    │  │
│  │  buffer.rs (554 LOC)       ← Infrastructure              │  │
│  │  config.rs (444 LOC)       ← Infrastructure              │  │
│  │  error.rs (265 LOC)        ← Domain Types                │  │
│  │  metrics.rs (329 LOC)      ← Infrastructure              │  │
│  │  response_helpers.rs (924) ← Protocol Formatting         │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                  │
│  Problems:                                                       │
│  • Business logic tightly coupled to HTTP framework             │
│  • Protocol implementations duplicate logic                     │
│  • Infrastructure mixed with domain                             │
│  • Difficult to test in isolation                               │
│  • Violates dependency inversion principle                      │
└──────────────────────────────────────────────────────────────────┘
```

---

## Target Architecture (Hexagonal/Ports & Adapters)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    HTTP Layer (riptide-api)                              │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐               │
│  │ /crawl/ndjson │  │  /crawl/sse   │  │  /crawl/ws    │               │
│  │   Handler     │  │   Handler     │  │   Handler     │               │
│  │   (<50 LOC)   │  │   (<50 LOC)   │  │   (<50 LOC)   │               │
│  └───────┬───────┘  └───────┬───────┘  └───────┬───────┘               │
│          │                   │                   │                        │
│          └───────────────────┴───────────────────┘                        │
│                              │                                            │
└──────────────────────────────┼────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────────┐
│              Protocol Adapters (riptide-api/adapters/)                   │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐      │
│  │ NdjsonTransport  │  │   SseTransport   │  │ WebSocketTransport│      │
│  │    (250 LOC)     │  │    (300 LOC)     │  │    (350 LOC)      │      │
│  │                  │  │                  │  │                   │      │
│  │ impl             │  │ impl             │  │ impl              │      │
│  │ StreamingTransport│ │ StreamingTransport│ │ StreamingTransport│      │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘      │
│                              │                                            │
└──────────────────────────────┼────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────────┐
│           Application Layer (riptide-facade)                             │
│  ┌────────────────────────────────────────────────────────────────┐     │
│  │              StreamingFacade (1,200 LOC)                       │     │
│  │                                                                 │     │
│  │  Business Logic Methods:                                       │     │
│  │  • create_crawl_stream()      - URL crawling orchestration     │     │
│  │  • create_deepsearch_stream() - Search + crawl orchestration   │     │
│  │  • process_urls_concurrent()  - Parallel URL processing        │     │
│  │  • execute_stream()           - Generic stream execution       │     │
│  │                                                                 │     │
│  │  Lifecycle Management:                                         │     │
│  │  • start_stream(), pause_stream(), resume_stream()             │     │
│  │  • cancel_stream(), get_stream_status()                        │     │
│  │  • get_stream_metrics(), list_active_streams()                 │     │
│  │                                                                 │     │
│  │  Features:                                                      │     │
│  │  ✓ Protocol-agnostic business logic                            │     │
│  │  ✓ Event emission (stream.started, stream.completed, etc.)     │     │
│  │  ✓ Metrics collection & reporting                              │     │
│  │  ✓ Backpressure management                                     │     │
│  │  ✓ Error handling & recovery                                   │     │
│  └────────────────────────────────────────────────────────────────┘     │
│                              │                                            │
└──────────────────────────────┼────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────────┐
│              Domain Layer (riptide-types/ports/)                         │
│  ┌────────────────────────────────────────────────────────────────┐     │
│  │                   streaming.rs (400 LOC)                       │     │
│  │                                                                 │     │
│  │  Ports (Traits):                                               │     │
│  │  ┌───────────────────────────────────────────────────────┐    │     │
│  │  │ trait StreamingTransport {                            │    │     │
│  │  │   async fn send_event(event: StreamEvent) -> Result;  │    │     │
│  │  │   async fn send_metadata(...) -> Result;              │    │     │
│  │  │   async fn close() -> Result;                         │    │     │
│  │  │   fn protocol_name() -> &str;                         │    │     │
│  │  │ }                                                      │    │     │
│  │  └───────────────────────────────────────────────────────┘    │     │
│  │                                                                 │     │
│  │  ┌───────────────────────────────────────────────────────┐    │     │
│  │  │ trait StreamProcessor {                               │    │     │
│  │  │   async fn process_urls(...) -> Result;               │    │     │
│  │  │   async fn create_progress() -> StreamProgress;       │    │     │
│  │  │   fn should_send_progress(...) -> bool;               │    │     │
│  │  │ }                                                      │    │     │
│  │  └───────────────────────────────────────────────────────┘    │     │
│  │                                                                 │     │
│  │  ┌───────────────────────────────────────────────────────┐    │     │
│  │  │ trait StreamLifecycle {                               │    │     │
│  │  │   async fn on_connection_established(...);            │    │     │
│  │  │   async fn on_stream_started(...);                    │    │     │
│  │  │   async fn on_completed(...);                         │    │     │
│  │  │ }                                                      │    │     │
│  │  └───────────────────────────────────────────────────────┘    │     │
│  │                                                                 │     │
│  │  Domain Types:                                                 │     │
│  │  • StreamEvent, StreamState, StreamMetrics                     │     │
│  │  • StreamMetadata, StreamProgress, StreamSummary               │     │
│  │  • StreamingError, StreamingResult                             │     │
│  └────────────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    Infrastructure Layer                                  │
│  ┌───────────────────┐  ┌───────────────────┐  ┌──────────────────┐    │
│  │ BufferManager     │  │  StreamConfig     │  │  Metrics         │    │
│  │ (riptide-         │  │  (riptide-config) │  │  (riptide-api)   │    │
│  │  reliability)     │  │                   │  │                  │    │
│  │                   │  │  • Buffer config  │  │  • Prometheus    │    │
│  │  • Buffer pool    │  │  • Protocol cfg   │  │  • Event tracking│    │
│  │  • Backpressure   │  │  • Limits & TTLs  │  │  • Health checks │    │
│  │  • Memory mgmt    │  │  • Env loading    │  │                  │    │
│  └───────────────────┘  └───────────────────┘  └──────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Data Flow: NDJSON Streaming Example

```
┌──────────────────────────────────────────────────────────────────┐
│ 1. HTTP Request                                                   │
│    POST /crawl/ndjson                                             │
│    { "urls": ["https://example.com", ...] }                      │
└────────┬──────────────────────────────────────────────────────────┘
         │
         ▼
┌──────────────────────────────────────────────────────────────────┐
│ 2. Handler (40 LOC)                                               │
│    • Validate request                                             │
│    • Create facade                                                │
│    • Convert to domain request                                    │
└────────┬──────────────────────────────────────────────────────────┘
         │
         ▼
┌──────────────────────────────────────────────────────────────────┐
│ 3. StreamingFacade::create_crawl_stream()                         │
│    • Emit event: stream.started                                   │
│    • Create StreamProcessor                                       │
│    • Set up backpressure management                               │
│    • Return StreamHandle                                          │
└────────┬──────────────────────────────────────────────────────────┘
         │
         ▼
┌──────────────────────────────────────────────────────────────────┐
│ 4. StreamingFacade::execute_stream()                              │
│    • Process URLs concurrently                                    │
│    • For each result:                                             │
│    │   - Convert to StreamEvent                                   │
│    │   - Send via transport.send_event()                          │
│    │   - Check backpressure                                       │
│    │   - Emit lifecycle events                                    │
│    • Send final summary                                           │
└────────┬──────────────────────────────────────────────────────────┘
         │
         ▼
┌──────────────────────────────────────────────────────────────────┐
│ 5. NdjsonTransport::send_event()                                  │
│    • Format as NDJSON line                                        │
│    • {"type":"result","index":0,"data":{...}}                    │
│    • Send to HTTP stream                                          │
│    • Update metrics                                               │
└────────┬──────────────────────────────────────────────────────────┘
         │
         ▼
┌──────────────────────────────────────────────────────────────────┐
│ 6. HTTP Response Stream                                           │
│    {"type":"metadata","total_urls":5}\n                          │
│    {"type":"result","index":0,"url":"..."}\n                     │
│    {"type":"progress","completed":1,"total":5}\n                 │
│    {"type":"summary","successful":5,"failed":0}\n                │
└──────────────────────────────────────────────────────────────────┘
```

---

## Component Interaction Diagram

```
┌─────────────────┐
│   HTTP Client   │
└────────┬────────┘
         │ HTTP Request
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│                      API Layer                               │
│  ┌────────────┐     ┌──────────────────┐                    │
│  │  Handler   │────▶│ NdjsonTransport  │                    │
│  └─────┬──────┘     │ (StreamingTransport)│                 │
│        │            └──────────┬───────────┘                 │
│        │                       │                              │
└────────┼───────────────────────┼──────────────────────────────┘
         │                       │
         │  Delegate             │  Protocol impl
         │  business logic       │
         │                       │
         ▼                       │
┌────────────────────────────────┼──────────────────────────────┐
│       Application Layer        │                               │
│  ┌──────────────────────┐     │                               │
│  │  StreamingFacade     │◀────┘                               │
│  │                      │                                      │
│  │  • Business Logic    │                                      │
│  │  • Orchestration     │                                      │
│  │  • Event Emission    │                                      │
│  └──┬────────────┬──────┘                                     │
│     │            │                                             │
└─────┼────────────┼─────────────────────────────────────────────┘
      │            │
      │ Uses ports │ Uses infrastructure
      │            │
      ▼            ▼
┌──────────────┐ ┌──────────────────┐
│ Domain Ports │ │ Infrastructure   │
│ (Traits)     │ │ • BufferManager  │
│              │ │ • StreamConfig   │
│              │ │ • Metrics        │
└──────────────┘ └──────────────────┘
```

---

## Dependency Flow (Hexagonal Architecture)

```
                    ┌──────────────────────┐
                    │   Domain Layer       │
                    │   (riptide-types)    │
                    │                      │
                    │   • Ports (traits)   │
                    │   • Domain types     │
                    │   • Pure business    │
                    │     rules            │
                    └──────────┬───────────┘
                               │
                    Dependencies point INWARD
                               │
              ┌────────────────┴────────────────┐
              │                                  │
              ▼                                  ▼
    ┌──────────────────┐              ┌──────────────────┐
    │ Application Layer│              │ Infrastructure   │
    │ (riptide-facade) │              │ (reliability,    │
    │                  │              │  config, etc.)   │
    │ • StreamingFacade│              │                  │
    │ • Business logic │              │ • BufferManager  │
    │ • Orchestration  │              │ • StreamConfig   │
    └────────┬─────────┘              │ • Metrics        │
             │                        └────────┬─────────┘
             │                                 │
             └─────────────┬───────────────────┘
                           │
                Dependencies point INWARD
                           │
                           ▼
                 ┌──────────────────┐
                 │   API Layer      │
                 │   (riptide-api)  │
                 │                  │
                 │ • Thin handlers  │
                 │ • Protocol       │
                 │   adapters       │
                 │ • HTTP routes    │
                 └──────────────────┘
                           │
                           │
                           ▼
                 ┌──────────────────┐
                 │  External World  │
                 │  • HTTP clients  │
                 │  • WebSocket     │
                 │  • SSE           │
                 └──────────────────┘

Key Principle: Dependencies point INWARD
• Domain layer has NO dependencies
• Application layer depends on Domain
• Infrastructure implements Domain ports
• API layer depends on Application + Infrastructure
```

---

## Before/After Comparison

### Before: Monolithic Architecture

```
streaming/ (7,986 LOC)
├── ❌ Business logic in transport layer
├── ❌ Protocol duplication
├── ❌ Infrastructure coupling
└── ❌ Difficult to test

Dependencies: All mixed together
Testing: Mock entire HTTP framework
Maintainability: Low (tight coupling)
Extensibility: Hard (no abstraction)
```

### After: Hexagonal Architecture

```
Domain Ports (400 LOC)
    ↓
Facade (1,200 LOC) + Infrastructure (1,000 LOC)
    ↓
Adapters (900 LOC)
    ↓
Handlers (50 LOC)

Dependencies: Clean, unidirectional
Testing: Unit test each layer independently
Maintainability: High (loose coupling)
Extensibility: Easy (add new protocols)
```

---

## Benefits Summary

| Aspect | Before | After | Improvement |
|--------|--------|-------|-------------|
| **LOC** | 7,986 | ~3,500 | 56% reduction |
| **Handler LOC** | ~200 | <50 | 75% reduction |
| **Protocol Add** | ~700 LOC | ~300 LOC | 57% reduction |
| **Test Isolation** | Difficult | Easy | ✅ Unit testable |
| **Dependencies** | Mixed | Clean | ✅ Unidirectional |
| **Maintainability** | Low | High | ✅ Separated concerns |
| **Extensibility** | Hard | Easy | ✅ Protocol agnostic |

---

**See full plan:** `docs/execution/SPRINT_4.3_STREAMING_PLAN.md`
