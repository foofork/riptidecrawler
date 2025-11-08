# CacheStorage Trait: Usage & Migration Guide

## Overview

The `CacheStorage` trait provides a backend-agnostic caching interface that enables dependency inversion, improved testability, and flexible backend selection.

### Design Goals

- **Reduce Redis Dependencies**: From 6 crates to 2 (riptide-cache + riptide-workers)
- **Enable Testing**: In-memory implementation for unit tests
- **Support Multiple Backends**: Redis, in-memory, or custom implementations
- **Maintain Performance**: Async-first with batch operations

## Architecture

```
┌─────────────────────────────────────────────┐
│          Application Layer                  │
│  (riptide-api, riptide-workers, etc.)      │
└─────────────────┬───────────────────────────┘
                  │ depends on trait
                  ▼
┌─────────────────────────────────────────────┐
│         CacheStorage Trait                  │
│         (riptide-types/ports)               │
└─────────────────┬───────────────────────────┘
                  │ implemented by
        ┌─────────┴──────────┐
        ▼                    ▼
┌──────────────┐    ┌──────────────────┐
│ InMemoryCache│    │  RedisStorage    │
│(riptide-types)│   │(riptide-cache)   │
└──────────────┘    └──────────────────┘
```

## Quick Start

### 1. Basic Usage with Redis

```rust
use riptide_cache::RedisStorage;
use riptide_types::ports::CacheStorage;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create Redis-backed storage
    let cache = RedisStorage::new("redis://localhost:6379").await?;

    // Set value with TTL
    cache.set("user:123", b"user_data", Some(Duration::from_secs(3600))).await?;

    // Get value
    if let Some(data) = cache.get("user:123").await? {
        println!("Found: {:?}", String::from_utf8_lossy(&data));
    }

    // Check existence
    if cache.exists("user:123").await? {
        println!("User is cached");
    }

    Ok(())
}
```

### 2. Testing with In-Memory Cache

```rust
use riptide_types::ports::{CacheStorage, InMemoryCache};
use std::time::Duration;

#[tokio::test]
async fn test_user_service() {
    // Use in-memory cache for testing
    let cache = InMemoryCache::new();

    // Same API as Redis
    cache.set("test:key", b"value", Some(Duration::from_secs(60))).await.unwrap();

    let result = cache.get("test:key").await.unwrap();
    assert_eq!(result, Some(b"value".to_vec()));
}
```

### 3. Dependency Injection

```rust
use riptide_types::ports::CacheStorage;
use std::sync::Arc;

pub struct UserService {
    cache: Arc<dyn CacheStorage>,
}

impl UserService {
    pub fn new(cache: Arc<dyn CacheStorage>) -> Self {
        Self { cache }
    }

    pub async fn get_user(&self, id: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let key = format!("user:{}", id);
        self.cache.get(&key).await.map_err(Into::into)
    }

    pub async fn cache_user(&self, id: &str, data: &[u8]) -> anyhow::Result<()> {
        let key = format!("user:{}", id);
        self.cache.set(&key, data, Some(Duration::from_secs(3600))).await.map_err(Into::into)
    }
}

// Production: use Redis
let redis_cache = Arc::new(RedisStorage::new("redis://localhost:6379").await?);
let service = UserService::new(redis_cache);

// Testing: use in-memory
let memory_cache = Arc::new(InMemoryCache::new());
let test_service = UserService::new(memory_cache);
```

## API Reference

### Core Methods

#### `get(&self, key: &str) -> Result<Option<Vec<u8>>>`
Retrieve value by key. Returns `None` if key doesn't exist or is expired.

```rust
let data = cache.get("mykey").await?;
if let Some(bytes) = data {
    let value: MyType = serde_json::from_slice(&bytes)?;
}
```

#### `set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()>`
Store value with optional TTL.

```rust
let data = serde_json::to_vec(&my_object)?;
cache.set("mykey", &data, Some(Duration::from_secs(3600))).await?;
```

