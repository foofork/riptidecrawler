# Shared Utilities Analysis - riptide-utils Consolidation Plan

**Analysis Date:** 2025-11-04
**Objective:** Identify duplicated code patterns and shared utilities across crates for consolidation into `riptide-utils`

---

## Executive Summary

This analysis identified **significant code duplication** across the RipTide codebase, particularly in:
- **Redis connection management** (3+ crates duplicating pool creation)
- **HTTP client configuration** (8+ test files creating `reqwest::Client`)
- **Time/date utilities** (50+ files using `chrono::Utc::now()` patterns)
- **Error handling patterns** (extensive `thiserror::Error` usage)
- **Retry logic** (40+ implementations of exponential backoff)
- **Validation functions** (CLI and API duplicating validation)
- **Hash/crypto utilities** (5+ crates implementing hashing)

**Estimated Impact:**
- **Code reduction:** ~2,000-3,000 lines
- **Maintenance improvement:** Centralized error handling and retry logic
- **Bug reduction:** Single source of truth for critical utilities
- **Test coverage improvement:** Shared utilities = shared comprehensive tests

---

## 1. Redis Connection Management

### Current State: **CRITICAL DUPLICATION** ⚠️

#### Locations Found:
1. **`riptide-workers/src/scheduler.rs:193`**
   ```rust
   let client = redis::Client::open(url)?;
   let connection = client.get_multiplexed_async_connection().await?;
   ```

2. **`riptide-workers/src/queue.rs:56`**
   ```rust
   let client = redis::Client::open(redis_url)?;
   let redis = client.get_multiplexed_async_connection().await?;
   ```

3. **`riptide-persistence/tests/integration/mod.rs:92`**
   ```rust
   let client = redis::Client::open(redis_url)?;
   let mut conn = client.get_async_connection().await?;
   ```

### Issues Identified:
- ❌ **No connection pooling abstraction**
- ❌ **Inconsistent error handling** (some use `anyhow`, some use `context`)
- ❌ **No retry logic on connection failures**
- ❌ **No health checks**
- ❌ **Different connection types** (multiplexed vs async)

### Proposed Solution: `riptide-utils/src/redis.rs`

```rust
/// Centralized Redis connection management
pub struct RedisConnectionManager {
    client: redis::Client,
    config: RedisConfig,
}

pub struct RedisConfig {
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub retry_attempts: u32,
    pub health_check_interval: Duration,
}

impl RedisConnectionManager {
    /// Create a new Redis connection manager with health checks
    pub async fn new(url: &str, config: RedisConfig) -> Result<Self>;

    /// Get a multiplexed async connection with retry
    pub async fn get_multiplexed_connection(&self) -> Result<MultiplexedConnection>;

    /// Get a standard async connection
    pub async fn get_async_connection(&self) -> Result<AsyncConnection>;

    /// Health check with connection validation
    pub async fn health_check(&self) -> Result<HealthStatus>;
}
```

### Migration Complexity: **MEDIUM**
- 3 crates need updating
- Tests need updating to use shared utilities
- Existing code relatively straightforward

---

## 2. HTTP Client Configuration

### Current State: **HIGH DUPLICATION** ⚠️

#### Locations Found:
**8+ test files** creating identical `reqwest::Client` instances:

1. **`tests/e2e/real_world_tests.rs:12,49,84,119,153,205,243,268`** (8 instances!)
   ```rust
   let client = reqwest::Client::builder()
       .user_agent("Mozilla/5.0 (compatible; RipTideBot/1.0)")
       .build()?;
   ```

2. **`tests/component/cli/performance_tests.rs:61,176`**
   ```rust
   let client = reqwest::Client::new();
   ```

3. **`tests/cli/performance_tests.rs:61,176`** (duplicate of above)

### Issues Identified:
- ❌ **No centralized user-agent configuration**
- ❌ **No timeout defaults**
- ❌ **No retry configuration**
- ❌ **No connection pooling settings**
- ❌ **Tests creating clients repeatedly**

### Proposed Solution: `riptide-utils/src/http.rs`

