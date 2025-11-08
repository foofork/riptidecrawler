# Phase 4: Infrastructure Consolidation Roadmap
**Version:** 2.1 (Enhanced Scope)
**Date:** 2025-11-08
**Duration:** 2 weeks
**Status:** Ready for Implementation

---

## Phase Overview

**Goal:** Centralize infrastructure concerns in reliability and cache layers

**Objectives:**
- Consolidate ALL HTTP clients to use ReliableHttpClient
- Implement single Redis manager with versioned keys
- Refactor streaming system (5,427 LOC) to use ports
- Consolidate resource manager (1,845 LOC) to facades
- Split metrics system (1,670 LOC) business vs transport
- Achieve graceful degradation for all external dependencies

**Enhanced Coverage:**
This phase now addresses **7,000+ LOC** of infrastructure violations (was 800 LOC), representing critical architectural gaps identified in the API coverage analysis.

---

## Prerequisites from Previous Phases

**Phase 3 Must Be Complete:**
- ✅ All handlers <50 LOC
- ✅ All business logic in facades
- ✅ Facades depend only on ports (traits)
- ✅ Zero serde_json::Value in facades
- ✅ ≥90% facade test coverage

**Phase 1 & 2 Must Be Complete:**
- ✅ All port traits defined
- ✅ All adapters implemented
- ✅ ApplicationContext DI working
- ✅ Authorization, idempotency, events infrastructure ready

---

## Sprint 4.1: HTTP Client Consolidation (Week 7, Days 1-2)

**Duration:** 2 days
**Priority:** HIGH (eliminate scattered HTTP client usage)

### Current State Analysis

**Problem:**
- Multiple crates create their own `reqwest::Client` instances
- No centralized circuit breaker strategy
- Inconsistent retry logic
- No shared connection pooling
- No request/response telemetry

**Files with Direct reqwest Usage:**
```
crates/riptide-fetch/src/lib.rs
crates/riptide-spider/src/lib.rs
crates/riptide-pdf/src/lib.rs
crates/riptide-browser/src/lib.rs
crates/riptide-search/src/lib.rs
```

### Solution: ReliableHttpClient with Circuit Breakers

**Enhanced Implementation:**

```rust
// crates/riptide-reliability/src/http_client.rs (ENHANCED)
use reqwest::{Client, Request, Response};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct ReliableHttpClient {
    client: Client,
    circuit_breakers: Arc<DashMap<String, CircuitBreaker>>,
    retry_policy: RetryPolicy,
    telemetry: Arc<dyn HttpTelemetry>,
}

/// Circuit breaker presets for different endpoint types
pub enum CircuitBreakerPreset {
    /// Browser rendering: high failure tolerance (50%), long timeout (30s)
    BrowserRendering,
    /// PDF processing: medium tolerance (30%), long timeout (20s)
    PdfProcessing,
    /// Search indexing: low tolerance (10%), medium timeout (5s)
    SearchIndexing,
    /// External APIs: medium tolerance (20%), short timeout (3s)
    ExternalApi,
    /// Internal services: low tolerance (5%), very short timeout (1s)
    InternalService,
    /// Web scraping: high tolerance (40%), medium timeout (10s)
    WebScraping,
}

impl ReliableHttpClient {
    pub fn new(config: HttpClientConfig) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(config.pool_size)
            .timeout(Duration::from_secs(config.default_timeout_secs))
            .tcp_keepalive(Some(Duration::from_secs(60)))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            circuit_breakers: Arc::new(DashMap::new()),
            retry_policy: config.retry_policy,
            telemetry: config.telemetry,
        }
    }

    /// Create client with preset circuit breaker for endpoint type
    pub fn with_preset(preset: CircuitBreakerPreset) -> Self {
        let config = match preset {
            CircuitBreakerPreset::BrowserRendering => HttpClientConfig {
                pool_size: 10,
                default_timeout_secs: 30,
                retry_policy: RetryPolicy::exponential(3, Duration::from_secs(1)),
                circuit_breaker: CircuitBreakerConfig {
                    failure_threshold: 0.5,  // 50% failure rate
                    min_requests: 10,
                    timeout: Duration::from_secs(60),
                },
            },
            CircuitBreakerPreset::PdfProcessing => HttpClientConfig {
                pool_size: 20,
                default_timeout_secs: 20,
                retry_policy: RetryPolicy::exponential(2, Duration::from_millis(500)),
                circuit_breaker: CircuitBreakerConfig {
                    failure_threshold: 0.3,  // 30% failure rate
                    min_requests: 20,
                    timeout: Duration::from_secs(45),
                },
            },
            CircuitBreakerPreset::SearchIndexing => HttpClientConfig {
                pool_size: 50,
                default_timeout_secs: 5,
                retry_policy: RetryPolicy::exponential(3, Duration::from_millis(200)),
                circuit_breaker: CircuitBreakerConfig {
                    failure_threshold: 0.1,  // 10% failure rate
                    min_requests: 50,
                    timeout: Duration::from_secs(30),
                },
            },
            CircuitBreakerPreset::ExternalApi => HttpClientConfig {
                pool_size: 30,
                default_timeout_secs: 3,
                retry_policy: RetryPolicy::exponential(2, Duration::from_millis(300)),
                circuit_breaker: CircuitBreakerConfig {
                    failure_threshold: 0.2,  // 20% failure rate
                    min_requests: 30,
                    timeout: Duration::from_secs(20),
                },
            },
            CircuitBreakerPreset::InternalService => HttpClientConfig {
                pool_size: 100,
                default_timeout_secs: 1,
                retry_policy: RetryPolicy::fixed(1, Duration::from_millis(100)),
                circuit_breaker: CircuitBreakerConfig {
                    failure_threshold: 0.05,  // 5% failure rate
                    min_requests: 100,
                    timeout: Duration::from_secs(10),
                },
            },
            CircuitBreakerPreset::WebScraping => HttpClientConfig {
                pool_size: 50,
                default_timeout_secs: 10,
                retry_policy: RetryPolicy::exponential_with_jitter(3, Duration::from_secs(1)),
                circuit_breaker: CircuitBreakerConfig {
                    failure_threshold: 0.4,  // 40% failure rate
                    min_requests: 20,
                    timeout: Duration::from_secs(30),
                },
            },
        };

        Self::new(config)
    }

    pub async fn execute(&self, request: Request) -> Result<Response> {
        let endpoint = request.url().host_str().unwrap_or("unknown").to_string();

        // Check circuit breaker
        if let Some(cb) = self.circuit_breakers.get(&endpoint) {
            if cb.is_open() {
                return Err(RiptideError::CircuitBreakerOpen { endpoint });
            }
        }

        // Telemetry: start
        let start = Instant::now();
        self.telemetry.record_request_start(&request);

        // Execute with retry
        let result = self.retry_policy.execute(|| async {
            self.client.execute(request.try_clone().unwrap()).await
                .map_err(|e| RiptideError::HttpRequest { source: e })
        }).await;

        // Telemetry: end
        let duration = start.elapsed();
        self.telemetry.record_request_end(&request, &result, duration);

        // Update circuit breaker
        self.circuit_breakers
            .entry(endpoint)
            .or_insert_with(|| CircuitBreaker::new(self.config.circuit_breaker.clone()))
            .record_result(&result);

        result
    }
}
```

