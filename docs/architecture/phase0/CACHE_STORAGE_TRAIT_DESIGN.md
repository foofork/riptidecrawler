# CacheStorage Trait Design - Phase 0 Sprint 0.1.3

**Version:** 1.0
**Date:** 2025-11-08
**Author:** System Architect
**Status:** ✅ Ready for Implementation

---

## Executive Summary

Define a **port interface** in `riptide-types` that enables **dependency inversion** for caching, allowing Redis to be an **implementation detail** rather than a direct dependency in 4+ crates. This follows **Hexagonal Architecture** principles and reduces coupling.

---

## 1. Problem Statement

### Current Architecture Issues

```
❌ CURRENT (Tight Coupling):
  riptide-utils ━━━┓
  riptide-api  ━━━━╋━━> redis crate (direct dependency)
  riptide-performance ━┛

Problems:
- 6 crates directly depend on Redis
- Cannot swap cache backends without changing 6+ crates
- Testing requires Redis infrastructure
- Violates Dependency Inversion Principle
```

### Desired Architecture

```
✅ TARGET (Dependency Inversion):
  riptide-types (Port)
       │
       ├──> CacheStorage trait (abstract interface)
       │
       └──> Used by:
            ├─ riptide-api
            ├─ riptide-utils
            └─ riptide-performance

  riptide-cache (Adapter)
       └──> RedisCacheStorage (concrete implementation)
              └──> redis crate (dependency scoped here)
```

---

## 2. Trait Specification

### 2.1 Core Trait Definition

Location: `crates/riptide-types/src/ports/cache.rs` (NEW)

```rust
//! Cache storage port - Domain abstraction for persistent caching
//!
//! This trait defines the contract for cache storage implementations.
//! It is intentionally minimal to support multiple backends (Redis, DragonflyDB, etc.)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Result type for cache operations
pub type CacheResult<T> = Result<T, CacheError>;

/// Cache operation errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum CacheError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Serialization failed: {0}")]
    SerializationError(String),

    #[error("Operation timeout after {0:?}")]
    Timeout(Duration),

    #[error("Storage backend error: {0}")]
    BackendError(String),

    #[error("Invalid key format: {0}")]
    InvalidKey(String),

    #[error("Value too large: {size} bytes exceeds max {max}")]
    ValueTooLarge { size: usize, max: usize },
}

/// Port interface for cache storage backends
///
/// # Design Principles
///
/// 1. **Dependency Inversion**: Domain depends on abstraction, not concrete Redis
/// 2. **Technology Agnostic**: Can be implemented by Redis, DragonflyDB, Memcached, etc.
/// 3. **Minimal Interface**: Only essential operations (YAGNI principle)
/// 4. **Type Safety**: Generic over serializable types
/// 5. **Async First**: All operations are async for non-blocking I/O
///
/// # Performance Targets
///
/// - `get`: <5ms p95 latency
/// - `set`: <10ms p95 latency
/// - `delete`: <5ms p95 latency
/// - `exists`: <3ms p95 latency
///
#[async_trait]
pub trait CacheStorage: Send + Sync {
    /// Retrieve a value from cache by key
    ///
    /// Returns `None` if key doesn't exist or has expired.
    /// Returns `Some(value)` if found and valid.
    ///
    /// # Performance
    /// Target: <5ms p95 latency
    ///
    /// # Errors
    /// - `CacheError::ConnectionFailed`: Cannot reach backend
    /// - `CacheError::SerializationError`: Cannot deserialize value
    /// - `CacheError::Timeout`: Operation exceeded deadline
    async fn get<T>(&self, key: &str) -> CacheResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>;

    /// Store a value in cache with optional TTL
    ///
    /// # Arguments
    /// - `key`: Cache key (must be non-empty)
    /// - `value`: Value to store (must be serializable)
    /// - `ttl`: Time-to-live (None = no expiration)
    ///
    /// # Performance
    /// Target: <10ms p95 latency
    ///
    /// # Errors
    /// - `CacheError::ConnectionFailed`: Cannot reach backend
    /// - `CacheError::SerializationError`: Cannot serialize value
    /// - `CacheError::ValueTooLarge`: Value exceeds backend limits
    /// - `CacheError::InvalidKey`: Key format is invalid
    async fn set<T>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()>
    where
        T: Serialize;

    /// Delete a key from cache
    ///
    /// Returns `true` if key was deleted, `false` if it didn't exist.
    ///
    /// # Performance
    /// Target: <5ms p95 latency
    async fn delete(&self, key: &str) -> CacheResult<bool>;

    /// Check if a key exists in cache
    ///
    /// # Performance
    /// Target: <3ms p95 latency
    async fn exists(&self, key: &str) -> CacheResult<bool>;

    /// Batch get operation (optional optimization)
    ///
    /// Default implementation calls `get()` in parallel.
    /// Backends may override for better performance (e.g., Redis MGET).
    async fn get_many<T>(&self, keys: &[&str]) -> CacheResult<Vec<Option<T>>>
    where
        T: for<'de> Deserialize<'de>,
    {
        use futures::future::try_join_all;

        let futures = keys.iter().map(|key| self.get::<T>(key));
        try_join_all(futures).await
    }

    /// Batch set operation (optional optimization)
    ///
    /// Default implementation calls `set()` in parallel.
    /// Backends may override for better performance (e.g., Redis MSET).
    async fn set_many<T>(
        &self,
        items: &[(&str, &T)],
        ttl: Option<Duration>,
    ) -> CacheResult<()>
    where
        T: Serialize,
    {
        use futures::future::try_join_all;

        let futures = items.iter().map(|(key, value)| self.set(key, value, ttl));
        try_join_all(futures).await?;
        Ok(())
    }

    /// Get cache statistics (optional, for monitoring)
    ///
    /// Returns `None` if backend doesn't support metrics.
    async fn stats(&self) -> CacheResult<Option<CacheStats>> {
        Ok(None)
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of keys
    pub total_keys: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Hit rate (0.0 to 1.0)
    pub hit_rate: f64,
    /// Miss rate (0.0 to 1.0)
    pub miss_rate: f64,
    /// Average operation latency (microseconds)
    pub avg_latency_us: u64,
}
```