#### `delete(&self, key: &str) -> Result<()>`
Delete a key.

```rust
cache.delete("mykey").await?;
```

#### `exists(&self, key: &str) -> Result<bool>`
Check if key exists and is not expired.

```rust
if cache.exists("mykey").await? {
    println!("Key exists");
}
```

### Batch Operations

#### `mset(&self, items: Vec<(&str, &[u8])>, ttl: Option<Duration>) -> Result<()>`
Set multiple keys atomically (where supported).

```rust
let items = vec![
    ("key1", b"value1" as &[u8]),
    ("key2", b"value2"),
];
cache.mset(items, Some(Duration::from_secs(3600))).await?;
```

#### `mget(&self, keys: &[&str]) -> Result<Vec<Option<Vec<u8>>>>`
Get multiple values efficiently.

```rust
let keys = vec!["key1", "key2", "key3"];
let values = cache.mget(&keys).await?;
for (key, value) in keys.iter().zip(values.iter()) {
    if let Some(data) = value {
        println!("{}: found", key);
    }
}
```

#### `delete_many(&self, keys: &[&str]) -> Result<usize>`
Delete multiple keys and return count deleted.

```rust
let deleted = cache.delete_many(&["key1", "key2"]).await?;
println!("Deleted {} keys", deleted);
```

### Advanced Operations

#### `expire(&self, key: &str, ttl: Duration) -> Result<bool>`
Set expiration on existing key. Returns `true` if key exists.

```rust
cache.expire("mykey", Duration::from_secs(600)).await?;
```

#### `ttl(&self, key: &str) -> Result<Option<Duration>>`
Get remaining time-to-live for a key.

```rust
if let Some(remaining) = cache.ttl("mykey").await? {
    println!("Expires in {} seconds", remaining.as_secs());
}
```

#### `incr(&self, key: &str, delta: i64) -> Result<i64>`
Atomically increment/decrement a numeric value.

```rust
let new_count = cache.incr("counter", 1).await?;
println!("Counter: {}", new_count);
```

#### `clear_pattern(&self, pattern: &str) -> Result<usize>`
Delete all keys matching a pattern (use with caution).

```rust
// Delete all user cache entries
let deleted = cache.clear_pattern("user:*").await?;
println!("Cleared {} user entries", deleted);
```

### Monitoring

#### `stats(&self) -> Result<CacheStats>`
Get cache statistics including hit rate and memory usage.

```rust
let stats = cache.stats().await?;
println!("Keys: {}", stats.total_keys);
println!("Memory: {} bytes", stats.memory_usage);
if let Some(hit_rate) = stats.hit_rate {
    println!("Hit rate: {:.2}%", hit_rate * 100.0);
}
```

#### `health_check(&self) -> Result<bool>`
Verify cache backend is operational.

```rust
if cache.health_check().await? {
    println!("Cache is healthy");
} else {
    println!("Cache is down");
}
```

## Migration Guide

### Step 1: Update Dependencies

No changes needed - `async-trait` already in workspace dependencies.

### Step 2: Replace Direct Redis Usage

**Before:**
```rust
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};

struct MyService {
    redis: MultiplexedConnection,
}

impl MyService {
    async fn get_data(&mut self, key: &str) -> anyhow::Result<Option<String>> {
        let value: Option<String> = self.redis.get(key).await?;
        Ok(value)
    }
}
```

**After:**
```rust
use riptide_types::ports::CacheStorage;
use std::sync::Arc;

struct MyService {
    cache: Arc<dyn CacheStorage>,
}

impl MyService {
    async fn get_data(&self, key: &str) -> anyhow::Result<Option<String>> {
        let bytes = self.cache.get(key).await?;
        match bytes {
            Some(data) => {
                let value = String::from_utf8(data)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}
```

### Step 3: Update Initialization

**Before:**
```rust
let client = redis::Client::open("redis://localhost:6379")?;
let conn = client.get_multiplexed_tokio_connection().await?;
```