### Migration Tasks (Sprint 4.1)

**Task 4.1.1: Replace reqwest::Client in All Crates**

```rust
// BEFORE (crates/riptide-fetch/src/lib.rs)
use reqwest::Client;

pub struct Fetcher {
    client: Client,  // ❌ Direct usage
}

impl Fetcher {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap()
        }
    }

    pub async fn fetch(&self, url: &str) -> Result<String> {
        self.client.get(url).send().await?.text().await
            .map_err(Into::into)
    }
}

// AFTER (crates/riptide-fetch/src/lib.rs)
use riptide_reliability::http_client::{ReliableHttpClient, CircuitBreakerPreset};

pub struct Fetcher {
    client: Arc<ReliableHttpClient>,  // ✅ Via reliability layer
}

impl Fetcher {
    pub fn new(client: Arc<ReliableHttpClient>) -> Self {
        Self { client }
    }

    pub async fn fetch(&self, url: &str) -> Result<String> {
        let request = reqwest::Request::new(reqwest::Method::GET, url.parse()?);
        let response = self.client.execute(request).await?;
        let text = response.text().await?;
        Ok(text)
    }
}
```

**Files to Modify:**
```
UPDATE: crates/riptide-fetch/src/lib.rs (use ReliableHttpClient)
UPDATE: crates/riptide-spider/src/lib.rs (use ReliableHttpClient)
UPDATE: crates/riptide-pdf/src/lib.rs (use ReliableHttpClient with PdfProcessing preset)
UPDATE: crates/riptide-browser/src/lib.rs (use ReliableHttpClient with BrowserRendering preset)
UPDATE: crates/riptide-search/src/lib.rs (use ReliableHttpClient with SearchIndexing preset)
UPDATE: crates/riptide-reliability/src/http_client.rs (add presets)
UPDATE: crates/riptide-api/src/composition.rs (wire ReliableHttpClient)
```

### Validation (Sprint 4.1)

```bash
# No direct reqwest usage outside reliability
rg "reqwest::Client::new\|reqwest::Client::builder" crates/riptide-{facade,api,spider,fetch,pdf,browser,search} && echo "FAIL: Direct reqwest found" || echo "PASS"

# All HTTP via reliability layer
rg "ReliableHttpClient" crates/riptide-{fetch,spider,pdf,browser,search} || echo "FAIL: Not using ReliableHttpClient"

# Tests pass
cargo test -p riptide-reliability
cargo test -p riptide-fetch
cargo test -p riptide-spider
```

---

## Sprint 4.2: Redis Consolidation (Week 7, Days 3-4)

**Duration:** 2 days
**Priority:** CRITICAL (eliminate scattered Redis usage)

### Current State

**Problem:** 6 crates with Redis dependencies (covered in Phase 0, but implementation here)

**Solution:** Single RedisManager in riptide-cache

See Phase 0 Task 0.1.3 for full implementation. This sprint VALIDATES the consolidation.

### Validation (Sprint 4.2)

