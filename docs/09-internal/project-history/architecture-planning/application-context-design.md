# ApplicationContext Architecture Design

**Phase**: 1-2 Architecture Design
**Date**: 2025-11-11
**Status**: Complete
**Architect**: System Architecture Designer

---

## Executive Summary

This document specifies the complete `ApplicationContext` structure with all ports, the facade factory pattern for dependency injection, and the strategy for zero-circular-dependency composition.

## Design Goals

1. **Single Composition Root**: All dependencies wired in one place
2. **Zero Circular Dependencies**: Strict layering with dependency inversion
3. **Testability**: Easy to swap implementations for testing
4. **Type Safety**: Compile-time dependency validation
5. **Performance**: Minimal overhead, Arc-based sharing

---

## ApplicationContext Structure

### Complete Port Composition

```rust
/// ApplicationContext - Composition Root for Hexagonal Architecture
///
/// This struct serves as the dependency injection container, holding all
/// port implementations wired together. It represents the composition root
/// where all dependencies are assembled.
///
/// # Layer Architecture
///
/// ```text
/// API Layer (riptide-api)
///       ↓ creates
/// APPLICATION CONTEXT ← YOU ARE HERE
///       ↓ injects ports into
/// Application Layer (riptide-facade)
///       ↓ uses ports (traits)
/// Domain Layer (riptide-types)
///       ↑ implemented by
/// Infrastructure Layer (riptide-*)
/// ```
#[derive(Clone)]
pub struct ApplicationContext {
    // ========================================================================
    // CORE INFRASTRUCTURE PORTS
    // ========================================================================

    /// System clock (real or fake for testing)
    pub clock: Arc<dyn Clock>,

    /// Entropy source (real or deterministic for testing)
    pub entropy: Arc<dyn Entropy>,

    // ========================================================================
    // PERSISTENCE LAYER PORTS
    // ========================================================================

    /// Transaction manager for ACID operations
    #[cfg(feature = "postgres")]
    pub transaction_manager: Arc<dyn TransactionManager<Transaction = PostgresTransaction>>,

    #[cfg(not(feature = "postgres"))]
    pub transaction_manager: Arc<dyn TransactionManager<Transaction = InMemoryTransaction>>,

    /// Generic repository factory (creates type-specific repositories)
    pub repository_factory: Arc<dyn RepositoryFactory>,

    // ========================================================================
    // EVENT SYSTEM PORTS
    // ========================================================================

    /// Event bus for domain events (outbox pattern in production)
    pub event_bus: Arc<dyn EventBus>,

    // ========================================================================
    // CACHING & STORAGE PORTS
    // ========================================================================

    /// Cache storage for caching arbitrary data
    pub cache: Arc<dyn CacheStorage>,

    /// Idempotency store for duplicate request prevention
    pub idempotency_store: Arc<dyn IdempotencyStore>,

    /// Session storage for browser/user sessions
    pub session_storage: Arc<dyn SessionStorage>,

    // ========================================================================
    // RESOURCE MANAGEMENT PORTS
    // ========================================================================

    /// WASM instance pool for sandboxed extraction
    pub wasm_pool: Arc<dyn Pool<WasmInstance>>,

    /// Browser session pool for headless rendering
    #[cfg(feature = "browser")]
    pub browser_pool: Arc<dyn Pool<BrowserSession>>,

    /// LLM client pool for AI operations
    #[cfg(feature = "llm")]
    pub llm_pool: Arc<dyn Pool<LlmClient>>,

    // ========================================================================
    // RELIABILITY & RESILIENCE PORTS
    // ========================================================================

    /// Circuit breaker for headless service
    pub headless_circuit_breaker: Arc<dyn CircuitBreaker>,

    /// Circuit breaker for LLM service
    #[cfg(feature = "llm")]
    pub llm_circuit_breaker: Arc<dyn CircuitBreaker>,

    /// Rate limiter for external requests
    pub rate_limiter: Arc<dyn RateLimiter>,

    // ========================================================================
    // HTTP CLIENT PORTS
    // ========================================================================

    /// HTTP client for external requests
    pub http_client: Arc<dyn HttpClient>,

    /// HTTP client for internal headless service
    pub headless_http_client: Arc<dyn HttpClient>,

    // ========================================================================
    // OBSERVABILITY PORTS
    // ========================================================================

    /// Health check registry
    pub health_registry: Arc<dyn HealthRegistry>,

    /// Metrics collector (low-level)
    pub metrics_collector: Arc<dyn MetricsCollector>,

    /// Business metrics (high-level)
    pub business_metrics: Arc<dyn BusinessMetrics>,

    // ========================================================================
    // STREAMING PORTS
    // ========================================================================

