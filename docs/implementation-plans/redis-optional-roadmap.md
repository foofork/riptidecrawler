# Redis Optional Implementation Roadmap

**Goal**: Make Redis optional with progressive enhancement (Minimal → Enhanced → Distributed)

**Estimated Timeline**: 2-3 weeks
**Risk Level**: Low (architecture already supports this)

---

## Phase 1: Configuration Infrastructure (2-3 days)

### 1.1 Add Cache Backend Configuration

**File**: `crates/riptide-config/src/lib.rs`

Add new config enum and settings:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheBackend {
    /// In-memory cache (no Redis required)
    Memory,
    /// Redis-backed cache (requires Redis connection)
    Redis,
}

impl Default for CacheBackend {
    fn default() -> Self {
        CacheBackend::Memory // Default to simplest mode
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache backend to use
    pub backend: CacheBackend,

    /// Redis URL (required only if backend = Redis)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_url: Option<String>,

    /// Memory cache TTL in seconds (used for in-memory backend)
    #[serde(default = "default_memory_ttl")]
    pub memory_ttl: u64,

    /// Max memory cache entries (LRU eviction)
    #[serde(default = "default_max_entries")]
    pub max_memory_entries: usize,
}

fn default_memory_ttl() -> u64 { 3600 }
fn default_max_entries() -> usize { 10_000 }

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            backend: CacheBackend::Memory,
            redis_url: None,
            memory_ttl: 3600,
            max_memory_entries: 10_000,
        }
    }
}
```

### 1.2 Add Worker Mode Configuration

**File**: `crates/riptide-config/src/lib.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    /// Enable worker service (requires Redis)
    pub enabled: bool,

    /// Redis URL for job queue (required if enabled = true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_url: Option<String>,

    /// Worker pool size
    #[serde(default = "default_worker_count")]
    pub worker_count: usize,
}

fn default_worker_count() -> usize { 4 }

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default
            redis_url: None,
            worker_count: 4,
        }
    }
}
```

### 1.3 Update Main Application Config

**File**: `crates/riptide-config/src/lib.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiptideConfig {
    // ... existing fields ...

    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,

    /// Worker service configuration
    #[serde(default)]
    pub workers: WorkerConfig,
}
```

### 1.4 Create Example Configurations

**File**: `config/minimal.toml` (NEW)

```toml
# Minimal configuration - No Redis required
# Perfect for: Local development, CI/CD, simple extraction

[cache]
backend = "memory"
memory_ttl = 3600
max_memory_entries = 10000

[workers]
enabled = false

[server]
host = "0.0.0.0"
port = 8080

[extraction]
timeout_ms = 30000
```

**File**: `config/enhanced.toml` (NEW)

```toml
# Enhanced configuration - Redis for caching
# Perfect for: Production single-instance, persistent cache

[cache]
backend = "redis"
redis_url = "redis://localhost:6379/0"

[workers]
enabled = false  # Still single-process, just with Redis cache

[server]
host = "0.0.0.0"
port = 8080
```

**File**: `config/distributed.toml` (NEW)

```toml
# Distributed configuration - Full Redis features
# Perfect for: Enterprise scale, multi-instance, job queue

[cache]
backend = "redis"
redis_url = "redis://localhost:6379/0"

[workers]
enabled = true
redis_url = "redis://localhost:6379/1"
worker_count = 8

[server]
host = "0.0.0.0"
port = 8080
```

**Acceptance Criteria**:
- [ ] Config structs compile and serialize correctly
- [ ] Default config uses in-memory cache
- [ ] Three example configs validate without errors
- [ ] Environment variables override config file settings

---

## Phase 2: Wire Up InMemoryCache (2-3 days)

### 2.1 Move InMemoryCache to Production Crate

**Current**: `crates/riptide-types/src/ports/memory_cache.rs` (476 lines)
**Action**: Already production-ready! Just needs to be exposed.

**File**: `crates/riptide-types/src/ports/mod.rs`

```rust
pub mod cache;
pub mod memory_cache; // Ensure this is public

