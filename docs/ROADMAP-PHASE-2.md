# Phase 2: P1 Infrastructure (Weeks 4-6)

**Duration**: 3 weeks
**Goal**: Complete port-adapter infrastructure and migrate 12 facades
**Key Outcomes**:
- 7 additional P1 ports created (STOP recreating existing ports)
- 12 production adapters implemented
- 12 facades migrated to ApplicationContext
- Infrastructure violations: 32 → 12

---

## Phase Overview

Phase 2 builds out the remaining infrastructure layer:

1. **Sprint 4**: Create 2 missing P1 ports (SearchEngine, PdfProcessor) - **NOT 7!**
   - Port audit in Week 0 found we already have most ports
   - Only create genuinely missing ports

2. **Sprint 5**: Implement 12 production adapters
   - For all 12 ports (5 from Sprint 2 + 2 from Sprint 4 + 5 existing)
   - Replace in-memory adapters with production (Redis, Postgres, etc.)

3. **Sprint 6**: Migrate 12 facades to ApplicationContext
   - One facade per day strategy
   - Per-facade smoke tests

---

## Sprint 4: Missing P1 Ports (Week 4)

**CRITICAL**: Week 0 port audit found we only need **2 new ports**, not 7!

### Day 1: Port Audit Review (Prevent Duplicate Work)

#### Task 4.1: Review Week 0 Port Inventory
Review `docs/architecture/PORT-INVENTORY.md`:

**Existing Ports (DO NOT RECREATE)**:
- ✅ BrowserDriver
- ✅ HttpClient
- ✅ CacheStorage
- ✅ SessionStorage
- ✅ EventBus
- ✅ MetricsCollector
- ✅ HealthChecker
- ✅ Repository<T>
- ✅ TransactionManager
- ✅ Clock
- ✅ Entropy

**Created in Sprint 2**:
- ✅ IdempotencyStore
- ✅ CircuitBreaker
- ✅ RateLimiter
- ✅ Validator
- ✅ Authorizer

**Actually Missing** (create in Sprint 4):
- ❌ SearchEngine (for full-text search)
- ❌ PdfProcessor (for PDF generation)

**Acceptance Criteria**:
- [ ] Port inventory reviewed and validated
- [ ] Team understands only 2 ports needed
- [ ] Duplicate work avoided (saves 1 week!)

### Day 2: Create SearchEngine Port

#### Task 4.2: SearchEngine Port Trait
`crates/riptide-types/src/search.rs`:

```rust
use async_trait::async_trait;

/// Port for full-text search operations
#[async_trait]
pub trait SearchEngine: Send + Sync {
    /// Index document for search
    async fn index<T>(&self, id: &str, document: &T) -> Result<(), SearchError>
    where
        T: serde::Serialize + Send + Sync;

    /// Search documents by query
    async fn search(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
    ) -> Result<SearchResults, SearchError>;

    /// Delete document from index
    async fn delete(&self, id: &str) -> Result<(), SearchError>;

    /// Bulk index documents
    async fn bulk_index<T>(&self, documents: Vec<(String, T)>) -> Result<(), SearchError>
    where
        T: serde::Serialize + Send + Sync;
}

#[derive(Debug, Clone)]
pub struct SearchFilters {
    pub tags: Option<Vec<String>>,
    pub date_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct SearchResults {
    pub hits: Vec<SearchHit>,
    pub total: usize,
    pub took_ms: u64,
}

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub id: String,
    pub score: f32,
    pub document: serde_json::Value,
}

#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("Index error: {0}")]
    Index(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
```

**Acceptance Criteria**:
- [ ] Trait supports indexing, searching, deletion
- [ ] Bulk operations supported for efficiency
- [ ] Filters allow date ranges, tags, pagination
- [ ] Compile-time validation passes

### Day 3: Create PdfProcessor Port

#### Task 4.3: PdfProcessor Port Trait
`crates/riptide-types/src/pdf.rs`:

