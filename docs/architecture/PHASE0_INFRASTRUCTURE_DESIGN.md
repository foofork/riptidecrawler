# Phase 0 Infrastructure Consolidation - Architecture Design

**Document Version:** 1.0
**Date:** 2025-01-08
**Aligned With:** Sprint 0.1-0.5 Roadmap
**Status:** Design Complete - Pending Implementation

## Executive Summary

This document defines clean architectural patterns for consolidating duplicate infrastructure code across the Riptide workspace while maintaining separation of concerns and hexagonal architecture principles.

**Core Principles:**
1. **Hexagonal Architecture** - Ports (traits) & Adapters (implementations)
2. **Dependency Inversion** - Depend on abstractions, not concrete types
3. **Single Responsibility** - Pure logic ≠ I/O layer
4. **DRY** - One canonical implementation per component
5. **Clean Separation** - No async/http in pure utility code

---

## Design 1: Robots.txt Split Architecture (Sprint 0.1.1)

### Problem Analysis

**Current State:**
- `riptide-fetch/src/robots.rs` (482 lines) contains mixed concerns:
  - Pure parsing logic (robots.txt format interpretation)
  - HTTP fetching with retry logic
  - Rate limiting with token bucket
  - Caching with TTL
  - Async/await throughout

**Issues:**
- Violation of SRP (Single Responsibility Principle)
- Cannot unit test parsing without HTTP stack
- Hard to mock for testing
- Retry logic coupled to parsing

### Proposed Architecture

#### Layer 1: Pure Logic Layer (riptide-utils/src/robots.rs)

**Location:** `/workspaces/eventmesh/crates/riptide-utils/src/robots.rs`

**Responsibilities:**
- Parse robots.txt content (text → structured data)
- Check if URL is allowed by rules
- Extract crawl-delay directives
- Pure, synchronous functions only

```rust
// Pure trait - no async, no I/O
pub trait RobotsParser: Send + Sync {
    /// Parse robots.txt content into structured rules
    fn parse(&self, content: &str, user_agent: &str) -> Result<RobotRules>;

    /// Check if path is allowed according to rules
    fn is_allowed(&self, rules: &RobotRules, path: &str) -> bool;

    /// Extract crawl delay for user agent
    fn extract_crawl_delay(&self, content: &str, user_agent: &str) -> Option<f64>;
}

#[derive(Debug, Clone)]
pub struct RobotRules {
    pub user_agent: String,
    pub allowed_paths: Vec<String>,
    pub disallowed_paths: Vec<String>,
    pub crawl_delay: Option<f64>,
}

pub struct DefaultRobotsParser {
    config: RobotsConfig,
}

impl RobotsParser for DefaultRobotsParser {
    fn parse(&self, content: &str, user_agent: &str) -> Result<RobotRules> {
        // Pure parsing logic using robotstxt::DefaultMatcher
        // No async, no HTTP, no I/O
    }

    fn is_allowed(&self, rules: &RobotRules, path: &str) -> bool {
        // Pure boolean logic
    }
}
```

**Testing Benefits:**
```rust
#[test]
fn test_robots_parsing() {
    let parser = DefaultRobotsParser::new();
    let content = "User-agent: *\nDisallow: /admin\n";
    let rules = parser.parse(content, "MyBot").unwrap();
    assert!(!parser.is_allowed(&rules, "/admin/panel"));
    assert!(parser.is_allowed(&rules, "/public"));
}
```

#### Layer 2: HTTP/Retry Layer (riptide-reliability/src/robots_fetcher.rs)

**Location:** `/workspaces/eventmesh/crates/riptide-reliability/src/robots_fetcher.rs`

**Responsibilities:**
- Fetch robots.txt via HTTP
- Circuit breaker integration
- Retry logic with exponential backoff
- Timeout handling
- Use ReliableHttpClient from riptide-reliability

```rust
#[async_trait]
pub trait RobotsFetcher: Send + Sync {
    /// Fetch robots.txt from URL with retry/circuit breaker
    async fn fetch_robots_txt(&self, base_url: &str) -> Result<String>;

    /// Check if URL is allowed (combines fetch + parse)
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
        // Check cache first
        if let Some(cached) = self.cache.get(base_url) {
            if !cached.is_expired() {
                return Ok(cached.content.clone());
            }
        }

        // Use ReliableHttpClient (already has circuit breaker + retry)
        let robots_url = format!("{}/robots.txt", base_url);
        let response = self.http_client
            .get(&robots_url)
            .send()
            .await?;

        let content = response.text().await?;

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

        // Fetch robots.txt (uses cache + circuit breaker)
        let content = self.fetch_robots_txt(&base_url).await?;

        // Parse using pure parser
        let rules = self.parser.parse(&content, &self.config.user_agent)?;

        // Check if allowed (pure logic)
        Ok(self.parser.is_allowed(&rules, parsed_url.path()))
    }
}
```