// Re-export for convenience
pub use cache::CacheStorage;
pub use memory_cache::InMemoryCache;
```

### 2.2 Create Cache Factory

**File**: `crates/riptide-cache/src/factory.rs` (NEW)

```rust
use anyhow::{Context, Result};
use std::sync::Arc;
use riptide_types::ports::{CacheStorage, InMemoryCache};
use crate::redis::RedisCacheManager;
use riptide_config::CacheConfig;

/// Factory for creating cache backends based on configuration
pub struct CacheFactory;

impl CacheFactory {
    /// Create a cache backend from configuration
    pub async fn create(config: &CacheConfig) -> Result<Arc<dyn CacheStorage>> {
        match config.backend {
            CacheBackend::Memory => {
                tracing::info!("Initializing in-memory cache backend");
                let cache = InMemoryCache::new();
                Ok(Arc::new(cache))
            }
            CacheBackend::Redis => {
                let redis_url = config.redis_url.as_ref()
                    .context("Redis URL required when cache backend = 'redis'")?;

                tracing::info!("Initializing Redis cache backend at {}", redis_url);
                let cache = RedisCacheManager::new(redis_url)
                    .await
                    .context("Failed to connect to Redis")?;

                Ok(Arc::new(cache))
            }
        }
    }

    /// Create cache with automatic fallback: try Redis, fall back to memory
    pub async fn create_with_fallback(config: &CacheConfig) -> Arc<dyn CacheStorage> {
        match Self::create(config).await {
            Ok(cache) => cache,
            Err(e) => {
                tracing::warn!(
                    "Failed to initialize {:?} cache: {}. Falling back to in-memory cache.",
                    config.backend,
                    e
                );
                Arc::new(InMemoryCache::new())
            }
        }
    }
}
```

### 2.3 Update ApplicationContext to Use Factory

**File**: `crates/riptide-api/src/context.rs`

Find the current Redis initialization (around line 1745):

```rust
// OLD CODE (remove this):
match CacheManager::new(&redis_url).await {
    Ok(cm) => Arc::new(Mutex::new(cm)),
    Err(e) => {
        panic!("Redis required for integration tests")
    }
}

// NEW CODE (replace with this):
use riptide_cache::factory::CacheFactory;

let cache = CacheFactory::create(&config.cache)
    .await
    .context("Failed to initialize cache backend")?;

// Store in context
let cache_manager = Arc::new(Mutex::new(cache));
```

**Acceptance Criteria**:
- [ ] API starts successfully with `cache.backend = "memory"`
- [ ] API starts successfully with `cache.backend = "redis"` (Redis running)
- [ ] API returns helpful error if Redis backend selected but Redis unavailable
- [ ] Cache operations work identically with both backends
- [ ] InMemoryCache handles concurrent requests correctly

---

## Phase 3: Make Workers Optional (3-4 days)

### 3.1 Add Feature Flag to riptide-workers

**File**: `crates/riptide-workers/Cargo.toml`

```toml
[package]
name = "riptide-workers"
# ... existing metadata ...

[features]
default = []
redis-queue = ["dep:redis"]  # Optional Redis dependency

[dependencies]
redis = { workspace = true, optional = true }
# ... other deps ...
```

### 3.2 Create Job Queue Abstraction

**File**: `crates/riptide-workers/src/queue/mod.rs` (NEW)

```rust
use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;

/// Abstract job queue interface
#[async_trait]
pub trait JobQueue: Send + Sync {
    async fn enqueue(&self, job: Job) -> Result<Uuid>;
    async fn dequeue(&self) -> Result<Option<Job>>;
    async fn ack(&self, job_id: Uuid) -> Result<()>;
    async fn nack(&self, job_id: Uuid) -> Result<()>;
    async fn get_status(&self, job_id: Uuid) -> Result<JobStatus>;
}

// Implementations:
#[cfg(feature = "redis-queue")]
pub mod redis_queue;

