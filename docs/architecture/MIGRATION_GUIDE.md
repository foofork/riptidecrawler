# Phase 0 Infrastructure Migration Guide

**Document Version:** 1.0
**Date:** 2025-01-08
**Purpose:** Step-by-step migration instructions for consolidating infrastructure

## Table of Contents

1. [Migration Overview](#migration-overview)
2. [CacheStorage Migration](#cachestorage-migration)
3. [Robots.txt Split Migration](#robotstxt-split-migration)
4. [Memory Manager Consolidation](#memory-manager-consolidation)
5. [Pipeline Core Migration](#pipeline-core-migration)
6. [SchemaStore Setup](#schemastore-setup)
7. [Rollback Procedures](#rollback-procedures)

---

## Migration Overview

### Migration Strategy

**Principle:** Incremental, non-breaking changes with parallel operation

**Phases:**
1. **Create New** - Build new trait-based infrastructure
2. **Parallel Run** - Run old and new code side-by-side
3. **Validate** - Verify behavior matches
4. **Switch** - Feature flag to new implementation
5. **Deprecate** - Mark old code as deprecated
6. **Remove** - Delete old code after grace period

### Feature Flags

All migrations use feature flags for safe rollout:

```toml
[features]
default = []

# Phase 0 migrations
cache-trait = []          # Use new CacheStorage trait
robots-split = []         # Use split robots.txt architecture
unified-memory = []       # Use unified memory manager
pipeline-core = []        # Use pipeline core infrastructure
schema-store = []         # Use schema store

# Convenience meta-features
phase0-all = ["cache-trait", "robots-split", "unified-memory", "pipeline-core", "schema-store"]
```

---

## CacheStorage Migration

### Current State

**Before:**
- Direct Redis usage in multiple crates
- Hard to test without Redis instance
- Cannot swap backends
- Code duplication

### Target State

**After:**
- Trait-based abstraction (`CacheStorage`)
- Easy testing with in-memory backend
- Pluggable implementations
- Single source of truth

### Step 1: Create Port Interface

**File:** `/workspaces/eventmesh/crates/riptide-types/src/ports/cache.rs`

```rust
// See TRAIT_SPECIFICATIONS.md for full implementation
pub trait CacheStorage: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<bool>;
    async fn exists(&self, key: &str) -> Result<bool>;
    async fn clear(&self) -> Result<u64>;
    async fn stats(&self) -> Result<CacheStats>;
}
```

**Add to `riptide-types/src/ports/mod.rs`:**
```rust
#[cfg(feature = "cache-trait")]
pub mod cache;
```

### Step 2: Create Redis Adapter

**File:** `/workspaces/eventmesh/crates/riptide-cache/src/adapters/redis.rs`

```rust
use riptide_types::ports::cache::CacheStorage;
use redis::aio::MultiplexedConnection;

pub struct RedisCache {
    conn: MultiplexedConnection,
    default_ttl: Duration,
    key_prefix: String,
}

#[async_trait]
impl CacheStorage for RedisCache {
    // Implement all trait methods
    // See TRAIT_SPECIFICATIONS.md
}
```

**Add to `riptide-cache/src/adapters/mod.rs`:**
```rust
#[cfg(feature = "redis")]
pub mod redis;

#[cfg(feature = "redis")]
pub use redis::RedisCache;
```

### Step 3: Create In-Memory Adapter (for testing)

**File:** `/workspaces/eventmesh/crates/riptide-cache/src/adapters/memory.rs`

```rust
use dashmap::DashMap;

pub struct InMemoryCache {
    storage: DashMap<String, CacheEntry>,
    default_ttl: Duration,
}

#[async_trait]
impl CacheStorage for InMemoryCache {
    // Implement all trait methods
}
```

### Step 4: Update Consumer Code

**Before:**
```rust
// Old way - direct Redis dependency
use redis::Client;

pub struct ExtractionService {
    redis_client: Client,
}

impl ExtractionService {
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)?;
        Ok(Self { redis_client: client })
    }

    pub async fn extract_with_cache(&self, url: &str) -> Result<String> {
        let mut conn = self.redis_client.get_connection()?;
        if let Some(cached) = conn.get::<_, Option<String>>(url)? {
            return Ok(cached);
        }
        // ... extract and cache
    }
}
```

**After:**
```rust
// New way - trait-based dependency injection
use riptide_types::ports::cache::CacheStorage;
use std::sync::Arc;

pub struct ExtractionService {
    cache: Arc<dyn CacheStorage>,
}

impl ExtractionService {
    pub fn new(cache: Arc<dyn CacheStorage>) -> Self {
        Self { cache }
    }

    pub async fn extract_with_cache(&self, url: &str) -> Result<String> {
        // Check cache
        if let Some(cached_bytes) = self.cache.get(url).await? {
            let cached_str = String::from_utf8(cached_bytes)?;
            return Ok(cached_str);
        }

        // Extract content
        let content = self.extract_fresh(url).await?;

        // Store in cache
        self.cache.set(
            url,
            content.as_bytes(),
            Some(Duration::from_secs(3600)),
        ).await?;

        Ok(content)
    }
}
```

### Step 5: Update Tests

**Before (requires Redis):**
```rust
#[tokio::test]
async fn test_extraction_with_cache() {
    // Requires Docker/Redis running!
    let service = ExtractionService::new("redis://localhost").unwrap();
    // ... test
}
```

**After (no Redis needed):**
```rust
#[tokio::test]
async fn test_extraction_with_cache() {
    // No external dependencies!
    let cache = Arc::new(InMemoryCache::new(Duration::from_secs(60)));
    let service = ExtractionService::new(cache);

    let result = service.extract_with_cache("https://example.com").await.unwrap();
    assert_eq!(result, "expected content");
}
```

### Step 6: Update Production Initialization

**File:** `main.rs` or initialization code

```rust
use riptide_cache::adapters::RedisCache;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Create Redis cache adapter
    let cache = Arc::new(
        RedisCache::new(
            &config.redis_url,
            Duration::from_secs(3600)
        ).await?
    ) as Arc<dyn CacheStorage>;

    // Inject into services
    let extraction_service = ExtractionService::new(Arc::clone(&cache));

    // Start server
    // ...
}
```

### Step 7: Parallel Operation

Run both old and new implementations side-by-side:

```rust
#[cfg(feature = "cache-trait")]
use riptide_types::ports::cache::CacheStorage;

pub struct ExtractionService {
    #[cfg(feature = "cache-trait")]
    cache: Arc<dyn CacheStorage>,

    #[cfg(not(feature = "cache-trait"))]
    redis_client: redis::Client,
}

impl ExtractionService {
    pub async fn extract_with_cache(&self, url: &str) -> Result<String> {
        #[cfg(feature = "cache-trait")]
        {
            // New implementation
            self.extract_with_trait_cache(url).await
        }

        #[cfg(not(feature = "cache-trait"))]
        {
            // Old implementation
            self.extract_with_redis(url).await
        }
    }
}
```

### Step 8: Validation

Run comprehensive tests with both implementations:

```bash
# Test with old implementation
cargo test --no-default-features

# Test with new implementation
cargo test --features cache-trait

# Test both in parallel (if supported)
cargo test --all-features
```

### Step 9: Switch Over

After validation passes:

```toml
# Update Cargo.toml
[features]
default = ["cache-trait"]  # Enable new implementation by default
```

### Step 10: Deprecate Old Code

```rust
#[deprecated(
    since = "0.2.0",
    note = "Use CacheStorage trait instead. Will be removed in 0.3.0"
)]
pub struct OldRedisCache {
    // ...
}
```

### Step 11: Remove Old Code (after grace period)

After 1-2 sprint cycles, remove old implementation:

```bash
# Remove old files
rm crates/riptide-cache/src/redis_old.rs

# Update Cargo.toml to remove feature flag
# [features]
# default = []  # Remove cache-trait from options
```

---

## Robots.txt Split Migration

### Current State

**Before:**
- Mixed sync/async in single file
- Parsing + HTTP + retry in one module
- Hard to unit test parsing logic

### Target State

**After:**
- Pure parsing in `riptide-utils`
- HTTP/retry in `riptide-reliability`
- Easy to test each layer independently

### Step 1: Create Pure Parser Layer

**File:** `/workspaces/eventmesh/crates/riptide-utils/src/robots.rs`

```rust
// Pure, synchronous parsing - NO async, NO http
pub trait RobotsParser: Send + Sync {
    fn parse(&self, content: &str, user_agent: &str) -> Result<RobotRules>;
    fn is_allowed(&self, rules: &RobotRules, path: &str) -> bool;
    fn extract_crawl_delay(&self, content: &str, user_agent: &str) -> Option<f64>;
}

pub struct DefaultRobotsParser {
    config: RobotsConfig,
}

impl RobotsParser for DefaultRobotsParser {
    fn parse(&self, content: &str, user_agent: &str) -> Result<RobotRules> {
        // Extract parsing logic from riptide-fetch/src/robots.rs
        // Lines 284-304: extract_crawl_delay
        // Use robotstxt::DefaultMatcher for actual parsing
    }
}
```

### Step 2: Create HTTP Fetcher Layer

**File:** `/workspaces/eventmesh/crates/riptide-reliability/src/robots_fetcher.rs`

```rust
use riptide_utils::robots::{RobotsParser, RobotRules};

#[async_trait]
pub trait RobotsFetcher: Send + Sync {
    async fn fetch_robots_txt(&self, base_url: &str) -> Result<String>;
    async fn is_allowed(&self, url: &str) -> Result<bool>;
}

pub struct ReliableRobotsFetcher {
    http_client: Arc<ReliableHttpClient>,
    parser: Arc<dyn RobotsParser>,
    cache: DashMap<String, CachedRobots>,
    config: RobotsConfig,
}

impl RobotsFetcher for ReliableRobotsFetcher {
    async fn fetch_robots_txt(&self, base_url: &str) -> Result<String> {
        // Check cache
        if let Some(cached) = self.cache.get(base_url) {
            if !cached.is_expired() {
                return Ok(cached.content.clone());
            }
        }

        // Fetch via ReliableHttpClient (has circuit breaker + retry)
        let robots_url = format!("{}/robots.txt", base_url);
        let response = self.http_client.get(&robots_url).send().await?;

        let content = if response.status().is_success() {
            response.text().await?
        } else {
            String::new() // 404 = permissive
        };

        // Cache result
        self.cache.insert(base_url.to_string(), CachedRobots {
            content: content.clone(),
            cached_at: Instant::now(),
            ttl: Duration::from_secs(self.config.cache_ttl),
        });

        Ok(content)
    }

    async fn is_allowed(&self, url: &str) -> Result<bool> {
        let parsed_url = Url::parse(url)?;
        let base_url = format!("{}://{}",
            parsed_url.scheme(),
            parsed_url.host_str().unwrap()
        );

        // Fetch robots.txt (cached)
        let content = self.fetch_robots_txt(&base_url).await?;

        // Parse using pure parser
        let rules = self.parser.parse(&content, &self.config.user_agent)?;

        // Check if allowed (pure logic)
        Ok(self.parser.is_allowed(&rules, parsed_url.path()))
    }
}
```

### Step 3: Write Pure Tests

**File:** `/workspaces/eventmesh/crates/riptide-utils/src/robots/tests.rs`

```rust
#[test]
fn test_parse_robots_txt() {
    let parser = DefaultRobotsParser::new();

    let content = r#"
        User-agent: *
        Disallow: /admin
        Crawl-delay: 2.5
    "#;

    let rules = parser.parse(content, "*").unwrap();
    assert_eq!(rules.crawl_delay, Some(2.5));
    assert!(!parser.is_allowed(&rules, "/admin/panel"));
    assert!(parser.is_allowed(&rules, "/public"));
}

#[test]
fn test_crawl_delay_clamping() {
    let parser = DefaultRobotsParser::new();

    let content = "Crawl-delay: 100"; // Very high
    let delay = parser.extract_crawl_delay(content, "*");

    // Should be clamped to max (e.g., 10.0)
    assert!(delay.unwrap() <= 10.0);
}
```

### Step 4: Update Spider to Use New Architecture

**Before:**
```rust
use riptide_fetch::robots::RobotsManager;

pub struct Spider {
    robots: Arc<RobotsManager>,
}
```

**After:**
```rust
use riptide_reliability::robots_fetcher::RobotsFetcher;

pub struct Spider {
    robots: Arc<dyn RobotsFetcher>,
}
```

### Step 5: Remove Old Implementation

After migration complete:

```bash
# Move old implementation to deprecated module
git mv crates/riptide-fetch/src/robots.rs \
       crates/riptide-fetch/src/robots_deprecated.rs

# Add deprecation notice
echo "#[deprecated(since = \"0.2.0\")]" >> \
     crates/riptide-fetch/src/robots_deprecated.rs
```

---

## Memory Manager Consolidation

### Current State

**Before:**
- `riptide-pool/src/memory_manager.rs` - WASM-specific (1122 lines)
- `riptide-api/src/resource_manager/memory_manager.rs` - API-level (988 lines)
- Separate tracking, duplicate logic

### Target State

**After:**
- Unified memory manager in `riptide-pool`
- Tracks all resource types
- Single metrics interface
- Shared leak detection

### Step 1: Define Resource Types

**File:** `/workspaces/eventmesh/crates/riptide-pool/src/resource_types.rs`

```rust
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ResourceType {
    WasmInstance {
        instance_id: String,
        component: Arc<Component>,
    },
    HttpConnection {
        pool_name: String,
        pool_size: usize,
    },
    BrowserSession {
        session_id: String,
        browser_type: BrowserType,
    },
    PdfProcessor {
        processor_id: String,
        slots_used: usize,
    },
    Custom {
        name: String,
        metadata: HashMap<String, String>,
    },
}
```

### Step 2: Extend Memory Manager

**File:** `/workspaces/eventmesh/crates/riptide-pool/src/memory_manager.rs`

Add resource trackers:

```rust
pub struct UnifiedMemoryManager {
    // Existing WASM pool
    wasm_pool: Option<WasmPoolManager>,

    // NEW: Additional resource trackers
    http_resources: Arc<HttpResourceTracker>,
    browser_sessions: Arc<BrowserSessionTracker>,
    pdf_processors: Arc<PdfProcessorTracker>,

    // Unified metrics
    metrics: Arc<MemoryStats>,
    config: MemoryManagerConfig,
}

impl UnifiedMemoryManager {
    pub async fn track_allocation(
        &self,
        resource: ResourceType,
        size_mb: u64,
    ) -> Result<()> {
        match resource {
            ResourceType::WasmInstance { .. } => {
                if let Some(wasm) = &self.wasm_pool {
                    wasm.track_allocation(size_mb).await?;
                }
            }
            ResourceType::HttpConnection { ref pool_name, pool_size } => {
                self.http_resources
                    .track_allocation(pool_name, pool_size, size_mb)
                    .await?;
            }
            // ... other types
        }

        // Update unified metrics
        self.metrics
            .total_allocated_mb
            .fetch_add(size_mb, Ordering::Relaxed);

        Ok(())
    }
}
```

### Step 3: Migrate API Resource Manager

**Before:**
```rust
// riptide-api/src/resource_manager/memory_manager.rs
pub struct MemoryManager {
    current_usage: AtomicUsize,
    // ... API-specific fields
}
```

**After:**
```rust
// Use unified manager from riptide-pool
use riptide_pool::UnifiedMemoryManager;

pub struct ApiMemoryManager {
    unified: Arc<UnifiedMemoryManager>,
}

impl ApiMemoryManager {
    pub async fn track_allocation(&self, size_mb: usize) {
        self.unified
            .track_allocation(
                ResourceType::Custom {
                    name: "api".to_string(),
                    metadata: HashMap::new(),
                },
                size_mb as u64,
            )
            .await
            .unwrap();
    }
}
```

### Step 4: Update All Resource Allocators

Find all places that allocate resources:

```bash
# Find resource allocations
rg "track_allocation|track_deallocation" --type rust
```

Update each to use unified manager:

```rust
// Before
memory_manager.track_allocation(size_mb);

// After
memory_manager.track_allocation(
    ResourceType::BrowserSession {
        session_id: id.clone(),
        browser_type: BrowserType::Chromium,
    },
    size_mb,
).await?;
```

---

## Pipeline Core Migration

### Step 1: Create Pipeline Core Crate

```bash
cargo new --lib crates/riptide-pipeline-core
```

**File:** `/workspaces/eventmesh/crates/riptide-pipeline-core/Cargo.toml`

```toml
[package]
name = "riptide-pipeline-core"
version = "0.1.0"

[dependencies]
riptide-types = { path = "../riptide-types" }
riptide-reliability = { path = "../riptide-reliability" }
anyhow = "1.0"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"

[features]
default = []
streaming = []
dual-mode = []
strategies = []
```

### Step 2: Extract Common Infrastructure

**File:** `/workspaces/eventmesh/crates/riptide-pipeline-core/src/lib.rs`

```rust
pub struct PipelineCore {
    config: PipelineConfig,
    metrics: Arc<PipelineMetrics>,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl PipelineCore {
    pub async fn execute_with_infrastructure<F, T>(
        &self,
        name: &str,
        operation: F,
    ) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        // Common pipeline infrastructure
        // - Circuit breaker check
        // - Retry logic
        // - Metrics tracking
        // - Error handling
    }
}
```

### Step 3: Convert Existing Pipelines

**Pattern for conversion:**

```rust
// Before
pub struct OldPipeline {
    // Lots of duplicate infrastructure
}

// After
pub struct NewPipeline {
    core: PipelineCore,  // Reuse common infrastructure
    // Only pipeline-specific fields
}

impl Pipeline for NewPipeline {
    async fn execute(&self, request: Self::Request) -> Result<Self::Response> {
        self.core.execute_with_infrastructure("my_pipeline", async {
            // Pipeline-specific logic only
            self.process_internal(request).await
        }).await
    }
}
```

### Step 4: Feature-Flag Variants

```rust
#[cfg(feature = "streaming")]
pub mod streaming {
    use super::*;

    pub struct StreamingPipeline {
        core: PipelineCore,
        stream_config: StreamConfig,
    }
}

#[cfg(feature = "dual-mode")]
pub mod dual {
    use super::*;

    pub struct DualModePipeline {
        core: PipelineCore,
        primary: Box<dyn Pipeline>,
        fallback: Box<dyn Pipeline>,
    }
}
```

### Step 5: Gradual Migration

Migrate pipelines one at a time:

1. **Week 1:** Simple pipeline (riptide-api/src/pipeline.rs)
2. **Week 2:** Streaming pipeline
3. **Week 3:** Dual-mode pipeline
4. **Week 4:** Strategies pipeline

After each migration, run full test suite to verify behavior.

---

## SchemaStore Setup

### Step 1: Create Stub Implementation

**File:** `/workspaces/eventmesh/crates/riptide-persistence/src/schema_store/memory.rs`

```rust
pub struct InMemorySchemaStore {
    schemas: DashMap<String, Value>,
    compiled_cache: DashMap<String, Arc<jsonschema::JSONSchema>>,
}

#[async_trait]
impl SchemaStore for InMemorySchemaStore {
    async fn put(&self, schema_uri: &str, schema: Value) -> Result<()> {
        // Validate schema is valid JSON Schema
        let compiled = jsonschema::JSONSchema::compile(&schema)?;

        // Store schema and compiled version
        self.schemas.insert(schema_uri.to_string(), schema);
        self.compiled_cache.insert(
            schema_uri.to_string(),
            Arc::new(compiled),
        );

        Ok(())
    }

    async fn validate(&self, schema_uri: &str, data: &Value) -> Result<bool> {
        // Get or compile schema
        let compiled = if let Some(cached) = self.compiled_cache.get(schema_uri) {
            cached.clone()
        } else {
            let schema = self.get(schema_uri).await?
                .ok_or_else(|| anyhow!("Schema not found: {}", schema_uri))?;

            let compiled = Arc::new(jsonschema::JSONSchema::compile(&schema)?);
            self.compiled_cache.insert(schema_uri.to_string(), compiled.clone());
            compiled
        };

        // Validate
        Ok(compiled.is_valid(data))
    }
}
```

### Step 2: Use in Validation Code

```rust
use riptide_types::ports::schema_store::SchemaStore;

pub struct Validator {
    schema_store: Arc<dyn SchemaStore>,
}

impl Validator {
    pub async fn validate_request(&self, request: &Value) -> Result<bool> {
        self.schema_store
            .validate("request.v1", request)
            .await
    }
}
```

### Step 3: Prepare for Future Backends

Document where Redis/S3 implementations will go:

```rust
// Future implementation
#[cfg(feature = "redis-schemas")]
pub mod redis {
    pub struct RedisSchemaStore {
        // Will use CacheStorage trait
        cache: Arc<dyn CacheStorage>,
        // Compile cache
        compiled: DashMap<String, Arc<jsonschema::JSONSchema>>,
    }
}

#[cfg(feature = "s3-schemas")]
pub mod s3 {
    pub struct S3SchemaStore {
        // Will use AWS SDK
        client: S3Client,
        bucket: String,
    }
}
```

---

## Rollback Procedures

### Emergency Rollback

If critical issue discovered:

```bash
# 1. Disable new feature immediately
cargo build --no-default-features

# 2. Deploy old version
git revert <migration-commit>
cargo build
cargo test

# 3. Redeploy
./deploy.sh
```

### Gradual Rollback

For non-critical issues:

```toml
# 1. Make new feature opt-in again
[features]
default = []  # Remove new feature from default
cache-trait = []  # Keep as optional

# 2. Update documentation
# docs/ROLLBACK.md

# 3. Notify team
# Slack/email announcement
```

### Data Migration Rollback

If data was migrated:

```rust
// Implement reverse migration
pub async fn rollback_cache_data(
    old_cache: &OldCache,
    new_cache: &Arc<dyn CacheStorage>,
) -> Result<()> {
    // Copy data back from new to old
    let keys = new_cache.list_keys().await?;

    for key in keys {
        if let Some(value) = new_cache.get(&key).await? {
            old_cache.set(&key, &value).await?;
        }
    }

    Ok(())
}
```

---

## Testing Checklist

Before completing any migration:

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Performance benchmarks show no regression
- [ ] Memory usage is stable
- [ ] Documentation updated
- [ ] Migration guide reviewed
- [ ] Rollback procedure tested
- [ ] Feature flag tested in both states
- [ ] Code review completed
- [ ] Team trained on new code

---

## Timeline

**Week 0-1: CacheStorage + SchemaStore**
- Day 1-2: Create trait + Redis adapter
- Day 3-4: Create in-memory adapter
- Day 5: Update first consumer
- Day 6-7: Testing and validation

**Week 1-2: Robots.txt Split**
- Day 1-2: Create pure parser layer
- Day 3-4: Create HTTP fetcher layer
- Day 5: Update spider integration
- Day 6-7: Testing and migration

**Week 2-3: Memory Manager**
- Day 1-3: Extend unified manager
- Day 4-5: Migrate API resource manager
- Day 6-7: Update all allocators

**Week 3-4: Pipeline Core**
- Day 1-5: Create core + migrate pipelines
- Day 6-7: Testing and validation

---

**Document Maintainer:** System Architect
**Last Review:** 2025-01-08
**Next Review:** After each migration phase