### Migration Strategy

**Phase 1: Create Pure Layer**
1. Create `riptide-utils/src/robots.rs` with pure parsing logic
2. Extract `RobotRules` struct and parsing from existing code
3. Write comprehensive unit tests (no async)

**Phase 2: Create HTTP Layer**
1. Create `riptide-reliability/src/robots_fetcher.rs`
2. Integrate with existing `ReliableHttpClient`
3. Move caching logic from riptide-fetch
4. Write integration tests with mock HTTP server

**Phase 3: Update Consumers**
1. Update `riptide-spider` to use `ReliableRobotsFetcher`
2. Update `riptide-fetch` to use new architecture
3. Remove old `riptide-fetch/src/robots.rs`
4. Update tests to use new structure

**Feature Flag:**
```toml
[dependencies]
riptide-utils = { version = "0.1", features = ["robots-parser"] }
riptide-reliability = { version = "0.1", features = ["robots-fetcher"] }
```

---

## Design 2: CacheStorage Trait (Sprint 0.1.3)

### Problem Analysis

**Current State:**
- `riptide-cache/src/redis.rs` - Direct Redis implementation
- `riptide-persistence/src/cache.rs` - Another Redis implementation
- Both tightly coupled to Redis
- Hard to test without Redis instance
- Cannot swap backends

**Issues:**
- Violates Dependency Inversion Principle
- Code duplication
- Testing requires Docker/Redis
- No flexibility for different backends

### Proposed Architecture

#### Port Interface (riptide-types/src/ports/cache.rs)

**Location:** `/workspaces/eventmesh/crates/riptide-types/src/ports/cache.rs`

```rust
use async_trait::async_trait;
use std::time::Duration;

/// Cache storage port - backend-agnostic caching interface
///
/// This trait defines the contract for cache implementations.
/// Implementations can be Redis, in-memory, disk-based, or distributed.
#[async_trait]
pub trait CacheStorage: Send + Sync {
    /// Get value from cache
    ///
    /// Returns None if key doesn't exist or is expired
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Set value in cache with TTL
    ///
    /// # Arguments
    /// * `key` - Cache key
    /// * `value` - Value to store (raw bytes)
    /// * `ttl` - Optional TTL, uses default if None
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()>;

    /// Delete value from cache
    async fn delete(&self, key: &str) -> Result<bool>;

    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Batch get operation (optional optimization)
    async fn get_batch(&self, keys: &[String]) -> Result<Vec<Option<Vec<u8>>>> {
        // Default implementation - can be optimized by specific backends
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.get(key).await?);
        }
        Ok(results)
    }

    /// Batch set operation (optional optimization)
    async fn set_batch(
        &self,
        entries: Vec<(String, Vec<u8>)>,
        ttl: Option<Duration>,
    ) -> Result<()> {
        // Default implementation - can be optimized by specific backends
        for (key, value) in entries {
            self.set(&key, &value, ttl).await?;
        }
        Ok(())
    }

    /// Clear all cache entries (use with caution)
    async fn clear(&self) -> Result<u64>;

    /// Get cache statistics
    async fn stats(&self) -> Result<CacheStats>;
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_keys: u64,
    pub memory_usage_bytes: u64,
    pub hit_rate: f64,
    pub miss_rate: f64,
}
```

#### Redis Adapter (riptide-cache/src/adapters/redis.rs)

**Location:** `/workspaces/eventmesh/crates/riptide-cache/src/adapters/redis.rs`