```rust
/// Standard HTTP client builder with RipTide defaults
pub struct HttpClientBuilder {
    user_agent: String,
    timeout: Duration,
    pool_max_idle_per_host: usize,
    pool_idle_timeout: Option<Duration>,
}

impl HttpClientBuilder {
    /// Create with RipTide defaults
    pub fn default() -> Self {
        Self {
            user_agent: "Mozilla/5.0 (compatible; RipTideBot/1.0)".into(),
            timeout: Duration::from_secs(30),
            pool_max_idle_per_host: 10,
            pool_idle_timeout: Some(Duration::from_secs(90)),
        }
    }

    /// Build a configured reqwest::Client
    pub fn build(self) -> Result<reqwest::Client>;

    /// Create a test client with shorter timeouts
    pub fn for_testing() -> reqwest::Client;
}
```

### Migration Complexity: **LOW**
- Simple find/replace across test files
- Backward compatible
- No breaking changes

---

## 3. Time/Date Utilities

### Current State: **EXTENSIVE USAGE**

#### Pattern Analysis:
- **50+ files** use `chrono::Utc::now()`
- **Common patterns:**
  - Timestamps: `chrono::Utc::now().to_rfc3339()`
  - Expiry calculations: `Utc::now() + Duration::days(7)`
  - Time comparisons: `Utc::now() - last_time`
  - Session IDs: `format!("session_{}", Utc::now().timestamp())`

#### Key Locations:
1. **`riptide-intelligence/src/domain_profiling/profiler.rs`**
   - Lines: 100, 167, 173, 179, 208-209, 245, 265
   - Pattern: Timestamp generation and expiry logic

2. **`riptide-intelligence/src/failover.rs`**
   - Lines: 293, 308, 369, 491, 528, 537, 564, 568, 574
   - Pattern: Circuit breaker timing

3. **`riptide-security/src/api_keys.rs`**
   - Lines: 61, 96, 108, 116, 125
   - Pattern: API key expiration

### Proposed Solution: `riptide-utils/src/time.rs`

```rust
/// Centralized time utilities with consistent formatting
pub struct TimeUtils;

impl TimeUtils {
    /// Current UTC timestamp in RFC3339 format
    pub fn now_rfc3339() -> String {
        Utc::now().to_rfc3339()
    }

    /// Current UTC timestamp as Unix epoch
    pub fn now_timestamp() -> i64 {
        Utc::now().timestamp()
    }

    /// Generate time-based session ID
    pub fn session_id(prefix: &str) -> String {
        format!("{}_{}", prefix, Self::now_timestamp())
    }

    /// Check if timestamp has expired
    pub fn is_expired(expires_at: DateTime<Utc>) -> bool {
        Utc::now() > expires_at
    }

    /// Calculate expiry time from now
    pub fn expires_in(duration: Duration) -> DateTime<Utc> {
        Utc::now() + duration
    }

    /// Time elapsed since timestamp
    pub fn elapsed_since(since: DateTime<Utc>) -> Duration {
        Utc::now() - since
    }
}
```

### Migration Complexity: **LOW-MEDIUM**
- Many files to update (50+)
- Simple mechanical refactoring
- Good test coverage needed

---

## 4. Validation Functions

### Current State: **MODERATE DUPLICATION**

#### Locations Found:

1. **CLI Validation (`riptide-cli/src/commands/*.rs`):**
   - `search.rs:279` - `validate_args()`
   - `spider.rs:150` - `validate_args()`
   - `extract.rs:153` - `validate_args()`
   - `render.rs:149` - `validate_args()`

2. **API Validation:**
   - `cli-spec/src/validation.rs:15-109` - Spec validation
   - `riptide-streaming/src/config.rs:310` - Config validation
   - `riptide-persistence/src/config.rs:640` - Config validation

### Common Patterns:
```rust
// URL validation
if url.is_empty() {
    return Err(anyhow!("URL cannot be empty"));
}

// Numeric range validation
if limit == 0 {
    return Err(anyhow!("Limit must be greater than 0"));
}

// Timeout validation
if timeout == 0 {
    return Err(anyhow!("Timeout must be greater than 0"));
}
```

### Proposed Solution: `riptide-utils/src/validation.rs`