```rust
use async_trait::async_trait;
use bytes::Bytes;

/// Port for PDF generation and manipulation
#[async_trait]
pub trait PdfProcessor: Send + Sync {
    /// Generate PDF from HTML content
    async fn html_to_pdf(&self, html: &str, options: PdfOptions) -> Result<Bytes, PdfError>;

    /// Generate PDF from URL
    async fn url_to_pdf(&self, url: &str, options: PdfOptions) -> Result<Bytes, PdfError>;

    /// Extract text from PDF
    async fn extract_text(&self, pdf: &[u8]) -> Result<String, PdfError>;

    /// Merge multiple PDFs
    async fn merge(&self, pdfs: Vec<Bytes>) -> Result<Bytes, PdfError>;

    /// Get PDF metadata
    async fn get_metadata(&self, pdf: &[u8]) -> Result<PdfMetadata, PdfError>;
}

#[derive(Debug, Clone)]
pub struct PdfOptions {
    pub page_size: PageSize,
    pub margin: Margin,
    pub landscape: bool,
    pub print_background: bool,
}

#[derive(Debug, Clone)]
pub enum PageSize {
    A4,
    Letter,
    Legal,
    Custom { width: f32, height: f32 },
}

#[derive(Debug, Clone)]
pub struct Margin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone)]
pub struct PdfMetadata {
    pub page_count: usize,
    pub author: Option<String>,
    pub title: Option<String>,
    pub created: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    #[error("Generation error: {0}")]
    Generation(String),

    #[error("Parse error: {0}")]
    Parse(String),
}
```

**Acceptance Criteria**:
- [ ] Supports HTML → PDF, URL → PDF
- [ ] Text extraction for PDF parsing
- [ ] Merge capability for combining reports
- [ ] Metadata extraction

### Day 4-5: Wire New Ports & Testing

#### Task 4.4: Update ApplicationContext
```rust
pub struct ApplicationContext {
    // ... existing 15 ports ...

    // NEW: Sprint 4 Ports
    pub search_engine: Arc<dyn SearchEngine>,
    pub pdf_processor: Arc<dyn PdfProcessor>,

    // Total: 17 ports
}
```

**Acceptance Criteria**:
- [ ] ApplicationContext compiles with 17 ports
- [ ] Builder methods added for new ports
- [ ] `validate()` checks new ports

#### Task 4.5: Create In-Memory Test Adapters
`crates/riptide-infrastructure/src/search/memory.rs`:

```rust
pub struct InMemorySearchEngine {
    documents: RwLock<HashMap<String, serde_json::Value>>,
}

#[async_trait]
impl SearchEngine for InMemorySearchEngine {
    async fn index<T>(&self, id: &str, document: &T) -> Result<(), SearchError>
    where
        T: serde::Serialize + Send + Sync,
    {
        let json = serde_json::to_value(document)
            .map_err(|e| SearchError::Serialization(e.to_string()))?;

        let mut docs = self.documents.write().unwrap();
        docs.insert(id.to_string(), json);
        Ok(())
    }

    async fn search(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
    ) -> Result<SearchResults, SearchError> {
        // Simple substring search for testing
        let docs = self.documents.read().unwrap();
        let hits: Vec<SearchHit> = docs
            .iter()
            .filter(|(_, doc)| {
                doc.to_string().to_lowercase().contains(&query.to_lowercase())
            })
            .map(|(id, doc)| SearchHit {
                id: id.clone(),
                score: 1.0,
                document: doc.clone(),
            })
            .collect();

        Ok(SearchResults {
            total: hits.len(),
            hits,
            took_ms: 1,
        })
    }

    // ... implement remaining methods ...
}
```

Similarly create `InMemoryPdfProcessor` for testing.

**Acceptance Criteria**:
- [ ] In-memory adapters functional
- [ ] Used in tests and development
- [ ] Unit tests pass