```rust
use riptide_types::ports::cache::CacheStorage;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};

pub struct RedisCache {
    conn: MultiplexedConnection,
    default_ttl: Duration,
    key_prefix: String,
}

impl RedisCache {
    pub async fn new(redis_url: &str, default_ttl: Duration) -> Result<Self> {
        let client = Client::open(redis_url)?;
        let conn = client.get_multiplexed_tokio_connection().await?;

        Ok(Self {
            conn,
            default_ttl,
            key_prefix: "riptide".to_string(),
        })
    }

    fn make_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }
}

#[async_trait]
impl CacheStorage for RedisCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let cache_key = self.make_key(key);
        let mut conn = self.conn.clone();
        let result: Option<Vec<u8>> = conn.get(&cache_key).await?;
        Ok(result)
    }

    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()> {
        let cache_key = self.make_key(key);
        let ttl_secs = ttl.unwrap_or(self.default_ttl).as_secs();
        let mut conn = self.conn.clone();
        conn.set_ex::<_, _, ()>(&cache_key, value, ttl_secs).await?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let cache_key = self.make_key(key);
        let mut conn = self.conn.clone();
        let deleted: u64 = conn.del(&cache_key).await?;
        Ok(deleted > 0)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let cache_key = self.make_key(key);
        let mut conn = self.conn.clone();
        let exists: bool = conn.exists(&cache_key).await?;
        Ok(exists)
    }

    // Optimized batch operations using Redis pipelining
    async fn get_batch(&self, keys: &[String]) -> Result<Vec<Option<Vec<u8>>>> {
        let cache_keys: Vec<String> = keys.iter()
            .map(|k| self.make_key(k))
            .collect();

        let mut conn = self.conn.clone();
        let results: Vec<Option<Vec<u8>>> = conn.get(&cache_keys).await?;
        Ok(results)
    }

    async fn set_batch(
        &self,
        entries: Vec<(String, Vec<u8>)>,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let ttl_secs = ttl.unwrap_or(self.default_ttl).as_secs();
        let mut conn = self.conn.clone();
        let mut pipe = redis::pipe();

        for (key, value) in entries {
            let cache_key = self.make_key(&key);
            pipe.set_ex(&cache_key, value, ttl_secs);
        }

        pipe.query_async::<()>(&mut conn).await?;
        Ok(())
    }

    async fn clear(&self) -> Result<u64> {
        let pattern = format!("{}:*", self.key_prefix);
        let mut conn = self.conn.clone();

        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await?;

        if keys.is_empty() {
            return Ok(0);
        }

        let deleted: u64 = conn.del(&keys).await?;
        Ok(deleted)
    }

    async fn stats(&self) -> Result<CacheStats> {
        let pattern = format!("{}:*", self.key_prefix);
        let mut conn = self.conn.clone();

        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await?;

        let info: String = redis::cmd("INFO")
            .arg("memory")
            .query_async(&mut conn)
            .await?;

        let memory_usage = parse_redis_memory(&info);

        Ok(CacheStats {
            total_keys: keys.len() as u64,
            memory_usage_bytes: memory_usage,
            hit_rate: 0.0, // Would need separate metrics tracking
            miss_rate: 0.0,
        })
    }
}

fn parse_redis_memory(info: &str) -> u64 {
    for line in info.lines() {
        if line.starts_with("used_memory:") {
            if let Some(value) = line.split(':').nth(1) {
                return value.trim().parse().unwrap_or(0);
            }
        }
    }
    0
}
```

#### In-Memory Adapter (riptide-cache/src/adapters/memory.rs)

**Location:** `/workspaces/eventmesh/crates/riptide-cache/src/adapters/memory.rs`

```rust
use riptide_types::ports::cache::CacheStorage;
use dashmap::DashMap;
use std::time::{Duration, Instant};

struct CacheEntry {
    data: Vec<u8>,
    expires_at: Instant,
}

pub struct InMemoryCache {
    storage: DashMap<String, CacheEntry>,
    default_ttl: Duration,
}

impl InMemoryCache {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            storage: DashMap::new(),
            default_ttl,
        }
    }

    fn is_expired(&self, entry: &CacheEntry) -> bool {
        Instant::now() > entry.expires_at
    }
}

#[async_trait]
impl CacheStorage for InMemoryCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        if let Some(entry) = self.storage.get(key) {
            if self.is_expired(&entry) {
                drop(entry);
                self.storage.remove(key);
                return Ok(None);
            }
            return Ok(Some(entry.data.clone()));
        }
        Ok(None)
    }

    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()> {
        let ttl = ttl.unwrap_or(self.default_ttl);
        let entry = CacheEntry {
            data: value.to_vec(),
            expires_at: Instant::now() + ttl,
        };
        self.storage.insert(key.to_string(), entry);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        Ok(self.storage.remove(key).is_some())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        if let Some(entry) = self.storage.get(key) {
            if self.is_expired(&entry) {
                drop(entry);
                self.storage.remove(key);
                return Ok(false);
            }
            return Ok(true);
        }
        Ok(false)
    }

    async fn clear(&self) -> Result<u64> {
        let count = self.storage.len() as u64;
        self.storage.clear();
        Ok(count)
    }

    async fn stats(&self) -> Result<CacheStats> {
        // Clean up expired entries first
        self.storage.retain(|_, entry| !self.is_expired(entry));

        let total_keys = self.storage.len() as u64;
        let memory_usage: usize = self.storage
            .iter()
            .map(|entry| entry.data.len())
            .sum();

        Ok(CacheStats {
            total_keys,
            memory_usage_bytes: memory_usage as u64,
            hit_rate: 0.0,
            miss_rate: 0.0,
        })
    }
}
```