**After:**
```rust
use riptide_cache::RedisStorage;

let cache: Arc<dyn CacheStorage> = Arc::new(
    RedisStorage::new("redis://localhost:6379").await?
);
```

### Step 4: Update Tests

**Before:**
```rust
#[tokio::test]
async fn test_service() {
    // Requires Redis running
    let client = redis::Client::open("redis://localhost:6379").unwrap();
    let conn = client.get_multiplexed_tokio_connection().await.unwrap();
    // ...
}
```

**After:**
```rust
use riptide_types::ports::InMemoryCache;

#[tokio::test]
async fn test_service() {
    // No Redis required!
    let cache: Arc<dyn CacheStorage> = Arc::new(InMemoryCache::new());
    // ...
}
```

## Implementation Details

### RedisStorage

Located in `crates/riptide-cache/src/redis_storage.rs`

**Features:**
- Multiplexed connections for concurrent access
- Native Redis commands (MSET, MGET, EXPIRE, TTL, INCR)
- Atomic batch operations with pipelines
- Statistics tracking (hits, misses)
- Health monitoring with PING

**Performance:**
- Connection pooling via multiplexed connections
- Pipelined batch operations
- Minimal allocations for hot paths

### InMemoryCache

Located in `crates/riptide-types/src/ports/memory_cache.rs`

**Features:**
- Thread-safe with `RwLock<HashMap>`
- TTL support with automatic expiration checking
- Statistics tracking (hits, misses, memory usage)
- Pattern matching for `clear_pattern`
- Zero external dependencies

**Performance:**
- Optimized for read-heavy workloads
- Efficient concurrent reads with `RwLock`
- Lazy expiration on access
- Manual cleanup via `cleanup_expired()`

## Best Practices

### 1. Use Dependency Injection

```rust
// Good: Accept trait for flexibility
pub struct Service {
    cache: Arc<dyn CacheStorage>,
}

// Bad: Hardcode Redis dependency
pub struct Service {
    redis: redis::aio::MultiplexedConnection,
}
```

### 2. Serialize Complex Types

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
}

// Set
let user = User { id: "123".into(), name: "Alice".into() };
let data = serde_json::to_vec(&user)?;
cache.set("user:123", &data, Some(Duration::from_secs(3600))).await?;

// Get
if let Some(bytes) = cache.get("user:123").await? {
    let user: User = serde_json::from_slice(&bytes)?;
}
```

### 3. Use Batch Operations

```rust
// Good: Single batch operation
let items = vec![("k1", b"v1"), ("k2", b"v2"), ("k3", b"v3")];
cache.mset(items, Some(ttl)).await?;

