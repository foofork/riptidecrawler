# Phase 2 Hexagonal Architecture - Visual Diagrams

## Complete System Architecture

```
┌───────────────────────────────────────────────────────────────────────────────────┐
│                                                                                   │
│                            EXTERNAL SYSTEMS & CLIENTS                             │
│                                                                                   │
│   HTTP Clients  │  WebSocket  │  SSE Clients  │  CLI Tools  │  Browser Apps      │
│                                                                                   │
└────────────────────────────────────┬──────────────────────────────────────────────┘
                                     │
                                     ▼
┌───────────────────────────────────────────────────────────────────────────────────┐
│                                                                                   │
│                            APPLICATION LAYER (Ports)                              │
│                          REST API + WebSocket + SSE                               │
│                                                                                   │
│  ┌─────────────────────────────────────────────────────────────────────────────┐ │
│  │                       REQUEST HANDLERS                                      │ │
│  │                                                                             │ │
│  │  /scrape   /extract   /pdf   /health   /metrics   /spider   /search       │ │
│  │  /stream   /traces    /jobs  /browser  /sessions  /events                 │ │
│  │                                                                             │ │
│  └──────────────────────────────┬──────────────────────────────────────────────┘ │
│                                 │                                                 │
│                                 ▼                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────────┐ │
│  │                   ApplicationContext (29 fields)                            │ │
│  │                                                                             │ │
│  │  ┌────────────────────────────────────────────────────────────────────┐    │ │
│  │  │           PORT TRAIT ABSTRACTIONS (24 fields)                      │    │ │
│  │  │                                                                    │    │ │
│  │  │  Infrastructure Access (All via trait objects):                   │    │ │
│  │  │                                                                    │    │ │
│  │  │  Arc<dyn HttpClient>                                               │    │ │
│  │  │  Arc<dyn CacheStorage>                                             │    │ │
│  │  │  Arc<dyn ContentExtractor>                                         │    │ │
│  │  │  Arc<dyn ReliableContentExtractor>                                 │    │ │
│  │  │  Arc<dyn ResourceManagement>                                       │    │ │
│  │  │  Arc<dyn MetricsCollectorPort>                                     │    │ │
│  │  │  Arc<dyn HealthCheck>                                              │    │ │
│  │  │  Arc<dyn SessionStorage>                                           │    │ │
│  │  │  Arc<dyn StreamingProvider>                                        │    │ │
│  │  │  Arc<dyn EventPublisher>                                           │    │ │
│  │  │  Arc<dyn CircuitBreaker>                                           │    │ │
│  │  │  Arc<dyn MonitoringBackend>                                        │    │ │
│  │  │  Arc<dyn BrowserDriver>                                            │    │ │
│  │  │  Arc<dyn WebScraping>                                              │    │ │
│  │  │  Arc<dyn SearchProvider>                                           │    │ │
│  │  │  Arc<dyn EngineSelection>                                          │    │ │
│  │  │  Arc<dyn TelemetryBackend>                                         │    │ │
│  │  │  Arc<dyn SpiderEngine>                                             │    │ │
│  │  │  Arc<dyn WorkerService>                                            │    │ │
│  │  │  Arc<dyn TraceBackend>                                             │    │ │
│  │  │  + 4 more facade abstractions                                      │    │ │
│  │  │                                                                    │    │ │
│  │  └────────────────────────────────────────────────────────────────────┘    │ │
│  │                                                                             │ │
│  │  ┌────────────────────────────────────────────────────────────────────┐    │ │
│  │  │           CONFIGURATION (5 fields)                                 │    │ │
│  │  │                                                                    │    │ │
│  │  │  AppConfig, RiptideApiConfig, AuthConfig                          │    │ │
│  │  │  cache_warmer_enabled, persistence_adapter                        │    │ │
│  │  │                                                                    │    │ │
│  │  └────────────────────────────────────────────────────────────────────┘    │ │
│  └─────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                   │
└───────────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     │ Depends on (Dependency Inversion)
                                     │
                                     ▼
┌───────────────────────────────────────────────────────────────────────────────────┐
│                                                                                   │
│                         PORT TRAITS LAYER (Interfaces)                            │
│                         Location: riptide_types::ports                            │
│                                                                                   │
│  ┌─────────────────────────────────────────────────────────────────────────────┐ │
│  │                        INFRASTRUCTURE PORTS                                 │ │
│  │                                                                             │ │
│  │  pub trait HttpClient                                                       │ │
│  │  pub trait CacheStorage                                                     │ │
│  │  pub trait ContentExtractor                                                 │ │
│  │  pub trait ReliableContentExtractor                                         │ │
│  │  pub trait ResourceManagement                                               │ │
│  │  pub trait MetricsCollectorPort                                             │ │
│  │  pub trait HealthCheck                                                      │ │
│  │  pub trait SessionStorage                                                   │ │
│  │  pub trait StreamingProvider                                                │ │
│  │  pub trait EventPublisher                                                   │ │
│  │  pub trait CircuitBreaker                                                   │ │
│  │  pub trait MonitoringBackend                                                │ │
│  │  pub trait BrowserDriver                                                    │ │
│  │  pub trait WebScraping                                                      │ │
│  │  pub trait SearchProvider                                                   │ │
│  │  pub trait EngineSelection                                                  │ │
│  │  pub trait TelemetryBackend                                                 │ │
│  │  pub trait SpiderEngine                                                     │ │
│  │  pub trait WorkerService                                                    │ │
│  │  pub trait TraceBackend                                                     │ │
│  │  pub trait Pool<T>                                                          │ │
│  │  pub trait RateLimiter                                                      │ │
│  │                                                                             │ │
│  └─────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                   │
└───────────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     │ Implemented by
                                     │
                                     ▼
┌───────────────────────────────────────────────────────────────────────────────────┐
│                                                                                   │
│                          ADAPTER LAYER (Implementations)                          │
│                        Location: riptide-api/src/adapters/                        │
│                                                                                   │
│  ┌─────────────────────────────────────────────────────────────────────────────┐ │
│  │                     ADAPTER IMPLEMENTATIONS (13 total)                      │ │
│  │                                                                             │ │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │ │
│  │  │  EventBusAdapter                                                     │  │ │
│  │  │    impl EventPublisher for EventBusAdapter { ... }                  │  │ │
│  │  │    Wraps: EventBus                                                   │  │ │
│  │  └──────────────────────────────────────────────────────────────────────┘  │ │
│  │                                                                             │ │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │ │
│  │  │  HealthCheckerAdapter                                                │  │ │
│  │  │    impl HealthCheck for HealthCheckerAdapter { ... }                │  │ │
│  │  │    Wraps: HealthChecker                                              │  │ │
│  │  └──────────────────────────────────────────────────────────────────────┘  │ │
│  │                                                                             │ │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │ │
│  │  │  MetricsCollectorAdapter                                             │  │ │
│  │  │    impl MetricsCollectorPort for MetricsCollectorAdapter { ... }    │  │ │
│  │  │    Wraps: MetricsCollector                                           │  │ │
│  │  └──────────────────────────────────────────────────────────────────────┘  │ │
│  │                                                                             │ │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │ │
│  │  │  MonitoringAdapter                                                   │  │ │
│  │  │    impl MonitoringBackend for MonitoringAdapter { ... }             │  │ │
│  │  │    Wraps: MonitoringSystem                                           │  │ │
│  │  └──────────────────────────────────────────────────────────────────────┘  │ │
│  │                                                                             │ │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │ │
│  │  │  ResourceManagerAdapter                                              │  │ │
│  │  │    impl ResourceManagement for ResourceManagerAdapter { ... }       │  │ │
│  │  │    Wraps: ResourceManager                                            │  │ │
│  │  └──────────────────────────────────────────────────────────────────────┘  │ │
│  │                                                                             │ │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │ │
│  │  │  SessionManagerAdapter                                               │  │ │
│  │  │    impl SessionStorage for SessionManagerAdapter { ... }            │  │ │
│  │  │    Wraps: SessionManager                                             │  │ │
│  │  └──────────────────────────────────────────────────────────────────────┘  │ │
│  │                                                                             │ │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │ │
│  │  │  StreamingProviderAdapter                                            │  │ │
│  │  │    impl StreamingProvider for StreamingProviderAdapter { ... }      │  │ │
│  │  │    Wraps: StreamingModule                                            │  │ │
│  │  └──────────────────────────────────────────────────────────────────────┘  │ │
│  │                                                                             │ │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │ │
│  │  │  TelemetryAdapter                                                    │  │ │
│  │  │    impl TelemetryBackend for TelemetryAdapter { ... }               │  │ │
│  │  │    Wraps: TelemetrySystem                                            │  │ │
│  │  └──────────────────────────────────────────────────────────────────────┘  │ │
│  │                                                                             │ │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │ │
│  │  │  CircuitBreakerAdapter                                               │  │ │
│  │  │    impl CircuitBreaker for CircuitBreakerAdapter { ... }            │  │ │
│  │  │    Wraps: CircuitBreakerState                                        │  │ │
│  │  └──────────────────────────────────────────────────────────────────────┘  │ │
│  │                                                                             │ │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │ │
│  │  │  SseTransportAdapter                                                 │  │ │
│  │  │    impl StreamingProvider for SseTransportAdapter { ... }           │  │ │
│  │  │    Wraps: SSE transport logic                                        │  │ │
│  │  └──────────────────────────────────────────────────────────────────────┘  │ │
│  │                                                                             │ │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │ │
│  │  │  WebSocketTransportAdapter                                           │  │ │
│  │  │    impl StreamingProvider for WebSocketTransportAdapter { ... }     │  │ │
│  │  │    Wraps: WebSocket transport logic                                  │  │ │
│  │  └──────────────────────────────────────────────────────────────────────┘  │ │
│  │                                                                             │ │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │ │
│  │  │  ResourceManagerPoolAdapter                                          │  │ │
│  │  │    impl Pool<ResourceSlot> for ResourceManagerPoolAdapter { ... }   │  │ │
│  │  │    Wraps: ResourceManager (pool semantics)                           │  │ │
│  │  └──────────────────────────────────────────────────────────────────────┘  │ │
│  │                                                                             │ │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │ │
│  │  │  HealthCheckAdapter                                                  │  │ │
│  │  │    impl HealthCheck for HealthCheckAdapter { ... }                  │  │ │
│  │  │    Wraps: Custom health check logic                                  │  │ │
│  │  └──────────────────────────────────────────────────────────────────────┘  │ │
│  │                                                                             │ │
│  └─────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                   │
└───────────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     │ Wraps & Delegates to
                                     │
                                     ▼
┌───────────────────────────────────────────────────────────────────────────────────┐
│                                                                                   │
│                      INFRASTRUCTURE LAYER (Concrete Types)                        │
│                       Distributed across multiple crates                          │
│                                                                                   │
│  ┌─────────────────────────────────────────────────────────────────────────────┐ │
│  │                    CONCRETE IMPLEMENTATIONS                                 │ │
│  │                                                                             │ │
│  │  riptide-api:                                                               │ │
│  │    • ResourceManager       - Resource controls & throttling                 │ │
│  │    • HealthChecker         - Enhanced health diagnostics                    │ │
│  │    • SessionManager        - Session lifecycle management                   │ │
│  │    • StreamingModule       - Real-time streaming infrastructure             │ │
│  │                                                                             │ │
│  │  riptide-cache:                                                             │ │
│  │    • RedisStorage          - Redis-backed cache storage                     │ │
│  │    • RedisManager          - Redis connection pool                          │ │
│  │    • RedisRateLimiter      - Distributed rate limiting                      │ │
│  │                                                                             │ │
│  │  riptide-fetch:                                                             │ │
│  │    • ReqwestHttpClient     - HTTP client adapter                            │ │
│  │    • FetchEngine           - Per-host circuit breakers                      │ │
│  │                                                                             │ │
│  │  riptide-extraction:                                                        │ │
│  │    • UnifiedExtractor      - WASM/native content extraction                 │ │
│  │                                                                             │ │
│  │  riptide-reliability:                                                       │ │
│  │    • ReliableExtractor     - Retry + circuit breaker wrapper                │ │
│  │    • CircuitBreakerState   - Fault tolerance state machine                  │ │
│  │                                                                             │ │
│  │  riptide-monitoring:                                                        │ │
│  │    • MetricsCollector      - Prometheus metrics aggregation                 │ │
│  │    • MonitoringSystem      - Performance tracking + alerting                │ │
│  │    • TelemetrySystem       - OpenTelemetry integration                      │ │
│  │                                                                             │ │
│  │  riptide-events:                                                            │ │
│  │    • EventBus              - Centralized event coordination                 │ │
│  │                                                                             │ │
│  │  riptide-headless:                                                          │ │
│  │    • HeadlessLauncher      - Browser pool management                        │ │
│  │                                                                             │ │
│  │  riptide-facade:                                                            │ │
│  │    • ExtractionFacade      - Extraction business logic                      │ │
│  │    • ScraperFacade         - Scraping business logic                        │ │
│  │    • SpiderFacade          - Crawling business logic                        │ │
│  │    • SearchFacade          - Search integration logic                       │ │
│  │    • EngineFacade          - Engine selection logic                         │ │
│  │    • ResourceFacade        - Resource orchestration logic                   │ │
│  │                                                                             │ │
│  │  riptide-spider:                                                            │ │
│  │    • Spider                - Deep web crawling engine                       │ │
│  │                                                                             │ │
│  │  riptide-workers:                                                           │ │
│  │    • WorkerService         - Background job processing                      │ │
│  │                                                                             │ │
│  │  riptide-performance:                                                       │ │
│  │    • PerformanceManager    - Profiling & resource limiting                  │ │
│  │                                                                             │ │
│  └─────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                   │
└───────────────────────────────────────────────────────────────────────────────────┘
```