```rust
pub struct Validators;

impl Validators {
    /// Validate URL is non-empty and parseable
    pub fn validate_url(url: &str) -> Result<()> {
        if url.is_empty() {
            return Err(anyhow!("URL cannot be empty"));
        }
        url::Url::parse(url)
            .map(|_| ())
            .context("Invalid URL format")
    }

    /// Validate positive number
    pub fn validate_positive<T: PartialOrd + Default + Display>(
        value: T,
        name: &str,
    ) -> Result<()> {
        if value <= T::default() {
            return Err(anyhow!("{} must be greater than 0", name));
        }
        Ok(())
    }

    /// Validate range
    pub fn validate_range<T: PartialOrd + Display>(
        value: T,
        min: T,
        max: T,
        name: &str,
    ) -> Result<()> {
        if value < min || value > max {
            return Err(anyhow!("{} must be between {} and {}", name, min, max));
        }
        Ok(())
    }

    /// Validate non-empty collection
    pub fn validate_non_empty<T>(items: &[T], name: &str) -> Result<()> {
        if items.is_empty() {
            return Err(anyhow!("{} cannot be empty", name));
        }
        Ok(())
    }
}
```

### Migration Complexity: **LOW**
- Clear patterns to replace
- Improves consistency
- Better error messages

---

## 5. Error Handling & Conversion

### Current State: **EXTENSIVE USAGE**

#### Pattern Analysis:
- **100+ files** use `thiserror::Error`
- **50+ files** use `anyhow::Error`
- **Common conversions:**
  - `From<std::io::Error>`
  - `From<reqwest::Error>`
  - `From<redis::RedisError>`
  - `From<url::ParseError>`

#### Key Implementations:
1. **`riptide-types/src/errors.rs:84`**
   ```rust
   Other(#[from] anyhow::Error)
   ```

2. **`riptide-api/src/errors.rs`** - Comprehensive API error types
3. **`riptide-streaming/src/lib.rs:30`** - Streaming errors
4. **Resource manager errors** - Timeout/exhaustion helpers

### Proposed Solution: `riptide-utils/src/errors.rs`

```rust
/// Common error conversion utilities
pub trait ErrorExt {
    /// Add context to any error
    fn with_context<C: Display>(self, context: C) -> anyhow::Error;

    /// Check if error is retryable
    fn is_retryable(&self) -> bool;
}

/// Standard error builders
pub struct ErrorBuilders;

impl ErrorBuilders {
    pub fn timeout(operation: impl Into<String>, duration: Duration) -> anyhow::Error {
        anyhow!(
            "Operation '{}' timed out after {:?}",
            operation.into(),
            duration
        )
    }

    pub fn exhausted(resource: impl Into<String>) -> anyhow::Error {
        anyhow!("Resource '{}' exhausted", resource.into())
    }

    pub fn not_found(item: impl Into<String>) -> anyhow::Error {
        anyhow!("'{}' not found", item.into())
    }

    pub fn invalid_config(reason: impl Into<String>) -> anyhow::Error {
        anyhow!("Invalid configuration: {}", reason.into())
    }
}
```

### Migration Complexity: **MEDIUM-HIGH**
- Many error types across crates
- Need to maintain backward compatibility
- Gradual migration recommended

---

## 6. Retry Logic & Exponential Backoff

### Current State: **CRITICAL DUPLICATION** ⚠️

#### Locations Found (40+ implementations):

1. **`riptide-fetch/src/fetch.rs`**
   - Lines: 141, 223, 777, 933, 952, 988, 1008, 1169, 1202, 1228
   - Full retry implementation with backoff

2. **`riptide-intelligence/src/smart_retry.rs`**
   - Lines: 423, 432, 449, 460, 545, 721, 757
   - Smart retry with provider-specific logic

3. **`riptide-workers/src/job.rs:241`**
   - Retry calculation for job scheduling

4. **`riptide-cli/src/client.rs:311`**
   - HTTP request retry logic

5. **`riptide-api/src/pipeline.rs:174`**
   - Pipeline retry configuration

### Common Pattern:
```rust
let mut delay = initial_delay;
for attempt in 0..max_attempts {
    match operation().await {
        Ok(result) => return Ok(result),
        Err(e) if is_retryable(&e) => {
            tokio::time::sleep(delay).await;
            delay = (delay * 2).min(max_delay);
        }
        Err(e) => return Err(e),
    }
}
```

