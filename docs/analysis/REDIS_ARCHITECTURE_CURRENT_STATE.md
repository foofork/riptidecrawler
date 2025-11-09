# Redis Architecture - Current State
## Sprint 4.2 Analysis Results

**Date:** 2025-11-08
**Status:** PARTIALLY CONSOLIDATED (71% compliance)

---

## Visual Architecture Map

```
┌─────────────────────────────────────────────────────────────────┐
│                    RIPTIDE APPLICATION LAYERS                    │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                         API LAYER                                │
│  riptide-api/                                                    │
│  └── errors.rs ⚠️ From<redis::RedisError>                        │
│                 (SHOULD NOT DEPEND ON REDIS)                     │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                       FACADE LAYER ✅                             │
│  riptide-facade/src/facades/                                     │
│  ├── engine.rs → Arc<dyn CacheStorage>                          │
│  └── llm.rs    → Arc<dyn CacheStorage>                          │
│                                                                  │
│  (CORRECTLY USES TRAIT ABSTRACTION)                              │
└─────────────────────────────────────────────────────────────────┘
                              ↓
                   ┌──────────┴──────────┐
                   ↓                     ↓
┌──────────────────────────┐   ┌──────────────────────────┐
│  INFRASTRUCTURE PORTS    │   │  PERSISTENCE LAYER ⚠️     │
│  riptide-types/          │   │  riptide-persistence/    │
│  └── CacheStorage        │   │  ├── tenant.rs           │
│      (Trait Definition)  │   │  ├── state.rs            │
│                          │   │  ├── cache.rs            │
│                          │   │  └── sync.rs             │
│                          │   │                          │
│                          │   │  (BYPASSES ABSTRACTION)  │
└──────────────────────────┘   └──────────────────────────┘
         ↓                              ↓ redis::Client
         ↓                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    CACHE IMPLEMENTATION ✅                        │
│  riptide-cache/src/                                              │
│  ├── redis_storage.rs (441 lines)                               │
│  │   └── impl CacheStorage for RedisStorage                     │
│  ├── adapters/                                                   │
│  │   ├── redis_idempotency.rs                                   │
│  │   └── redis_session_storage.rs                               │
│  ├── key.rs (SHA256 key generation)                             │
│  └── manager.rs                                                  │
│                                                                  │
│  (CORRECT - PRIMARY REDIS ABSTRACTION)                           │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   UTILITY LAYER ⚠️                                │
│  riptide-utils/src/                                              │
│  └── redis.rs (RedisPool, RedisConfig)                          │
│                                                                  │
│  (SHOULD BE IN riptide-cache)                                    │
└─────────────────────────────────────────────────────────────────┘
         ↓                              ↓
         ↓                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   WORKER LAYER ✅                                 │
│  riptide-workers/src/                                            │
│  ├── queue.rs (Redis job queue)                                 │
│  ├── processors.rs                                               │
│  └── scheduler.rs                                                │
│                                                                  │
│  (CORRECT - NEEDS DIRECT REDIS FOR QUEUE)                        │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                        REDIS SERVER                              │
│  redis://localhost:6379                                          │
│                                                                  │
│  Key Namespaces:                                                 │
│  ├── riptide:v1:{hash}                                          │
│  ├── session:v1:{session_id}                                    │
│  ├── idempotency:v1:{user_key}                                  │
│  └── riptide:strategies:v1:{hash}                               │
└─────────────────────────────────────────────────────────────────┘
```

---

## Dependency Flow Analysis

### Current State (6 Crates with Redis)

```
┌─────────────────┐
│  redis crate    │ (workspace dependency)
└────────┬────────┘
         │
    ┌────┴────────────────────────────────┐
    │                                     │
    ↓                                     ↓
┌───────────────┐                 ┌──────────────────┐
│ riptide-cache │ ✅              │ riptide-workers  │ ✅
│               │                 │                  │
│ (PRIMARY)     │                 │ (JOB QUEUE)      │
└───────────────┘                 └──────────────────┘
         │
    ┌────┴────────────────────────────────┐
    │                                     │
    ↓                                     ↓
┌───────────────┐                 ┌──────────────────┐
│ riptide-utils │ ⚠️               │ riptide-persist  │ ⚠️
│               │                 │                  │
│ (MOVE POOL)   │                 │ (USE TRAIT)      │
└───────────────┘                 └──────────────────┘
    │                                     │
    ↓                                     ↓
┌───────────────┐                 ┌──────────────────┐
│ riptide-api   │ ⚠️               │ riptide-perf     │ ⚠️
│               │                 │                  │
│ (REMOVE ERR)  │                 │ (OPTIONAL FEAT)  │
└───────────────┘                 └──────────────────┘
```

