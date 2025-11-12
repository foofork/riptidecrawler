# 🚪 RipTide API - HTTP Entry Point & Composition Root

> **Layer**: API (Entry Points & Composition Root)
> **Role**: HTTP API server, dependency injection, request/response handling
> **Architecture**: Hexagonal (Ports & Adapters)

The REST API server for RipTide web crawler. This is where all application dependencies are wired together through the **ApplicationContext** composition root.

---

## 📋 Quick Overview

RipTide API is the HTTP entry point that exposes 59+ REST endpoints for web crawling, content extraction, and system monitoring. It follows hexagonal architecture principles with a clean separation between handlers (API layer), facades (application layer), and domain logic.

**Key Responsibilities:**
- ✅ HTTP request handling via Axum framework
- ✅ Dependency injection through ApplicationContext
- ✅ Request validation and authentication
- ✅ Rate limiting and resource management
- ✅ Metrics collection and health monitoring

---

## 🎯 Composition Root Pattern

### What is ApplicationContext?

The `ApplicationContext` struct is the **Composition Root** - the single place where all application dependencies are created and wired together. This pattern follows Dependency Injection principles and makes testing easier.

```rust
// ApplicationContext contains ALL dependencies
pub struct ApplicationContext {
    pub http_client: Client,
    pub cache: Arc<Mutex<CacheManager>>,
    pub extractor: Arc<UnifiedExtractor>,
    pub resource_manager: Arc<ResourceManager>,
    pub session_manager: Arc<SessionManager>,
    pub extraction_facade: Arc<ExtractionFacade>,
    pub scraper_facade: Arc<ScraperFacade>,
    // ... 20+ more dependencies
}
```

### Initialization Flow

```rust
// 1. Load configuration from environment
let config = AppConfig::default();
let health_checker = Arc::new(HealthChecker::new());

// 2. Initialize ApplicationContext (composition root)
let app_context = ApplicationContext::new(config, health_checker).await?;

// 3. Build router with handlers
let app = Router::new()
    .route("/crawl", post(handlers::crawl))
    .route("/extract", post(handlers::extract))
    .with_state(app_context); // Inject dependencies into all handlers

// 4. Start server
axum::serve(listener, app).await?;
```

### Why This Matters

**Before (God Object Anti-Pattern):**
```rust
// ❌ Bad: Handlers directly access global state
async fn extract(State(state): State<Arc<AppState>>) {
    state.http_client.get(...);  // Tight coupling
    state.cache.lock().await;     // Hard to test
    state.extractor.extract(...);  // No abstraction
}
```

**After (Composition Root Pattern):**
```rust
// ✅ Good: Handlers receive dependencies through state
async fn extract(State(ctx): State<ApplicationContext>) {
    // Use high-level facades instead of low-level components
    ctx.extraction_facade.extract_content(url).await?;
}
```

**Benefits:**
- **Testability**: Easy to mock dependencies for unit tests
- **Flexibility**: Swap implementations without changing handlers
- **Clarity**: Single source of truth for dependency graph
- **Type Safety**: Compiler enforces correct wiring

---

## 🔌 API Endpoints

### Core Endpoints (18 routes)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/healthz` | GET | Basic health check |
| `/api/health/detailed` | GET | Detailed health with components |
| `/health/:component` | GET | Component-specific health |
| `/metrics` | GET | Prometheus metrics |
| `/crawl` | POST | Start web crawl |
| `/crawl/stream` | POST | Streaming crawl (NDJSON) |
| `/extract` | POST | Extract content from URL |
| `/search` | GET | Web search (feature-gated) |
| `/deepsearch` | POST | Deep search with crawling |
| `/render` | POST | Headless browser rendering |
| `/spider/crawl` | POST | Deep crawl with frontier |
| `/spider/status` | POST | Spider crawl status |
| `/spider/control` | POST | Control spider crawl |
| `/sessions` | POST/GET | Session management |
| `/resources/status` | GET | Resource monitoring |
| `/monitoring/health-score` | GET | System health score |
| `/api/profiling/memory` | GET | Memory profiling |
| `/api/telemetry/status` | GET | Telemetry status |