pub mod memory_queue;  // In-memory queue (no persistence)

#[cfg(feature = "redis-queue")]
pub use redis_queue::RedisJobQueue;
pub use memory_queue::InMemoryJobQueue;
```

### 3.3 Implement InMemoryJobQueue

**File**: `crates/riptide-workers/src/queue/memory_queue.rs` (NEW)

```rust
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use uuid::Uuid;

/// In-memory job queue (no persistence, single-process only)
///
/// **Limitations:**
/// - Jobs lost on restart
/// - Single-process only (no distributed workers)
/// - No retry logic across restarts
///
/// **Use for:**
/// - Development and testing
/// - Simple single-instance deployments
/// - Low-volume background tasks
pub struct InMemoryJobQueue {
    queue: Arc<RwLock<VecDeque<Job>>>,
    status_map: Arc<RwLock<HashMap<Uuid, JobStatus>>>,
}

impl InMemoryJobQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            status_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl JobQueue for InMemoryJobQueue {
    async fn enqueue(&self, job: Job) -> Result<Uuid> {
        let mut queue = self.queue.write().await;
        let mut statuses = self.status_map.write().await;

        let job_id = job.id;
        queue.push_back(job);
        statuses.insert(job_id, JobStatus::Pending);

        tracing::debug!("Enqueued job {} to in-memory queue", job_id);
        Ok(job_id)
    }

    async fn dequeue(&self) -> Result<Option<Job>> {
        let mut queue = self.queue.write().await;
        let mut statuses = self.status_map.write().await;

        if let Some(job) = queue.pop_front() {
            statuses.insert(job.id, JobStatus::InProgress);
            Ok(Some(job))
        } else {
            Ok(None)
        }
    }

    async fn ack(&self, job_id: Uuid) -> Result<()> {
        let mut statuses = self.status_map.write().await;
        statuses.insert(job_id, JobStatus::Completed);
        Ok(())
    }

    async fn nack(&self, job_id: Uuid) -> Result<()> {
        let mut statuses = self.status_map.write().await;
        statuses.insert(job_id, JobStatus::Failed);
        Ok(())
    }

    async fn get_status(&self, job_id: Uuid) -> Result<JobStatus> {
        let statuses = self.status_map.read().await;
        Ok(statuses.get(&job_id).cloned().unwrap_or(JobStatus::Unknown))
    }
}
```

### 3.4 Update Worker Service to be Optional

**File**: `crates/riptide-api/src/main.rs`

```rust
// Initialize worker service only if enabled
let worker_service = if config.workers.enabled {
    let redis_url = config.workers.redis_url.as_ref()
        .context("Worker service enabled but redis_url not configured")?;

    tracing::info!("Starting worker service with Redis queue");
    let service = WorkerService::new(WorkerServiceConfig {
        redis_url: redis_url.clone(),
        worker_count: config.workers.worker_count,
        // ... other config
    })
    .await
    .context("Failed to start worker service")?;

    Some(Arc::new(service))
} else {
    tracing::info!("Worker service disabled - running in single-process mode");
    None
};

// Store in app context
let context = ApplicationContext {
    // ... existing fields ...
    worker_service,  // Option<Arc<WorkerService>>
};
```

### 3.5 Update Job Submission Endpoints

**File**: `crates/riptide-api/src/routes/jobs.rs`

```rust
pub async fn submit_job(
    State(context): State<Arc<ApplicationContext>>,
    Json(request): Json<JobRequest>,
) -> Result<Json<JobResponse>, ApiError> {
    match &context.worker_service {
        Some(workers) => {
            // Async job via worker queue
            let job_id = workers.submit(request).await?;
            Ok(Json(JobResponse {
                job_id,
                status: "queued",
                message: "Job submitted to worker queue",
            }))
        }
        None => {
            // Synchronous execution (no workers)
            tracing::warn!("Worker service not enabled - executing job synchronously");
            let result = execute_job_sync(&context, request).await?;
            Ok(Json(JobResponse {
                job_id: Uuid::new_v4(),
                status: "completed",
                message: "Job executed synchronously (workers disabled)",
                result: Some(result),
            }))
        }
    }
}
```

**Acceptance Criteria**:
- [ ] API starts successfully with `workers.enabled = false`
- [ ] Job submission works in sync mode when workers disabled
- [ ] Job submission works in async mode when workers enabled
- [ ] Clear warning logged when submitting job without workers
- [ ] InMemoryJobQueue handles concurrent operations correctly

---

## Phase 4: Graceful Degradation (2-3 days)

### 4.1 Add Capability Detection

**File**: `crates/riptide-api/src/capabilities.rs` (NEW)

```rust
use serde::{Serialize, Deserialize};

