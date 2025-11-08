# Phase 2: Application Layer Enhancements Roadmap
**Version:** 2.0 (Enhanced)
**Date:** 2025-11-08
**Duration:** 3 weeks (updated from 2 weeks)
**Goal:** Add authorization, idempotency, events, transactions, and middleware refactoring

---

## Overview

Phase 2 builds upon the Ports & Adapters foundation by implementing critical application-layer concerns: authorization policies, idempotency management, domain events, transactional workflows, backpressure handling, and middleware refactoring. These cross-cutting concerns belong in the application layer (riptide-facade), not in handlers or infrastructure.

### Objectives

1. **Authorization Policies**: Tenant scoping, RBAC, resource-level permissions
2. **Idempotency Management**: Request deduplication at entry points
3. **Transactional Workflows**: ACID guarantees with outbox pattern
4. **Domain Events**: Event emission and transactional outbox publishing
5. **Backpressure & Cancellation**: Resource management and graceful degradation
6. **Business Metrics**: Domain-level observability (not just transport)
7. **Middleware Refactoring**: Move business validation to facades (NEW)

---

## 🚨 Quality Gates (MANDATORY - Every Task)

**Zero-tolerance policy for errors/warnings. Every commit must:**

```bash
# 1. Tests pass (NO #[ignore], NO skipped tests)
cargo test -p [affected-crate]  # NOT --workspace (conserve disk)

# 2. Clippy clean (ZERO warnings)
cargo clippy -p [affected-crate] -- -D warnings

# 3. Cargo check passes
cargo check -p [affected-crate]

# 4. Full workspace ONLY for final phase validation
# Use targeted builds: cargo build -p [crate] to save disk space
```

**Commit Rules:**
- ❌ NO commits with failing tests
- ❌ NO commits with clippy warnings
- ❌ NO commits with compilation errors
- ❌ NO #[ignore] on tests without tracking issue
- ✅ Each phase MUST be fully complete before moving to next

---

### Success Criteria

- ✅ Authorization policies enforced in all facades
- ✅ Idempotency keys at all entry points
- ✅ Transactional workflows with outbox pattern
- ✅ Domain events emitted from entities, published via EventBus
- ✅ Backpressure + cancellation tokens
- ✅ Business metrics (not just transport metrics)
- ✅ Middleware layer: only I/O validation (<300 LOC total)
- ✅ **All tests pass (ZERO ignored)**
- ✅ **Clippy clean (ZERO warnings)**
- ✅ **Cargo check passes**

---

## Sprint 2.1: Authorization Policies (Week 1, Days 1-2)

### Task 2.1.1: Define Authorization Framework

**File:** `crates/riptide-facade/src/authorization/mod.rs` (NEW)

```rust
use riptide_types::ports::Repository;

pub struct AuthorizationContext {
    pub user_id: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub permissions: HashSet<String>,
}

pub trait AuthorizationPolicy: Send + Sync {
    fn authorize(&self, ctx: &AuthorizationContext, resource: &Resource) -> Result<()>;
}

pub struct TenantScopingPolicy;

impl AuthorizationPolicy for TenantScopingPolicy {
    fn authorize(&self, ctx: &AuthorizationContext, resource: &Resource) -> Result<()> {
        if resource.tenant_id != ctx.tenant_id {
            Err(RiptideError::Unauthorized {
                reason: "Cross-tenant access denied".to_string()
            })
        } else {
            Ok(())
        }
    }
}

pub struct RbacPolicy {
    required_permission: String,
}

impl AuthorizationPolicy for RbacPolicy {
    fn authorize(&self, ctx: &AuthorizationContext, resource: &Resource) -> Result<()> {
        if ctx.permissions.contains(&self.required_permission) {
            Ok(())
        } else {
            Err(RiptideError::Forbidden {
                permission: self.required_permission.clone()
            })
        }
    }
}
```

---

### Task 2.1.2: Integrate Authorization into Facades

**File:** `crates/riptide-facade/src/facades/extraction.rs` (ENHANCED)