#### Task 4.6: Unit Tests for New Ports
Write comprehensive tests:
- [ ] SearchEngine: indexing, searching, filtering, pagination
- [ ] PdfProcessor: HTML→PDF, text extraction, merge

**Acceptance Criteria**:
- [ ] 90%+ coverage
- [ ] Edge cases tested
- [ ] Tests pass in CI

### Sprint 4 Quality Gates

**All 6 gates must pass** ✅

---

## Sprint 5: Production Adapters (Week 5)

**Goal**: Replace all in-memory adapters with production implementations
**Strategy**: One adapter per half-day, smoke test after each

### Adapter Implementation Plan

| Adapter | Port | Technology | Priority | Days |
|---------|------|------------|----------|------|
| RedisIdempotencyAdapter | IdempotencyStore | Redis | P0 | 0.5 |
| CircuitBreakerAdapter | CircuitBreaker | governor crate | P0 | 0.5 |
| TokenBucketRateLimiter | RateLimiter | governor crate | P0 | 0.5 |
| JsonSchemaValidatorAdapter | Validator | jsonschema crate | P0 | 0.5 |
| RbacAuthorizerAdapter | Authorizer | Postgres RBAC | P0 | 1.0 |
| MeilisearchAdapter | SearchEngine | Meilisearch | P1 | 1.0 |
| WeasyPrintAdapter | PdfProcessor | WeasyPrint | P1 | 0.5 |
| PostgresRepositoryAdapter | Repository<T> | Postgres | Existing | 0 |
| NatsBusAdapter | EventBus | NATS | Existing | 0 |
| RedisCacheAdapter | CacheStorage | Redis | Existing | 0 |
| PrometheusMetricsAdapter | MetricsCollector | Prometheus | Existing | 0 |

**Total**: 5 days (12 adapters, many already exist)

### Day 1-2: P0 Resilience Adapters

#### Task 5.1: RedisIdempotencyAdapter
`crates/riptide-infrastructure/src/idempotency/redis.rs`:

```rust
use redis::AsyncCommands;
use riptide_types::{IdempotencyStore, IdempotencyError};

pub struct RedisIdempotencyAdapter {
    client: redis::Client,
}

impl RedisIdempotencyAdapter {
    pub fn new(redis_url: &str) -> Result<Self, IdempotencyError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| IdempotencyError::Storage(e.to_string()))?;
        Ok(Self { client })
    }
}

#[async_trait]
impl IdempotencyStore for RedisIdempotencyAdapter {
    async fn is_duplicate(&self, operation_id: &str) -> Result<bool, IdempotencyError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| IdempotencyError::Storage(e.to_string()))?;

        let exists: bool = conn.exists(format!("idempotency:{}", operation_id)).await
            .map_err(|e| IdempotencyError::Storage(e.to_string()))?;

        Ok(exists)
    }

    async fn record(&self, operation_id: &str, ttl: Duration) -> Result<(), IdempotencyError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| IdempotencyError::Storage(e.to_string()))?;

        conn.set_ex(
            format!("idempotency:{}", operation_id),
            "1",
            ttl.as_secs() as usize,
        ).await
        .map_err(|e| IdempotencyError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn get_result(&self, operation_id: &str) -> Result<Option<Vec<u8>>, IdempotencyError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| IdempotencyError::Storage(e.to_string()))?;

        let result: Option<Vec<u8>> = conn.get(format!("idempotency:result:{}", operation_id)).await
            .map_err(|e| IdempotencyError::Storage(e.to_string()))?;

        Ok(result)
    }

    async fn store_result(
        &self,
        operation_id: &str,
        result: Vec<u8>,
        ttl: Duration,
    ) -> Result<(), IdempotencyError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| IdempotencyError::Storage(e.to_string()))?;

        conn.set_ex(
            format!("idempotency:result:{}", operation_id),
            result,
            ttl.as_secs() as usize,
        ).await
        .map_err(|e| IdempotencyError::Storage(e.to_string()))?;

        Ok(())
    }
}
```