---

## 3. Redis Adapter Implementation

### 3.1 Implementation in `riptide-cache`

Location: `crates/riptide-cache/src/redis_storage.rs` (NEW)

```rust
//! Redis implementation of CacheStorage trait

use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};
use riptide_types::ports::cache::{
    CacheError, CacheResult, CacheStats, CacheStorage,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Redis-backed cache storage adapter
///
/// Implements `CacheStorage` trait using Redis as the backend.
/// Includes connection pooling, automatic reconnection, and metrics.
pub struct RedisCacheStorage {
    /// Connection pool
    connections: Arc<RwLock<Vec<MultiplexedConnection>>>,
    /// Configuration
    config: RedisConfig,
    /// Metrics collector
    metrics: Arc<RwLock<RedisMetrics>>,
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// Redis URL (e.g., "redis://localhost:6379")
    pub url: String,
    /// Connection pool size
    pub pool_size: usize,
    /// Key prefix for namespacing
    pub key_prefix: String,
    /// Default TTL if none specified
    pub default_ttl: Option<Duration>,
    /// Maximum value size in bytes
    pub max_value_size: usize,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            pool_size: 10,
            key_prefix: "riptide".to_string(),
            default_ttl: Some(Duration::from_secs(3600)), // 1 hour
            max_value_size: 20 * 1024 * 1024, // 20MB
        }
    }
}

impl RedisCacheStorage {
    pub async fn new(config: RedisConfig) -> CacheResult<Self> {
        let client = Client::open(config.url.as_str())
            .map_err(|e| CacheError::ConnectionFailed(e.to_string()))?;

        let mut connections = Vec::new();
        for _ in 0..config.pool_size {
            let conn = client
                .get_multiplexed_tokio_connection()
                .await
                .map_err(|e| CacheError::ConnectionFailed(e.to_string()))?;
            connections.push(conn);
        }

        Ok(Self {
            connections: Arc::new(RwLock::new(connections)),
            config,
            metrics: Arc::new(RwLock::new(RedisMetrics::default())),
        })
    }

    /// Get a connection from the pool
    async fn get_connection(&self) -> CacheResult<MultiplexedConnection> {
        let connections = self.connections.read().await;
        connections
            .first()
            .cloned()
            .ok_or_else(|| CacheError::ConnectionFailed("No connections available".to_string()))
    }

    /// Generate namespaced key
    fn make_key(&self, key: &str) -> String {
        format!("{}:{}", self.config.key_prefix, key)
    }
}

#[async_trait]
impl CacheStorage for RedisCacheStorage {
    async fn get<T>(&self, key: &str) -> CacheResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let start = std::time::Instant::now();
        let redis_key = self.make_key(key);

        let mut conn = self.get_connection().await?;
        let result: Option<Vec<u8>> = conn
            .get(&redis_key)
            .await
            .map_err(|e| CacheError::BackendError(e.to_string()))?;

        let elapsed = start.elapsed();

        // Record metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_get(elapsed, result.is_some());
        }

        match result {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes)
                    .map_err(|e| CacheError::SerializationError(e.to_string()))?;
                debug!(key = %key, elapsed_us = elapsed.as_micros(), "Cache hit");
                Ok(Some(value))
            }
            None => {
                debug!(key = %key, elapsed_us = elapsed.as_micros(), "Cache miss");
                Ok(None)
            }
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> CacheResult<()>
    where
        T: Serialize,
    {
        let start = std::time::Instant::now();
        let redis_key = self.make_key(key);

        // Serialize value
        let bytes = serde_json::to_vec(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        // Check size limits
        if bytes.len() > self.config.max_value_size {
            return Err(CacheError::ValueTooLarge {
                size: bytes.len(),
                max: self.config.max_value_size,
            });
        }

        let mut conn = self.get_connection().await?;

        let ttl_seconds = ttl
            .or(self.config.default_ttl)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if ttl_seconds > 0 {
            conn.set_ex::<_, _, ()>(&redis_key, &bytes, ttl_seconds)
                .await
                .map_err(|e| CacheError::BackendError(e.to_string()))?;
        } else {
            conn.set::<_, _, ()>(&redis_key, &bytes)
                .await
                .map_err(|e| CacheError::BackendError(e.to_string()))?;
        }

        let elapsed = start.elapsed();

        // Record metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_set(elapsed, bytes.len());
        }

        debug!(
            key = %key,
            size = bytes.len(),
            ttl = ttl_seconds,
            elapsed_us = elapsed.as_micros(),
            "Cache set"
        );

        Ok(())
    }

    async fn delete(&self, key: &str) -> CacheResult<bool> {
        let redis_key = self.make_key(key);
        let mut conn = self.get_connection().await?;

        let deleted: u64 = conn
            .del(&redis_key)
            .await
            .map_err(|e| CacheError::BackendError(e.to_string()))?;

        Ok(deleted > 0)
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        let redis_key = self.make_key(key);
        let mut conn = self.get_connection().await?;

        conn.exists(&redis_key)
            .await
            .map_err(|e| CacheError::BackendError(e.to_string()))
    }

    async fn stats(&self) -> CacheResult<Option<CacheStats>> {
        let metrics = self.metrics.read().await;
        Ok(Some(metrics.to_cache_stats()))
    }
}

#[derive(Debug, Default)]
struct RedisMetrics {
    total_gets: u64,
    cache_hits: u64,
    total_sets: u64,
    total_latency_us: u64,
    operations: u64,
}

impl RedisMetrics {
    fn record_get(&mut self, elapsed: Duration, hit: bool) {
        self.total_gets += 1;
        if hit {
            self.cache_hits += 1;
        }
        self.total_latency_us += elapsed.as_micros() as u64;
        self.operations += 1;
    }

    fn record_set(&mut self, elapsed: Duration, _size: usize) {
        self.total_sets += 1;
        self.total_latency_us += elapsed.as_micros() as u64;
        self.operations += 1;
    }

    fn to_cache_stats(&self) -> CacheStats {
        let hit_rate = if self.total_gets > 0 {
            self.cache_hits as f64 / self.total_gets as f64
        } else {
            0.0
        };

        let avg_latency_us = if self.operations > 0 {
            self.total_latency_us / self.operations
        } else {
            0
        };

        CacheStats {
            total_keys: 0, // Would need Redis INFO query
            memory_usage_bytes: 0, // Would need Redis INFO query
            hit_rate,
            miss_rate: 1.0 - hit_rate,
            avg_latency_us,
        }
    }
}
```