/// System capabilities based on configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCapabilities {
    /// Cache backend in use
    pub cache_backend: String,

    /// Whether workers are available
    pub async_jobs: bool,

    /// Whether distributed mode is available
    pub distributed: bool,

    /// Whether persistent cache is available
    pub persistent_cache: bool,

    /// Whether session persistence is available
    pub session_persistence: bool,
}

impl SystemCapabilities {
    pub fn detect(config: &RiptideConfig) -> Self {
        let has_redis = matches!(config.cache.backend, CacheBackend::Redis);
        let has_workers = config.workers.enabled;

        Self {
            cache_backend: format!("{:?}", config.cache.backend),
            async_jobs: has_workers,
            distributed: has_redis && has_workers,
            persistent_cache: has_redis,
            session_persistence: has_redis,
        }
    }
}
```

### 4.2 Add /health/capabilities Endpoint

**File**: `crates/riptide-api/src/routes/health.rs`

```rust
/// Get system capabilities
pub async fn get_capabilities(
    State(context): State<Arc<ApplicationContext>>,
) -> Json<SystemCapabilities> {
    Json(context.capabilities.clone())
}
```

### 4.3 Update Session Management

**File**: `crates/riptide-api/src/session/mod.rs`

```rust
pub enum SessionBackend {
    Redis(RedisSessionStore),
    Memory(InMemorySessionStore),
}

impl SessionManager {
    pub async fn new(config: &CacheConfig) -> Result<Self> {
        let backend = match config.backend {
            CacheBackend::Redis => {
                let store = RedisSessionStore::new(config.redis_url.as_ref().unwrap()).await?;
                SessionBackend::Redis(store)
            }
            CacheBackend::Memory => {
                tracing::warn!(
                    "Using in-memory session store - sessions will not persist across restarts"
                );
                SessionBackend::Memory(InMemorySessionStore::new())
            }
        };

        Ok(Self { backend })
    }
}
```

**Acceptance Criteria**:
- [ ] `/health/capabilities` endpoint returns accurate system state
- [ ] Sessions work with memory backend (warns about non-persistence)
- [ ] Clear warnings logged for degraded features
- [ ] Documentation updated to explain capability differences

---

## Phase 5: Deployment Configurations (1-2 days)

### 5.1 Create Minimal Docker Compose

**File**: `docker-compose.minimal.yml` (NEW)

```yaml
version: '3.8'

services:
  riptide-api:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - CONFIG_PATH=/app/config/minimal.toml
    volumes:
      - ./config/minimal.toml:/app/config/minimal.toml:ro
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 10s
      timeout: 5s
      retries: 3
    labels:
      description: "Minimal Riptide - No Redis required"
      use_case: "Development, CI/CD, simple extraction"

# No Redis service!
# No Chrome service (using WASM)
```

### 5.2 Update Existing Docker Compose

**File**: `docker-compose.simple.yml` (RENAME from docker-compose.lite.yml)

```yaml
version: '3.8'

services:
  riptide-api:
    # ... existing config ...
    environment:
      - REDIS_URL=redis://redis:6379/0
      - CONFIG_PATH=/app/config/enhanced.toml
    depends_on:
      redis:
        condition: service_healthy
    labels:
      description: "Enhanced Riptide - Redis cache, single-instance"
      use_case: "Production single-instance, persistent cache"

  redis:
    image: redis:7-alpine
    # ... existing Redis config ...