### Usage Pattern

```rust
// Dependency injection - use trait, not concrete type
pub struct ExtractionService {
    cache: Arc<dyn CacheStorage>,
}

impl ExtractionService {
    pub fn new(cache: Arc<dyn CacheStorage>) -> Self {
        Self { cache }
    }

    pub async fn extract_with_cache(&self, url: &str) -> Result<String> {
        // Check cache
        if let Some(cached) = self.cache.get(url).await? {
            return Ok(String::from_utf8(cached)?);
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

// Production: Use Redis
let redis_cache = Arc::new(RedisCache::new("redis://localhost", Duration::from_secs(3600)).await?);
let service = ExtractionService::new(redis_cache);

// Testing: Use in-memory
let mem_cache = Arc::new(InMemoryCache::new(Duration::from_secs(60)));
let service = ExtractionService::new(mem_cache);
```

### Migration Strategy

1. **Create Port** (`riptide-types/src/ports/cache.rs`)
2. **Create Redis Adapter** (`riptide-cache/src/adapters/redis.rs`)
3. **Create In-Memory Adapter** (for testing)
4. **Update Consumers** (inject `Arc<dyn CacheStorage>` instead of `RedisClient`)
5. **Remove Old Implementations** (consolidate duplicate code)

### Testing Benefits

```rust
#[tokio::test]
async fn test_extraction_with_mock_cache() {
    // No Redis needed!
    let cache = Arc::new(InMemoryCache::new(Duration::from_secs(60)));
    let service = ExtractionService::new(cache);

    let result = service.extract_with_cache("https://example.com").await.unwrap();
    assert_eq!(result, "expected content");
}
```

---

## Design 3: Unified Memory Manager (Sprint 0.1.2)

### Problem Analysis

**Current State:**
- `riptide-pool/src/memory_manager.rs` - WASM pool management (1122 lines)
- `riptide-api/src/resource_manager/memory_manager.rs` - API-level memory tracking (988 lines)
- Both track memory but with different approaches
- Duplication of leak detection logic
- No unified view of system memory

**Issues:**
- Cannot track memory across different resource types
- Duplicate monitoring logic
- Separate metrics for different resource types
- Hard to get unified memory pressure view

### Proposed Architecture

#### Resource Type Enumeration

```rust
/// Types of resources that consume memory
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ResourceType {
    /// WASM instance with component info
    WasmInstance {
        component: Arc<Component>,
        instance_id: String,
    },
    /// HTTP connection pool
    HttpConnection {
        pool_name: String,
        pool_size: usize,
    },
    /// Browser session (Chromium/WebKit)
    BrowserSession {
        session_id: String,
        browser_type: BrowserType,
    },
    /// PDF processor instance
    PdfProcessor {
        processor_id: String,
        slots_used: usize,
    },
    /// Custom resource type
    Custom {
        name: String,
        metadata: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum BrowserType {
    Chromium,
    WebKit,
}
```

#### Unified Memory Manager

**Location:** `/workspaces/eventmesh/crates/riptide-pool/src/memory_manager.rs` (enhanced)