### Feature-Gated Endpoints

| Feature | Endpoints | Count |
|---------|-----------|-------|
| `browser` | Browser pool, session management | 4 |
| `workers` | Job queue, scheduling | 9 |
| `search` | Search integration | 3 |
| `spider` | Deep crawling | 3 |
| `persistence` | Multi-tenancy, admin | 13 |

**Total**: 59+ production-ready endpoints

---

## 🚀 Getting Started

### Prerequisites

```bash
# Required
rust >= 1.75
redis-server  # For caching and sessions

# Optional (for full features)
chromium      # For headless rendering
docker        # For containerized deployment
```

### Quick Start (3 steps)

```bash
# 1. Start Redis
docker run -d -p 6379:6379 redis

# 2. Set environment variables
export REDIS_URL="redis://localhost:6379"
export WASM_EXTRACTOR_PATH="./target/wasm32-wasip2/release/riptide_extractor_wasm.wasm"

# 3. Run the server
cargo run --release -p riptide-api -- --bind 0.0.0.0:8080
```

### First Request

```bash
# Health check
curl http://localhost:8080/healthz

# Extract content
curl -X POST http://localhost:8080/extract \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com",
    "strategy": "auto"
  }'

# Crawl website
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com",
    "max_depth": 2,
    "respect_robots": true
  }'
```

---

## 📦 Request/Response Models

### DTOs (Data Transfer Objects)

All request/response models are in `/src/dto/`:

```rust
// Request DTO
#[derive(Deserialize)]
pub struct ExtractRequest {
    pub url: String,
    pub strategy: Option<String>,
    pub quality_threshold: Option<f64>,
}

// Response DTO
#[derive(Serialize)]
pub struct ExtractResponse {
    pub content: String,
    pub quality_score: f64,
    pub strategy_used: String,
    pub metadata: ExtractionMetadata,
}
```

### Validation

All DTOs implement validation:

```rust
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CrawlRequest {
    #[validate(url)]
    pub url: String,

    #[validate(range(min = 1, max = 10))]
    pub max_depth: Option<u32>,

    #[validate(range(min = 1, max = 1000))]
    pub max_pages: Option<u32>,
}

// Validated in middleware
async fn request_validation_middleware(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, StatusCode> {
    // Extract and validate DTO
    // Return 400 Bad Request if invalid
}
```

### Error Responses

Consistent error format across all endpoints:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid URL format",
    "status": 400,
    "details": {
      "field": "url",
      "reason": "Must be a valid HTTP/HTTPS URL"
    }
  }
}
```

---

## 🔧 Configuration

### Environment Variables

#### Core Settings

```bash
# Redis (required)
REDIS_URL="redis://localhost:6379"

# WASM Extractor (optional)
WASM_EXTRACTOR_PATH="./target/wasm32-wasip2/release/riptide_extractor_wasm.wasm"

# Server Binding
BIND_ADDRESS="0.0.0.0:8080"

# Logging
RUST_LOG="info,riptide_api=debug"
```

#### Resource Limits

```bash
# Browser Pool
RIPTIDE_HEADLESS_POOL_SIZE=3
RIPTIDE_MAX_CONCURRENT_RENDERS=10

# PDF Processing
RIPTIDE_MAX_CONCURRENT_PDF=2

# Rate Limiting
RIPTIDE_RATE_LIMIT_RPS=1.5          # Requests per second per host
RIPTIDE_RATE_LIMIT_JITTER=0.1       # Random jitter (0.0-1.0)

# Memory
RIPTIDE_MEMORY_LIMIT_MB=2048         # Global memory limit

# Timeouts
RIPTIDE_RENDER_TIMEOUT=3             # Headless render timeout (seconds)
```

#### Feature Flags

```bash
# Search Backend
SEARCH_BACKEND=serper                 # serper, searxng, or none
SERPER_API_KEY=your_key_here         # Required for Serper