// Bad: Multiple individual sets
for (k, v) in items {
    cache.set(k, v, Some(ttl)).await?;
}
```

### 4. Handle Errors Gracefully

```rust
match cache.get(key).await {
    Ok(Some(data)) => {
        // Use cached data
    }
    Ok(None) => {
        // Cache miss - fetch from source
    }
    Err(e) => {
        // Cache backend error - fallback strategy
        warn!("Cache error: {}", e);
        // Fetch from source directly
    }
}
```

### 5. Monitor Cache Performance

```rust
// Periodically check cache health
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;

        match cache.stats().await {
            Ok(stats) => {
                if let Some(hit_rate) = stats.hit_rate {
                    if hit_rate < 0.5 {
                        warn!("Low cache hit rate: {:.2}%", hit_rate * 100.0);
                    }
                }
            }
            Err(e) => error!("Failed to get cache stats: {}", e),
        }
    }
});
```

## Comparison: Redis vs In-Memory

| Feature | RedisStorage | InMemoryCache |
|---------|-------------|---------------|
| **Persistence** | Persistent | Volatile |
| **Distribution** | Shared across processes | Local only |
| **TTL Precision** | Native Redis TTL | Checked on access |
| **Pattern Clearing** | SCAN + DEL | String prefix match |
| **Increment** | Atomic INCR | Lock-based |
| **Best For** | Production, multi-instance | Testing, single-instance |
| **Setup Required** | Redis server | None |

## Performance Considerations

### RedisStorage

**Pros:**
- Distributed caching across instances
- Persistent across restarts
- Native atomic operations
- Mature, battle-tested backend

**Cons:**
- Network latency for operations
- Requires Redis infrastructure
- Additional operational complexity

**Optimization Tips:**
- Use batch operations (`mset`, `mget`) to reduce round trips
- Set appropriate TTLs to prevent memory bloat
- Monitor Redis memory usage with `INFO MEMORY`
- Consider Redis clustering for horizontal scaling

### InMemoryCache

**Pros:**
- Zero latency (in-process)
- No external dependencies
- Simple setup and testing
- Predictable performance

**Cons:**
- Memory limited to process heap
- No sharing across instances
- Lost on restart
- TTL cleanup is lazy (on access)

**Optimization Tips:**
- Call `cleanup_expired()` periodically to free memory
- Use `with_capacity()` if size is known upfront
- Monitor memory usage with `stats()`
- Consider size limits for cache entries

## Next Steps

### Crates to Migrate

Based on the roadmap, these crates should migrate from direct Redis to `CacheStorage`:

1. **riptide-api** - Use dependency injection in handlers
2. **riptide-utils** - Replace utility cache functions
3. **riptide-persistence** - Use trait for persistence layer caching
4. **riptide-performance** - Use trait for performance metrics caching

### Migration Priority

**Phase 1** (Sprint 0.1.3):
- ✅ Create `CacheStorage` trait
- ✅ Implement `RedisStorage` adapter
- ✅ Implement `InMemoryCache` for testing

**Phase 2** (Sprint 0.1.4):
- Migrate `riptide-api` to use `CacheStorage`
- Update tests to use `InMemoryCache`

**Phase 3** (Sprint 0.1.5):
- Migrate `riptide-utils` to use `CacheStorage`
- Migrate `riptide-persistence` to use `CacheStorage`

**Phase 4** (Sprint 0.2.0):
- Migrate `riptide-performance` to use `CacheStorage`
- Remove direct Redis dependencies from migrated crates

## Troubleshooting

### Issue: Type conversion errors

```rust
// Error: expected `&[u8]`, found `String`
cache.set("key", "string value", None).await?;

// Solution: Convert to bytes
cache.set("key", b"string value", None).await?;
// Or
cache.set("key", "string value".as_bytes(), None).await?;
```

### Issue: Trait object safety

```rust
// Error: cannot be made into an object
fn get_cache() -> Box<dyn CacheStorage> {
    Box::new(InMemoryCache::new())
}

// Solution: Use Arc instead of Box
use std::sync::Arc;

fn get_cache() -> Arc<dyn CacheStorage> {
    Arc::new(InMemoryCache::new())
}
```

### Issue: Lifetime errors with cache reference

```rust
// Error: borrowed value does not live long enough
async fn use_cache(cache: &dyn CacheStorage) {
    cache.set("key", b"value", None).await.unwrap();
}

// Solution: Use Arc<dyn CacheStorage>
async fn use_cache(cache: Arc<dyn CacheStorage>) {
    cache.set("key", b"value", None).await.unwrap();
}
```

## Additional Resources

- **Source Code**: `crates/riptide-types/src/ports/cache.rs`
- **Redis Adapter**: `crates/riptide-cache/src/redis_storage.rs`
- **In-Memory Impl**: `crates/riptide-types/src/ports/memory_cache.rs`
- **Tests**: See `#[cfg(test)]` modules in each file
- **Refactoring Roadmap**: `/workspaces/eventmesh/docs/REFACTORING_ROADMAP_v3.1.md`

## Support

For questions or issues:
1. Check existing tests for usage examples
2. Review the trait documentation in source code
3. Consult the refactoring roadmap for migration guidance