```rust
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tokio::sync::RwLock;

/// Unified memory manager tracking all resource types
pub struct UnifiedMemoryManager {
    /// WASM pool manager (existing)
    wasm_pool: Option<WasmPoolManager>,

    /// HTTP resource tracking (NEW)
    http_resources: Arc<HttpResourceTracker>,

    /// Browser session tracking (NEW)
    browser_sessions: Arc<BrowserSessionTracker>,

    /// PDF processor tracking (NEW)
    pdf_processors: Arc<PdfProcessorTracker>,

    /// Custom resource tracking (NEW)
    custom_resources: Arc<CustomResourceTracker>,

    /// Unified metrics
    metrics: Arc<MemoryStats>,

    /// Configuration
    config: MemoryManagerConfig,
}

impl UnifiedMemoryManager {
    pub async fn new(config: MemoryManagerConfig) -> Result<Self> {
        Ok(Self {
            wasm_pool: if config.enable_wasm_pool {
                Some(WasmPoolManager::new(config.wasm_config.clone()).await?)
            } else {
                None
            },
            http_resources: Arc::new(HttpResourceTracker::new()),
            browser_sessions: Arc::new(BrowserSessionTracker::new()),
            pdf_processors: Arc::new(PdfProcessorTracker::new()),
            custom_resources: Arc::new(CustomResourceTracker::new()),
            metrics: Arc::new(MemoryStats::default()),
            config,
        })
    }

    /// Track resource allocation
    pub async fn track_allocation(&self, resource: ResourceType, size_mb: u64) -> Result<()> {
        match resource {
            ResourceType::WasmInstance { .. } => {
                if let Some(wasm_pool) = &self.wasm_pool {
                    wasm_pool.track_allocation(size_mb).await?;
                }
            }
            ResourceType::HttpConnection { ref pool_name, pool_size } => {
                self.http_resources.track_allocation(pool_name, pool_size, size_mb).await?;
            }
            ResourceType::BrowserSession { ref session_id, browser_type } => {
                self.browser_sessions.track_allocation(session_id, browser_type, size_mb).await?;
            }
            ResourceType::PdfProcessor { ref processor_id, slots_used } => {
                self.pdf_processors.track_allocation(processor_id, slots_used, size_mb).await?;
            }
            ResourceType::Custom { ref name, .. } => {
                self.custom_resources.track_allocation(name, size_mb).await?;
            }
        }

        // Update unified metrics
        self.metrics.total_allocated_mb.fetch_add(size_mb, Ordering::Relaxed);

        Ok(())
    }

    /// Track resource deallocation
    pub async fn track_deallocation(&self, resource: ResourceType, size_mb: u64) -> Result<()> {
        match resource {
            ResourceType::WasmInstance { .. } => {
                if let Some(wasm_pool) = &self.wasm_pool {
                    wasm_pool.track_deallocation(size_mb).await?;
                }
            }
            ResourceType::HttpConnection { ref pool_name, .. } => {
                self.http_resources.track_deallocation(pool_name, size_mb).await?;
            }
            ResourceType::BrowserSession { ref session_id, .. } => {
                self.browser_sessions.track_deallocation(session_id, size_mb).await?;
            }
            ResourceType::PdfProcessor { ref processor_id, .. } => {
                self.pdf_processors.track_deallocation(processor_id, size_mb).await?;
            }
            ResourceType::Custom { ref name, .. } => {
                self.custom_resources.track_deallocation(name, size_mb).await?;
            }
        }

        // Update unified metrics
        self.metrics.total_allocated_mb.fetch_sub(size_mb, Ordering::Relaxed);

        Ok(())
    }

    /// Get unified memory statistics
    pub async fn get_stats(&self) -> UnifiedMemoryStats {
        let wasm_stats = if let Some(wasm_pool) = &self.wasm_pool {
            Some(wasm_pool.stats().await)
        } else {
            None
        };

        let http_stats = self.http_resources.stats().await;
        let browser_stats = self.browser_sessions.stats().await;
        let pdf_stats = self.pdf_processors.stats().await;
        let custom_stats = self.custom_resources.stats().await;

        UnifiedMemoryStats {
            total_allocated_mb: self.metrics.total_allocated_mb.load(Ordering::Relaxed),
            wasm: wasm_stats,
            http: http_stats,
            browser: browser_stats,
            pdf: pdf_stats,
            custom: custom_stats,
        }
    }

    /// Check overall memory pressure
    pub fn is_under_pressure(&self) -> bool {
        let current_usage = self.metrics.total_allocated_mb.load(Ordering::Relaxed);
        let memory_pressure = (current_usage as f64 / self.config.max_total_memory_mb as f64) * 100.0;
        memory_pressure > self.config.memory_pressure_threshold
    }
}
```

#### HTTP Resource Tracker

```rust
pub struct HttpResourceTracker {
    pools: DashMap<String, HttpPoolStats>,
    total_memory_mb: AtomicU64,
}

struct HttpPoolStats {
    pool_name: String,
    pool_size: usize,
    memory_mb: u64,
    connection_count: usize,
    last_updated: Instant,
}

impl HttpResourceTracker {
    pub fn new() -> Self {
        Self {
            pools: DashMap::new(),
            total_memory_mb: AtomicU64::new(0),
        }
    }

    pub async fn track_allocation(
        &self,
        pool_name: &str,
        pool_size: usize,
        size_mb: u64,
    ) -> Result<()> {
        self.pools
            .entry(pool_name.to_string())
            .and_modify(|stats| {
                stats.memory_mb += size_mb;
                stats.connection_count += 1;
                stats.last_updated = Instant::now();
            })
            .or_insert_with(|| HttpPoolStats {
                pool_name: pool_name.to_string(),
                pool_size,
                memory_mb: size_mb,
                connection_count: 1,
                last_updated: Instant::now(),
            });

        self.total_memory_mb.fetch_add(size_mb, Ordering::Relaxed);
        Ok(())
    }

    pub async fn track_deallocation(
        &self,
        pool_name: &str,
        size_mb: u64,
    ) -> Result<()> {
        if let Some(mut stats) = self.pools.get_mut(pool_name) {
            stats.memory_mb = stats.memory_mb.saturating_sub(size_mb);
            stats.connection_count = stats.connection_count.saturating_sub(1);
            stats.last_updated = Instant::now();
        }

        self.total_memory_mb.fetch_sub(size_mb, Ordering::Relaxed);
        Ok(())
    }

    pub async fn stats(&self) -> HttpResourceStats {
        HttpResourceStats {
            total_pools: self.pools.len(),
            total_connections: self.pools.iter().map(|s| s.connection_count).sum(),
            total_memory_mb: self.total_memory_mb.load(Ordering::Relaxed),
        }
    }
}
```