# Spider
SPIDER_ENABLE=true
SPIDER_MAX_DEPTH=5
SPIDER_MAX_PAGES=1000

# Workers
WORKER_POOL_SIZE=4
WORKER_ENABLE_SCHEDULER=true

# Telemetry
OTEL_ENDPOINT="http://jaeger:4317"   # OpenTelemetry collector
```

### Configuration File

Optional YAML configuration in `config/application/riptide.yml`:

```yaml
resources:
  max_concurrent_renders: 10
  max_concurrent_pdf: 2
  global_timeout_secs: 30

performance:
  render_timeout_secs: 3
  pdf_timeout_secs: 10

rate_limiting:
  enabled: true
  requests_per_second_per_host: 1.5
  jitter_factor: 0.1
  burst_capacity_per_host: 3

headless:
  max_pool_size: 3
  min_pool_size: 1
  idle_timeout_secs: 300

memory:
  global_memory_limit_mb: 2048
  pressure_threshold: 0.85
  auto_gc: true
```

---

## 🧪 Testing

### Running Tests

```bash
# All tests
cargo test -p riptide-api

# With output
cargo test -p riptide-api -- --nocapture

# Specific test module
cargo test -p riptide-api handlers::crawl

# Integration tests only
cargo test -p riptide-api --test '*'

# With all features
cargo test -p riptide-api --features full
```

### Integration Tests

Located in `/tests/integration/`:

```rust
use riptide_api::ApplicationContext;
use riptide_api::context::AppConfig;

#[tokio::test]
async fn test_extract_endpoint() {
    // 1. Create test context
    let ctx = ApplicationContext::new_test_minimal().await;

    // 2. Build router with test state
    let app = Router::new()
        .route("/extract", post(handlers::extract))
        .with_state(ctx);

    // 3. Make request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/extract")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"url":"https://example.com"}"#))
                .unwrap()
        )
        .await
        .unwrap();

    // 4. Assert
    assert_eq!(response.status(), StatusCode::OK);
}
```

### Mock Context for Testing

```rust
// Create minimal test context
let ctx = ApplicationContext::new_test_minimal().await;

// Or create with custom config
let config = AppConfig {
    redis_url: "redis://localhost:6379".to_string(),
    max_concurrency: 5,
    ..Default::default()
};
let ctx = ApplicationContext::new(config, health_checker).await?;
```

---

## 🏗️ Extension Points

### Adding a New Endpoint

**1. Create Handler**

```rust
// src/handlers/my_feature.rs
use axum::{extract::State, Json};
use crate::context::ApplicationContext;
use crate::dto::{MyRequest, MyResponse};

pub async fn my_handler(
    State(ctx): State<ApplicationContext>,
    Json(req): Json<MyRequest>,
) -> Result<Json<MyResponse>, StatusCode> {
    // Use facades from context
    let result = ctx.extraction_facade
        .extract_content(&req.url)
        .await?;

    Ok(Json(MyResponse { result }))
}
```

**2. Register Route**

```rust
// src/main.rs
let app = Router::new()
    .route("/my-feature", post(handlers::my_handler))
    .with_state(app_context);
```

**3. Add DTO Models**

```rust
// src/dto/my_feature.rs
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct MyRequest {
    #[validate(url)]
    pub url: String,
}

#[derive(Serialize)]
pub struct MyResponse {
    pub result: String,
}
```

### Implementing a New Facade

If you need a new facade (application service):

```rust
// In riptide-facade crate
pub struct MyFacade {
    extractor: Arc<UnifiedExtractor>,
    cache: Arc<CacheManager>,
}

impl MyFacade {
    pub async fn new(config: RiptideConfig) -> Result<Self> {
        Ok(Self {
            extractor: Arc::new(UnifiedExtractor::new(None).await?),
            cache: Arc::new(CacheManager::new(&config.redis_url).await?),
        })
    }