---

## 4. Migration Path

### 4.1 Phase 1: Define Trait (Sprint 0.1.3 - Day 1)

```bash
# Create port interface
mkdir -p crates/riptide-types/src/ports
touch crates/riptide-types/src/ports/cache.rs
touch crates/riptide-types/src/ports/mod.rs

# Update riptide-types/src/lib.rs
echo "pub mod ports;" >> crates/riptide-types/src/lib.rs

# Implement trait specification (from Section 2.1)
# Test compilation
cargo check -p riptide-types
```

### 4.2 Phase 2: Implement Redis Adapter (Sprint 0.1.3 - Day 2)

```bash
# Create Redis adapter
touch crates/riptide-cache/src/redis_storage.rs

# Update riptide-cache/Cargo.toml
# Add dependency: riptide-types = { path = "../riptide-types" }

# Update riptide-cache/src/lib.rs
echo "pub mod redis_storage;" >> crates/riptide-cache/src/lib.rs
echo "pub use redis_storage::RedisCacheStorage;" >> crates/riptide-cache/src/lib.rs

# Test Redis implementation
cargo test -p riptide-cache
```

### 4.3 Phase 3: Migrate Consumers (Sprint 0.1.3 - Day 3)

```bash
# Migrate riptide-persistence
# BEFORE: use redis::Client;
# AFTER:  use riptide_types::ports::cache::CacheStorage;

# Update Cargo.toml - REMOVE redis dependency
sed -i '/^redis =/d' crates/riptide-persistence/Cargo.toml

# Add riptide-cache dependency instead
echo 'riptide-cache = { path = "../riptide-cache" }' >> crates/riptide-persistence/Cargo.toml

# Repeat for:
# - riptide-utils
# - riptide-api
# - riptide-performance

# Verify no direct Redis usage outside riptide-cache
rg "use redis::" crates/ --type rust | grep -v riptide-cache
```