#### Browser Session Tracker

```rust
pub struct BrowserSessionTracker {
    sessions: DashMap<String, BrowserSessionStats>,
    total_memory_mb: AtomicU64,
}

struct BrowserSessionStats {
    session_id: String,
    browser_type: BrowserType,
    memory_mb: u64,
    created_at: Instant,
    last_activity: Instant,
}

impl BrowserSessionTracker {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
            total_memory_mb: AtomicU64::new(0),
        }
    }

    pub async fn track_allocation(
        &self,
        session_id: &str,
        browser_type: BrowserType,
        size_mb: u64,
    ) -> Result<()> {
        self.sessions
            .entry(session_id.to_string())
            .and_modify(|stats| {
                stats.memory_mb += size_mb;
                stats.last_activity = Instant::now();
            })
            .or_insert_with(|| BrowserSessionStats {
                session_id: session_id.to_string(),
                browser_type,
                memory_mb: size_mb,
                created_at: Instant::now(),
                last_activity: Instant::now(),
            });

        self.total_memory_mb.fetch_add(size_mb, Ordering::Relaxed);
        Ok(())
    }

    pub async fn track_deallocation(
        &self,
        session_id: &str,
        size_mb: u64,
    ) -> Result<()> {
        if let Some(mut stats) = self.sessions.get_mut(session_id) {
            stats.memory_mb = stats.memory_mb.saturating_sub(size_mb);
            stats.last_activity = Instant::now();
        }

        self.total_memory_mb.fetch_sub(size_mb, Ordering::Relaxed);
        Ok(())
    }

    pub async fn stats(&self) -> BrowserResourceStats {
        let chromium_count = self.sessions.iter()
            .filter(|s| matches!(s.browser_type, BrowserType::Chromium))
            .count();

        let webkit_count = self.sessions.iter()
            .filter(|s| matches!(s.browser_type, BrowserType::WebKit))
            .count();

        BrowserResourceStats {
            total_sessions: self.sessions.len(),
            chromium_sessions: chromium_count,
            webkit_sessions: webkit_count,
            total_memory_mb: self.total_memory_mb.load(Ordering::Relaxed),
        }
    }
}
```

### Migration Strategy

**Phase 1: Extend Existing** (riptide-pool)
1. Add `ResourceType` enum
2. Add resource-specific trackers
3. Enhance `UnifiedMemoryManager`

**Phase 2: Integrate API Layer**
1. Update `riptide-api` to use `UnifiedMemoryManager`
2. Remove duplicate tracking logic
3. Consolidate leak detection

**Phase 3: Update Consumers**
1. Update WASM pool to report via unified manager
2. Update HTTP pools to report via unified manager
3. Update browser management to report via unified manager

---

## Design 4: Pipeline Common Core (Sprint 0.2)

### Problem Analysis

**Current State:**
Found 15+ pipeline implementations:
- `riptide-api/src/pipeline.rs`
- `riptide-api/src/pipeline_dual.rs`
- `riptide-api/src/pipeline_enhanced.rs`
- `riptide-api/src/streaming/pipeline.rs`
- `riptide-api/src/strategies_pipeline.rs`
- `riptide-facade/src/facades/pipeline.rs`

**Common Patterns Identified:**
1. All have: request validation → extraction → processing → response
2. All use: circuit breakers, retries, timeouts
3. All track: metrics, errors, timing
4. Variance: streaming vs batch, strategies vs fixed, dual-mode

### Proposed Architecture

#### Core Pipeline Trait

**Location:** `/workspaces/eventmesh/crates/riptide-types/src/ports/pipeline.rs`