```rust
pub struct ExtractionFacade {
    extractor: Arc<dyn BrowserDriver>,
    cache: Arc<dyn CacheStorage>,
    event_bus: Arc<dyn EventBus>,
    authz_policies: Vec<Box<dyn AuthorizationPolicy>>,  // NEW
}

impl ExtractionFacade {
    pub async fn extract_content(
        &self,
        url: &str,
        authz_ctx: &AuthorizationContext,  // NEW
    ) -> Result<ExtractedData> {
        // 1. Authorization checks (Application layer)
        for policy in &self.authz_policies {
            policy.authorize(authz_ctx, &Resource::Url(url.to_string()))?;
        }

        // 2. Business logic
        let data = self.extractor.extract(url).await?;

        // 3. Emit domain event
        self.event_bus.publish(DomainEvent {
            event_type: "content.extracted".to_string(),
            aggregate_id: url.to_string(),
            payload: serde_json::to_value(&data)?,
            // ...
        }).await?;

        Ok(data)
    }
}
```

**Files Modified:**
```
CREATE:  crates/riptide-facade/src/authorization/mod.rs (~200 LOC)
CREATE:  crates/riptide-facade/src/authorization/policies.rs (~150 LOC)
UPDATE:  crates/riptide-facade/src/facades/extraction.rs (add authz)
UPDATE:  crates/riptide-facade/src/facades/browser.rs (add authz)
UPDATE:  crates/riptide-facade/src/facades/pipeline.rs (add authz)
UPDATE:  crates/riptide-facade/src/facades/spider.rs (add authz)
UPDATE:  crates/riptide-facade/src/facades/search.rs (add authz)
UPDATE:  crates/riptide-facade/src/facades/pdf.rs (add authz)
```

---

## Sprint 2.2: Idempotency & Transactions (Week 1, Days 3-4)

### Task 2.2.1: Implement Transactional Workflow

**File:** `crates/riptide-facade/src/workflows/transactional.rs` (NEW)

```rust
use riptide_types::ports::{TransactionManager, EventBus, IdempotencyStore};

pub struct TransactionalWorkflow<T> {
    tx_manager: Arc<dyn TransactionManager>,
    event_bus: Arc<dyn EventBus>,
    idempotency_store: Arc<dyn IdempotencyStore>,
    _phantom: PhantomData<T>,
}

impl<T> TransactionalWorkflow<T> {
    pub async fn execute<F, R>(
        &self,
        idempotency_key: &str,
        workflow_fn: F,
    ) -> Result<R>
    where
        F: FnOnce(&mut dyn Transaction) -> BoxFuture<'_, Result<(R, Vec<DomainEvent>)>>,
    {
        // 1. Check idempotency
        let idem_token = self.idempotency_store
            .try_acquire(idempotency_key, Duration::from_secs(300))
            .await?;

        // 2. Begin transaction
        let mut tx = self.tx_manager.begin().await?;

        // 3. Execute workflow (returns result + events)
        let (result, events) = match workflow_fn(&mut tx).await {
            Ok(r) => r,
            Err(e) => {
                self.tx_manager.rollback(tx).await?;
                self.idempotency_store.release(idem_token).await?;
                return Err(e);
            }
        };

        // 4. Write events to outbox (transactional)
        for event in events {
            self.event_bus.publish(event).await?;
        }

        // 5. Commit transaction
        self.tx_manager.commit(tx).await?;

        // 6. Release idempotency lock
        self.idempotency_store.release(idem_token).await?;

        Ok(result)
    }
}
```

---

### Task 2.2.2: Apply to Profile Facade

**File:** `crates/riptide-facade/src/facades/profile.rs` (ENHANCED)

```rust
impl ProfileFacade {
    pub async fn create_profile(
        &self,
        request: CreateProfileRequest,
        authz_ctx: &AuthorizationContext,
    ) -> Result<Profile> {
        let idempotency_key = format!("profile:create:{}", request.user_id);

        self.workflow.execute(&idempotency_key, |tx| async move {
            // Business logic within transaction
            let profile = Profile {
                id: self.context.entropy.random_id(),
                user_id: request.user_id.clone(),
                tenant_id: authz_ctx.tenant_id.clone(),
                created_at: self.context.clock.now(),
                // ...
            };

            // Save to repository (within TX)
            self.profile_repository.save(&profile).await?;

            // Emit domain event (written to outbox in same TX)
            let events = vec![DomainEvent {
                event_type: "profile.created".to_string(),
                aggregate_id: profile.id.clone(),
                payload: serde_json::to_value(&profile)?,
                // ...
            }];

            Ok((profile, events))
        }.boxed()).await
    }
}
```