**Acceptance Criteria**:
- [ ] Connects to Redis successfully
- [ ] TTL expiry works correctly
- [ ] Handles connection failures gracefully
- [ ] Integration tests pass with real Redis

#### Task 5.2: CircuitBreakerAdapter
Using `governor` crate for production circuit breaker:

```rust
use governor::{Quota, RateLimiter as GovernorLimiter, state::InMemoryState};
use riptide_types::{CircuitBreaker, CircuitBreakerError, CircuitState};

pub struct CircuitBreakerAdapter {
    failure_threshold: u32,
    timeout: Duration,
    state: Arc<RwLock<CircuitState>>,
    failures: Arc<AtomicU32>,
}

#[async_trait]
impl CircuitBreaker for CircuitBreakerAdapter {
    async fn call<F, T>(&self, operation: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<T, Box<dyn std::error::Error>>> + Send,
        T: Send,
    {
        // Check circuit state
        let state = *self.state.read().unwrap();
        match state {
            CircuitState::Open => {
                return Err(CircuitBreakerError::CircuitOpen);
            }
            CircuitState::HalfOpen | CircuitState::Closed => {}
        }

        // Execute operation
        match operation().await {
            Ok(result) => {
                // Success: reset failures
                self.failures.store(0, Ordering::SeqCst);
                *self.state.write().unwrap() = CircuitState::Closed;
                Ok(result)
            }
            Err(e) => {
                // Failure: increment counter
                let failures = self.failures.fetch_add(1, Ordering::SeqCst) + 1;

                if failures >= self.failure_threshold {
                    *self.state.write().unwrap() = CircuitState::Open;

                    // Schedule half-open transition
                    let state_clone = self.state.clone();
                    let timeout = self.timeout;
                    tokio::spawn(async move {
                        tokio::time::sleep(timeout).await;
                        *state_clone.write().unwrap() = CircuitState::HalfOpen;
                    });
                }

                Err(CircuitBreakerError::OperationFailed(e.to_string()))
            }
        }
    }

    fn state(&self) -> CircuitState {
        *self.state.read().unwrap()
    }

    async fn reset(&self) {
        self.failures.store(0, Ordering::SeqCst);
        *self.state.write().unwrap() = CircuitState::Closed;
    }
}
```

**Acceptance Criteria**:
- [ ] Opens circuit after threshold failures
- [ ] Half-open state allows testing recovery
- [ ] Automatic reset after timeout
- [ ] Thread-safe state management

#### Task 5.3: TokenBucketRateLimiter
```rust
use governor::{Quota, RateLimiter as GovernorLimiter};
use riptide_types::{RateLimiter, RateLimitError};

pub struct TokenBucketRateLimiter {
    limiters: Arc<RwLock<HashMap<String, GovernorLimiter<String, InMemoryState>>>>,
    default_quota: Quota,
}

#[async_trait]
impl RateLimiter for TokenBucketRateLimiter {
    async fn check_rate_limit(&self, key: &str) -> Result<bool, RateLimitError> {
        let limiters = self.limiters.read().unwrap();
        if let Some(limiter) = limiters.get(key) {
            Ok(limiter.check().is_ok())
        } else {
            // No limit for this key yet, allow
            Ok(true)
        }
    }

    async fn wait_for_slot(&self, key: &str) -> Result<(), RateLimitError> {
        let mut limiters = self.limiters.write().unwrap();
        let limiter = limiters
            .entry(key.to_string())
            .or_insert_with(|| GovernorLimiter::keyed(self.default_quota));

        limiter.until_ready().await;
        Ok(())
    }

    async fn remaining(&self, key: &str) -> Result<u64, RateLimitError> {
        let limiters = self.limiters.read().unwrap();
        if let Some(limiter) = limiters.get(key) {
            // Governor doesn't expose remaining capacity directly
            // Return estimate based on check result
            Ok(if limiter.check().is_ok() { 1 } else { 0 })
        } else {
            Ok(u64::MAX) // No limit set
        }
    }

    async fn reset(&self, key: &str) -> Result<(), RateLimitError> {
        let mut limiters = self.limiters.write().unwrap();
        limiters.remove(key);
        Ok(())
    }
}
```