```rust
#[async_trait]
pub trait Pipeline: Send + Sync {
    type Request;
    type Response;

    /// Execute pipeline with full lifecycle
    async fn execute(&self, request: Self::Request) -> Result<Self::Response>;

    /// Validate request before processing
    async fn validate(&self, request: &Self::Request) -> Result<()>;

    /// Pre-processing hook
    async fn pre_process(&self, request: &Self::Request) -> Result<()> {
        Ok(())
    }

    /// Post-processing hook
    async fn post_process(&self, response: &Self::Response) -> Result<()> {
        Ok(())
    }

    /// Error handling hook
    async fn on_error(&self, error: &anyhow::Error) -> Result<()> {
        Ok(())
    }
}
```

#### Pipeline Core Implementation

**Location:** `/workspaces/eventmesh/crates/riptide-pipeline-core/src/lib.rs` (NEW CRATE)

```rust
/// Common pipeline infrastructure
pub struct PipelineCore {
    config: PipelineConfig,
    metrics: Arc<PipelineMetrics>,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl PipelineCore {
    /// Execute with common infrastructure (retry, circuit breaker, metrics)
    pub async fn execute_with_infrastructure<F, T>(
        &self,
        name: &str,
        operation: F,
    ) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        let start = Instant::now();

        // Check circuit breaker
        if self.circuit_breaker.is_open() {
            return Err(anyhow!("Circuit breaker open for {}", name));
        }

        // Execute with retry
        let result = self.retry_operation(operation).await;

        // Record metrics
        let duration = start.elapsed();
        self.metrics.record_operation(name, duration, result.is_ok()).await;

        // Update circuit breaker
        match &result {
            Ok(_) => self.circuit_breaker.record_success().await,
            Err(_) => self.circuit_breaker.record_failure().await,
        }

        result
    }

    async fn retry_operation<F, T>(&self, operation: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        // Retry logic with exponential backoff
        // Implementation from riptide-reliability
    }
}
```

#### Feature-Based Variants

```rust
// Feature flags for optional behaviors
#[cfg(feature = "streaming")]
pub mod streaming {
    pub struct StreamingPipeline {
        core: PipelineCore,
        stream_config: StreamConfig,
    }
}

#[cfg(feature = "dual-mode")]
pub mod dual {
    pub struct DualModePipeline {
        core: PipelineCore,
        primary: Box<dyn Pipeline>,
        fallback: Box<dyn Pipeline>,
    }
}

#[cfg(feature = "strategies")]
pub mod strategies {
    pub struct StrategyPipeline {
        core: PipelineCore,
        strategies: Vec<Box<dyn ExtractionStrategy>>,
    }
}
```

### Migration Strategy

1. **Create Core Crate** (`riptide-pipeline-core`)
2. **Extract Common Logic** from existing pipelines
3. **Convert Variants** to use core + feature flags
4. **Deprecate Old Pipelines** gradually
5. **Remove Duplicates** after migration complete

---

## Design 5: SchemaStore Runtime Interface (Sprint 0.5.1)

### Problem Analysis

**Future Requirement:**
- Need runtime JSON schemas for validation
- May back with Redis or S3 later
- Should be pluggable
- Currently no implementation exists

### Proposed Architecture

#### Port Interface

**Location:** `/workspaces/eventmesh/crates/riptide-types/src/ports/schema_store.rs`

```rust
use serde_json::Value;

#[async_trait]
pub trait SchemaStore: Send + Sync {
    /// Store a JSON schema
    async fn put(&self, schema_uri: &str, schema: Value) -> Result<()>;

    /// Retrieve a JSON schema
    async fn get(&self, schema_uri: &str) -> Result<Option<Value>>;

    /// Validate data against a schema
    async fn validate(&self, schema_uri: &str, data: &Value) -> Result<bool>;

    /// List all available schemas
    async fn list(&self) -> Result<Vec<String>>;

    /// Delete a schema
    async fn delete(&self, schema_uri: &str) -> Result<bool>;
}
```

#### In-Memory Stub

**Location:** `/workspaces/eventmesh/crates/riptide-persistence/src/schema_store/memory.rs`