    /// SSE transport for server-sent events
    pub sse_transport: Arc<dyn StreamingTransport>,

    /// WebSocket transport for bidirectional streaming
    pub websocket_transport: Arc<dyn StreamingTransport>,

    // ========================================================================
    // CONFIGURATION
    // ========================================================================

    /// Feature flags and runtime configuration
    pub config: DiConfig,
}
```

---

## Facade Factory Pattern

### Design

The facade factory pattern creates facades on-demand, injecting the required ports from ApplicationContext. This enables:

1. **Lazy Construction**: Facades created only when needed
2. **Testability**: Easy to inject mock ports
3. **Type Safety**: Compiler validates required ports
4. **Separation**: Facades don't know about ApplicationContext

### Factory Trait

```rust
/// Facade factory for creating application use-cases
///
/// This trait enables facade creation with injected dependencies,
/// following the Abstract Factory pattern.
pub trait FacadeFactory: Send + Sync {
    /// Create crawl facade with required ports
    fn create_crawl_facade(&self) -> Arc<CrawlFacade>;

    /// Create browser facade with required ports
    #[cfg(feature = "browser")]
    fn create_browser_facade(&self) -> Arc<BrowserFacade>;

    /// Create scraper facade with required ports
    fn create_scraper_facade(&self) -> Arc<ScraperFacade>;

    /// Create search facade with required ports
    #[cfg(feature = "search")]
    fn create_search_facade(&self) -> Arc<SearchFacade>;

    /// Create LLM facade with required ports
    #[cfg(feature = "llm")]
    fn create_llm_facade(&self) -> Arc<LlmFacade>;

    /// Create pipeline facade with required ports
    fn create_pipeline_facade(&self) -> Arc<PipelineFacade>;

    /// Create spider facade with required ports
    fn create_spider_facade(&self) -> Arc<SpiderFacade>;
}
```

### Factory Implementation

```rust
/// Default facade factory implementation
///
/// Creates facades by injecting ports from ApplicationContext.
pub struct DefaultFacadeFactory {
    ctx: Arc<ApplicationContext>,
}

impl DefaultFacadeFactory {
    pub fn new(ctx: Arc<ApplicationContext>) -> Self {
        Self { ctx }
    }
}

impl FacadeFactory for DefaultFacadeFactory {
    fn create_crawl_facade(&self) -> Arc<CrawlFacade> {
        Arc::new(CrawlFacade::new(
            self.ctx.wasm_pool.clone(),
            self.ctx.cache.clone(),
            self.ctx.http_client.clone(),
            self.ctx.rate_limiter.clone(),
            self.ctx.event_bus.clone(),
            self.ctx.idempotency_store.clone(),
            self.ctx.metrics_collector.clone(),
        ))
    }

    #[cfg(feature = "browser")]
    fn create_browser_facade(&self) -> Arc<BrowserFacade> {
        Arc::new(BrowserFacade::new(
            self.ctx.browser_pool.clone(),
            self.ctx.headless_http_client.clone(),
            self.ctx.headless_circuit_breaker.clone(),
            self.ctx.cache.clone(),
            self.ctx.session_storage.clone(),
            self.ctx.event_bus.clone(),
            self.ctx.metrics_collector.clone(),
        ))
    }

    fn create_scraper_facade(&self) -> Arc<ScraperFacade> {
        Arc::new(ScraperFacade::new(
            self.ctx.http_client.clone(),
            self.ctx.wasm_pool.clone(),
            self.ctx.cache.clone(),
            self.ctx.rate_limiter.clone(),
            self.ctx.idempotency_store.clone(),
        ))
    }

    fn create_pipeline_facade(&self) -> Arc<PipelineFacade> {
        Arc::new(PipelineFacade::new(
            self.create_crawl_facade(),
            self.create_scraper_facade(),
            #[cfg(feature = "browser")]
            self.create_browser_facade(),
            self.ctx.event_bus.clone(),
            self.ctx.metrics_collector.clone(),
        ))
    }