**Acceptance Criteria**:
- [ ] Token bucket algorithm enforces limits
- [ ] Per-key rate limiting (isolation)
- [ ] `wait_for_slot` blocks until capacity available
- [ ] Production-ready performance

### Day 3-4: Security & Search Adapters

#### Task 5.4: JsonSchemaValidatorAdapter
```rust
use jsonschema::{Draft, JSONSchema};
use riptide_types::{Validator, ValidationError};

pub struct JsonSchemaValidatorAdapter {
    schemas: Arc<RwLock<HashMap<String, JSONSchema>>>,
}

#[async_trait]
impl Validator for JsonSchemaValidatorAdapter {
    async fn validate<T>(&self, input: &T) -> Result<(), ValidationError>
    where
        T: serde::Serialize + Send + Sync,
    {
        let json = serde_json::to_value(input)
            .map_err(|e| ValidationError::Invalid(e.to_string()))?;

        // Get schema for type (based on type name)
        let type_name = std::any::type_name::<T>();
        let schemas = self.schemas.read().unwrap();

        if let Some(schema) = schemas.get(type_name) {
            let result = schema.validate(&json);

            if let Err(errors) = result {
                let error_msgs: Vec<String> = errors
                    .map(|e| e.to_string())
                    .collect();

                return Err(ValidationError::Invalid(error_msgs.join(", ")));
            }
        }

        Ok(())
    }

    // ... implement validate_and_coerce ...
}
```

**Acceptance Criteria**:
- [ ] JSON Schema Draft 7 support
- [ ] Clear validation error messages
- [ ] Schema registry for different types
- [ ] Coercion support (type conversion)

#### Task 5.5: RbacAuthorizerAdapter (Postgres-backed)
```rust
use sqlx::PgPool;
use riptide_types::{Authorizer, AuthorizationError, Permission};

pub struct RbacAuthorizerAdapter {
    pool: PgPool,
}

#[async_trait]
impl Authorizer for RbacAuthorizerAdapter {
    async fn authorize(
        &self,
        user_id: &str,
        resource: &str,
        action: &str,
    ) -> Result<bool, AuthorizationError> {
        let result: Option<bool> = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM user_permissions
                WHERE user_id = $1 AND resource = $2 AND action = $3
            )"
        )
        .bind(user_id)
        .bind(resource)
        .bind(action)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthorizationError::PermissionDenied)?;

        Ok(result.unwrap_or(false))
    }

    async fn get_permissions(&self, user_id: &str) -> Result<Vec<Permission>, AuthorizationError> {
        let permissions = sqlx::query_as::<_, (String, String)>(
            "SELECT resource, action FROM user_permissions WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| AuthorizationError::UserNotFound(user_id.to_string()))?;

        Ok(permissions
            .into_iter()
            .map(|(resource, action)| Permission { resource, action })
            .collect())
    }

    async fn grant_permission(
        &self,
        user_id: &str,
        permission: Permission,
    ) -> Result<(), AuthorizationError> {
        sqlx::query(
            "INSERT INTO user_permissions (user_id, resource, action)
             VALUES ($1, $2, $3)
             ON CONFLICT DO NOTHING"
        )
        .bind(user_id)
        .bind(&permission.resource)
        .bind(&permission.action)
        .execute(&self.pool)
        .await
        .map_err(|_| AuthorizationError::PermissionDenied)?;

        Ok(())
    }
}
```

**Acceptance Criteria**:
- [ ] Postgres schema created
- [ ] RBAC permission checks work
- [ ] Grant/revoke functionality
- [ ] Migration script for user_permissions table

### Day 5: Search & PDF Adapters