**Files Modified:**
```
CREATE:  crates/riptide-facade/src/workflows/transactional.rs (~250 LOC)
UPDATE:  crates/riptide-facade/src/facades/profile.rs (use TransactionalWorkflow)
UPDATE:  crates/riptide-facade/src/facades/session.rs (use TransactionalWorkflow)
```

---

## Sprint 2.3: Backpressure & Cancellation (Week 2, Days 1-2)

### Task 2.3.1: Implement Backpressure Manager

**File:** `crates/riptide-facade/src/workflows/backpressure.rs` (NEW)

```rust
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

pub struct BackpressureManager {
    semaphore: Arc<Semaphore>,
    active_count: Arc<AtomicUsize>,
    max_concurrency: usize,
}

impl BackpressureManager {
    pub async fn acquire(&self, cancel_token: &CancellationToken) -> Result<BackpressureGuard> {
        tokio::select! {
            permit = self.semaphore.acquire() => {
                let permit = permit.map_err(|_| RiptideError::ResourceExhausted)?;
                self.active_count.fetch_add(1, Ordering::SeqCst);

                Ok(BackpressureGuard {
                    _permit: permit.into(),
                    active_count: self.active_count.clone(),
                })
            }
            _ = cancel_token.cancelled() => {
                Err(RiptideError::Cancelled)
            }
        }
    }

    pub fn current_load(&self) -> f64 {
        self.active_count.load(Ordering::SeqCst) as f64 / self.max_concurrency as f64
    }
}

pub struct BackpressureGuard {
    _permit: OwnedSemaphorePermit,
    active_count: Arc<AtomicUsize>,
}

impl Drop for BackpressureGuard {
    fn drop(&mut self) {
        self.active_count.fetch_sub(1, Ordering::SeqCst);
    }
}
```

**Files Modified:**
```
CREATE:  crates/riptide-facade/src/workflows/backpressure.rs (~200 LOC)
UPDATE:  crates/riptide-facade/src/facades/extraction.rs (use BackpressureManager)
UPDATE:  crates/riptide-facade/src/facades/browser.rs (use BackpressureManager)
```

---

## Sprint 2.4: Business Metrics (Week 2, Days 3-4)

### Task 2.4.1: Define Business Metrics

**File:** `crates/riptide-facade/src/metrics/business.rs` (NEW)

```rust
use prometheus::{Counter, Histogram, IntGauge};

pub struct BusinessMetrics {
    // Domain-level metrics (not transport)
    profiles_created: Counter,
    extractions_completed: Counter,
    extractions_duration: Histogram,
    active_sessions: IntGauge,

    // Business SLOs
    extraction_success_rate: Counter,
    extraction_failure_rate: Counter,
}

impl BusinessMetrics {
    pub fn record_extraction_completed(&self, duration: Duration, success: bool) {
        self.extractions_completed.inc();
        self.extractions_duration.observe(duration.as_secs_f64());

        if success {
            self.extraction_success_rate.inc();
        } else {
            self.extraction_failure_rate.inc();
        }
    }
}
```

**Files Modified:**
```
CREATE:  crates/riptide-facade/src/metrics/business.rs (~200 LOC)
UPDATE:  All facades to record business metrics
```

---

## Sprint 2.5: Middleware Refactoring (NEW - Week 3, Days 1-3)

**Priority:** CRITICAL
**Source:** API_CRATE_COVERAGE_ANALYSIS.md - Gap #1

### Problem

**Middleware system has business logic in API layer (1,879 LOC):**
- `middleware/auth.rs` (846 LOC) - Authentication logic
- `middleware/request_validation.rs` (669 LOC) - Business validation mixed with I/O validation
- `middleware/rate_limit.rs` (178 LOC) - Should use port pattern
- `middleware/payload_limit.rs` (176 LOC) - Pure I/O (OK to keep)

**Current Roadmap:** No mention
**Gap:** Business logic in wrong layer, tight coupling

### Tasks

**Task 2.5.1: Split Request Validation (1 day)**

