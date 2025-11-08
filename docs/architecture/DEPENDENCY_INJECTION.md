# Dependency Injection Patterns for Riptide

**Document Version:** 1.0
**Date:** 2025-01-08
**Purpose:** Best practices for dependency injection in Phase 0 and beyond

## Table of Contents

1. [Why Dependency Injection](#why-dependency-injection)
2. [Injection Patterns](#injection-patterns)
3. [Trait Object Patterns](#trait-object-patterns)
4. [Testing Patterns](#testing-patterns)
5. [Ownership Patterns](#ownership-patterns)
6. [Common Pitfalls](#common-pitfalls)
7. [Examples](#examples)

---

## Why Dependency Injection

### Problems with Direct Dependencies

**Before (tightly coupled):**
```rust
pub struct ExtractionService {
    redis_client: redis::Client,  // Concrete type
}

impl ExtractionService {
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { redis_client: client })
    }
}

// Problems:
// ❌ Cannot test without Redis
// ❌ Cannot swap implementations
// ❌ Hard to mock for testing
// ❌ Violates Dependency Inversion Principle
```

### Benefits of Dependency Injection

**After (loosely coupled):**
```rust
pub struct ExtractionService {
    cache: Arc<dyn CacheStorage>,  // Abstract interface
}

impl ExtractionService {
    pub fn new(cache: Arc<dyn CacheStorage>) -> Self {
        Self { cache }
    }
}

// Benefits:
// ✅ Easy to test with mocks
// ✅ Can swap implementations at runtime
// ✅ Follows Dependency Inversion Principle
// ✅ Clear dependencies in constructor
```

---

## Injection Patterns

### Pattern 1: Constructor Injection (Preferred)

**When to use:** Most cases - clear, explicit dependencies

```rust
pub struct ExtractionService {
    cache: Arc<dyn CacheStorage>,
    parser: Arc<dyn RobotsParser>,
    metrics: Arc<Metrics>,
}

impl ExtractionService {
    pub fn new(
        cache: Arc<dyn CacheStorage>,
        parser: Arc<dyn RobotsParser>,
        metrics: Arc<Metrics>,
    ) -> Self {
        Self {
            cache,
            parser,
            metrics,
        }
    }
}

// Usage
let service = ExtractionService::new(
    Arc::new(RedisCache::new(...)),
    Arc::new(DefaultRobotsParser::new()),
    Arc::new(Metrics::new()),
);
```

**Pros:**
- Dependencies are explicit
- Immutable after construction
- Easy to see what's required
- Compiler enforces completeness

**Cons:**
- Long parameter lists for many dependencies
- All dependencies required upfront

### Pattern 2: Builder Pattern

**When to use:** Many optional dependencies or complex configuration

```rust
pub struct ExtractionServiceBuilder {
    cache: Option<Arc<dyn CacheStorage>>,
    parser: Option<Arc<dyn RobotsParser>>,
    metrics: Option<Arc<Metrics>>,
    config: ExtractionConfig,
}

impl ExtractionServiceBuilder {
    pub fn new() -> Self {
        Self {
            cache: None,
            parser: None,
            metrics: None,
            config: ExtractionConfig::default(),
        }
    }

    pub fn cache(mut self, cache: Arc<dyn CacheStorage>) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn parser(mut self, parser: Arc<dyn RobotsParser>) -> Self {
        self.parser = Some(parser);
        self
    }

    pub fn config(mut self, config: ExtractionConfig) -> Self {
        self.config = config;
        self
    }

    pub fn build(self) -> Result<ExtractionService> {
        Ok(ExtractionService {
            cache: self.cache.ok_or_else(|| anyhow!("cache required"))?,
            parser: self.parser.unwrap_or_else(|| Arc::new(DefaultRobotsParser::new())),
            metrics: self.metrics.unwrap_or_else(|| Arc::new(Metrics::new())),
            config: self.config,
        })
    }
}

// Usage
let service = ExtractionServiceBuilder::new()
    .cache(Arc::new(RedisCache::new(...)))
    .config(my_config)
    .build()?;
```

**Pros:**
- Clean API for many dependencies
- Optional dependencies clear
- Can provide sensible defaults
- Fluent, readable code

**Cons:**
- More boilerplate
- Runtime errors if required deps missing

### Pattern 3: Trait-Based Factory

**When to use:** Need different implementations based on configuration

```rust
pub trait CacheFactory: Send + Sync {
    fn create(&self, config: &CacheConfig) -> Result<Arc<dyn CacheStorage>>;
}

pub struct RedisCacheFactory;

impl CacheFactory for RedisCacheFactory {
    fn create(&self, config: &CacheConfig) -> Result<Arc<dyn CacheStorage>> {
        Ok(Arc::new(RedisCache::new(&config.redis_url, config.ttl)?))
    }
}

pub struct InMemoryCacheFactory;

impl CacheFactory for InMemoryCacheFactory {
    fn create(&self, config: &CacheConfig) -> Result<Arc<dyn CacheStorage>> {
        Ok(Arc::new(InMemoryCache::new(config.ttl)))
    }
}

// Usage
let factory: Box<dyn CacheFactory> = if config.use_redis {
    Box::new(RedisCacheFactory)
} else {
    Box::new(InMemoryCacheFactory)
};

let cache = factory.create(&config.cache)?;
let service = ExtractionService::new(cache);
```

**Pros:**
- Deferred creation
- Easy to swap implementations
- Can encapsulate complex initialization

**Cons:**
- Extra indirection
- More complex than direct injection

### Pattern 4: Service Locator (Avoid)

**When to use:** Legacy code only - not recommended for new code

```rust
// ❌ Anti-pattern - don't use in new code
pub struct ServiceLocator {
    cache: Arc<dyn CacheStorage>,
    parser: Arc<dyn RobotsParser>,
}

impl ServiceLocator {
    pub fn get_cache(&self) -> Arc<dyn CacheStorage> {
        Arc::clone(&self.cache)
    }

    pub fn get_parser(&self) -> Arc<dyn RobotsParser> {
        Arc::clone(&self.parser)
    }
}

pub struct ExtractionService {
    locator: Arc<ServiceLocator>,  // Hidden dependencies
}
```

**Problems:**
- Dependencies hidden
- Runtime failures instead of compile-time
- Hard to test
- Violates explicit dependencies principle

---

## Trait Object Patterns

### Pattern 1: Arc\<dyn Trait\> (Preferred)

**When to use:** Shared ownership, multiple consumers

```rust
pub struct ExtractionService {
    cache: Arc<dyn CacheStorage>,  // Shared, thread-safe
}

// Multiple services can share same cache
let cache = Arc::new(RedisCache::new(...));
let service1 = ExtractionService::new(Arc::clone(&cache));
let service2 = AnotherService::new(Arc::clone(&cache));
```

**Pros:**
- Thread-safe (Send + Sync)
- Shared ownership
- No lifetime parameters
- Can pass across threads

**Cons:**
- Slight runtime overhead (vtable)
- Heap allocation
- No static dispatch

### Pattern 2: Box\<dyn Trait\>

**When to use:** Single owner, ownership transfer

```rust
pub struct Pipeline {
    strategy: Box<dyn ExtractionStrategy>,  // Owned, exclusive
}

impl Pipeline {
    pub fn new(strategy: Box<dyn ExtractionStrategy>) -> Self {
        Self { strategy }
    }

    pub fn replace_strategy(&mut self, new_strategy: Box<dyn ExtractionStrategy>) {
        self.strategy = new_strategy;
    }
}
```

**Pros:**
- Clear ownership
- Can replace at runtime
- No shared state concerns

**Cons:**
- Not thread-safe (not Send + Sync by default)
- Only one owner
- No cloning

### Pattern 3: &dyn Trait

**When to use:** Temporary, borrowed access

```rust
pub fn validate_data(
    data: &Value,
    schema_store: &dyn SchemaStore,  // Borrowed, no ownership
) -> Result<bool> {
    schema_store.validate("schema.v1", data).await
}

// Usage
let store = InMemorySchemaStore::new();
let valid = validate_data(&my_data, &store)?;
```

**Pros:**
- No heap allocation
- No ownership transfer
- Fast for short operations

**Cons:**
- Lifetime parameters needed
- Cannot store in structs easily
- Not suitable for long-lived dependencies

### Pattern 4: Generic Types (Alternative)

**When to use:** Performance critical, compile-time dispatch

```rust
pub struct ExtractionService<C: CacheStorage> {
    cache: Arc<C>,  // Concrete type, monomorphized
}

impl<C: CacheStorage> ExtractionService<C> {
    pub fn new(cache: Arc<C>) -> Self {
        Self { cache }
    }
}

// Usage
let cache = Arc::new(RedisCache::new(...));
let service: ExtractionService<RedisCache> = ExtractionService::new(cache);
```

**Pros:**
- Zero-cost abstraction (static dispatch)
- No vtable overhead
- Inlined method calls

**Cons:**
- Type signature complexity
- Cannot swap implementations at runtime
- Code bloat from monomorphization

---

## Testing Patterns

### Pattern 1: Mock Implementation

```rust
// Create mock for testing
struct MockCache {
    storage: DashMap<String, Vec<u8>>,
}

#[async_trait]
impl CacheStorage for MockCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.storage.get(key).map(|v| v.clone()))
    }

    async fn set(&self, key: &str, value: &[u8], _ttl: Option<Duration>) -> Result<()> {
        self.storage.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    // ... other methods
}

// Use in test
#[tokio::test]
async fn test_extraction_with_cache() {
    let cache = Arc::new(MockCache::new());
    let service = ExtractionService::new(cache);

    let result = service.extract("https://example.com").await.unwrap();
    assert_eq!(result, "expected");
}
```

### Pattern 2: Test Doubles with Verification

```rust
struct SpyCache {
    storage: DashMap<String, Vec<u8>>,
    get_calls: Arc<AtomicUsize>,
    set_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl CacheStorage for SpyCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.get_calls.fetch_add(1, Ordering::Relaxed);
        Ok(self.storage.get(key).map(|v| v.clone()))
    }

    async fn set(&self, key: &str, value: &[u8], _ttl: Option<Duration>) -> Result<()> {
        self.set_calls.fetch_add(1, Ordering::Relaxed);
        self.storage.insert(key.to_string(), value.to_vec());
        Ok(())
    }
}

#[tokio::test]
async fn test_cache_usage() {
    let cache = Arc::new(SpyCache::new());
    let service = ExtractionService::new(Arc::clone(&cache));

    // Perform operations
    service.extract("https://example.com").await.unwrap();

    // Verify cache was used
    assert_eq!(cache.get_calls.load(Ordering::Relaxed), 1);
    assert_eq!(cache.set_calls.load(Ordering::Relaxed), 1);
}
```

### Pattern 3: Conditional Mock (mockall crate)

```rust
#[cfg(test)]
use mockall::mock;

#[cfg(test)]
mock! {
    Cache {}

    #[async_trait]
    impl CacheStorage for Cache {
        async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
        async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_with_mock() {
        let mut mock_cache = MockCache::new();

        mock_cache
            .expect_get()
            .with(eq("test-key"))
            .returning(|_| Ok(Some(b"cached-value".to_vec())));

        let service = ExtractionService::new(Arc::new(mock_cache));
        // ... test
    }
}
```

---

## Ownership Patterns

### Pattern 1: Shared Dependencies (Arc)

**When to use:** Multiple consumers need same dependency

```rust
// Shared cache across services
let cache = Arc::new(RedisCache::new(...));

let extraction_service = ExtractionService::new(Arc::clone(&cache));
let validation_service = ValidationService::new(Arc::clone(&cache));
let metrics_service = MetricsService::new(Arc::clone(&cache));

// All share same cache instance
```

### Pattern 2: Owned Dependencies

**When to use:** Single owner, exclusive access

```rust
pub struct Pipeline {
    owned_strategy: Box<dyn Strategy>,  // Owned exclusively
}

impl Pipeline {
    pub fn take_strategy(mut self) -> Box<dyn Strategy> {
        std::mem::replace(
            &mut self.owned_strategy,
            Box::new(DefaultStrategy::new())
        )
    }
}
```

### Pattern 3: Borrowed Dependencies

**When to use:** Function-level, temporary access

```rust
pub async fn process_with_cache(
    data: &Data,
    cache: &dyn CacheStorage,  // Borrowed
) -> Result<ProcessedData> {
    // Use cache temporarily
    if let Some(cached) = cache.get(&data.id).await? {
        return Ok(serde_json::from_slice(&cached)?);
    }

    // Process and cache
    let processed = process(data)?;
    cache.set(&data.id, &serde_json::to_vec(&processed)?, None).await?;

    Ok(processed)
}
```

### Pattern 4: Interior Mutability (Arc\<RwLock\> or Arc\<Mutex\>)

**When to use:** Shared mutable state (use sparingly)

```rust
pub struct StatefulService {
    cache: Arc<dyn CacheStorage>,  // Immutable trait object
    state: Arc<RwLock<ServiceState>>,  // Mutable state
}

impl StatefulService {
    pub async fn update_state(&self, new_state: ServiceState) {
        let mut state = self.state.write().await;
        *state = new_state;
    }
}
```

---

## Common Pitfalls

### Pitfall 1: Cloning Arc in Loops

```rust
// ❌ Bad - unnecessary clones
for item in items {
    let cache = Arc::clone(&self.cache);  // Clone in loop
    tokio::spawn(async move {
        cache.get(&item.id).await
    });
}

// ✅ Good - clone once, share reference
let cache = Arc::clone(&self.cache);
for item in items {
    let cache_ref = Arc::clone(&cache);  // Or just use &cache
    tokio::spawn(async move {
        cache_ref.get(&item.id).await
    });
}
```

### Pitfall 2: Trait Object Lifetime Issues

```rust
// ❌ Bad - lifetime parameters everywhere
pub struct Service<'a> {
    cache: &'a dyn CacheStorage,
}

impl<'a> Service<'a> {
    pub fn new(cache: &'a dyn CacheStorage) -> Self {
        Self { cache }
    }
}

// ✅ Good - use Arc to avoid lifetimes
pub struct Service {
    cache: Arc<dyn CacheStorage>,
}

impl Service {
    pub fn new(cache: Arc<dyn CacheStorage>) -> Self {
        Self { cache }
    }
}
```

### Pitfall 3: Not Using Send + Sync

```rust
// ❌ Bad - not thread-safe
pub trait CacheStorage {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
}

// Cannot use Arc<dyn CacheStorage> across threads!

// ✅ Good - require Send + Sync
pub trait CacheStorage: Send + Sync {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
}

// Can use Arc<dyn CacheStorage> across threads
```

### Pitfall 4: Unnecessary Boxing

```rust
// ❌ Bad - double indirection
pub struct Service {
    cache: Box<Arc<dyn CacheStorage>>,  // Box + Arc = unnecessary
}

// ✅ Good - just Arc
pub struct Service {
    cache: Arc<dyn CacheStorage>,
}
```

---

## Examples

### Example 1: Complete Service with DI

```rust
use std::sync::Arc;
use anyhow::Result;

// Dependencies
pub trait CacheStorage: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: &[u8]) -> Result<()>;
}

pub trait RobotsParser: Send + Sync {
    fn is_allowed(&self, url: &str) -> bool;
}

pub trait Metrics: Send + Sync {
    async fn record(&self, name: &str, value: f64);
}

// Service with constructor injection
pub struct ExtractionService {
    cache: Arc<dyn CacheStorage>,
    robots: Arc<dyn RobotsParser>,
    metrics: Arc<dyn Metrics>,
}

impl ExtractionService {
    pub fn new(
        cache: Arc<dyn CacheStorage>,
        robots: Arc<dyn RobotsParser>,
        metrics: Arc<dyn Metrics>,
    ) -> Self {
        Self {
            cache,
            robots,
            metrics,
        }
    }

    pub async fn extract(&self, url: &str) -> Result<String> {
        // Check robots.txt
        if !self.robots.is_allowed(url) {
            return Err(anyhow::anyhow!("Blocked by robots.txt"));
        }

        // Check cache
        if let Some(cached) = self.cache.get(url).await? {
            self.metrics.record("cache_hit", 1.0).await;
            return Ok(String::from_utf8(cached)?);
        }

        // Extract content
        let content = self.fetch_content(url).await?;

        // Cache result
        self.cache.set(url, content.as_bytes()).await?;
        self.metrics.record("extraction", 1.0).await;

        Ok(content)
    }

    async fn fetch_content(&self, _url: &str) -> Result<String> {
        // Implementation
        Ok("content".to_string())
    }
}

// Production initialization
#[tokio::main]
async fn main() -> Result<()> {
    let cache = Arc::new(RedisCache::new("redis://localhost")?);
    let robots = Arc::new(DefaultRobotsParser::new());
    let metrics = Arc::new(PrometheusMetrics::new());

    let service = ExtractionService::new(cache, robots, metrics);

    let result = service.extract("https://example.com").await?;
    println!("Extracted: {}", result);

    Ok(())
}

// Test with mocks
#[cfg(test)]
mod tests {
    use super::*;

    struct MockCache;
    #[async_trait]
    impl CacheStorage for MockCache {
        async fn get(&self, _: &str) -> Result<Option<Vec<u8>>> {
            Ok(None)
        }
        async fn set(&self, _: &str, _: &[u8]) -> Result<()> {
            Ok(())
        }
    }

    struct MockRobots;
    impl RobotsParser for MockRobots {
        fn is_allowed(&self, _: &str) -> bool {
            true
        }
    }

    struct MockMetrics;
    #[async_trait]
    impl Metrics for MockMetrics {
        async fn record(&self, _: &str, _: f64) {}
    }

    #[tokio::test]
    async fn test_extraction() {
        let service = ExtractionService::new(
            Arc::new(MockCache),
            Arc::new(MockRobots),
            Arc::new(MockMetrics),
        );

        let result = service.extract("https://example.com").await.unwrap();
        assert_eq!(result, "content");
    }
}
```

### Example 2: Builder Pattern with DI

```rust
pub struct ExtractionServiceBuilder {
    cache: Option<Arc<dyn CacheStorage>>,
    robots: Option<Arc<dyn RobotsParser>>,
    metrics: Option<Arc<dyn Metrics>>,
}

impl ExtractionServiceBuilder {
    pub fn new() -> Self {
        Self {
            cache: None,
            robots: None,
            metrics: None,
        }
    }

    pub fn cache(mut self, cache: Arc<dyn CacheStorage>) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn robots(mut self, robots: Arc<dyn RobotsParser>) -> Self {
        self.robots = Some(robots);
        self
    }

    pub fn metrics(mut self, metrics: Arc<dyn Metrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn build(self) -> Result<ExtractionService> {
        Ok(ExtractionService {
            cache: self.cache.ok_or_else(|| anyhow::anyhow!("cache required"))?,
            robots: self.robots.unwrap_or_else(|| Arc::new(DefaultRobotsParser::new())),
            metrics: self.metrics.unwrap_or_else(|| Arc::new(NoOpMetrics)),
        })
    }
}

// Usage
let service = ExtractionServiceBuilder::new()
    .cache(Arc::new(RedisCache::new(...)?))
    .metrics(Arc::new(PrometheusMetrics::new()))
    .build()?;
```

---

## Summary

**Recommended Patterns:**
1. **Constructor Injection** with `Arc<dyn Trait>` for most cases
2. **Builder Pattern** for complex initialization
3. **Mock implementations** for testing
4. **Send + Sync** requirement for all shared traits

**Key Principles:**
- Depend on abstractions, not concretions
- Inject dependencies explicitly
- Make dependencies clear in constructors
- Use Arc for shared ownership
- Prefer compile-time errors over runtime errors

---

**Document Maintainer:** System Architect
**Last Review:** 2025-01-08
**Next Review:** After Phase 0 completion