#### Task 5.6: MeilisearchAdapter
```rust
use meilisearch_sdk::Client as MeilisearchClient;
use riptide_types::{SearchEngine, SearchError, SearchResults};

pub struct MeilisearchAdapter {
    client: MeilisearchClient,
}

#[async_trait]
impl SearchEngine for MeilisearchAdapter {
    async fn index<T>(&self, id: &str, document: &T) -> Result<(), SearchError>
    where
        T: serde::Serialize + Send + Sync,
    {
        let index = self.client.index("documents");
        index.add_documents(&[document], Some("id"))
            .await
            .map_err(|e| SearchError::Index(e.to_string()))?;

        Ok(())
    }

    async fn search(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
    ) -> Result<SearchResults, SearchError> {
        let index = self.client.index("documents");
        let mut search = index.search();
        search.with_query(query);

        if let Some(f) = filters {
            if let Some(tags) = f.tags {
                search.with_filter(&format!("tags IN {}", tags.join(",")));
            }
            search.with_limit(f.limit);
            search.with_offset(f.offset);
        }

        let results = search.execute::<serde_json::Value>()
            .await
            .map_err(|e| SearchError::Query(e.to_string()))?;

        Ok(SearchResults {
            hits: results.hits.into_iter().map(|hit| SearchHit {
                id: hit.result.get("id").unwrap().to_string(),
                score: 1.0,
                document: hit.result,
            }).collect(),
            total: results.estimated_total_hits.unwrap_or(0),
            took_ms: results.processing_time_ms as u64,
        })
    }

    // ... implement remaining methods ...
}
```

**Acceptance Criteria**:
- [ ] Meilisearch integration works
- [ ] Indexing and search functional
- [ ] Filters applied correctly
- [ ] Performance acceptable (<100ms searches)

#### Task 5.7: WeasyPrintAdapter (PDF Generation)
```rust
use tokio::process::Command;
use riptide_types::{PdfProcessor, PdfError, PdfOptions};

pub struct WeasyPrintAdapter {
    binary_path: String,
}

#[async_trait]
impl PdfProcessor for WeasyPrintAdapter {
    async fn html_to_pdf(&self, html: &str, options: PdfOptions) -> Result<Bytes, PdfError> {
        // Write HTML to temp file
        let temp_html = tempfile::NamedTempFile::new()
            .map_err(|e| PdfError::Generation(e.to_string()))?;

        tokio::fs::write(temp_html.path(), html).await
            .map_err(|e| PdfError::Generation(e.to_string()))?;

        // Generate PDF
        let output = Command::new(&self.binary_path)
            .arg(temp_html.path())
            .arg("-")
            .arg("--format").arg("pdf")
            .output()
            .await
            .map_err(|e| PdfError::Generation(e.to_string()))?;

        if !output.status.success() {
            return Err(PdfError::Generation(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(Bytes::from(output.stdout))
    }

    // ... implement url_to_pdf, extract_text, merge ...
}
```

**Acceptance Criteria**:
- [ ] WeasyPrint binary available
- [ ] HTML → PDF generation works
- [ ] Options (page size, margins) respected
- [ ] Error handling for malformed HTML

### Sprint 5 Quality Gates

**All 6 gates must pass** ✅

**Additional Sprint 5 Validation**:
- [ ] All 12 adapters pass integration tests
- [ ] Production configs for Redis, Postgres, Meilisearch documented
- [ ] Rollback to in-memory adapters tested
- [ ] Performance benchmarks show <10% overhead

---

## Sprint 6: Facade Migration (Week 6)

**Goal**: Migrate 12 facades from AppState to ApplicationContext
**Strategy**: One facade per day, smoke test after each

### Facade Migration Priority