### Proposed Solution: `riptide-utils/src/retry.rs`

```rust
/// Configurable retry policy with exponential backoff
#[derive(Clone, Debug)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
    pub jitter: bool,
}

impl RetryPolicy {
    /// Standard defaults
    pub fn standard() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
            jitter: true,
        }
    }

    /// Aggressive retry for transient failures
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(10),
            backoff_factor: 1.5,
            jitter: true,
        }
    }

    /// Execute with retry
    pub async fn execute<F, T, E, Fut>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: RetryableError,
    {
        // Implementation with exponential backoff
    }

    /// Calculate delay for attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay = self.initial_delay.as_millis() as f64
            * self.backoff_factor.powi(attempt as i32);
        let delay = delay.min(self.max_delay.as_millis() as f64);

        if self.jitter {
            // Add random jitter ±25%
            let jitter = rand::random::<f64>() * 0.5 - 0.25;
            delay * (1.0 + jitter)
        } else {
            delay
        }

        Duration::from_millis(delay as u64)
    }
}

/// Trait for errors that can be retried
pub trait RetryableError {
    fn is_retryable(&self) -> bool;
}
```

### Migration Complexity: **HIGH**
- 40+ implementations to consolidate
- Different retry strategies across crates
- Need careful testing
- Phased migration recommended

---

## 7. Hash & Cryptography Utilities

### Current State: **MODERATE DUPLICATION**

#### Locations Found:

1. **`riptide-security/src/api_keys.rs:83`**
   ```rust
   fn hash_key(key: &str) -> String {
       // SHA-256 hashing
   }
   ```

2. **`riptide-cache/src/integrated.rs:258,274`**
   ```rust
   fn hash_options(options: &HashMap<String, String>) -> String
   fn hash_url(url: &str) -> String
   ```

3. **`riptide-intelligence/src/hot_reload.rs:563`**
   ```rust
   fn calculate_config_hash(config: &IntelligenceConfig) -> String
   ```

4. **`riptide-spider/src/url_utils.rs:128`**
   ```rust
   fn hash_function(&self, item: &str, seed: usize) -> usize
   ```

5. **`riptide-persistence/src/cache.rs:613`**
   ```rust
   fn calculate_hash<T: Serialize>(&self, data: &T) -> Result<String> {
       let hash = blake3::hash(&bytes);
   }
   ```

### Issues:
- ❌ **Different hash algorithms** (SHA-256, Blake3, DefaultHasher)
- ❌ **Inconsistent serialization** for hashing
- ❌ **No standard cache key generation**

### Proposed Solution: `riptide-utils/src/crypto.rs`

```rust
pub struct HashUtils;

impl HashUtils {
    /// SHA-256 hash of string
    pub fn sha256(input: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Blake3 hash (faster)
    pub fn blake3(input: &[u8]) -> String {
        blake3::hash(input).to_hex().to_string()
    }

    /// Hash serializable data consistently
    pub fn hash_data<T: Serialize>(data: &T) -> Result<String> {
        let bytes = serde_json::to_vec(data)?;
        Ok(Self::blake3(&bytes))
    }

    /// Generate cache key from URL and options
    pub fn cache_key(url: &str, options: &HashMap<String, String>) -> String {
        let combined = format!("{}{:?}", url, options);
        Self::blake3(combined.as_bytes())
    }

    /// Bloom filter hash with seed
    pub fn bloom_hash(item: &str, seed: usize) -> usize {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        item.hash(&mut hasher);
        hasher.finish() as usize
    }
}
```

### Migration Complexity: **MEDIUM**
- 5 crates to update
- Need to ensure hash compatibility for cache keys
- Test thoroughly

---

## 8. Serialization Helpers

### Current State: **WIDESPREAD USAGE**

#### Pattern Analysis:
- **100+ occurrences** of `serde_json::to_string`
- **100+ occurrences** of `serde_json::from_str`
- Common in tests and API responses

### Proposed Solution: `riptide-utils/src/serde.rs`