### Target State (2 Crates with Redis)

```
┌─────────────────┐
│  redis crate    │ (workspace dependency)
└────────┬────────┘
         │
    ┌────┴──────────┐
    │               │
    ↓               ↓
┌───────────────┐   ┌──────────────────┐
│ riptide-cache │   │ riptide-workers  │
│               │   │                  │
│ (CONSOLIDATED)│   │ (JOB QUEUE)      │
│ ├── redis.rs  │   │ ├── queue.rs     │
│ ├── pool.rs   │   │ └── scheduler.rs │
│ └── storage.rs│   │                  │
└───────────────┘   └──────────────────┘
         │
         ↓ (via CacheStorage trait)
┌─────────────────────────────────────┐
│  All Other Crates                   │
│  ├── riptide-api                    │
│  ├── riptide-facade                 │
│  ├── riptide-persistence            │
│  └── riptide-performance            │
└─────────────────────────────────────┘
```

---

## Cache Key Patterns

### Versioned Key Hierarchy

```
redis://localhost:6379
│
├── riptide:v1:{sha256}
│   └── General cache entries
│       ├── URL-based keys
│       ├── Method-based keys
│       └── Option-based keys
│
├── riptide:strategies:v1:{sha256}
│   └── Strategy-specific cache
│       └── Extraction strategies
│
├── session:v1:{session_id}
│   └── User session data
│       ├── Authentication tokens
│       └── Session state
│
└── idempotency:v1:{user_key}
    ├── {user_key} (lock)
    └── {user_key}:result (cached result)
```

### Key Generation Algorithm

```rust
// SHA256-based deterministic hashing
CacheKeyBuilder::new()
    .url("https://example.com")
    .method("fetch")
    .version("v1")
    .namespace("riptide")
    .options({
        "option1": "value1",
        "option2": "value2"  // BTreeMap ensures order independence
    })
    .build()

// Output: riptide:v1:a1b2c3d4e5f6...
```

---

## Connection Architecture

### Current Connection Pooling

```
┌──────────────────────────────────────────────────────────────┐
│                    riptide-utils                             │
│                                                              │
│  RedisConfig                                                 │
│  ├── url: "redis://localhost:6379"                         │
│  ├── timeout_ms: 5000                                       │
│  ├── max_retries: 3                                         │
│  └── health_check_interval_secs: 30                         │
│                                                              │
│  RedisPool                                                   │
│  └── MultiplexedConnection                                  │
│      ├── Clone for concurrent access                        │
│      ├── PING/PONG health checks                           │
│      └── Automatic reconnection                             │
└──────────────────────────────────────────────────────────────┘
                           ↓
┌──────────────────────────────────────────────────────────────┐
│                    riptide-cache                             │
│                                                              │
│  RedisStorage (impl CacheStorage)                           │
│  ├── conn: MultiplexedConnection                           │
│  ├── hits: AtomicUsize                                      │
│  ├── misses: AtomicUsize                                    │
│  └── client: Client                                         │
└──────────────────────────────────────────────────────────────┘
                           ↓
┌──────────────────────────────────────────────────────────────┐
│                  Redis Server                                │
│                                                              │
│  Commands Used:                                              │
│  ├── GET/SET/DEL                                            │
│  ├── MGET/MSET (batch operations)                          │
│  ├── EXPIRE/TTL                                             │
│  ├── INCR                                                   │
│  ├── SCAN (pattern matching)                               │
│  ├── PING (health checks)                                  │
│  ├── DBSIZE (statistics)                                   │
│  └── INFO MEMORY (monitoring)                              │
└──────────────────────────────────────────────────────────────┘
```

---

## CacheStorage Trait Operations

### Operation Coverage

| Operation | RedisStorage | InMemoryCache | Use Case |
|-----------|--------------|---------------|----------|
| get | ✅ | ✅ | Single key retrieval |
| set | ✅ | ✅ | Single key storage |
| delete | ✅ | ✅ | Single key removal |
| exists | ✅ | ✅ | Key existence check |
| mset | ✅ | ✅ | Batch storage |
| mget | ✅ | ✅ | Batch retrieval |
| expire | ✅ | ✅ | Set TTL |
| ttl | ✅ | ✅ | Get TTL |
| incr | ✅ | ✅ | Atomic counter |
| delete_many | ✅ | ✅ | Batch deletion |
| clear_pattern | ✅ | ✅ | Pattern-based deletion |
| stats | ✅ | ✅ | Cache statistics |
| health_check | ✅ | ✅ | Connection health |