```

**File**: `docker-compose.yml` (UPDATE existing)

```yaml
version: '3.8'

services:
  riptide-api:
    # ... existing config ...
    environment:
      - CONFIG_PATH=/app/config/distributed.toml
    labels:
      description: "Distributed Riptide - Full features"
      use_case: "Enterprise production, multi-instance scaling"
    deploy:
      replicas: 3  # Multiple instances

  riptide-worker:
    # ... existing worker config ...

  redis:
    # ... existing Redis config ...

  chrome:
    # ... existing Chrome config ...
```

### 5.3 Update README Quick Start

**File**: `README.md`

```markdown
## Quick Start

Choose your deployment mode based on your needs:

### 🚀 Option 1: Minimal (No Dependencies)

Perfect for: Local development, CI/CD, simple extraction

```bash
# Using Docker
docker-compose -f docker-compose.minimal.yml up

# Or locally
cargo run --release -- --config config/minimal.toml
```

**Features:**
- ✅ Fast extraction and crawling
- ✅ In-memory cache
- ✅ Single-process
- ⚠️ No persistent cache
- ⚠️ No async job queue

### ⚡ Option 2: Enhanced (With Redis)

Perfect for: Production single-instance, persistent cache

```bash
docker-compose -f docker-compose.simple.yml up
```

**Features:**
- ✅ Everything in Minimal
- ✅ Persistent cache across restarts
- ✅ Session management
- ⚠️ Single-instance only

### 🏢 Option 3: Distributed (Full Stack)

Perfect for: Enterprise scale, multi-instance

```bash
docker-compose up --scale riptide-api=5
```

**Features:**
- ✅ Everything in Enhanced
- ✅ Distributed job queue
- ✅ Multi-instance scaling
- ✅ Worker pool
- ✅ Multi-tenancy
```

**Acceptance Criteria**:
- [ ] `docker-compose.minimal.yml` starts without Redis
- [ ] `docker-compose.simple.yml` starts with Redis only
- [ ] `docker-compose.yml` starts full stack
- [ ] All three modes pass health checks
- [ ] README clearly explains differences

---

## Phase 6: Documentation (2-3 days)

### 6.1 Update FAQ

**File**: `docs/00-getting-started/faq.md`

Update the "Do I need Redis?" section:

```markdown
### Do I need Redis?

**Short answer**: No! Redis is optional.

**Long answer**: It depends on your use case:

| Use Case | Redis Needed? | Recommended Mode |
|----------|---------------|------------------|
| Extract data from 1-100 URLs | ❌ No | Minimal |
| Local development & testing | ❌ No | Minimal |
| CI/CD integration tests | ❌ No | Minimal |
| Production single-instance | ⚠️ Recommended | Enhanced |
| Multi-instance deployment | ✅ Yes | Distributed |
| Background job processing | ✅ Yes | Distributed |

