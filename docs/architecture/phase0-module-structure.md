# Phase 0: riptide-utils Module Structure

**Quick Reference Guide for Developers**

## Module Organization

```
crates/riptide-utils/
├── Cargo.toml                    # Dependencies: redis, reqwest, tokio, governor, chrono
├── src/
│   ├── lib.rs                    # Public API exports
│   │   pub use redis::RedisPool;
│   │   pub use http::{create_default_client, create_custom_client};
│   │   pub use retry::RetryPolicy;
│   │   pub use time::{now, now_unix, format_iso8601};
│   │   pub use error::{RiptideError, Result};
│   │   pub use rate_limit::SimpleRateLimiter;
│   │
│   ├── redis.rs                  # 📦 RedisPool (2 days) - CREATE NEW
│   │   pub struct RedisPool
│   │   pub struct RedisConfig
│   │   impl RedisPool::new()
│   │   impl RedisPool::get()
│   │   fn start_health_checks()
│   │
│   ├── http.rs                   # 🌐 HTTP client factory (1 day) - CREATE NEW
│   │   pub fn create_default_client() -> Result<Client>
│   │   pub fn create_custom_client(timeout, ua) -> Result<Client>
│   │
│   ├── retry.rs                  # 🔄 RetryPolicy (2-3 days) - REFACTOR
│   │   pub struct RetryPolicy
│   │   impl RetryPolicy::execute<F, Fut, T, E>()
│   │   - Exponential backoff
│   │   - Jitter support
│   │   - Generic over error types
│   │
│   ├── time.rs                   # ⏰ Time utilities (0.5 days) - CREATE NEW
│   │   pub fn now() -> DateTime<Utc>
│   │   pub fn now_unix() -> i64
│   │   pub fn format_iso8601(dt) -> String
│   │   pub fn parse_iso8601(s) -> Result<DateTime<Utc>>
│   │
│   ├── error.rs                  # ⚠️ Error re-exports (0.5 days) - CREATE NEW
│   │   pub use riptide_types::{RiptideError, Result};
│   │
│   └── rate_limit.rs             # 🚦 Rate limiting (0.5 days) - CREATE NEW
│       pub struct SimpleRateLimiter
│       impl SimpleRateLimiter::new(rpm)
│       impl SimpleRateLimiter::check()
│
└── tests/
    ├── redis_pool_tests.rs       # Redis connection pooling tests (8 tests)
    ├── retry_tests.rs            # Retry policy tests (10 tests)
    ├── http_tests.rs             # HTTP factory tests (5 tests)
    ├── time_tests.rs             # Time utility tests (2 tests)
    └── integration/              # Integration tests
        └── mod.rs
```

## Dependency Graph

```
External Dependencies (No internal riptide-* dependencies!)
├── redis (0.26) - workspace version
├── reqwest (0.12) - workspace version
├── tokio (1.x) - workspace version
├── governor (0.6) - rate limiting
├── chrono (0.4) - time utilities
├── thiserror (1.0) - error types
└── anyhow (1.0) - error handling

Consumers (Crates that depend on riptide-utils)
├── riptide-workers
│   ├── scheduler.rs:193 → RedisPool
│   └── queue.rs:56 → RedisPool
├── riptide-persistence
│   └── tests/integration/mod.rs:92 → RedisPool
├── riptide-fetch
│   └── fetch.rs → RetryPolicy (migrated from)
├── riptide-intelligence
│   └── smart_retry.rs → RetryPolicy
└── 8+ test files → create_default_client()
```

## Migration Checklist

### Week 0 Day 1-2: Redis Pooling

**Files to Update:**
```
✅ Create: crates/riptide-utils/src/redis.rs
✅ Update: crates/riptide-workers/src/scheduler.rs:193
   - Replace: redis::Client::open()
   - With: riptide_utils::redis::RedisPool::new()

✅ Update: crates/riptide-workers/src/queue.rs:56
   - Replace: redis::Client::open()
   - With: riptide_utils::redis::RedisPool::new()

✅ Update: crates/riptide-persistence/tests/integration/mod.rs:92
   - Replace: redis::Client::open()
   - With: riptide_utils::redis::RedisPool::new()

✅ Add: 8 tests in tests/redis_pool_tests.rs
```

**Estimated Removal:** ~150 lines

### Week 0 Day 3: HTTP Client Factory

**Files to Update:**
```
✅ Create: crates/riptide-utils/src/http.rs
✅ Update: 8+ test files with reqwest::Client::builder()
   - Replace: Client::builder().timeout(...).build()
   - With: riptide_utils::http::create_default_client()

✅ Add: 5 tests in tests/http_tests.rs
```

**Estimated Removal:** ~80 lines

### Week 0 Day 4-6: Retry Logic

**Phase 1 - High Priority (Week 0):**
```
✅ Extract: crates/riptide-fetch/src/fetch.rs:32-57 (canonical)
✅ Create: crates/riptide-utils/src/retry.rs
✅ Update: 10 high-priority files
   - riptide-intelligence/src/smart_retry.rs
   - riptide-workers/src/job.rs
   - riptide-spider/src/core.rs (where applicable)
   - 7 other critical files

✅ Add: 10 tests in tests/retry_tests.rs
```

**Phase 2 - Deferred (Week 1-2):**
```
⏳ Update: Remaining 91 files with retry patterns
   - Document in cleanup backlog
   - Prioritize by usage frequency
```

**Estimated Removal:** ~400 lines (high-priority only)

### Week 0 Day 7: Utilities + Rate Limiting

**Files to Create:**
```
✅ Create: crates/riptide-utils/src/time.rs (0.5 days)
✅ Create: crates/riptide-utils/src/error.rs (0.5 days)
✅ Create: crates/riptide-utils/src/rate_limit.rs (0.5 days)
✅ Add: 2 tests in tests/time_tests.rs
```