### Performance Characteristics

```
Operation           | Time Complexity | Network Calls | Notes
--------------------|-----------------|---------------|------------------
get(key)           | O(1)            | 1             | Single roundtrip
set(key, val, ttl) | O(1)            | 1             | With TTL support
mget(n keys)       | O(n)            | 1             | Batch optimized
mset(n items)      | O(n)            | 1             | Pipeline atomic
clear_pattern(pat) | O(N)            | N/100         | SCAN in batches
stats()            | O(1)            | 2             | DBSIZE + INFO
```

---

## Issue Summary

### Violations Found

1. **riptide-persistence** (4 files)
   - Direct `redis::Client` usage
   - Bypasses CacheStorage abstraction
   - Pipeline operations without trait

2. **riptide-utils** (1 file)
   - Owns Redis infrastructure
   - Should be in riptide-cache

3. **riptide-api** (1 file)
   - Direct Redis error dependency
   - Should use generic errors

4. **riptide-performance** (minor)
   - Optional Redis dependency
   - Could use CacheStorage instead

### Refactoring Path

```
Priority 1: Move Pool (2 hours)
  riptide-utils/redis.rs → riptide-cache/pool.rs

Priority 2: Persistence (8 hours)
  riptide-persistence/*.rs
    BEFORE: redis::Client
    AFTER:  Arc<dyn CacheStorage>

Priority 3: API Errors (1 hour)
  riptide-api/errors.rs
    BEFORE: From<redis::RedisError>
    AFTER:  Generic RiptideError mapping

Priority 4: Performance (4 hours)
  riptide-performance
    Use CacheStorage trait instead of direct Redis
```

---

## Metrics & Statistics

### Code Distribution

```
Crate                  | Redis Lines | Percentage | Status
-----------------------|-------------|------------|--------
riptide-cache          | ~1,500      | 60%        | ✅ Core
riptide-persistence    | ~400        | 16%        | ⚠️ Fix
riptide-workers        | ~300        | 12%        | ✅ OK
riptide-utils          | ~150        | 6%         | ⚠️ Move
riptide-api            | ~10         | 0.4%       | ⚠️ Remove
riptide-performance    | ~50         | 2%         | ⚠️ Optional
-----------------------|-------------|------------|--------
TOTAL                  | ~2,410      | 100%       | 71% OK
```

### Test Coverage

```
Crate                  | Tests | Coverage | Notes
-----------------------|-------|----------|-------------------
riptide-cache          | 25+   | High     | Comprehensive
riptide-persistence    | 15+   | Medium   | Needs integration
riptide-workers        | 10+   | Medium   | Queue-focused
riptide-utils          | 5+    | Low      | Basic unit tests
```

---

## Configuration Matrix

### Environment Variables

| Variable | Default | Purpose | Used By |
|----------|---------|---------|---------|
| REDIS_URL | redis://127.0.0.1:6379 | Connection string | All |
| CACHE_TTL_SECS | 3600 | Default TTL | riptide-cache |
| REDIS_TIMEOUT_MS | 5000 | Connection timeout | riptide-utils |
| REDIS_MAX_RETRIES | 3 | Retry attempts | riptide-utils |
| HEALTH_CHECK_INTERVAL | 30 | Health check interval | riptide-utils |

### YAML Configuration

```yaml
cache:
  redis_url: "redis://localhost:6379"
  default_ttl_secs: 3600
  max_size_mb: 1024

# Used by riptide-config
# Documented in: crates/riptide-config/README.md
```

---

## Next Steps

### Immediate (Sprint 4.2)
- ✅ Document current state (this file)
- ✅ Identify all violations
- ✅ Create refactoring plan

### Sprint 4.3 (Consolidation)
- Move RedisPool to riptide-cache
- Refactor riptide-persistence
- Remove API Redis dependency
- Update tests

### Sprint 4.4 (Optimization)
- Performance benchmarking
- Connection pool tuning
- Monitoring integration
- Documentation updates

---

## References

- Main Analysis: `/workspaces/eventmesh/docs/analysis/REDIS_CONSOLIDATION_VALIDATION.md`
- Cache Implementation: `/workspaces/eventmesh/crates/riptide-cache/src/redis_storage.rs`
- Trait Definition: `/workspaces/eventmesh/crates/riptide-types/src/ports/cache.rs`
- Configuration: `/workspaces/eventmesh/crates/riptide-config/README.md`

---

**Document Status:** ✅ COMPLETE
**Last Updated:** 2025-11-08
**Compliance Score:** 71% (5/7 checks passed)