---

## 5. Testing Strategy

### 5.1 Unit Tests for Trait

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    /// In-memory cache for testing (no Redis required)
    struct InMemoryCacheStorage {
        store: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    }

    #[async_trait]
    impl CacheStorage for InMemoryCacheStorage {
        async fn get<T>(&self, key: &str) -> CacheResult<Option<T>>
        where
            T: for<'de> Deserialize<'de>,
        {
            let store = self.store.read().await;
            match store.get(key) {
                Some(bytes) => {
                    let value = serde_json::from_slice(bytes)?;
                    Ok(Some(value))
                }
                None => Ok(None),
            }
        }

        async fn set<T>(&self, key: &str, value: &T, _ttl: Option<Duration>) -> CacheResult<()>
        where
            T: Serialize,
        {
            let bytes = serde_json::to_vec(value)?;
            self.store.write().await.insert(key.to_string(), bytes);
            Ok(())
        }

        async fn delete(&self, key: &str) -> CacheResult<bool> {
            Ok(self.store.write().await.remove(key).is_some())
        }

        async fn exists(&self, key: &str) -> CacheResult<bool> {
            Ok(self.store.read().await.contains_key(key))
        }
    }

    #[tokio::test]
    async fn test_cache_storage_contract() {
        let cache = InMemoryCacheStorage {
            store: Arc::new(RwLock::new(HashMap::new())),
        };

        // Test set and get
        cache.set("key1", &"value1", None).await.unwrap();
        let result: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(result, Some("value1".to_string()));

        // Test exists
        assert!(cache.exists("key1").await.unwrap());
        assert!(!cache.exists("nonexistent").await.unwrap());

        // Test delete
        assert!(cache.delete("key1").await.unwrap());
        assert!(!cache.exists("key1").await.unwrap());
    }
}
```

### 5.2 Integration Tests for Redis

```rust
#[cfg(test)]
#[cfg(feature = "integration-tests")]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_cache_storage() {
        let config = RedisConfig {
            url: "redis://localhost:6379".to_string(),
            ..Default::default()
        };

        let cache = RedisCacheStorage::new(config).await.unwrap();

        // Test basic operations
        cache.set("test_key", &42i32, Some(Duration::from_secs(60)))
            .await
            .unwrap();

        let value: Option<i32> = cache.get("test_key").await.unwrap();
        assert_eq!(value, Some(42));

        // Cleanup
        cache.delete("test_key").await.unwrap();
    }
}
```

---

## 6. Architecture Decision Records (ADRs)

### ADR-001: Use Trait Instead of Concrete Type

**Status:** ✅ Accepted
**Context:** Need to decouple domain logic from Redis implementation
**Decision:** Define `CacheStorage` trait in `riptide-types`
**Consequences:**
- ✅ Enables testing without Redis (in-memory mock)
- ✅ Allows future backend swaps (DragonflyDB, Memcached)
- ✅ Follows Dependency Inversion Principle
- ⚠️ Adds trait dispatch overhead (negligible for async I/O)

### ADR-002: Keep Interface Minimal

**Status:** ✅ Accepted
**Context:** Balance between feature-richness and simplicity
**Decision:** Only include essential operations (get, set, delete, exists)
**Consequences:**
- ✅ Easy to implement new backends
- ✅ Clear contract with minimal surface area
- ⚠️ Advanced features (pub/sub, transactions) require backend-specific APIs

### ADR-003: Use JSON Serialization

**Status:** ✅ Accepted
**Context:** Need type-safe serialization
**Decision:** Use `serde_json` for value serialization
**Consequences:**
- ✅ Works with all Rust types that implement `Serialize`
- ✅ Human-readable in Redis for debugging
- ⚠️ Slightly less efficient than binary formats (acceptable tradeoff)

---

## 7. Success Metrics

| Metric | Target | Validation |
|--------|--------|------------|
| **Redis Dependencies** | ≤2 crates | `grep -c "redis =" crates/*/Cargo.toml` |
| **Direct Redis Imports** | 0 outside `riptide-cache` | `rg "use redis::" crates/ --type rust` |
| **Performance (p95)** | <5ms get, <10ms set | Integration benchmarks |
| **Test Coverage** | 100% trait methods | `cargo tarpaulin -p riptide-types` |
| **Build Time** | No increase | Compare before/after |

---

## 8. Implementation Checklist

### Sprint 0.1.3 - Day 1
- [ ] Create `crates/riptide-types/src/ports/cache.rs`
- [ ] Define `CacheStorage` trait
- [ ] Define `CacheError` enum
- [ ] Define `CacheStats` struct
- [ ] Add unit tests with in-memory mock
- [ ] Verify `cargo check -p riptide-types`

### Sprint 0.1.3 - Day 2
- [ ] Create `crates/riptide-cache/src/redis_storage.rs`
- [ ] Implement `RedisCacheStorage`
- [ ] Add connection pooling
- [ ] Add metrics tracking
- [ ] Add integration tests (behind feature flag)
- [ ] Verify `cargo test -p riptide-cache`

### Sprint 0.1.3 - Day 3
- [ ] Migrate `riptide-persistence` to use trait
- [ ] Migrate `riptide-utils` to use trait
- [ ] Migrate `riptide-api` to use trait
- [ ] Migrate `riptide-performance` to use trait
- [ ] Remove Redis dependencies from these crates
- [ ] Verify `cargo test --workspace`
- [ ] Verify only 2 crates depend on Redis

---

## 9. Files Modified Summary

```
CREATE:
  crates/riptide-types/src/ports/mod.rs
  crates/riptide-types/src/ports/cache.rs
  crates/riptide-cache/src/redis_storage.rs

UPDATE:
  crates/riptide-types/src/lib.rs (expose ports module)
  crates/riptide-cache/src/lib.rs (expose redis_storage)
  crates/riptide-cache/Cargo.toml (add riptide-types dep)
  crates/riptide-persistence/Cargo.toml (replace redis with riptide-cache)
  crates/riptide-utils/Cargo.toml (replace redis with riptide-cache)
  crates/riptide-api/Cargo.toml (replace redis with riptide-cache)
  crates/riptide-performance/Cargo.toml (replace redis with riptide-cache)
  crates/riptide-persistence/src/*.rs (use CacheStorage trait)
  crates/riptide-utils/src/*.rs (use CacheStorage trait)
  crates/riptide-api/src/*.rs (use CacheStorage trait)
  crates/riptide-performance/src/*.rs (use CacheStorage trait)
```

**Total LOC Impact:** +400 LOC trait/implementation, -0 LOC (refactoring only)

---

## 10. Risk Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **Performance regression** | LOW | MEDIUM | Benchmark before/after, trait dispatch is negligible for async I/O |
| **Breaking existing code** | MEDIUM | HIGH | Comprehensive test suite, gradual migration per crate |
| **Redis-specific features needed** | LOW | LOW | Keep `RedisCacheStorage` public for advanced use cases |

---

## 11. References

- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- [Dependency Inversion Principle](https://en.wikipedia.org/wiki/Dependency_inversion_principle)
- [Phase 0 Cleanup Roadmap](/workspaces/eventmesh/docs/roadmap/PHASE_0_CLEANUP_ROADMAP.md)
- [riptide-types traits.rs](/workspaces/eventmesh/crates/riptide-types/src/traits.rs)

---

**Document Status:** ✅ Ready for Implementation
**Next Review:** After Sprint 0.1.3 completion
**Owner:** System Architect