```bash
# Only 2 crates have Redis
REDIS_COUNT=$(find crates -name "Cargo.toml" -exec grep -l "redis" {} \; | wc -l)
[ "$REDIS_COUNT" -le 2 ] && echo "PASS: Redis in $REDIS_COUNT crates" || echo "FAIL: Redis in $REDIS_COUNT crates (max: 2)"

# No direct redis usage outside cache/workers
rg "redis::" crates/riptide-{facade,api,persistence,utils,performance} && echo "FAIL: Redis found outside cache" || echo "PASS"

# All cache access via CacheStorage trait
rg "CacheStorage" crates/riptide-facade/src/ || echo "FAIL: Facades not using CacheStorage"
```

---

## Sprint 4.3: Streaming System Refactoring (Week 7, Day 5 + Week 8, Days 1-3)

**Duration:** 4 days
**Priority:** CRITICAL (5,427 LOC of business logic in API layer)

### Current State Analysis

**CRITICAL ISSUE:**
The streaming system is entirely in `crates/riptide-api/src/streaming/`, violating clean architecture. It contains:
- Business logic (processor, pipeline, lifecycle)
- Domain orchestration (response_helpers)
- Transport concerns (websocket, sse)
- Configuration (config, buffer, metrics)

**Files (15 total, 5,427 LOC):**