## Dependency Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                       DEPENDENCY FLOW                            │
│                (Hexagonal Architecture Pattern)                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  Application Layer (Domain Logic)                                │
│  • Request handlers use ApplicationContext                       │
│  • Depends ONLY on port trait abstractions                       │
│  • Zero knowledge of concrete infrastructure                     │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           │ Depends on
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│  Port Traits (Interface Contracts)                               │
│  • Define infrastructure capabilities                            │
│  • Technology-agnostic specifications                            │
│  • Compiler-enforced contracts                                   │
└──────────────────────────▲──────────────────────────────────────┘
                           │
                           │ Implements
                           │
┌──────────────────────────┴──────────────────────────────────────┐
│  Adapter Layer (Bridge Pattern)                                  │
│  • Wraps concrete infrastructure                                 │
│  • Implements port trait interfaces                              │
│  • Translates between layers                                     │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           │ Wraps & Delegates to
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│  Infrastructure Layer (Concrete Implementations)                 │
│  • Actual technology implementations                             │
│  • Redis, HTTP clients, WebSocket, etc.                          │
│  • External system integrations                                  │
└─────────────────────────────────────────────────────────────────┘

KEY PRINCIPLE: Dependencies point INWARD (↑↑↑)
Infrastructure depends on ports, NOT the reverse
```

## Testing Architecture

```
┌───────────────────────────────────────────────────────────────────┐
│                      TESTING STRATEGY                              │
└───────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────┐
│  UNIT TESTS (Domain Logic)                                         │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │  Application Handler                                         │ │
│  │    ↓ uses                                                    │ │
│  │  ApplicationContext                                          │ │
│  │    ↓ with                                                    │ │
│  │  Mock<dyn HttpClient>        ← Fake/mock implementation      │ │
│  │  Mock<dyn CacheStorage>      ← In-memory mock               │ │
│  │  Mock<dyn HealthCheck>       ← Returns canned responses     │ │
│  │  Mock<dyn ResourceManagement>← Simulated resources          │ │
│  │  [... all other mocked ports]                                │ │
│  │                                                              │ │
│  │  Result: Fast, isolated, deterministic tests                │ │
│  └──────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────┐
│  INTEGRATION TESTS (Infrastructure)                                │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │  Application Handler                                         │ │
│  │    ↓ uses                                                    │ │
│  │  ApplicationContext                                          │ │
│  │    ↓ with                                                    │ │
│  │  RedisStorage (real Redis)   ← Actual Redis instance        │ │
│  │  ReqwestHttpClient (real)    ← Real HTTP client             │ │
│  │  Mock<dyn HealthCheck>       ← Some mocks still used        │ │
│  │  Mock<dyn ResourceManagement>← Focus on specific layer      │ │
│  │  [... mixed real/mock]                                       │ │
│  │                                                              │ │
│  │  Result: Real infrastructure integration testing            │ │
│  └──────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────┐
│  END-TO-END TESTS (Full System)                                    │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │  HTTP Client                                                 │ │
│  │    ↓ requests                                                │ │
│  │  Application Server                                          │ │
│  │    ↓ uses                                                    │ │
│  │  ApplicationContext (fully initialized)                      │ │
│  │    ↓ with                                                    │ │
│  │  ALL REAL INFRASTRUCTURE                                     │ │
│  │  • Redis                                                     │ │
│  │  • HTTP clients                                              │ │
│  │  • WebSocket                                                 │ │
│  │  • Database                                                  │ │
│  │  • External APIs                                             │ │
│  │                                                              │ │
│  │  Result: Complete system behavior validation                │ │
│  └──────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────┘
```

## Adapter Implementation Pattern

```
┌─────────────────────────────────────────────────────────────────────┐
│             ADAPTER IMPLEMENTATION PATTERN                           │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  Port Trait (Interface)                                        │ │
│  │                                                                │ │
│  │  pub trait HealthCheck: Send + Sync {                         │ │
│  │      async fn check(&self) -> Result<HealthStatus>;           │ │
│  │      async fn detailed_check(&self) -> Result<DetailedHealth>;│ │
│  │  }                                                             │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                 ▲                                    │
│                                 │ implements                         │
│                                 │                                    │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  Adapter (Bridge)                                              │ │
│  │                                                                │ │
│  │  pub struct HealthCheckerAdapter {                            │ │
│  │      inner: Arc<HealthChecker>,  // Wraps concrete type       │ │
│  │  }                                                             │ │
│  │                                                                │ │
│  │  impl HealthCheck for HealthCheckerAdapter {                  │ │
│  │      async fn check(&self) -> Result<HealthStatus> {          │ │
│  │          // Delegate to concrete implementation               │ │
│  │          self.inner.check().await                             │ │
│  │      }                                                         │ │
│  │                                                                │ │
│  │      async fn detailed_check(&self) -> Result<DetailedHealth> {│ │
│  │          self.inner.detailed_check().await                    │ │
│  │      }                                                         │ │
│  │  }                                                             │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                 │                                    │
│                                 │ wraps                              │
│                                 │                                    │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  Concrete Implementation                                       │ │
│  │                                                                │ │
│  │  pub struct HealthChecker {                                    │ │
│  │      // Actual implementation details                          │ │
│  │      redis_client: RedisClient,                                │ │
│  │      http_client: HttpClient,                                  │ │
│  │      // ... infrastructure dependencies                        │ │
│  │  }                                                             │ │
│  │                                                                │ │
│  │  impl HealthChecker {                                          │ │
│  │      pub async fn check(&self) -> Result<HealthStatus> {      │ │
│  │          // Real implementation with Redis, HTTP, etc.         │ │
│  │          // ...                                                │ │
│  │      }                                                         │ │
│  │  }                                                             │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Benefits Summary