    fn create_spider_facade(&self) -> Arc<SpiderFacade> {
        Arc::new(SpiderFacade::new(
            self.create_scraper_facade(),
            self.ctx.http_client.clone(),
            self.ctx.cache.clone(),
            self.ctx.rate_limiter.clone(),
        ))
    }
}
```

---

## Dependency Injection Strategy

### 1. Composition Root Pattern

**Location**: `crates/riptide-api/src/composition/mod.rs`

All dependency wiring happens in one place:

```rust
impl ApplicationContext {
    /// Create production ApplicationContext with real adapters
    pub async fn new(config: &DiConfig) -> Result<Self> {
        // Validate configuration
        config.validate()?;

        // Wire infrastructure ports
        let clock = Arc::new(SystemClock) as Arc<dyn Clock>;
        let entropy = Arc::new(SystemEntropy) as Arc<dyn Entropy>;

        // Wire persistence layer
        let pool = create_postgres_pool(config).await?;
        let transaction_manager = Arc::new(PostgresTransactionManager::new(pool.clone()));
        let repository_factory = Arc::new(PostgresRepositoryFactory::new(pool.clone()));

        // Wire caching layer
        let cache = Arc::new(RedisCache::new(&config.redis_url).await?);
        let idempotency_store = Arc::new(RedisIdempotencyStore::new(&config.redis_url).await?);
        let session_storage = Arc::new(PostgresSessionStorage::new(pool.clone()));

        // Wire event system
        let event_bus = Arc::new(OutboxEventBus::new(pool.clone()));

        // Wire resource pools
        let wasm_pool = Arc::new(WasmInstancePool::new(config.wasm_pool_size));

        #[cfg(feature = "browser")]
        let browser_pool = Arc::new(BrowserPool::new(config.browser_pool_size).await?);

        #[cfg(feature = "llm")]
        let llm_pool = Arc::new(LlmClientPool::new(config.llm_pool_size));

        // Wire reliability layer
        let headless_circuit_breaker = Arc::new(AtomicCircuitBreaker::new(
            config.headless_circuit_breaker_config.clone(),
        ));

        #[cfg(feature = "llm")]
        let llm_circuit_breaker = Arc::new(AtomicCircuitBreaker::new(
            config.llm_circuit_breaker_config.clone(),
        ));

        let rate_limiter = Arc::new(TokenBucketRateLimiter::new(config.rate_limit_config.clone()));

        // Wire HTTP clients
        let http_client = Arc::new(ReqwestHttpClient::new(config.http_config.clone())?);
        let headless_http_client = Arc::new(ReqwestHttpClient::new(config.headless_http_config.clone())?);

        // Wire observability
        let health_registry = Arc::new(DefaultHealthRegistry::new());
        let metrics_collector = Arc::new(PrometheusMetrics::new());
        let business_metrics = Arc::new(DefaultBusinessMetrics::new(metrics_collector.clone()));

        // Wire streaming
        let sse_transport = Arc::new(SseTransport::new());
        let websocket_transport = Arc::new(WebSocketTransport::new());

        // Register health checks
        register_health_checks(&health_registry, &cache, &pool, &wasm_pool).await;

        Ok(Self {
            clock,
            entropy,
            transaction_manager,
            repository_factory,
            event_bus,
            cache,
            idempotency_store,
            session_storage,
            wasm_pool,
            #[cfg(feature = "browser")]
            browser_pool,
            #[cfg(feature = "llm")]
            llm_pool,
            headless_circuit_breaker,
            #[cfg(feature = "llm")]
            llm_circuit_breaker,
            rate_limiter,
            http_client,
            headless_http_client,
            health_registry,
            metrics_collector,
            business_metrics,
            sse_transport,
            websocket_transport,
            config: config.clone(),
        })
    }
}
```

### 2. Testing Strategy

```rust
impl ApplicationContext {
    /// Create testing ApplicationContext with in-memory implementations
    pub fn for_testing() -> Self {
        let clock = Arc::new(FakeClock::at_epoch());
        let entropy = Arc::new(DeterministicEntropy::new(42));

        let transaction_manager = Arc::new(InMemoryTransactionManager::new());
        let repository_factory = Arc::new(InMemoryRepositoryFactory::new());

        let cache = Arc::new(InMemoryCache::new());
        let idempotency_store = Arc::new(InMemoryIdempotencyStore::new());
        let session_storage = Arc::new(InMemorySessionStorage::new());

        let event_bus = Arc::new(InMemoryEventBus::new());

        let wasm_pool = Arc::new(InMemoryPool::new());

        #[cfg(feature = "browser")]
        let browser_pool = Arc::new(InMemoryPool::new());

        #[cfg(feature = "llm")]
        let llm_pool = Arc::new(InMemoryPool::new());

        let headless_circuit_breaker = Arc::new(AlwaysClosedCircuitBreaker::new());

        #[cfg(feature = "llm")]
        let llm_circuit_breaker = Arc::new(AlwaysClosedCircuitBreaker::new());

        let rate_limiter = Arc::new(UnlimitedRateLimiter::new());

        let http_client = Arc::new(MockHttpClient::new());
        let headless_http_client = Arc::new(MockHttpClient::new());

        let health_registry = Arc::new(DefaultHealthRegistry::new());
        let metrics_collector = Arc::new(InMemoryMetrics::new());
        let business_metrics = Arc::new(DefaultBusinessMetrics::new(metrics_collector.clone()));

        let sse_transport = Arc::new(MockStreamingTransport::new());
        let websocket_transport = Arc::new(MockStreamingTransport::new());

        let config = DiConfig::for_testing();

        Self {
            clock,
            entropy,
            transaction_manager,
            repository_factory,
            event_bus,
            cache,
            idempotency_store,
            session_storage,
            wasm_pool,
            #[cfg(feature = "browser")]
            browser_pool,
            #[cfg(feature = "llm")]
            llm_pool,
            headless_circuit_breaker,
            #[cfg(feature = "llm")]
            llm_circuit_breaker,
            rate_limiter,
            http_client,
            headless_http_client,
            health_registry,
            metrics_collector,
            business_metrics,
            sse_transport,
            websocket_transport,
            config,
        }
    }
}
```

### 3. Builder Pattern for Custom Testing

```rust
impl ApplicationContext {
    pub fn builder() -> ApplicationContextBuilder {
        ApplicationContextBuilder::new()
    }
}