```rust
pub struct SerdeUtils;

impl SerdeUtils {
    /// Serialize to JSON string with error context
    pub fn to_json<T: Serialize>(value: &T) -> Result<String> {
        serde_json::to_string(value)
            .context("Failed to serialize to JSON")
    }

    /// Serialize to pretty JSON
    pub fn to_json_pretty<T: Serialize>(value: &T) -> Result<String> {
        serde_json::to_string_pretty(value)
            .context("Failed to serialize to pretty JSON")
    }

    /// Deserialize from JSON string
    pub fn from_json<T: DeserializeOwned>(json: &str) -> Result<T> {
        serde_json::from_str(json)
            .context("Failed to deserialize from JSON")
    }

    /// Safe deserialize with default fallback
    pub fn from_json_or_default<T: DeserializeOwned + Default>(json: &str) -> T {
        serde_json::from_str(json).unwrap_or_default()
    }
}
```

### Migration Complexity: **LOW**
- Optional enhancement
- Can coexist with direct usage
- Provides better error messages

---

## 9. Timeout Utilities

### Current State: **EXTENSIVE USAGE**

#### Pattern Analysis:
- **150+ occurrences** of `tokio::time::timeout`
- Common pattern: wrapping operations with timeouts
- Adaptive timeout in multiple places

#### Key Locations:
1. **`tests/regression/adaptive_timeout_tests.rs`** - Full timeout wrapper
2. **`riptide-pool/tests/*.rs`** - Pool operation timeouts
3. **`tests/lib.rs:128`** - Test helper `with_timeout`

### Proposed Solution: `riptide-utils/src/timeout.rs`

```rust
/// Timeout wrapper with operation tracking
pub struct TimeoutWrapper<T> {
    inner: T,
    timeout: Duration,
}

impl<T> TimeoutWrapper<T> {
    pub fn with_timeout(inner: T, timeout: Duration) -> Self {
        Self { inner, timeout }
    }

    pub async fn execute<F, R>(&self, operation: F) -> Result<R>
    where
        F: Future<Output = Result<R>>,
    {
        tokio::time::timeout(self.timeout, operation)
            .await
            .context("Operation timed out")?
    }
}

/// Adaptive timeout based on operation history
pub struct AdaptiveTimeout {
    min_timeout: Duration,
    max_timeout: Duration,
    percentile: f64, // e.g., 0.95 for p95
}

impl AdaptiveTimeout {
    pub fn new(min: Duration, max: Duration) -> Self {
        Self {
            min_timeout: min,
            max_timeout: max,
            percentile: 0.95,
        }
    }

    /// Calculate timeout based on history
    pub fn calculate_timeout(&self, durations: &[Duration]) -> Duration {
        // Implementation
    }
}
```

### Migration Complexity: **MEDIUM**
- Many uses across codebase
- Optional - can migrate gradually
- Good for standardization

---

## Priority Order for Consolidation

### Phase 1: **CRITICAL** (Immediate Impact)
1. **Redis Connection Management** - Eliminates critical duplication
2. **Retry Logic & Backoff** - 40+ implementations to consolidate
3. **Error Handling Utilities** - Improves consistency

**Estimated Time:** 2-3 weeks
**Lines Reduced:** ~1,500

### Phase 2: **HIGH VALUE** (Quality Improvement)
4. **HTTP Client Configuration** - Better testing
5. **Validation Functions** - Consistency
6. **Hash/Crypto Utilities** - Security

**Estimated Time:** 1-2 weeks
**Lines Reduced:** ~800

### Phase 3: **OPTIMIZATION** (Nice to Have)
7. **Time/Date Utilities** - Convenience
8. **Serialization Helpers** - Error messages
9. **Timeout Utilities** - Standardization

**Estimated Time:** 1 week
**Lines Reduced:** ~500

---

## What riptide-utils Should Contain

### Recommended Structure:

```
crates/riptide-utils/
├── src/
│   ├── lib.rs              # Re-exports
│   ├── redis.rs            # Redis connection management
│   ├── http.rs             # HTTP client builders
│   ├── retry.rs            # Retry policies & exponential backoff
│   ├── errors.rs           # Error utilities & conversions
│   ├── validation.rs       # Common validators
│   ├── crypto.rs           # Hash & crypto utilities
│   ├── time.rs             # Time/date helpers
│   ├── serde.rs            # Serialization helpers
│   └── timeout.rs          # Timeout wrappers
├── tests/
│   ├── redis_tests.rs
│   ├── retry_tests.rs
│   └── ...
└── Cargo.toml
```