## Interface Contracts

### RedisPool

```rust
// Usage Example
use riptide_utils::redis::{RedisPool, RedisConfig};

let config = RedisConfig {
    max_connections: 10,
    connection_timeout: Duration::from_secs(5),
    retry_attempts: 3,
    health_check_interval: Duration::from_secs(30),
};

let pool = RedisPool::new("redis://localhost:6379", config).await?;
let conn = pool.get().await?;

// Use connection
redis::cmd("SET").arg("key").arg("value").query_async(&mut conn).await?;
```

**Key Features:**
- Health checks (PING every 30s)
- Connection pooling
- Retry on connection failure
- Arc-based sharing

### RetryPolicy

```rust
// Usage Example
use riptide_utils::retry::RetryPolicy;

let policy = RetryPolicy {
    max_attempts: 3,
    initial_delay: Duration::from_millis(100),
    max_delay: Duration::from_secs(10),
    backoff_factor: 2.0,
};

let result = policy.execute(|| async {
    // Your async operation here
    fetch_data().await
}).await?;
```

**Key Features:**
- Exponential backoff
- Jitter (optional)
- Generic over error types
- Async-friendly

### HTTP Client Factory

```rust
// Usage Example
use riptide_utils::http::{create_default_client, create_custom_client};

// Default: 30s timeout, "RipTide/1.0.0" user agent
let client = create_default_client()?;

// Custom
let client = create_custom_client(60, "CustomBot/1.0")?;
```

**Key Features:**
- Consistent configuration
- Connection pooling (10 idle per host)
- Compression (gzip, brotli)
- Customizable timeout/user-agent

## Test Coverage

**Total Test Count:** 25+ tests

**Breakdown:**
- Redis pooling: 8 tests
  - Connection reuse
  - Health checks
  - Retry on failure
  - Concurrent access
  - Pool exhaustion
  - Configuration validation
  - Error handling
  - Metrics tracking

- Retry logic: 10 tests
  - Exponential backoff
  - Jitter
  - Max attempts
  - Success after retry
  - Failure after max attempts
  - Generic error types
  - Async operations
  - Cancellation
  - Timeout handling
  - Edge cases

- HTTP factory: 5 tests
  - Default client creation
  - Custom client creation
  - Timeout configuration
  - User agent setting
  - Error handling

- Time utilities: 2 tests
  - ISO8601 formatting
  - Parsing

**Coverage Goal:** >80%

## Quality Gates

**Before Committing:**
```bash
# 1. Build succeeds
cargo build -p riptide-utils

# 2. All tests pass
cargo test -p riptide-utils

# 3. Zero warnings
RUSTFLAGS="-D warnings" cargo clippy -p riptide-utils -- -D warnings

# 4. Format check
cargo fmt -p riptide-utils -- --check

# 5. Documentation builds
cargo doc -p riptide-utils --no-deps
```

## Success Metrics

**Week 0 Complete When:**
- [x] `cargo build -p riptide-utils` succeeds
- [x] All 25+ utils tests pass
- [x] 3 crates updated (Redis pooling)
- [x] 8+ test files updated (HTTP clients)
- [x] 10+ files updated (retry logic high-priority)
- [x] Simple rate limiting works
- [x] All existing 41 test targets still pass
- [x] ~630 lines removed from codebase

## Common Patterns

### Pattern 1: Migrate Redis Usage

**Before:**
```rust
// crates/riptide-workers/src/scheduler.rs:193
let client = redis::Client::open(redis_url)?;
let manager = ConnectionManager::new(client).await?;
```

**After:**
```rust
// crates/riptide-workers/src/scheduler.rs:193
use riptide_utils::redis::{RedisPool, RedisConfig};

let pool = RedisPool::new(redis_url, RedisConfig::default()).await?;
let conn = pool.get().await?;
```

### Pattern 2: Migrate Retry Logic

**Before:**
```rust
// Custom retry implementation
let mut attempts = 0;
loop {
    match fetch().await {
        Ok(data) => return Ok(data),
        Err(e) if attempts < 3 => {
            attempts += 1;
            sleep(Duration::from_millis(100 * 2_u64.pow(attempts))).await;
        }
        Err(e) => return Err(e),
    }
}
```

**After:**
```rust
use riptide_utils::retry::RetryPolicy;

let policy = RetryPolicy::default();
let data = policy.execute(|| fetch()).await?;
```

### Pattern 3: Migrate HTTP Client

**Before:**
```rust
// Test file
let client = Client::builder()
    .timeout(Duration::from_secs(30))
    .user_agent("test-agent")
    .build()?;
```

**After:**
```rust
use riptide_utils::http::create_default_client;

let client = create_default_client()?;
```

## Notes for Developers

1. **Version Alignment:** Always use `{ workspace = true }` for shared dependencies
2. **No Internal Dependencies:** riptide-utils must NOT depend on other riptide-* crates
3. **Async-First:** All public APIs should be async where I/O is involved
4. **Error Propagation:** Use `Result<T, E>` with proper error context
5. **Testing:** Write tests BEFORE implementation (TDD)
6. **Documentation:** All public APIs must have doc comments

## Related Documents

- [Phase 0 Architecture Analysis](./phase0-architecture-analysis.md) - Full analysis
- [Definitive Roadmap](../roadmap/RIPTIDE-V1-DEFINITIVE-ROADMAP.md) - Complete roadmap
- [Shared Utilities Analysis](../analysis/shared-utilities-analysis.md) - Code duplication analysis

---

**Last Updated:** 2025-11-04
**Status:** ✅ Ready for Implementation