**What you lose without Redis:**
- Persistent cache (in-memory cache clears on restart)
- Session persistence (browser contexts don't survive restarts)
- Distributed job queue (jobs execute synchronously)
- Multi-instance coordination (single process only)

**What still works without Redis:**
- ✅ HTML extraction (2-5ms)
- ✅ JavaScript rendering (WASM)
- ✅ AI schema generation
- ✅ Smart crawling/spider
- ✅ All core extraction features
```

### 6.2 Create Migration Guide

**File**: `docs/guides/redis-migration.md` (NEW)

```markdown
# Migrating Between Deployment Modes

## From Minimal → Enhanced (Adding Redis)

**Why?** You need persistent cache or session management.

**Steps:**
1. Install Redis: `docker run -d -p 6379:6379 redis:7-alpine`
2. Update config:
   ```toml
   [cache]
   backend = "redis"
   redis_url = "redis://localhost:6379/0"
   ```
3. Restart Riptide
4. Verify: `curl http://localhost:8080/health/capabilities`

**What changes:**
- Cache persists across restarts
- Sessions survive server restarts
- Slight latency increase (~1-2ms per cache operation)

## From Enhanced → Distributed (Adding Workers)

**Why?** You need background job processing or multi-instance scaling.

**Steps:**
1. Update config:
   ```toml
   [workers]
   enabled = true
   redis_url = "redis://localhost:6379/1"  # Different DB
   worker_count = 4
   ```
2. Start worker process: `cargo run --bin riptide-worker`
3. Scale API: `docker-compose up --scale riptide-api=3`

**What changes:**
- Jobs execute asynchronously
- Can scale horizontally
- Requires worker monitoring
```

### 6.3 Update Architecture Docs

**File**: `docs/architecture/cache-layer.md`

Add section on backend selection:

```markdown
## Cache Backend Selection

The cache layer supports two backends:

### InMemoryCache

**Implementation**: `DashMap<String, CacheEntry>`

**Pros:**
- Zero dependencies
- No network latency
- Simple operations (~50ns)
- Perfect for development

**Cons:**
- Not shared across instances
- Cleared on restart
- Limited by RAM
- No TTL persistence

**Use for:**
- Local development
- CI/CD tests
- Single-instance with high cache churn

### RedisCache

**Implementation**: `redis::AsyncCommands`

**Pros:**
- Shared across instances
- Persists across restarts
- Distributed coordination
- Production-proven

**Cons:**
- Requires Redis server
- Network latency (~1-2ms)
- Additional complexity

**Use for:**
- Production deployments
- Multi-instance scaling
- Long-lived cache entries
```

**Acceptance Criteria**:
- [ ] FAQ clearly explains when Redis is needed
- [ ] Migration guide covers all transition paths
- [ ] Architecture docs explain backend tradeoffs
- [ ] All examples use correct configuration format

---

## Phase 7: Testing (3-4 days)

### 7.1 Add Integration Tests for Both Modes

**File**: `crates/riptide-api/tests/cache_backends.rs` (NEW)

```rust
#[cfg(test)]
mod cache_tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_cache_basic_operations() {
        let config = CacheConfig {
            backend: CacheBackend::Memory,
            ..Default::default()
        };

        let cache = CacheFactory::create(&config).await.unwrap();

        // Test set/get
        cache.set("key1", "value1", 60).await.unwrap();
        let value = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Test expiration
        cache.set("key2", "value2", 1).await.unwrap();
        tokio::time::sleep(Duration::from_secs(2)).await;
        let value = cache.get("key2").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    #[ignore] // Only run when Redis available
    async fn test_redis_cache_basic_operations() {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let config = CacheConfig {
            backend: CacheBackend::Redis,
            redis_url: Some(redis_url),
            ..Default::default()
        };

        let cache = CacheFactory::create(&config).await.unwrap();

        // Same tests as memory cache
        // ...
    }

    #[tokio::test]
    async fn test_cache_fallback() {
        // Try Redis with invalid URL - should fall back to memory
        let config = CacheConfig {
            backend: CacheBackend::Redis,
            redis_url: Some("redis://invalid:9999".to_string()),
            ..Default::default()
        };

        let cache = CacheFactory::create_with_fallback(&config).await;

        // Should still work with memory backend
        cache.set("key", "value", 60).await.unwrap();
        let value = cache.get("key").await.unwrap();
        assert_eq!(value, Some("value".to_string()));
    }
}
```

### 7.2 Add Worker Mode Tests

**File**: `crates/riptide-workers/tests/queue_backends.rs` (NEW)

```rust
#[tokio::test]
async fn test_memory_queue_operations() {
    let queue = InMemoryJobQueue::new();

    let job = Job {
        id: Uuid::new_v4(),
        task: "test".to_string(),
        // ...
    };

    // Enqueue
    let job_id = queue.enqueue(job.clone()).await.unwrap();

    // Dequeue
    let dequeued = queue.dequeue().await.unwrap().unwrap();
    assert_eq!(dequeued.id, job_id);

    // Ack
    queue.ack(job_id).await.unwrap();
    let status = queue.get_status(job_id).await.unwrap();
    assert_eq!(status, JobStatus::Completed);
}
```

### 7.3 Add API Integration Tests

**File**: `crates/riptide-api/tests/deployment_modes.rs` (NEW)

```rust
/// Test API works in minimal mode (no Redis)
#[tokio::test]
async fn test_minimal_mode() {
    let config = load_config("config/minimal.toml");
    let app = build_application(config).await.unwrap();

    // Test extraction still works
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/extract")
                .method("POST")
                .body(Body::from(r#"{"url":"https://example.com"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

/// Test capabilities endpoint reports correct mode
#[tokio::test]
async fn test_capabilities_detection() {
    let config = load_config("config/minimal.toml");
    let app = build_application(config).await.unwrap();

    let response = app
        .oneshot(Request::builder().uri("/health/capabilities").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let capabilities: SystemCapabilities = serde_json::from_slice(
        &hyper::body::to_bytes(response.into_body()).await.unwrap()
    ).unwrap();

    assert_eq!(capabilities.cache_backend, "Memory");
    assert!(!capabilities.async_jobs);
    assert!(!capabilities.distributed);
}
```

**Acceptance Criteria**:
- [ ] All tests pass with `cache.backend = "memory"`
- [ ] All tests pass with `cache.backend = "redis"` (Redis running)
- [ ] Fallback tests verify graceful degradation
- [ ] API tests verify extraction works in all modes
- [ ] Worker tests verify both queue backends

---

## Phase 8: CI/CD Updates (1-2 days)

### 8.1 Update GitHub Actions

**File**: `.github/workflows/ci.yml`

```yaml
name: CI

on: [push, pull_request]

jobs:
  test-minimal-mode:
    name: Test Minimal Mode (No Redis)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      # No Redis service!

      - name: Run tests with memory backend
        run: |
          cargo test --all-features
        env:
          RIPTIDE_CONFIG: config/minimal.toml

  test-enhanced-mode:
    name: Test Enhanced Mode (With Redis)
    runs-on: ubuntu-latest
    services:
      redis:
        image: redis:7-alpine
        ports:
          - 6379:6379
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      - name: Run tests with Redis backend
        run: |
          cargo test --all-features -- --include-ignored
        env:
          REDIS_URL: redis://localhost:6379
          RIPTIDE_CONFIG: config/enhanced.toml

  test-distributed-mode:
    name: Test Distributed Mode (Full Stack)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Start full stack
        run: docker-compose up -d

      - name: Wait for services
        run: |
          timeout 60 bash -c 'until curl -f http://localhost:8080/health; do sleep 2; done'

      - name: Run integration tests
        run: |
          cargo test --test integration -- --test-threads=1
        env:
          RIPTIDE_URL: http://localhost:8080
```

### 8.2 Update Docker Build

**File**: `Dockerfile`

Ensure the image includes all config files:

```dockerfile
# Copy config files
COPY config/minimal.toml /app/config/minimal.toml
COPY config/enhanced.toml /app/config/enhanced.toml
COPY config/distributed.toml /app/config/distributed.toml

# Default to minimal mode
ENV CONFIG_PATH=/app/config/minimal.toml
```

**Acceptance Criteria**:
- [ ] CI tests pass without Redis service
- [ ] CI tests pass with Redis service
- [ ] Docker image builds successfully
- [ ] All three configs included in image
- [ ] Default mode is minimal (no Redis)

---

## Rollout Strategy

### Week 1: Foundation
- ✅ Phase 1: Configuration infrastructure
- ✅ Phase 2: Wire up InMemoryCache
- ✅ Basic testing

### Week 2: Workers & Testing
- ✅ Phase 3: Make workers optional
- ✅ Phase 7: Comprehensive tests
- ✅ Phase 8: CI/CD updates

### Week 3: Polish & Documentation
- ✅ Phase 4: Graceful degradation
- ✅ Phase 5: Deployment configs
- ✅ Phase 6: Documentation
- ✅ Final QA and launch

---

## Success Metrics

### Technical
- [ ] API starts in <5s without Redis (currently panics)
- [ ] Zero breaking changes to existing Redis users
- [ ] All tests pass in both modes
- [ ] Docker images work in all three modes

### User Experience
- [ ] New users can run `cargo run` immediately
- [ ] Clear error messages if Redis misconfigured
- [ ] Migration path documented for all transitions
- [ ] Performance benchmarks show <5% difference between modes

### Documentation
- [ ] README quick start covers all modes
- [ ] FAQ answers "Do I need Redis?"
- [ ] Architecture docs explain backend selection
- [ ] Migration guide helps users upgrade

---

## Risk Mitigation

### Risk: Breaking existing deployments

**Mitigation:**
- Default config includes Redis URL check
- Existing docker-compose.yml unchanged
- Backward compatible environment variables

### Risk: Performance regression with InMemoryCache

**Mitigation:**
- Benchmark both backends
- Document performance characteristics
- Recommend Redis for production

### Risk: Incomplete feature parity

**Mitigation:**
- Capabilities endpoint reports feature availability
- Clear warnings for degraded features
- Documentation explains limitations

### Risk: Increased support burden

**Mitigation:**
- Three clear modes with specific use cases
- Health endpoint shows current configuration
- Automated tests for all modes

---

## Future Enhancements

### Phase 9 (Future): Alternative Backends
- PostgreSQL cache backend (SQL-based persistence)
- SQLite backend (file-based, no server)
- Memcached support (for existing infrastructure)

### Phase 10 (Future): Hybrid Modes
- Memory cache with Redis fallback
- Distributed cache with local L1
- Smart backend selection based on workload

---

## Appendix: File Checklist

### New Files
- [ ] `config/minimal.toml`
- [ ] `config/enhanced.toml`
- [ ] `config/distributed.toml`
- [ ] `docker-compose.minimal.yml`
- [ ] `crates/riptide-cache/src/factory.rs`
- [ ] `crates/riptide-workers/src/queue/mod.rs`
- [ ] `crates/riptide-workers/src/queue/memory_queue.rs`
- [ ] `crates/riptide-api/src/capabilities.rs`
- [ ] `docs/guides/redis-migration.md`
- [ ] `crates/riptide-api/tests/cache_backends.rs`
- [ ] `crates/riptide-workers/tests/queue_backends.rs`
- [ ] `crates/riptide-api/tests/deployment_modes.rs`

### Modified Files
- [ ] `crates/riptide-config/src/lib.rs` (add configs)
- [ ] `crates/riptide-types/src/ports/mod.rs` (expose InMemoryCache)
- [ ] `crates/riptide-cache/Cargo.toml` (optional Redis)
- [ ] `crates/riptide-workers/Cargo.toml` (feature flags)
- [ ] `crates/riptide-api/src/context.rs` (use factory)
- [ ] `crates/riptide-api/src/main.rs` (optional workers)
- [ ] `crates/riptide-api/src/routes/jobs.rs` (sync fallback)
- [ ] `crates/riptide-api/src/routes/health.rs` (capabilities)
- [ ] `docker-compose.simple.yml` (rename from lite)
- [ ] `docker-compose.yml` (update labels)
- [ ] `README.md` (quick start options)
- [ ] `docs/00-getting-started/faq.md` (Redis question)
- [ ] `docs/architecture/cache-layer.md` (backends)
- [ ] `.github/workflows/ci.yml` (test both modes)
- [ ] `Dockerfile` (include configs)

---

**Total Estimate**: 2-3 weeks with 1 developer, 1-1.5 weeks with 2 developers

**Confidence**: High (architecture already supports this, just wiring needed)