### Dependencies:
```toml
[dependencies]
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1.35", features = ["time", "sync"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha2 = "0.10"
blake3 = "1.5"
url = "2.5"
rand = "0.8"
```

---

## What Can Stay in Place

### DO NOT Move:
1. **Domain-specific logic** - Keep in respective crates
2. **API handler functions** - Stay in `riptide-api`
3. **CLI command implementations** - Stay in `riptide-cli`
4. **WASM module pool** - Stay in `riptide-pool`
5. **Spider strategies** - Stay in `riptide-spider`
6. **Intelligence providers** - Stay in `riptide-intelligence`

### Keep Specialized:
- Config structs specific to one crate
- Business logic
- Complex state management
- Feature-specific implementations

---

## Migration Complexity Estimates

| Utility | Complexity | Files Affected | Risk Level | Testing Effort |
|---------|-----------|----------------|------------|----------------|
| Redis Pools | MEDIUM | 3 | LOW | Medium |
| HTTP Clients | LOW | 15+ | LOW | Low |
| Retry Logic | HIGH | 40+ | MEDIUM | High |
| Error Handling | MEDIUM-HIGH | 100+ | MEDIUM | High |
| Validation | LOW | 10 | LOW | Low |
| Crypto/Hash | MEDIUM | 5 | MEDIUM | Medium |
| Time Utils | LOW-MEDIUM | 50+ | LOW | Low |
| Serialization | LOW | Optional | LOW | Low |
| Timeouts | MEDIUM | Optional | LOW | Medium |

---

## Recommendations

### Immediate Actions:
1. ✅ **Create `riptide-utils` crate** with basic structure
2. ✅ **Start with Redis utilities** (Phase 1, highest impact)
3. ✅ **Implement retry logic consolidation** (Phase 1, critical)
4. ✅ **Add comprehensive tests** for all utilities

### Migration Strategy:
1. **Create utilities** without removing old code
2. **Update one crate at a time** to use new utilities
3. **Run full test suite** after each migration
4. **Remove old implementations** only after validation
5. **Update documentation** as you go

### Success Metrics:
- **Code reduction:** >2,000 lines
- **Test coverage:** >90% for utilities
- **Zero regressions** in existing functionality
- **Performance:** No degradation
- **DRY principle:** Single source of truth

---

## File Location Reference

### Redis Connections:
- `crates/riptide-workers/src/scheduler.rs:193`
- `crates/riptide-workers/src/queue.rs:56`
- `crates/riptide-persistence/tests/integration/mod.rs:92`

### HTTP Clients:
- `tests/e2e/real_world_tests.rs:12,49,84,119,153,205,243,268`
- `tests/component/cli/performance_tests.rs:61,176`
- `tests/cli/performance_tests.rs:61,176`

### Retry Logic:
- `crates/riptide-fetch/src/fetch.rs:141,223,777+`
- `crates/riptide-intelligence/src/smart_retry.rs:423+`
- `crates/riptide-workers/src/job.rs:241`
- `crates/riptide-cli/src/client.rs:311`

### Validation:
- `crates/riptide-cli/src/commands/search.rs:279`
- `crates/riptide-cli/src/commands/spider.rs:150`
- `crates/riptide-cli/src/commands/extract.rs:153`
- `crates/riptide-cli/src/commands/render.rs:149`

### Hash/Crypto:
- `crates/riptide-security/src/api_keys.rs:83`
- `crates/riptide-cache/src/integrated.rs:258,274`
- `crates/riptide-intelligence/src/hot_reload.rs:563`
- `crates/riptide-persistence/src/cache.rs:613`

---

## Conclusion

This analysis reveals **significant opportunities** for consolidation. The `riptide-utils` crate should focus on:

1. **Infrastructure utilities** (Redis, HTTP)
2. **Cross-cutting concerns** (retry, errors, validation)
3. **Common helpers** (time, crypto, serialization)

**Do not** try to consolidate everything at once. Follow the phased approach, starting with **Redis and Retry logic** for maximum impact.

**Next Steps:**
1. Review this analysis with the team
2. Create `riptide-utils` skeleton
3. Implement Phase 1 utilities
4. Begin migration one crate at a time