| Priority | Facade | Reason | Complexity |
|----------|--------|--------|------------|
| P0 | CrawlFacade | Most used, high impact | Medium |
| P0 | ExtractorFacade | Core functionality | Medium |
| P0 | StreamingFacade | Large (1464 LOC) | High |
| P1 | SessionFacade | Authentication critical | Low |
| P1 | MetricsFacade | Observability | Low |
| P1 | CacheFacade | Performance impact | Medium |
| P1 | HealthFacade | Monitoring | Low |
| P1 | ConfigFacade | Runtime config | Low |
| P2 | SearchFacade | New port (Sprint 5) | Medium |
| P2 | PdfFacade | New port (Sprint 5) | Low |
| P2 | EventFacade | Event bus integration | Medium |
| P2 | WorkflowFacade | Orchestration | High |

### Day 1-5: Migrate One Facade Per Half-Day

#### Facade Migration Template

For each facade, follow this pattern:

**BEFORE**:
```rust
pub struct [Facade] {
    state: Arc<AppState>,
}

impl [Facade] {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn [method](&self) -> Result<T> {
        // Direct AppState field access
        self.state.[field].operation()
    }
}
```

**AFTER**:
```rust
pub struct [Facade] {
    context: Arc<ApplicationContext>,
}

impl [Facade] {
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self { context }
    }

    pub async fn [method](&self) -> Result<T> {
        // Port trait usage
        self.context.[port].operation()
    }
}
```

#### Task 6.1: StreamingFacade Migration (Day 1)
```rust
// BEFORE (1464 LOC, many dependencies)
pub struct StreamingFacade {
    state: Arc<AppState>,
}

impl StreamingFacade {
    pub async fn stream_crawl(&self, urls: Vec<String>) -> BoxStream<CrawlResult> {
        let browser = self.state.browser_pool.checkout().await;
        let http_client = self.state.http_client.clone();
        let cache = self.state.cache.clone();

        // ... complex streaming logic ...
    }
}

// AFTER
pub struct StreamingFacade {
    context: Arc<ApplicationContext>,
}

impl StreamingFacade {
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self { context }
    }

    pub async fn stream_crawl(&self, urls: Vec<String>) -> BoxStream<CrawlResult> {
        let browser_driver = self.context.browser_driver.clone();
        let http_client = self.context.http_client.clone();
        let cache = self.context.cache_storage.clone();
        let circuit_breaker = self.context.circuit_breaker.clone();
        let rate_limiter = self.context.rate_limiter.clone();

        // Enhanced with resilience patterns
        stream::iter(urls)
            .map(move |url| {
                let browser = browser_driver.clone();
                let http = http_client.clone();
                let cache = cache.clone();
                let cb = circuit_breaker.clone();
                let rl = rate_limiter.clone();

                async move {
                    // Rate limit
                    rl.wait_for_slot("crawl").await?;

                    // Circuit breaker protection
                    cb.call(|| async {
                        // Check cache first
                        if let Some(cached) = cache.get(&url).await? {
                            return Ok(cached);
                        }

                        // Fetch with browser
                        let browser = browser.acquire().await?;
                        let result = browser.navigate(&url).await?;

                        // Cache result
                        cache.set(&url, &result, Duration::from_secs(3600)).await?;

                        Ok(result)
                    }).await
                }
            })
            .buffered(10) // Parallel requests
            .boxed()
    }
}
```

**Acceptance Criteria**:
- [ ] StreamingFacade compiles
- [ ] All methods use `self.context` ports
- [ ] Circuit breaker and rate limiter integrated
- [ ] Smoke tests pass (`/api/v1/stream/crawl`)

#### Per-Facade Checklist

For each of the 12 facades, complete:

1. **Code Migration**
   - [ ] Replace `state: Arc<AppState>` with `context: Arc<ApplicationContext>`
   - [ ] Update constructor signature
   - [ ] Replace all `self.state.field` with `self.context.port`
   - [ ] Add resilience patterns (circuit breaker, rate limiter)
   - [ ] Compile without errors

2. **Testing**
   - [ ] Unit tests updated to use `create_test_context()`
   - [ ] All unit tests pass
   - [ ] Integration test for facade added
   - [ ] Smoke test for top route passes