pub struct ApplicationContextBuilder {
    clock: Option<Arc<dyn Clock>>,
    cache: Option<Arc<dyn CacheStorage>>,
    http_client: Option<Arc<dyn HttpClient>>,
    // ... other overrides
}

impl ApplicationContextBuilder {
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = Some(clock);
        self
    }

    pub fn with_cache(mut self, cache: Arc<dyn CacheStorage>) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn build_for_testing(self) -> ApplicationContext {
        let base = ApplicationContext::for_testing();

        ApplicationContext {
            clock: self.clock.unwrap_or(base.clock),
            cache: self.cache.unwrap_or(base.cache),
            http_client: self.http_client.unwrap_or(base.http_client),
            ..base
        }
    }
}
```

---

## Zero Circular Dependencies

### Dependency Graph

```text
┌─────────────────────────────────────────────────────────┐
│                    riptide-api                           │
│          (Composition Root + HTTP Handlers)              │
│                                                          │
│  ┌─────────────────────────────────────────────┐        │
│  │       ApplicationContext                     │        │
│  │  - Wires all dependencies                    │        │
│  │  - Creates facade factory                    │        │
│  └──────────────┬──────────────────────────────┘        │
└─────────────────┼───────────────────────────────────────┘
                  │
                  ↓ injects ports into
┌─────────────────────────────────────────────────────────┐
│                 riptide-facade                           │
│           (Application Use-Cases)                        │
│                                                          │
│  CrawlFacade, BrowserFacade, ScraperFacade, etc.        │
│  - Only depends on port traits                           │
│  - No infrastructure imports                             │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ↓ uses
┌─────────────────────────────────────────────────────────┐
│                 riptide-types                            │
│            (Domain + Port Traits)                        │
│                                                          │
│  Port Traits (CacheStorage, CircuitBreaker, etc.)       │
│  Domain Types (ExtractedDoc, CrawlResult, etc.)         │
└──────────────────┬──────────────────────────────────────┘
                   ↑
                   │ implemented by
┌──────────────────────────────────────────────────────────┐
│         Infrastructure Crates (Adapters)                  │
│                                                           │
│  riptide-cache    → RedisCache                           │
│  riptide-persistence → PostgresRepository                │
│  riptide-browser  → BrowserPool                          │
│  riptide-reliability → AtomicCircuitBreaker              │
│  riptide-pool     → WasmInstancePool                     │
└───────────────────────────────────────────────────────────┘
```

**Key Rules**:
1. riptide-types has NO dependencies on infrastructure crates
2. riptide-facade ONLY depends on riptide-types
3. Infrastructure crates implement riptide-types ports
4. riptide-api wires everything together

---

## Extension Points

### Adding New Port

1. Define trait in `riptide-types/src/ports/`
2. Create adapter in infrastructure crate
3. Add to ApplicationContext
4. Inject into facades via factory

### Adding New Facade

1. Create facade in `riptide-facade/src/facades/`
2. Accept ports via constructor
3. Add factory method to FacadeFactory
4. Wire in DefaultFacadeFactory

### Feature Flags

```rust
#[cfg(feature = "browser")]
pub browser_pool: Arc<dyn Pool<BrowserSession>>,

#[cfg(feature = "llm")]
pub llm_circuit_breaker: Arc<dyn CircuitBreaker>,
```

---

## Next Steps

1. **Implement CircuitBreaker Port** (See: port-trait-specifications.md)
2. **Update ApplicationContext** (Add new ports)
3. **Create Facade Factory** (Implement pattern)
4. **Migration Execution** (See: migration-strategy.md)