```rust
// BEFORE: middleware/request_validation.rs (669 LOC - mixed concerns)
pub async fn validate_request(req: Request) -> Result<Request> {
    // I/O validation (format, bounds)
    if req.body_size > MAX_SIZE { return Err(...); }

    // BUSINESS VALIDATION (belongs in facade)
    if !user.has_permission("extract") { return Err(...); }
    if tenant.quota_exceeded() { return Err(...); }

    Ok(req)
}

// AFTER: middleware/request_validation.rs (~200 LOC - I/O only)
pub async fn validate_request_format(req: Request) -> Result<Request> {
    // Only I/O validation
    if req.body_size > MAX_SIZE {
        return Err(ApiError::payload_too_large(req.body_size, MAX_SIZE));
    }

    if req.headers.content_type.is_none() {
        return Err(ApiError::missing_header("Content-Type"));
    }

    Ok(req)
}

// Business validation moved to facades
// crates/riptide-facade/src/facades/extraction.rs
impl ExtractionFacade {
    pub async fn extract_content(...) -> Result<...> {
        // Authorization checks (business logic)
        self.authorize(authz_ctx, resource)?;

        // Quota checks (business logic)
        self.check_quota(authz_ctx.tenant_id).await?;

        // Extraction logic
        ...
    }
}
```

**Task 2.5.2: Port Authentication Middleware (1.5 days)**

```rust
// Define port in riptide-types
// crates/riptide-types/src/ports/authentication.rs (NEW)
#[async_trait]
pub trait AuthenticationProvider: Send + Sync {
    async fn authenticate(&self, token: &str) -> Result<AuthenticationContext>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair>;
}

pub struct AuthenticationContext {
    pub user_id: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub permissions: HashSet<String>,
}

// Implement adapter in infrastructure
// crates/riptide-api/src/adapters/jwt_authentication.rs (NEW)
pub struct JwtAuthenticationProvider {
    jwt_secret: Arc<String>,
    jwt_decoder: JwtDecoder,
}

#[async_trait]
impl AuthenticationProvider for JwtAuthenticationProvider {
    async fn authenticate(&self, token: &str) -> Result<AuthenticationContext> {
        // JWT validation logic
        let claims = self.jwt_decoder.decode(token, &self.jwt_secret)?;

        Ok(AuthenticationContext {
            user_id: claims.sub,
            tenant_id: claims.tenant_id,
            roles: claims.roles,
            permissions: claims.permissions.into_iter().collect(),
        })
    }
}

// Middleware uses port (thin layer)
// crates/riptide-api/src/middleware/auth.rs (REFACTORED - ~100 LOC)
pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response> {
    // Extract token
    let token = extract_bearer_token(&req)?;

    // Use authentication port
    let auth_ctx = state.context.authentication_provider
        .authenticate(&token)
        .await?;

    // Inject into request extensions
    req.extensions_mut().insert(auth_ctx);

    Ok(next.run(req).await)
}
```

**Task 2.5.3: Port Rate Limiting (0.5 days)**

```rust
// Define port in riptide-types
// crates/riptide-types/src/ports/rate_limit.rs (NEW)
#[async_trait]
pub trait RateLimiter: Send + Sync {
    async fn check_rate_limit(&self, key: &str, limit: RateLimit) -> Result<RateLimitStatus>;
}

pub struct RateLimit {
    pub requests: usize,
    pub window: Duration,
}

pub enum RateLimitStatus {
    Allowed { remaining: usize },
    Exceeded { retry_after: Duration },
}

// Implement Redis adapter
// crates/riptide-cache/src/adapters/redis_rate_limiter.rs (NEW)
impl RateLimiter for RedisRateLimiter {
    async fn check_rate_limit(&self, key: &str, limit: RateLimit) -> Result<RateLimitStatus> {
        // Redis sliding window algorithm
    }
}

// Middleware uses port
// crates/riptide-api/src/middleware/rate_limit.rs (REFACTORED - ~50 LOC)
pub async fn rate_limit_middleware(...) -> Result<Response> {
    let rate_limit_key = format!("ratelimit:{}:{}", tenant_id, endpoint);

    let status = state.context.rate_limiter
        .check_rate_limit(&rate_limit_key, RateLimit {
            requests: 100,
            window: Duration::from_secs(60),
        })
        .await?;

    match status {
        RateLimitStatus::Allowed { remaining } => {
            // Add headers, continue
        }
        RateLimitStatus::Exceeded { retry_after } => {
            Err(ApiError::rate_limit_exceeded(retry_after))
        }
    }
}
```