    pub async fn my_operation(&self, input: String) -> Result<String> {
        // Business logic here
        Ok(input)
    }
}
```

Then wire it into ApplicationContext:

```rust
// src/context.rs
pub struct ApplicationContext {
    // ... existing fields
    pub my_facade: Arc<MyFacade>,
}

impl ApplicationContext {
    pub async fn new(...) -> Result<Self> {
        // ... existing initialization

        let my_facade = Arc::new(
            MyFacade::new(facade_config.clone()).await?
        );

        Ok(Self {
            // ... existing fields
            my_facade,
        })
    }
}
```

### Wiring New Dependencies

All dependency wiring happens in `ApplicationContext::new()`:

```rust
impl ApplicationContext {
    pub async fn new(config: AppConfig, health_checker: Arc<HealthChecker>) -> Result<Self> {
        // 1. Initialize infrastructure
        let http_client = Client::new();
        let cache = CacheManager::new(&config.redis_url).await?;

        // 2. Initialize domain services
        let extractor = UnifiedExtractor::new(Some(&config.wasm_path)).await?;

        // 3. Initialize facades (application layer)
        let extraction_facade = ExtractionFacade::new(facade_config).await?;

        // 4. Wire everything together
        Ok(Self {
            http_client,
            cache: Arc::new(Mutex::new(cache)),
            extractor: Arc::new(extractor),
            extraction_facade: Arc::new(extraction_facade),
            // ... all other dependencies
        })
    }
}
```

---

## 🐳 Deployment

### Docker

```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p riptide-api --features full

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    chromium \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/riptide-api /usr/local/bin/
EXPOSE 8080
CMD ["riptide-api", "--bind", "0.0.0.0:8080"]
```

```bash
# Build
docker build -t riptide-api:latest .

# Run
docker run -d \
  -p 8080:8080 \
  -e REDIS_URL=redis://redis:6379 \
  -e RUST_LOG=info \
  --name riptide-api \
  riptide-api:latest
```

### Docker Compose

```yaml
version: '3.8'

services:
  api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - REDIS_URL=redis://redis:6379
      - RUST_LOG=info
      - RIPTIDE_MAX_CONCURRENT_RENDERS=10
    depends_on:
      - redis
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/healthz"]
      interval: 30s
      timeout: 10s
      retries: 3

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data

volumes:
  redis-data:
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: riptide-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: riptide-api
  template:
    metadata:
      labels:
        app: riptide-api
    spec:
      containers:
      - name: riptide-api
        image: riptide-api:latest
        ports:
        - containerPort: 8080
        env:
        - name: REDIS_URL
          value: "redis://redis-service:6379"
        - name: RIPTIDE_MAX_CONCURRENT_RENDERS
          value: "10"
        resources:
          limits:
            memory: "2Gi"
            cpu: "1000m"
          requests:
            memory: "512Mi"
            cpu: "250m"
        livenessProbe:
          httpGet:
            path: /healthz
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /healthz
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
```

---

## 📊 Monitoring

### Prometheus Metrics

```bash
# Scrape metrics
curl http://localhost:8080/metrics

# Key metrics
http_requests_total{method="POST",endpoint="/extract",status="200"} 1234
http_request_duration_seconds{method="POST",endpoint="/extract"} 0.123
extraction_quality_score{strategy="wasm"} 0.95
cache_hit_rate 0.87
browser_pool_size 3
memory_allocated_bytes 524288000
```

### Health Checks

```bash
# Basic health
curl http://localhost:8080/healthz

# Detailed health
curl http://localhost:8080/api/health/detailed

# Component health
curl http://localhost:8080/health/redis
curl http://localhost:8080/health/browser
curl http://localhost:8080/health/workers
```

### OpenTelemetry Integration

```bash
# Enable telemetry
export OTEL_ENDPOINT="http://jaeger:4317"
cargo run --release -p riptide-api --features telemetry

# View traces
curl http://localhost:8080/api/telemetry/traces
curl http://localhost:8080/api/telemetry/traces/{trace_id}
```

---

## 🔍 Troubleshooting

### Common Issues

**Redis Connection Failed**
```bash
# Check Redis is running
redis-cli ping
# Should return: PONG