| File | LOC | Violation | Target Layer |
|------|-----|-----------|--------------|
| response_helpers.rs | 924 | Business logic | Facade |
| websocket.rs | 684 | Transport + domain | Port + Adapter |
| processor.rs | 634 | Business logic | Facade |
| pipeline.rs | 628 | Business logic | Facade |
| lifecycle.rs | 622 | Business logic | Facade |
| sse.rs | 575 | Transport + domain | Port + Adapter |
| buffer.rs | 554 | Infrastructure | Reliability |
| mod.rs | 546 | Orchestration | Facade |
| config.rs | 444 | Configuration | Config |
| metrics.rs | 329 | Business metrics | Facade |
| error.rs | 265 | Domain errors | Types |
| ndjson/* | 725 | Format handling | Facade |

### Solution: Port-Based Streaming Architecture

**Step 1: Define Streaming Ports**

```rust
// crates/riptide-types/src/ports/streaming.rs (NEW)
use async_trait::async_trait;
use futures::Stream;

/// Streaming transport port (implemented by WebSocket, SSE adapters)
#[async_trait]
pub trait StreamingTransport: Send + Sync {
    type Item: Send;
    type Error: std::error::Error + Send + Sync + 'static;

    async fn send(&self, item: Self::Item) -> Result<(), Self::Error>;
    async fn close(&self) -> Result<(), Self::Error>;
    fn is_connected(&self) -> bool;
}

/// Streaming processor port (business logic interface)
#[async_trait]
pub trait StreamProcessor: Send + Sync {
    type Input;
    type Output;

    async fn process_stream(
        &self,
        input: impl Stream<Item = Self::Input> + Send,
    ) -> Result<impl Stream<Item = Self::Output> + Send>;
}

/// Lifecycle management port
#[async_trait]
pub trait StreamLifecycle: Send + Sync {
    async fn start(&self) -> Result<()>;
    async fn pause(&self) -> Result<()>;
    async fn resume(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    fn state(&self) -> StreamState;
}

pub enum StreamState {
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Failed(String),
}
```

**Step 2: Create Streaming Facade**

```rust
// crates/riptide-facade/src/facades/streaming.rs (NEW - consolidates processor, pipeline, lifecycle)
pub struct StreamingFacade {
    cache: Arc<dyn CacheStorage>,
    event_bus: Arc<dyn EventBus>,
    metrics: Arc<BusinessMetrics>,
}

impl StreamingFacade {
    pub async fn create_stream<T>(
        &self,
        source: StreamSource,
        config: StreamConfig,
        authz_ctx: &AuthorizationContext,
    ) -> Result<impl Stream<Item = T>>
    where
        T: Send + Serialize + DeserializeOwned,
    {
        // Authorization
        self.authorize(authz_ctx, &source)?;

        // Business logic from processor.rs, pipeline.rs, lifecycle.rs
        let stream = match source {
            StreamSource::WebCrawl(url) => self.crawl_stream(&url, config).await?,
            StreamSource::PdfPages(pdf) => self.pdf_stream(&pdf, config).await?,
            StreamSource::SearchResults(query) => self.search_stream(&query, config).await?,
        };

        // Wrap with lifecycle management
        let managed_stream = ManagedStream::new(stream, self.metrics.clone());

        // Emit event
        self.event_bus.publish(DomainEvent {
            event_type: "stream.created".to_string(),
            aggregate_id: source.id(),
            // ...
        }).await?;

        Ok(managed_stream)
    }

    // Business logic from processor.rs (634 LOC)
    async fn crawl_stream(&self, url: &str, config: StreamConfig) -> Result<impl Stream<Item = CrawlResult>> {
        // Move logic from streaming/processor.rs
    }

    // Business logic from pipeline.rs (628 LOC)
    async fn pdf_stream(&self, pdf: &[u8], config: StreamConfig) -> Result<impl Stream<Item = PdfPage>> {
        // Move logic from streaming/pipeline.rs
    }

    // Business logic from lifecycle.rs (622 LOC)
    // Moved to ManagedStream wrapper
}

struct ManagedStream<S> {
    inner: S,
    metrics: Arc<BusinessMetrics>,
    state: Arc<RwLock<StreamState>>,
}

impl<S: Stream> Stream for ManagedStream<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Lifecycle management from lifecycle.rs
        // Metrics collection from metrics.rs
        // Buffer management from buffer.rs
    }
}
```

**Step 3: Create Transport Adapters**

```rust
// crates/riptide-api/src/adapters/websocket_transport.rs (NEW - move from handlers/streaming/websocket.rs)
use riptide_types::ports::StreamingTransport;
use axum::extract::ws::{WebSocket, Message};

pub struct WebSocketTransport {
    socket: WebSocket,
}

#[async_trait]
impl StreamingTransport for WebSocketTransport {
    type Item = serde_json::Value;
    type Error = RiptideError;

    async fn send(&self, item: Self::Item) -> Result<(), Self::Error> {
        let json = serde_json::to_string(&item)?;
        self.socket.send(Message::Text(json)).await?;
        Ok(())
    }

    async fn close(&self) -> Result<(), Self::Error> {
        self.socket.close().await?;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        // Check connection state
        true
    }
}

// crates/riptide-api/src/adapters/sse_transport.rs (NEW - move from handlers/streaming/sse.rs)
pub struct SseTransport {
    // SSE implementation
}

#[async_trait]
impl StreamingTransport for SseTransport {
    // Similar to WebSocketTransport
}
```

**Step 4: Simplify Handlers**

```rust
// crates/riptide-api/src/handlers/streaming.rs (REFACTORED - was 300 LOC)
pub async fn stream_crawl(
    State(state): State<AppState>,
    AuthContext(authz): AuthContext,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        // Create transport adapter
        let transport = WebSocketTransport::new(socket);

        // Create stream via facade
        let stream = state.streaming_facade
            .create_stream(StreamSource::WebCrawl(url), config, &authz)
            .await
            .expect("Failed to create stream");

        // Wire stream to transport
        while let Some(item) = stream.next().await {
            transport.send(item).await.ok();
        }

        transport.close().await.ok();
    })
}
// Total: 20 LOC (was 300)
```

### Files Modified (Sprint 4.3)

```
CREATE: crates/riptide-types/src/ports/streaming.rs (~300 LOC)
CREATE: crates/riptide-facade/src/facades/streaming.rs (~2,000 LOC - consolidates processor, pipeline, lifecycle)
CREATE: crates/riptide-api/src/adapters/websocket_transport.rs (~400 LOC - from websocket.rs)
CREATE: crates/riptide-api/src/adapters/sse_transport.rs (~350 LOC - from sse.rs)
UPDATE: crates/riptide-api/src/handlers/streaming.rs (300 → 25 LOC, -275)
DELETE: crates/riptide-api/src/streaming/processor.rs (634 LOC)
DELETE: crates/riptide-api/src/streaming/pipeline.rs (628 LOC)
DELETE: crates/riptide-api/src/streaming/lifecycle.rs (622 LOC)
DELETE: crates/riptide-api/src/streaming/response_helpers.rs (924 LOC)
MOVE: crates/riptide-api/src/streaming/buffer.rs → crates/riptide-reliability/src/buffer.rs
MOVE: crates/riptide-api/src/streaming/config.rs → crates/riptide-config/src/streaming.rs
MOVE: crates/riptide-api/src/streaming/error.rs → crates/riptide-types/src/errors/streaming.rs
DELETE: crates/riptide-api/src/streaming/mod.rs (546 LOC)
DELETE: crates/riptide-api/src/streaming/ndjson/* (725 LOC - move to facade)
```

**LOC Impact (Sprint 4.3):**
- **-3,500 LOC** deleted from API layer
- **+2,300 LOC** added to facades
- **+750 LOC** added to adapters/ports
- **-450 LOC net reduction**
- **Critical:** 5,427 LOC architectural violation RESOLVED

### Validation (Sprint 4.3)

```bash
# No streaming directory in API
[ ! -d crates/riptide-api/src/streaming ] && echo "PASS: streaming/ deleted" || echo "FAIL: streaming/ still exists"

# Streaming ports defined
grep "StreamingTransport" crates/riptide-types/src/ports/streaming.rs && echo "PASS" || echo "FAIL"

# Streaming facade exists
[ -f crates/riptide-facade/src/facades/streaming.rs ] && echo "PASS" || echo "FAIL"

# Tests pass
cargo test -p riptide-facade --test streaming_tests
cargo test -p riptide-api --test websocket_integration_tests
```

---

## Sprint 4.4: Resource Manager Consolidation (Week 8, Days 4-5)

**Duration:** 2 days
**Priority:** MEDIUM (1,845 LOC partially addressed)

### Current State

**Problem:** `crates/riptide-api/src/resource_manager/` has business logic

**Files:**
- mod.rs (653 LOC) - orchestration logic
- performance.rs (384 LOC) - business metrics
- rate_limiter.rs (374 LOC) - should use port
- wasm_manager.rs (321 LOC) - should use WasmPoolPort
- guards.rs (237 LOC) - RAII guards (OK)
- metrics.rs (191 LOC) - business metrics
- errors.rs (84 LOC) - domain errors (OK)
- memory_manager.rs (DELETED in Phase 0)

### Solution: Migrate to Facades and Ports

**Step 1: Move Orchestration to Facade**

```rust
// crates/riptide-facade/src/facades/resource.rs (NEW)
pub struct ResourceFacade {
    wasm_pool: Arc<dyn WasmPool>,
    rate_limiter: Arc<dyn RateLimiter>,
    metrics: Arc<BusinessMetrics>,
}

impl ResourceFacade {
    pub async fn acquire_wasm_slot(&self, authz_ctx: &AuthorizationContext) -> Result<WasmSlot> {
        // Authorization
        self.authorize(authz_ctx)?;

        // Rate limiting
        self.rate_limiter.check_quota(&authz_ctx.tenant_id).await?;

        // Acquire slot (orchestration from mod.rs)
        let slot = self.wasm_pool.acquire().await?;

        // Metrics
        self.metrics.record_wasm_slot_acquired();

        Ok(slot)
    }
}
```

**Step 2: Define RateLimiter Port**

```rust
// crates/riptide-types/src/ports/rate_limit.rs (NEW)
#[async_trait]
pub trait RateLimiter: Send + Sync {
    async fn check_quota(&self, tenant_id: &str) -> Result<()>;
    async fn consume(&self, tenant_id: &str, amount: usize) -> Result<()>;
    async fn reset(&self, tenant_id: &str) -> Result<()>;
}
```

**Step 3: Implement Redis Rate Limiter**

```rust
// crates/riptide-cache/src/adapters/redis_rate_limiter.rs (NEW)
pub struct RedisRateLimiter {
    redis: Arc<RedisManager>,
}

#[async_trait]
impl RateLimiter for RedisRateLimiter {
    async fn check_quota(&self, tenant_id: &str) -> Result<()> {
        let key = format!("ratelimit:v1:{}", tenant_id);
        let count: usize = self.redis.get("ratelimit", &key).await?
            .and_then(|v| String::from_utf8(v).ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        if count >= MAX_REQUESTS_PER_MINUTE {
            Err(RiptideError::RateLimitExceeded { tenant_id: tenant_id.to_string() })
        } else {
            Ok(())
        }
    }

    async fn consume(&self, tenant_id: &str, amount: usize) -> Result<()> {
        let key = format!("ratelimit:v1:{}", tenant_id);
        self.redis.incr("ratelimit", &key, amount, Duration::from_secs(60)).await?;
        Ok(())
    }
}
```

### Files Modified (Sprint 4.4)

```
CREATE: crates/riptide-types/src/ports/rate_limit.rs (~150 LOC)
CREATE: crates/riptide-facade/src/facades/resource.rs (~500 LOC - from mod.rs)
CREATE: crates/riptide-cache/src/adapters/redis_rate_limiter.rs (~200 LOC - from rate_limiter.rs)
MOVE: crates/riptide-api/src/resource_manager/performance.rs → crates/riptide-facade/src/metrics/performance.rs
UPDATE: crates/riptide-pool/src/lib.rs (implement WasmPool trait)
DELETE: crates/riptide-api/src/resource_manager/mod.rs (653 LOC)
DELETE: crates/riptide-api/src/resource_manager/rate_limiter.rs (374 LOC)
DELETE: crates/riptide-api/src/resource_manager/wasm_manager.rs (321 LOC)
KEEP: crates/riptide-api/src/resource_manager/guards.rs (RAII guards are fine)
```

**LOC Impact (Sprint 4.4):**
- **-1,500 LOC** deleted from API layer
- **+850 LOC** added to facades/adapters
- **-650 LOC net reduction**

### Validation (Sprint 4.4)

```bash
# resource_manager mostly gone
[ ! -f crates/riptide-api/src/resource_manager/mod.rs ] && echo "PASS" || echo "FAIL"

# RateLimiter port exists
grep "trait RateLimiter" crates/riptide-types/src/ports/rate_limit.rs && echo "PASS" || echo "FAIL"

# Tests pass
cargo test -p riptide-facade --test resource_tests
```

---

## Sprint 4.5: Metrics System Split (Week 8, Day 5)

**Duration:** 1 day
**Priority:** HIGH (1,670 LOC mixed concerns)

### Current State

**Problem:** `crates/riptide-api/src/metrics.rs` (1,670 LOC) mixes:
- Business metrics (extraction counts, profile creations, etc.)
- Transport metrics (HTTP request counts, latency, etc.)
- Infrastructure metrics (Prometheus exporter)

### Solution: Split Business vs Transport Metrics

**Business Metrics → Facades:**
```rust
// crates/riptide-facade/src/metrics/business.rs (NEW)
pub struct BusinessMetrics {
    profiles_created: Counter,
    extractions_completed: Counter,
    extractions_duration: Histogram,
    searches_performed: Counter,
    pdf_pages_processed: Counter,
}

impl BusinessMetrics {
    pub fn record_extraction_completed(&self, duration: Duration) {
        self.extractions_completed.inc();
        self.extractions_duration.observe(duration.as_secs_f64());
    }
}
```

**Transport Metrics → Keep in API:**
```rust
// crates/riptide-api/src/metrics.rs (REFACTORED - ~500 LOC)
pub struct TransportMetrics {
    http_requests_total: Counter,
    http_request_duration: Histogram,
    http_response_size: Histogram,
    websocket_connections: Gauge,
    sse_connections: Gauge,
}

impl TransportMetrics {
    pub fn record_http_request(&self, method: &str, path: &str, status: u16, duration: Duration) {
        self.http_requests_total
            .with_label_values(&[method, path, &status.to_string()])
            .inc();
        self.http_request_duration.observe(duration.as_secs_f64());
    }
}
```

**Prometheus Exporter → Keep in API:**
```rust
// crates/riptide-api/src/metrics.rs (continued)
pub fn prometheus_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    Response::builder()
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Body::from(buffer))
        .unwrap()
}
```

### Files Modified (Sprint 4.5)

```
CREATE: crates/riptide-facade/src/metrics/business.rs (~800 LOC - business metrics from metrics.rs)
UPDATE: crates/riptide-api/src/metrics.rs (1,670 → 500 LOC, keep transport metrics + Prometheus exporter)
UPDATE: All facades to use BusinessMetrics
UPDATE: crates/riptide-api/src/composition.rs (inject BusinessMetrics into facades)
```

**LOC Impact (Sprint 4.5):**
- **-1,170 LOC** deleted/moved from API
- **+800 LOC** added to facades
- **-370 LOC net reduction**

### Validation (Sprint 4.5)

```bash
# Business metrics in facade
grep "BusinessMetrics" crates/riptide-facade/src/metrics/business.rs && echo "PASS" || echo "FAIL"

# Transport metrics in API
wc -l crates/riptide-api/src/metrics.rs | awk '{if ($1 < 600) print "PASS"; else print "FAIL"}'

# All metrics still exported
curl http://localhost:8080/metrics | grep "riptide_" && echo "PASS" || echo "FAIL"
```

---

## Sprint 4.6: Browser Crate Consolidation (1 day) ⚠️ CRITICAL

**Priority:** HIGH (Infrastructure Simplification)
**Source:** WORKSPACE_CRATE_ANALYSIS.md §4 - Critical Violation #4

### Problem

**3 browser crates with overlapping responsibilities (11,482 LOC total):**
- `riptide-browser-abstraction` (16 files, 4,208 LOC) - Supposed abstraction, still couples to `spider_chrome`
- `riptide-browser` (7 files, 4,403 LOC) - Browser pool management
- `riptide-headless` (9 files, 2,871 LOC) - Headless browser HTTP API

**Violations:**
- Abstraction leak: browser-abstraction depends on concrete CDP (Chromiumoxide)
- Duplicate browser pool logic
- Unclear separation of concerns
- Cannot swap browser implementations

### Solution

**Merge all three into single `riptide-browser` crate with modules:**

```rust
// NEW unified structure
crates/riptide-browser/
├── src/
│   ├── abstraction/     // Traits only (NO concrete CDP types)
│   │   ├── driver.rs    // BrowserDriver trait
│   │   ├── session.rs   // BrowserSession trait
│   │   └── mod.rs
│   ├── pool/            // Pool management
│   │   ├── manager.rs   // From old riptide-browser
│   │   └── mod.rs
│   ├── cdp/             // CDP-specific implementation
│   │   ├── chromium.rs  // Implements BrowserDriver
│   │   └── mod.rs
│   ├── http/            // HTTP API for headless
│   │   ├── server.rs    // From old riptide-headless
│   │   └── mod.rs
│   └── lib.rs           // Re-export modules
```

**Implementation:**
```bash
# Day 1: Merge crates
mkdir -p crates/riptide-browser/src/{abstraction,pool,cdp,http}

# Move abstraction (extract traits, remove concrete types)
mv crates/riptide-browser-abstraction/src/traits.rs \
   crates/riptide-browser/src/abstraction/

# Move pool
mv crates/riptide-browser/src/pool.rs \
   crates/riptide-browser/src/pool/manager.rs

# Move CDP implementation
mv crates/riptide-browser-abstraction/src/chromium.rs \
   crates/riptide-browser/src/cdp/

# Move headless HTTP API
mv crates/riptide-headless/src/* \
   crates/riptide-browser/src/http/

# Remove old crates
rm -rf crates/riptide-browser-abstraction
rm -rf crates/riptide-headless

# Update all imports
rg "use riptide_browser_abstraction" crates/ | cut -d: -f1 | sort -u | \
  xargs sed -i 's/riptide_browser_abstraction/riptide_browser::abstraction/g'
rg "use riptide_headless" crates/ | cut -d: -f1 | sort -u | \
  xargs sed -i 's/riptide_headless/riptide_browser::http/g'
```

**Files Modified:**
```
CREATE:  crates/riptide-browser/src/abstraction/ (traits only)
CREATE:  crates/riptide-browser/src/pool/ (from old browser)
CREATE:  crates/riptide-browser/src/cdp/ (from browser-abstraction)
CREATE:  crates/riptide-browser/src/http/ (from headless)
UPDATE:  crates/riptide-browser/Cargo.toml (merge dependencies)
UPDATE:  Cargo.toml (remove browser-abstraction, headless from workspace)
UPDATE:  All files using old crates → riptide_browser::*
DELETE:  crates/riptide-browser-abstraction/ (entire crate)
DELETE:  crates/riptide-headless/ (entire crate)
```

**Validation:**
```bash
# 1. Verify abstraction has no concrete types
rg "chromiumoxide|spider_chrome" crates/riptide-browser/src/abstraction/ \
  && echo "❌ FAIL: Concrete types in abstraction" \
  || echo "✅ PASS: Clean abstraction"

# 2. Verify single browser crate
find crates -name "*browser*" -type d | wc -l  # Expected: 1

# 3. All tests pass
cargo test -p riptide-browser

# 4. Build succeeds
cargo build --workspace
```

**Success Criteria:**
- ✅ Single riptide-browser crate (3 → 1)
- ✅ True trait abstraction (no concrete CDP types)
- ✅ Clear module separation (abstraction/pool/cdp/http)
- ✅ All browser features still work
- ✅ Tests pass

**Impact:**
- **Crates Removed:** -2 (browser-abstraction, headless)
- **LOC Saved:** ~1,500 LOC (deduplication)
- **Architecture:** Clean abstraction enables browser swapping

**References:**
- WORKSPACE_CRATE_ANALYSIS.md §4 - Critical Violation #4
- WORKSPACE_CRATE_ANALYSIS.md §5 - Consolidation Phase 2

---

## Sprint 4.7: Pool Abstraction Unification (0.5 days)

**Priority:** MEDIUM (Code Reuse)
**Source:** WORKSPACE_CRATE_ANALYSIS.md §4 - Minor Violation #9

### Problem

**Duplicate pool implementations in 3 crates:**
- `riptide-pool` - Generic pool management (10,086 LOC)
- `riptide-browser/src/pool/` - Browser-specific pool
- `riptide-intelligence` - LLM client pool

**Issue:** Similar pooling logic duplicated (~1,000 LOC wasted)

### Solution

**Extract common Pool<T> trait in domain layer:**

```rust
// crates/riptide-types/src/ports/pool.rs (NEW)
#[async_trait]
pub trait Pool<T>: Send + Sync {
    async fn acquire(&self) -> Result<PooledResource<T>>;
    async fn release(&self, resource: T) -> Result<()>;
    async fn size(&self) -> usize;
    async fn available(&self) -> usize;
    async fn health_check(&self) -> PoolHealth;
}

pub struct PooledResource<T> {
    resource: T,
    pool: Arc<dyn Pool<T>>,
}

pub struct PoolHealth {
    pub total: usize,
    pub available: usize,
    pub in_use: usize,
    pub failed: usize,
}
```

**Refactor implementations to use trait:**
```rust
// riptide-browser implements Pool<BrowserSession>
impl Pool<BrowserSession> for BrowserPool { ... }

// riptide-intelligence implements Pool<LlmClient>
impl Pool<LlmClient> for LlmClientPool { ... }

// riptide-pool provides generic Pool<T> implementation
impl<T> Pool<T> for GenericPool<T> { ... }
```

**Files Modified:**
```
CREATE:  crates/riptide-types/src/ports/pool.rs (~150 LOC)
UPDATE:  crates/riptide-browser/src/pool/manager.rs (impl Pool<BrowserSession>)
UPDATE:  crates/riptide-intelligence/src/llm/pool.rs (impl Pool<LlmClient>)
UPDATE:  crates/riptide-pool/src/generic.rs (impl Pool<T>)
REFACTOR: Extract common logic to shared utilities
```

**Validation:**
```bash
# All pools implement the trait
rg "impl.*Pool<" crates/ | wc -l  # Expected: 3+

# Tests pass
cargo test -p riptide-browser
cargo test -p riptide-intelligence
cargo test -p riptide-pool
```

**Success Criteria:**
- ✅ Pool<T> trait defined in riptide-types
- ✅ All pools implement Pool<T>
- ✅ Consistent pool interface across workspace
- ✅ Tests pass

**Impact:**
- **LOC Saved:** ~1,000 LOC (common logic extracted)
- **Consistency:** Uniform pool interface
- **Reusability:** Easy to add new pooled resources

**References:**
- WORKSPACE_CRATE_ANALYSIS.md §4 - Minor Violation #9
- WORKSPACE_CRATE_ANALYSIS.md §5 - Consolidation Phase 4

---

## Success Criteria for Phase 4

### Quantitative Metrics

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| **HTTP clients** | 6 | **1** | `rg "reqwest::Client::new" crates/ \| wc -l` |
| **Redis dependencies** | 6 | **≤2** | `find crates -name "Cargo.toml" -exec grep -l redis {} \; \| wc -l` |
| **streaming/ LOC in API** | 5,427 | **0** | `wc -l crates/riptide-api/src/streaming/*.rs 2>/dev/null \| tail -1` |
| **resource_manager/ LOC** | 2,832 | **<500** | `find crates/riptide-api/src/resource_manager -name "*.rs" -exec wc -l {} + \| tail -1` |
| **metrics.rs LOC** | 1,670 | **<600** | `wc -l crates/riptide-api/src/metrics.rs` |

### Qualitative Checks

- [ ] All HTTP via ReliableHttpClient (STRICT)
- [ ] Circuit breakers configured per endpoint type
- [ ] Redis via single manager with versioned keys
- [ ] Streaming system uses ports/adapters (STRICT)
- [ ] Resource manager logic in facades
- [ ] Business metrics separated from transport metrics
- [ ] Graceful degradation tested

### Validation Script

```bash
#!/bin/bash
# scripts/validate_phase4.sh

FAIL_COUNT=0

echo "🔍 Phase 4 Validation: Infrastructure Consolidation"
echo "===================================================="

# 1. HTTP client consolidation
echo ""
echo "🌐 Checking HTTP client usage..."
if rg "reqwest::Client::new\|reqwest::Client::builder" crates/riptide-{facade,api,spider,fetch} 2>/dev/null; then
    echo "❌ FAIL: Direct reqwest found"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: All HTTP via ReliableHttpClient"
fi

# 2. Redis consolidation
echo ""
echo "💾 Checking Redis dependencies..."
REDIS_COUNT=$(find crates -name "Cargo.toml" -exec grep -l "redis" {} \; | wc -l)
if [ "$REDIS_COUNT" -gt 2 ]; then
    echo "❌ FAIL: Redis in $REDIS_COUNT crates (max: 2)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: Redis in $REDIS_COUNT crates"
fi

# 3. Streaming system removed
echo ""
echo "🌊 Checking streaming system migration..."
if [ -d crates/riptide-api/src/streaming ]; then
    echo "❌ FAIL: streaming/ still exists in API layer"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: Streaming migrated to facades"
fi

# 4. Resource manager consolidated
echo ""
echo "📦 Checking resource manager..."
RM_LOC=$(find crates/riptide-api/src/resource_manager -name "*.rs" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
if [ "$RM_LOC" -gt 500 ]; then
    echo "❌ FAIL: resource_manager has $RM_LOC LOC (max: 500)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: resource_manager consolidated ($RM_LOC LOC)"
fi

# 5. Metrics split
echo ""
echo "📊 Checking metrics split..."
METRICS_LOC=$(wc -l crates/riptide-api/src/metrics.rs 2>/dev/null | awk '{print $1}')
if [ "$METRICS_LOC" -gt 600 ]; then
    echo "❌ FAIL: metrics.rs has $METRICS_LOC LOC (max: 600)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: Metrics split ($METRICS_LOC LOC)"
fi

# Summary
echo ""
echo "===================================================="
if [ $FAIL_COUNT -eq 0 ]; then
    echo "✅ ALL CHECKS PASSED - Phase 4 Complete!"
    exit 0
else
    echo "❌ $FAIL_COUNT CHECKS FAILED"
    exit 1
fi
```

---

## LOC Impact Summary

| Sprint | Deleted | Added | Net |
|--------|---------|-------|-----|
| 4.1 (HTTP) | -200 | +300 | +100 |
| 4.2 (Redis) | 0 | 0 | 0 (validated in Phase 0) |
| 4.3 (Streaming) | -3,500 | +3,050 | -450 |
| 4.4 (Resource Mgr) | -1,500 | +850 | -650 |
| 4.5 (Metrics) | -1,170 | +800 | -370 |
| **Total** | **-6,370** | **+4,000** | **-2,370** |

**Enhanced Coverage:**
- **Original Plan:** -800 LOC
- **Enhanced Plan:** -2,370 LOC
- **Improvement:** +196% more cleanup

---

## Dependencies and Risks

### Dependencies

**Requires Phase 3 Complete:**
- All handlers <50 LOC
- All facades created

**Requires Phase 1 & 2 Complete:**
- Port traits defined
- Adapters working

### Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Streaming refactoring breaks real-time features | CRITICAL | Comprehensive WebSocket/SSE integration tests |
| Circuit breakers too aggressive | HIGH | Tune thresholds per endpoint type |
| Redis consolidation causes cache misses | MEDIUM | Versioned keys, gradual migration |

---

## Timeline

**Week 7:**
- Days 1-2: Sprint 4.1 (HTTP consolidation)
- Days 3-4: Sprint 4.2 (Redis validation)
- Day 5: Sprint 4.3 Part 1 (Streaming - ports & facade start)

**Week 8:**
- Days 1-3: Sprint 4.3 Part 2 (Streaming - complete migration)
- Days 4-5: Sprint 4.4 (Resource manager)
- Day 5: Sprint 4.5 (Metrics split)

**Total:** 2 weeks (was 1 week in original plan)

---

## Related Documents

- [PHASE_3_HANDLER_REFACTORING_ROADMAP.md](./PHASE_3_HANDLER_REFACTORING_ROADMAP.md) (prerequisite)
- [PHASE_5_VALIDATION_ROADMAP.md](./PHASE_5_VALIDATION_ROADMAP.md) (follows this)
- [API_CRATE_COVERAGE_ANALYSIS.md](../architecture/API_CRATE_COVERAGE_ANALYSIS.md) (source of Sprint 4.3-4.5)

---

**Document Version:** 2.1
**Status:** ✅ Ready for Implementation
**Next Review:** After Sprint 4.3 completion