3. **Documentation**
   - [ ] Dependency matrix updated
   - [ ] Migration notes in ADR (if needed)
   - [ ] Code comments for complex port usage

4. **Quality Gates**
   - [ ] Builds in both modes (feature flags)
   - [ ] Top route returns 200
   - [ ] Tests pass
   - [ ] Rollback tested

### Day 5: Verify All Facades Migrated

#### Task 6.2: Validate Facade-AppState Separation
```bash
# Check that no facades import AppState
grep -r "use.*AppState" crates/riptide-facade/src/*.rs

# Expected: 0 results (facades no longer depend on AppState)
```

**Acceptance Criteria**:
- [ ] All 12 facades use ApplicationContext
- [ ] No AppState imports in facade code
- [ ] Dependency matrix shows 100% port usage

#### Task 6.3: Update FacadeRegistry
```rust
impl FacadeRegistry {
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        // All 35 facades initialized with context
        let browser = Arc::new(BrowserFacade::new(context.clone()));
        let crawl = Arc::new(CrawlFacade::new(context.clone()));
        let extractor = Arc::new(ExtractorFacade::new(context.clone()));
        let streaming = Arc::new(StreamingFacade::new(context.clone()));
        let session = Arc::new(SessionFacade::new(context.clone()));
        let metrics = Arc::new(MetricsFacade::new(context.clone()));
        let cache = Arc::new(CacheFacade::new(context.clone()));
        let health = Arc::new(HealthFacade::new(context.clone()));
        let config = Arc::new(ConfigFacade::new(context.clone()));
        let search = Arc::new(SearchFacade::new(context.clone()));
        let pdf = Arc::new(PdfFacade::new(context.clone()));
        let event = Arc::new(EventFacade::new(context.clone()));
        let workflow = Arc::new(WorkflowFacade::new(context.clone()));
        // ... 22 more facades

        Self {
            context,
            browser,
            crawl,
            extractor,
            streaming,
            session,
            metrics,
            cache,
            health,
            config,
            search,
            pdf,
            event,
            workflow,
            // ...
        }
    }
}
```

**Acceptance Criteria**:
- [ ] All 35+ facades in FacadeRegistry
- [ ] Single ApplicationContext shared
- [ ] No direct infrastructure dependencies

### Sprint 6 Quality Gates

**All 6 gates must pass** ✅

**Additional Sprint 6 Validation**:
- [ ] Infrastructure violations: 32 → 12 (63% reduction)
- [ ] All 12 target facades use ApplicationContext
- [ ] No AppState imports in facades
- [ ] Hexagonal compliance: 35% → 55%

---

## Phase 2 Success Metrics

| Metric | Phase 1 End | Sprint 6 Target | Actual |
|--------|-------------|-----------------|--------|
| Missing Ports | 2 | 0 | ___ |
| Production Adapters | 0 | 12 | ___ |
| Facades Using Context | 3 | 15 | ___ |
| Infrastructure Violations | 32 | 12 | ___ |
| Hexagonal Compliance | 35% | 55% | ___ |

---

## Phase 2 Deliverables

- [ ] 2 new P1 ports created (SearchEngine, PdfProcessor)
- [ ] Port audit prevented 5 duplicate ports (saved 1 week!)
- [ ] 12 production adapters implemented and tested
- [ ] 12 facades migrated to ApplicationContext
- [ ] FacadeRegistry updated for all facades
- [ ] Infrastructure violations reduced by 63%
- [ ] All quality gates passed for Sprints 4-6

---

## Next: Phase 3 (Weeks 7-9)

**Goal**: Complete testing of all untested facades and resilience patterns
**See**: [ROADMAP-PHASE-3.md](ROADMAP-PHASE-3.md)

---

**Status**: Ready for Sprint 4 kickoff
**Owner**: Backend Engineering Team
**Duration**: 3 weeks