```
┌───────────────────────────────────────────────────────────────────────┐
│                     HEXAGONAL ARCHITECTURE BENEFITS                    │
└───────────────────────────────────────────────────────────────────────┘

1. TESTABILITY ✅
   ┌──────────────────────────────────────────────────────────────────┐
   │  • Mock all infrastructure via trait implementations             │
   │  • Isolated unit tests without external dependencies             │
   │  • Fast CI/CD with deterministic test results                    │
   │  • Integration tests with partial mocking                        │
   └──────────────────────────────────────────────────────────────────┘

2. FLEXIBILITY ✅
   ┌──────────────────────────────────────────────────────────────────┐
   │  • Swap infrastructure without domain changes                    │
   │  • Multiple implementations per port trait                       │
   │  • Runtime configuration via dependency injection                │
   │  • Feature flags enable/disable integrations                     │
   └──────────────────────────────────────────────────────────────────┘

3. MAINTAINABILITY ✅
   ┌──────────────────────────────────────────────────────────────────┐
   │  • Clear layer boundaries and responsibilities                   │
   │  • Single responsibility per adapter                             │
   │  • Dependency contracts enforced by compiler                     │
   │  • Infrastructure changes don't impact domain                    │
   └──────────────────────────────────────────────────────────────────┘

4. SCALABILITY ✅
   ┌──────────────────────────────────────────────────────────────────┐
   │  • Add new infrastructure without domain changes                 │
   │  • Parallel development of layers                                │
   │  • Team can work on adapters independently                       │
   │  • Easier to refactor and optimize specific layers               │
   └──────────────────────────────────────────────────────────────────┘

5. DOMAIN PURITY ✅
   ┌──────────────────────────────────────────────────────────────────┐
   │  • Business logic free from infrastructure concerns              │
   │  • Domain models don't depend on external systems                │
   │  • Technology decisions isolated from business rules             │
   │  • Easier to understand and reason about domain logic            │
   └──────────────────────────────────────────────────────────────────┘
```

---

**Generated:** 2025-11-12
**Version:** 1.0
**Status:** Final