```rust
use dashmap::DashMap;

pub struct InMemorySchemaStore {
    schemas: DashMap<String, Value>,
}

impl InMemorySchemaStore {
    pub fn new() -> Self {
        Self {
            schemas: DashMap::new(),
        }
    }
}

#[async_trait]
impl SchemaStore for InMemorySchemaStore {
    async fn put(&self, schema_uri: &str, schema: Value) -> Result<()> {
        self.schemas.insert(schema_uri.to_string(), schema);
        Ok(())
    }

    async fn get(&self, schema_uri: &str) -> Result<Option<Value>> {
        Ok(self.schemas.get(schema_uri).map(|v| v.clone()))
    }

    async fn validate(&self, schema_uri: &str, data: &Value) -> Result<bool> {
        let schema = self.get(schema_uri).await?
            .ok_or_else(|| anyhow!("Schema not found: {}", schema_uri))?;

        // Use jsonschema crate for validation
        let compiled = jsonschema::JSONSchema::compile(&schema)
            .map_err(|e| anyhow!("Invalid schema: {}", e))?;

        Ok(compiled.is_valid(data))
    }

    async fn list(&self) -> Result<Vec<String>> {
        Ok(self.schemas.iter().map(|entry| entry.key().clone()).collect())
    }

    async fn delete(&self, schema_uri: &str) -> Result<bool> {
        Ok(self.schemas.remove(schema_uri).is_some())
    }
}
```

#### Future Redis Implementation

```rust
// Future implementation when needed
#[cfg(feature = "redis-schemas")]
pub struct RedisSchemaStore {
    cache: Arc<dyn CacheStorage>,
    key_prefix: String,
}
```

#### Future S3 Implementation

```rust
// Future implementation when needed
#[cfg(feature = "s3-schemas")]
pub struct S3SchemaStore {
    bucket: String,
    client: S3Client,
}
```

---

## Risk Assessment

### Design 1: Robots.txt Split
- **Risk:** LOW
- **Impact:** Medium (enables better testing)
- **Mitigation:** Phase rollout, feature flags

### Design 2: CacheStorage Trait
- **Risk:** MEDIUM
- **Impact:** HIGH (critical for Redis scoping)
- **Mitigation:** Comprehensive testing with both backends

### Design 3: Unified Memory Manager
- **Risk:** MEDIUM
- **Impact:** HIGH (affects resource management)
- **Mitigation:** Gradual integration, maintain backward compatibility

### Design 4: Pipeline Common Core
- **Risk:** HIGH (many files to migrate)
- **Impact:** VERY HIGH (touches most of API layer)
- **Mitigation:** Create new crate first, parallel run, gradual deprecation

### Design 5: SchemaStore
- **Risk:** LOW (future feature)
- **Impact:** LOW (not used yet)
- **Mitigation:** Start with stub, defer complex backends

---

## Implementation Priority

**Week 0-1:** Foundations
1. Design 2: CacheStorage Trait (enables Redis scoping)
2. Design 5: SchemaStore Stub (prepare for future)

**Week 1-2:** Resource Management
3. Design 1: Robots.txt Split (clean up fetch layer)
4. Design 3: Unified Memory Manager (consolidate tracking)

**Week 2-3:** Pipeline Consolidation
5. Design 4: Pipeline Common Core (most complex)

---

## ADR (Architecture Decision Records)

### ADR-001: Use Hexagonal Architecture

**Context:** Need to decouple business logic from infrastructure

**Decision:** Use ports (traits) and adapters (implementations) pattern

**Consequences:**
- ✅ Easy to test with mocks
- ✅ Can swap implementations
- ✅ Clear separation of concerns
- ❌ More boilerplate code
- ❌ Learning curve for team

### ADR-002: Dependency Injection via Traits

**Context:** Need flexible, testable dependencies

**Decision:** Inject `Arc<dyn Trait>` instead of concrete types

**Consequences:**
- ✅ Enables mocking for tests
- ✅ Runtime flexibility
- ❌ Slight runtime overhead (vtable dispatch)
- ❌ More complex type signatures

### ADR-003: Split Pure Logic from I/O

**Context:** Testing requires separating sync/async, pure/impure

**Decision:** Create separate layers for pure logic vs I/O operations

**Consequences:**
- ✅ Pure functions easily testable
- ✅ No async in utility code
- ✅ Can unit test without infrastructure
- ❌ More modules to maintain
- ❌ Requires discipline to maintain separation

---

## Metrics & Success Criteria

**Code Quality:**
- Clippy warnings: 0 (currently at 0 ✅)
- Test coverage: >80% for new code
- Duplicate code: <5% (measured by code-analyzer)

**Performance:**
- Cache access: <5ms (p95)
- Memory overhead: <10% vs current
- No regression in throughput

**Developer Experience:**
- New test setup time: <30s
- Mock creation complexity: LOW
- Documentation completeness: 100%

---

## Next Steps

1. **Review this design** with team
2. **Create feature branches** for each design
3. **Implement in priority order** (CacheStorage → SchemaStore → Robots → Memory → Pipeline)
4. **Write comprehensive tests** for each layer
5. **Document migration guides** for consumers
6. **Deprecate old code** gradually

---

**Document Maintainer:** System Architect
**Last Review:** 2025-01-08
**Next Review:** After Sprint 0.1 completion