# Check connection string
echo $REDIS_URL
```

**Browser Pool Exhausted**
```bash
# Increase pool size
export RIPTIDE_HEADLESS_POOL_SIZE=5

# Check pool status
curl http://localhost:8080/resources/browser-pool
```

**High Memory Usage**
```bash
# Enable memory profiling
cargo run --features jemalloc

# Check memory metrics
curl http://localhost:8080/resources/memory
curl http://localhost:8080/api/profiling/memory
```

**Rate Limiting Too Aggressive**
```bash
# Increase rate limit
export RIPTIDE_RATE_LIMIT_RPS=5.0

# Or disable temporarily
export RIPTIDE_RATE_LIMITING_ENABLED=false
```

### Debug Mode

```bash
# Enable verbose logging
export RUST_LOG=riptide_api=debug,riptide_facade=debug,riptide_core=debug

# With backtraces
export RUST_BACKTRACE=full

# Run
cargo run -p riptide-api
```

---

## 🏛️ Architecture Notes

### Hexagonal Architecture

```
┌─────────────────────────────────────────┐
│         API Layer (riptide-api)         │
│  ┌─────────────────────────────────┐   │
│  │   HTTP Handlers (Adapters)      │   │
│  │   - Request validation          │   │
│  │   - DTO transformation          │   │
│  │   - Error handling              │   │
│  └──────────┬──────────────────────┘   │
│             │ Uses                      │
│  ┌──────────▼──────────────────────┐   │
│  │  Application Context (DI Root)  │   │
│  │  - Wires all dependencies       │   │
│  │  - Provides facades to handlers │   │
│  └──────────┬──────────────────────┘   │
└─────────────┼─────────────────────────┘
              │ Delegates to
┌─────────────▼─────────────────────────┐
│    Application Layer (riptide-facade) │
│  - ExtractionFacade                    │
│  - ScraperFacade                       │
│  - SpiderFacade                        │
│  - SearchFacade                        │
└─────────────┬─────────────────────────┘
              │ Orchestrates
┌─────────────▼─────────────────────────┐
│      Domain Layer (riptide-core)      │
│  - Business logic                      │
│  - Domain models                       │
│  - Pure functions                      │
└─────────────┬─────────────────────────┘
              │ Uses
┌─────────────▼─────────────────────────┐
│   Infrastructure (riptide-*)           │
│  - HTTP clients                        │
│  - Cache (Redis)                       │
│  - Browser automation                  │
│  - Search engines                      │
└───────────────────────────────────────┘
```

### Dependency Flow

```
main.rs
  → ApplicationContext::new()  (Composition Root)
    → Initialize infrastructure (Redis, HTTP, Browser)
    → Initialize domain services (Extractor, Spider)
    → Initialize facades (Extraction, Scraper, Search)
    → Wire everything with Arc<> for thread safety
  → Router::new()
    → Register handlers
    → Inject ApplicationContext via .with_state()
  → axum::serve()
```

---

## 📚 Related Crates

| Crate | Purpose | Used By |
|-------|---------|---------|
| `riptide-cli` | CLI interface | Alternative entry point |
| `riptide-facade` | Application services | Handlers |
| `riptide-core` | Domain logic | Facades |
| `riptide-extraction` | Content extraction | Core |
| `riptide-spider` | Web crawling | Core |
| `riptide-search` | Search integration | Facades |
| `riptide-cache` | Redis caching | Infrastructure |
| `riptide-headless` | Browser automation | Infrastructure |

---

## 🤝 Contributing

See main project `CONTRIBUTING.md` for guidelines.

**Before adding features:**
1. Read architecture docs in `/docs/hexa-analysis.md`
2. Understand hexagonal architecture principles
3. Keep handlers thin - business logic belongs in facades
4. All dependencies must be wired through ApplicationContext
5. Write integration tests in `/tests/integration/`

---

## 📄 License

Apache-2.0