**Files Modified:**
```
CREATE:  crates/riptide-types/src/ports/authentication.rs (~100 LOC)
CREATE:  crates/riptide-types/src/ports/rate_limit.rs (~80 LOC)
CREATE:  crates/riptide-api/src/adapters/jwt_authentication.rs (~300 LOC)
CREATE:  crates/riptide-cache/src/adapters/redis_rate_limiter.rs (~150 LOC)
UPDATE:  crates/riptide-api/src/middleware/auth.rs (846 → 100 LOC)
UPDATE:  crates/riptide-api/src/middleware/request_validation.rs (669 → 200 LOC)
UPDATE:  crates/riptide-api/src/middleware/rate_limit.rs (178 → 50 LOC)
KEEP:    crates/riptide-api/src/middleware/payload_limit.rs (176 LOC - pure I/O)
UPDATE:  crates/riptide-facade/src/facades/* (move business validation from middleware)
```

**Validation:**
```bash
# Middleware layer: only I/O validation
find crates/riptide-api/src/middleware -name "*.rs" -exec wc -l {} + | awk '{sum+=$1} END {print sum}'
# Expected: <500 LOC total (was 1,879 LOC)

# No business logic in middleware
rg "has_permission|quota|tenant" crates/riptide-api/src/middleware/ && echo "FAIL" || echo "PASS"

# Tests pass
cargo test -p riptide-api -- middleware
```

**Expected Result:**
- Middleware layer: only I/O validation (<350 LOC total, was 1,879 LOC)
- Business validation: moved to facades (~1,200 LOC)
- Authentication: port-based (testable, swappable)
- Rate limiting: port-based

**LOC Impact:** +630 (ports + adapters), -1,200 (move to facades) = net -570 LOC from middleware

---

## Phase 2 Summary

### Duration Breakdown

| Sprint | Duration | LOC Added | LOC Deleted |
|--------|----------|-----------|-------------|
| 2.1: Authorization | 2 days | 350 | 0 |
| 2.2: Idempotency & Transactions | 2 days | 250 | 0 |
| 2.3: Backpressure | 2 days | 200 | 0 |
| 2.4: Business Metrics | 2 days | 200 | 0 |
| 2.5: Middleware Refactoring | 3 days | 630 | 1,200 |
| **Total** | **11 days (2.2 weeks)** | **1,630** | **1,200** |

### Total Impact

**Original Plan (Sprints 2.1-2.4 only):**
- Duration: 2 weeks
- LOC Impact: +1,000 LOC added

**Enhanced Plan (All Sprints):**
- **Duration:** 2.2 weeks (10% increase)
- **LOC Impact:** +1,630 added, -1,200 deleted (net +430 LOC)
- **Middleware LOC:** 1,879 → 350 (81% reduction)

### Success Criteria

- ✅ Authorization policies enforced
- ✅ Idempotency at all entry points
- ✅ Transactional outbox working
- ✅ Domain events emitted
- ✅ Business metrics instrumented
- ✅ Middleware: only I/O validation (<350 LOC)
- ✅ Authentication: port-based
- ✅ Rate limiting: port-based

### Risk Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Authorization bugs in facades | MEDIUM | HIGH | Comprehensive auth tests per facade |
| Middleware refactor breaks auth | HIGH | CRITICAL | Feature flag for gradual rollout |
| Transaction performance overhead | MEDIUM | MEDIUM | Benchmark transactional workflows |
| Idempotency key collisions | LOW | HIGH | Use UUIDs + tenant scoping |

---

## Dependencies

**Requires Completion Of:**
- [PHASE_1_PORTS_ADAPTERS_ROADMAP.md](./PHASE_1_PORTS_ADAPTERS_ROADMAP.md) - Need ports defined

**Enables:**
- [PHASE_3_HANDLER_REFACTORING_ROADMAP.md](./PHASE_3_HANDLER_REFACTORING_ROADMAP.md) - Can now refactor handlers to <50 LOC

---

## Document Status

**Version:** 2.0 (Enhanced with Middleware Refactoring)
**Status:** ✅ Ready for Implementation
**Date:** 2025-11-08
**Dependencies:** Phase 1 complete
**Next Review:** After Sprint 2.3 completion

**Related Documents:**
- [API_CRATE_COVERAGE_ANALYSIS.md](../architecture/API_CRATE_COVERAGE_ANALYSIS.md)
- [PHASE_3_HANDLER_REFACTORING_ROADMAP.md](./PHASE_3_HANDLER_REFACTORING_ROADMAP.md) (next phase)
