# Phase 4: Integration & Production Readiness (Weeks 10-16)

**Duration**: 7 weeks (extended from 3 weeks in v1)
**Goal**: Remove AppState wrapper, flip to new-context by default, production hardening
**Key Outcomes**:
- AppState wrapper removed (hybrid pattern complete)
- Feature flag flipped: `new-context` becomes default
- Production observability established
- Hexagonal compliance: 95%+
- Zero infrastructure violations

---

## Phase Overview

Phase 4 is the culmination of 9 weeks of refactoring:

1. **Sprint 10**: Remove AppState wrapper (CAREFUL!)
2. **Sprint 11**: Feature flag flip and migration validation
3. **Sprint 12**: Production observability and monitoring
4. **Sprint 13-14**: Production hardening (buffer weeks)
5. **Sprint 15**: Pre-production validation
6. **Sprint 16**: Production deployment and celebration 🎉

**Why 7 weeks instead of 3?**
- v1 underestimated AppState removal complexity
- Need buffer for unexpected issues
- Production hardening requires time
- Stakeholder confidence requires thorough validation

---

## Sprint 10: Remove AppState Wrapper (Week 10)

**Goal**: Eliminate hybrid `AppState { context, facades }` pattern
**Risk**: HIGH - This is the most dangerous sprint
**Strategy**: Incremental removal, one route at a time

### Preconditions (MUST be met before Sprint 10)

- [ ] All 35+ facades use ApplicationContext (verify with grep)
- [ ] No facades import AppState
- [ ] All tests pass in `new-context` mode
- [ ] Feature flags tested every sprint (Sprints 1-9)
- [ ] Rollback drill successful in Sprint 9

**If any precondition fails, STOP and remediate before proceeding.**

### Day 1: Analyze AppState Usage

#### Task 10.1: Find All AppState References
```bash
# Find all code referencing AppState
grep -r "AppState" crates/ --include="*.rs" > appstate-usage.txt

# Categorize by usage type
grep "state: AppState" appstate-usage.txt > appstate-fields.txt
grep "AppState::" appstate-usage.txt > appstate-constructors.txt
grep "with_state" appstate-usage.txt > appstate-axum.txt

# Expected results:
# - appstate-fields.txt: ~35 routes (Axum handlers)
# - appstate-constructors.txt: 1 (main.rs)
# - appstate-axum.txt: 1 (Axum router setup)
```

**Acceptance Criteria**:
- [ ] All AppState usages catalogued
- [ ] Migration plan for each usage
- [ ] No unexpected references

### Day 2-3: Migrate Routes to FacadeRegistry

#### Task 10.2: Update Axum Router
**BEFORE** (hybrid pattern):
```rust
// crates/riptide-api/src/main.rs

#[cfg(feature = "new-context")]
pub struct AppState {
    pub context: Arc<ApplicationContext>,
    pub facades: Arc<FacadeRegistry>,
}

async fn main() -> Result<()> {
    let context = ApplicationContext::builder()
        // ... build context
        .build()?;

    let state = AppState {
        context: Arc::new(context),
        facades: Arc::new(FacadeRegistry::new(context)),
    };

    let app = Router::new()
        .route("/api/v1/crawl", post(crawl_handler))
        .with_state(state);  // AppState

    // ...
}

// Route handler
async fn crawl_handler(
    State(state): State<AppState>,
    Json(req): Json<CrawlRequest>,
) -> Result<Json<CrawlResponse>, ApiError> {
    let result = state.facades.crawl().crawl(&req.url).await?;
    Ok(Json(result.into()))
}
```

**AFTER** (AppState removed):
```rust
// crates/riptide-api/src/main.rs

async fn main() -> Result<()> {
    let context = ApplicationContext::builder()
        // ... build context
        .build()?;

    let facades = FacadeRegistry::new(Arc::new(context));

    let app = Router::new()
        .route("/api/v1/crawl", post(crawl_handler))
        .with_state(Arc::new(facades));  // FacadeRegistry directly

    // ...
}

// Route handler
async fn crawl_handler(
    State(facades): State<Arc<FacadeRegistry>>,
    Json(req): Json<CrawlRequest>,
) -> Result<Json<CrawlResponse>, ApiError> {
    let result = facades.crawl().crawl(&req.url).await?;
    Ok(Json(result.into()))
}
```

**Acceptance Criteria**:
- [ ] All 35+ routes updated to use `State<Arc<FacadeRegistry>>`
- [ ] No `AppState` imports in route handlers
- [ ] Compiles successfully

#### Task 10.3: Update Middleware
```rust
// BEFORE
pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, ApiError> {
    let token = extract_token(&req)?;
    let authorized = state.facades.authorization().authorize(token).await?;
    // ...
}

// AFTER
pub async fn auth_middleware(
    State(facades): State<Arc<FacadeRegistry>>,
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, ApiError> {
    let token = extract_token(&req)?;
    let authorized = facades.authorization().authorize(token).await?;
    // ...
}
```

**Acceptance Criteria**:
- [ ] All middleware updated
- [ ] Auth, logging, metrics middleware work
- [ ] No AppState references

### Day 4: Remove AppState Definition

#### Task 10.4: Delete AppState Struct
`crates/riptide-api/src/state.rs`:

```rust
// DELETE THIS (with feature flags):

#[cfg(feature = "new-context")]
pub struct AppState {
    pub context: Arc<ApplicationContext>,
    pub facades: Arc<FacadeRegistry>,
}

#[cfg(feature = "legacy-appstate")]
pub struct AppState {
    pub http_client: Client,
    pub cache: Arc<tokio::sync::Mutex<CacheManager>>,
    // ... 40+ fields
}

// File can now be deleted entirely!
```

**Acceptance Criteria**:
- [ ] `state.rs` deleted
- [ ] `mod state` removed from `lib.rs`
- [ ] Compiles successfully
- [ ] All tests pass

### Day 5: Cleanup and Validation

#### Task 10.5: Remove Feature Flags (Partial)
```toml
# crates/riptide-api/Cargo.toml

# BEFORE
[features]
default = ["legacy-appstate"]
legacy-appstate = []
new-context = []

# AFTER (keep flags for Sprint 11 rollback safety)
[features]
default = ["new-context"]  # Flip default!
legacy-appstate = []  # Keep for emergency rollback
new-context = []
```

**Don't delete feature flags yet** - keep for Sprint 11 validation.

**Acceptance Criteria**:
- [ ] `new-context` is now default
- [ ] `legacy-appstate` still compiles (for rollback)
- [ ] CI tests both modes

#### Task 10.6: Verify No AppState References
```bash
# Final verification
grep -r "AppState" crates/ --include="*.rs"

# Expected: Only in conditional compilation for legacy mode
# If any unexpected references, FIX before proceeding
```

**Acceptance Criteria**:
- [ ] No AppState references outside legacy mode
- [ ] Grep returns 0 results for new-context mode

#### Task 10.7: Update Documentation
`docs/architecture/ADR-004-appstate-removed.md`:

```markdown
# ADR 004: AppState Removed

**Status**: Accepted
**Date**: 2025-11-[DATE]
**Supersedes**: ADR-002 (Hybrid AppState pattern)

## Context
After 9 sprints with hybrid `AppState { context, facades }`, all facades now use ApplicationContext directly.

## Decision
Remove AppState wrapper entirely:
- Routes use `State<Arc<FacadeRegistry>>`
- FacadeRegistry owns all facades
- ApplicationContext provides port-based DI

## Migration
- Sprint 1-9: Gradual facade migration
- Sprint 10: AppState removal
- Sprint 11: Feature flag flip

## Consequences
- **Reduced complexity**: One state type instead of two
- **Better testability**: FacadeRegistry easily mocked
- **Cleaner architecture**: No god object
- **Emergency rollback**: `legacy-appstate` flag available until Sprint 16
```

**Acceptance Criteria**:
- [ ] ADR documents removal
- [ ] Migration timeline explained
- [ ] Consequences documented

### Sprint 10 Quality Gates

**All 6 gates must pass** ✅

**Additional Sprint 10 Validation**:
- [ ] AppState references: 100+ → 0 (new-context mode)
- [ ] God object eliminated (2213 LOC deleted!)
- [ ] Routes use FacadeRegistry
- [ ] Rollback to legacy-appstate tested and works

---

## Sprint 11: Feature Flag Flip (Week 11)

**Goal**: Make `new-context` the production default, validate in staging
**Risk**: MEDIUM - Production-like validation required

### Day 1-2: Staging Deployment

#### Task 11.1: Deploy to Staging with new-context
```bash
# Build with new default
cargo build --release

# Verify feature flags
cargo tree -p riptide-api -e features | grep "new-context"

# Expected: new-context enabled by default

# Deploy to staging
./scripts/deploy-staging.sh
```

**Acceptance Criteria**:
- [ ] Staging deployment succeeds
- [ ] Health check passes
- [ ] Metrics dashboard shows green

#### Task 11.2: Smoke Test All Routes
```bash
# Automated smoke tests
./scripts/smoke-tests.sh staging

# Manual validation
curl https://staging.riptide.com/api/v1/crawl -X POST -d '{"url":"https://example.com"}'
curl https://staging.riptide.com/api/v1/extract -X POST -d '{"url":"https://example.com"}'
curl https://staging.riptide.com/health

# Expected: All return 200 OK
```

**Acceptance Criteria**:
- [ ] All smoke tests pass
- [ ] No errors in logs
- [ ] Latency within acceptable range

### Day 3: Load Testing

#### Task 11.3: Production-Like Load Test
```rust
// Load test script
use goose::prelude::*;

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .register_scenario(
            scenario!("Crawl Load Test")
                .register_transaction(transaction!(crawl_transaction))
        )
        .set_default(GooseDefault::Host, "https://staging.riptide.com")?
        .set_default(GooseDefault::Users, 100)?  // 100 concurrent users
        .set_default(GooseDefault::RunTime, 600)?  // 10 minutes
        .execute()
        .await?;

    Ok(())
}

async fn crawl_transaction(user: &mut GooseUser) -> TransactionResult {
    let request = user.post("/api/v1/crawl")
        .json(&json!({ "url": "https://example.com" }));

    let response = user.get_request_builder(&request)?
        .send()
        .await?;

    let status = response.status();
    match status {
        StatusCode::OK => Ok(()),
        _ => Err(GooseError::from(format!("Unexpected status: {}", status))),
    }
}
```

Run load test:
```bash
cargo run --release --bin load-test

# Monitor metrics:
# - Throughput: >100 req/sec
# - p95 latency: <500ms
# - Error rate: <1%
```

**Acceptance Criteria**:
- [ ] Sustains 100 concurrent users for 10 minutes
- [ ] Throughput ≥100 req/sec
- [ ] p95 latency <500ms
- [ ] Error rate <1%

### Day 4: Chaos Testing

#### Task 11.4: Resilience Validation
```bash
# Chaos testing scenarios

# 1. Simulate database failure
kubectl exec -it postgres-0 -- pg_ctl stop

# Verify circuit breaker opens
curl https://staging.riptide.com/api/v1/crawl
# Expected: 503 Service Unavailable (circuit open)

# Restart database
kubectl exec -it postgres-0 -- pg_ctl start

# Verify circuit closes
sleep 10
curl https://staging.riptide.com/api/v1/crawl
# Expected: 200 OK (circuit closed)

# 2. Simulate Redis failure
kubectl scale deployment redis --replicas=0

# Verify graceful degradation (cache misses)
curl https://staging.riptide.com/api/v1/crawl
# Expected: 200 OK (slower, but works)

# 3. Simulate network latency
tc qdisc add dev eth0 root netem delay 200ms

# Verify retries work
curl https://staging.riptide.com/api/v1/crawl
# Expected: 200 OK (with retries logged)
```

**Acceptance Criteria**:
- [ ] Circuit breaker prevents cascading failures
- [ ] Graceful degradation when cache unavailable
- [ ] Retries handle network latency
- [ ] No panics or crashes

### Day 5: Rollback Drill

#### Task 11.5: Emergency Rollback Test
```bash
# Simulate production issue discovered
echo "SIMULATED ISSUE: Memory leak detected"

# Immediate rollback (flip feature flag)
kubectl set env deployment/riptide-api FEATURES="legacy-appstate"
kubectl rollout status deployment/riptide-api

# Verify rollback successful
curl https://staging.riptide.com/health
# Expected: 200 OK (legacy mode)

# Verify metrics
curl https://staging.riptide.com/metrics | grep riptide_mode
# Expected: riptide_mode="legacy-appstate"

# Measure rollback time
# Target: <5 minutes
```

**Acceptance Criteria**:
- [ ] Rollback completes in <5 minutes
- [ ] No data loss
- [ ] Zero downtime
- [ ] All services healthy

#### Task 11.6: Roll Forward Again
```bash
# After "fix", roll forward to new-context
kubectl set env deployment/riptide-api FEATURES="new-context"
kubectl rollout status deployment/riptide-api

# Verify health
curl https://staging.riptide.com/health
# Expected: 200 OK (new-context mode)
```

**Acceptance Criteria**:
- [ ] Roll forward succeeds
- [ ] Feature flag flip proven reliable
- [ ] Confidence in production deployment

### Sprint 11 Quality Gates

**All 6 gates must pass** ✅

**Additional Sprint 11 Validation**:
- [ ] Staging deployment successful
- [ ] Load test passed (100 users, 10 min)
- [ ] Chaos testing passed (resilience validated)
- [ ] Rollback drill <5 minutes
- [ ] Zero production incidents in staging

---

## Sprint 12: Production Observability (Week 12)

**Goal**: Establish comprehensive monitoring and alerting
**Why**: Can't deploy to production without observability

### Day 1-2: Metrics & Dashboards

#### Task 12.1: Prometheus Metrics
`crates/riptide-api/src/observability/metrics.rs`:

```rust
use prometheus::{
    Registry, Counter, Histogram, Gauge, HistogramOpts, Opts,
    register_counter_vec, register_histogram_vec, register_gauge_vec,
};

lazy_static! {
    // Request metrics
    pub static ref HTTP_REQUESTS: Counter = register_counter_vec!(
        "riptide_http_requests_total",
        "Total HTTP requests",
        &["method", "endpoint", "status"]
    ).unwrap();

    pub static ref HTTP_DURATION: Histogram = register_histogram_vec!(
        HistogramOpts::new("riptide_http_duration_seconds", "HTTP request duration"),
        &["method", "endpoint"]
    ).unwrap();

    // Facade metrics
    pub static ref FACADE_CALLS: Counter = register_counter_vec!(
        "riptide_facade_calls_total",
        "Total facade method calls",
        &["facade", "method", "result"]
    ).unwrap();

    pub static ref FACADE_DURATION: Histogram = register_histogram_vec!(
        HistogramOpts::new("riptide_facade_duration_seconds", "Facade method duration"),
        &["facade", "method"]
    ).unwrap();

    // Circuit breaker metrics
    pub static ref CIRCUIT_STATE: Gauge = register_gauge_vec!(
        "riptide_circuit_breaker_state",
        "Circuit breaker state (0=closed, 1=half-open, 2=open)",
        &["circuit"]
    ).unwrap();

    pub static ref CIRCUIT_FAILURES: Counter = register_counter_vec!(
        "riptide_circuit_breaker_failures_total",
        "Circuit breaker failures",
        &["circuit"]
    ).unwrap();

    // Rate limiter metrics
    pub static ref RATE_LIMIT_HITS: Counter = register_counter_vec!(
        "riptide_rate_limit_hits_total",
        "Rate limit hits",
        &["key", "allowed"]
    ).unwrap();

    // Idempotency metrics
    pub static ref IDEMPOTENCY_CHECKS: Counter = register_counter_vec!(
        "riptide_idempotency_checks_total",
        "Idempotency checks",
        &["operation", "duplicate"]
    ).unwrap();

    // Port usage metrics
    pub static ref PORT_CALLS: Counter = register_counter_vec!(
        "riptide_port_calls_total",
        "Port trait method calls",
        &["port", "method", "result"]
    ).unwrap();

    // Hexagonal compliance
    pub static ref HEXAGONAL_COMPLIANCE: Gauge = register_gauge!(
        "riptide_hexagonal_compliance_percent",
        "Hexagonal architecture compliance percentage"
    ).unwrap();
}

pub fn init_metrics() {
    // Set initial compliance gauge
    HEXAGONAL_COMPLIANCE.set(95.0);  // From Phase 3
}
```

**Acceptance Criteria**:
- [ ] All metrics registered
- [ ] Prometheus scraping works
- [ ] Metrics exported on `/metrics` endpoint

#### Task 12.2: Grafana Dashboards
Create `deployments/grafana/dashboards/riptide-overview.json`:

**Dashboard Panels**:
1. **Request Metrics**
   - Total requests/sec
   - Request duration (p50, p95, p99)
   - Error rate (4xx, 5xx)

2. **Facade Metrics**
   - Top 10 facades by call volume
   - Facade latency breakdown
   - Facade error rates

3. **Resilience Metrics**
   - Circuit breaker states
   - Rate limit hit rate
   - Retry attempts
   - Idempotency duplicate rate

4. **Port Metrics**
   - Port call volume by port type
   - Port error rates
   - Hexagonal compliance gauge

5. **Infrastructure Metrics**
   - Database connection pool
   - Redis cache hit rate
   - Browser pool utilization

**Acceptance Criteria**:
- [ ] Dashboard created
- [ ] All panels populated
- [ ] Auto-refresh working

### Day 3: Alerting

#### Task 12.3: Alert Rules
`deployments/prometheus/alerts/riptide.yml`:

```yaml
groups:
  - name: riptide_alerts
    interval: 30s
    rules:
      # High error rate
      - alert: HighErrorRate
        expr: rate(riptide_http_requests_total{status=~"5.."}[5m]) > 0.05
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} (>5%) for 2 minutes"

      # High latency
      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(riptide_http_duration_seconds_bucket[5m])) > 1.0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High p95 latency detected"
          description: "p95 latency is {{ $value }}s (>1s) for 5 minutes"

      # Circuit breaker open
      - alert: CircuitBreakerOpen
        expr: riptide_circuit_breaker_state{circuit="crawl"} == 2
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Circuit breaker open"
          description: "{{ $labels.circuit }} circuit breaker has been open for 1 minute"

      # Low hexagonal compliance (should never happen after refactor!)
      - alert: LowHexagonalCompliance
        expr: riptide_hexagonal_compliance_percent < 90
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Hexagonal compliance dropped"
          description: "Compliance is {{ $value }}% (<90%)"

      # High rate limit hit rate
      - alert: HighRateLimitHits
        expr: rate(riptide_rate_limit_hits_total{allowed="false"}[5m]) > 10
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High rate limit rejection rate"
          description: "{{ $value }} requests/sec being rate limited"
```

**Acceptance Criteria**:
- [ ] All alerts loaded
- [ ] Test alerts trigger correctly
- [ ] PagerDuty/Slack integration works

### Day 4-5: Logging & Tracing

#### Task 12.4: Structured Logging
```rust
use tracing::{info, warn, error, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

// Example facade logging
#[instrument(skip(self), fields(facade = "crawl"))]
pub async fn crawl(&self, url: &str) -> Result<CrawlResult> {
    info!(url = url, "Starting crawl");

    let result = self.context.circuit_breaker.call(|| async {
        // Crawl logic
    }).await;

    match &result {
        Ok(_) => info!(url = url, "Crawl succeeded"),
        Err(e) => error!(url = url, error = %e, "Crawl failed"),
    }

    result
}
```

**Acceptance Criteria**:
- [ ] All facades use structured logging
- [ ] Logs in JSON format
- [ ] Log levels configurable via `RUST_LOG`

#### Task 12.5: Distributed Tracing (OpenTelemetry)
```rust
use opentelemetry::{global, sdk::trace as sdktrace};
use tracing_opentelemetry::OpenTelemetryLayer;

pub fn init_tracing() -> Result<()> {
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("riptide-api")
        .install_batch(opentelemetry::runtime::Tokio)?;

    let telemetry = OpenTelemetryLayer::new(tracer);

    tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}
```

**Acceptance Criteria**:
- [ ] Tracing spans created for facades
- [ ] Trace IDs propagated across services
- [ ] Jaeger UI shows traces

### Sprint 12 Quality Gates

**All 6 gates must pass** ✅

**Additional Sprint 12 Validation**:
- [ ] Prometheus metrics exported
- [ ] Grafana dashboards functional
- [ ] All alerts tested and working
- [ ] Structured logging in place
- [ ] Distributed tracing operational

---

## Sprint 13-14: Production Hardening (Weeks 13-14)

**Goal**: Buffer time for unexpected issues and final polish
**Why**: v1 was too optimistic, reality requires buffer

### Week 13: Security & Performance

#### Task 13.1: Security Audit
- [ ] Dependency audit: `cargo audit`
- [ ] OWASP Top 10 review
- [ ] SQL injection testing (prepared statements)
- [ ] XSS prevention (input sanitization)
- [ ] CSRF token validation
- [ ] Rate limiting per IP
- [ ] Authentication/authorization review

#### Task 13.2: Performance Optimization
- [ ] Profile with `perf` and flamegraphs
- [ ] Optimize hot paths (allocations, cloning)
- [ ] Database query optimization (EXPLAIN ANALYZE)
- [ ] Cache hit rate tuning (>80% target)
- [ ] Connection pool sizing
- [ ] Thread pool tuning

#### Task 13.3: Documentation Finalization
- [ ] API documentation (OpenAPI/Swagger)
- [ ] Architecture diagrams updated
- [ ] ADRs complete for all decisions
- [ ] Runbooks for common operations
- [ ] Onboarding guide for new developers
- [ ] Troubleshooting guide

### Week 14: Production Readiness Review

#### Task 14.1: Production Readiness Checklist
```markdown
## Infrastructure
- [ ] Load balancer configured
- [ ] Auto-scaling policies defined
- [ ] Database backups automated (hourly)
- [ ] Redis persistence enabled
- [ ] Secrets management (Vault/AWS Secrets Manager)
- [ ] TLS certificates configured
- [ ] CDN configured (if needed)

## Observability
- [ ] Prometheus scraping all services
- [ ] Grafana dashboards operational
- [ ] Alerts tested and PagerDuty integrated
- [ ] Logging aggregation (ELK/Splunk)
- [ ] Distributed tracing (Jaeger)
- [ ] Metrics retention policy (90 days)

## Reliability
- [ ] Circuit breakers configured
- [ ] Rate limiters per endpoint
- [ ] Retry policies with exponential backoff
- [ ] Idempotency for all mutations
- [ ] Health checks for all dependencies
- [ ] Graceful shutdown (SIGTERM handling)

## Security
- [ ] All dependencies up to date
- [ ] No known vulnerabilities (cargo audit)
- [ ] Authentication required for all routes
- [ ] Authorization checks in place
- [ ] Input validation on all endpoints
- [ ] SQL injection prevention
- [ ] XSS prevention
- [ ] CORS configured correctly

## Testing
- [ ] 90%+ test coverage
- [ ] All integration tests passing
- [ ] Load tests passed (100 concurrent users)
- [ ] Chaos tests passed (resilience validated)
- [ ] Rollback drill <5 minutes

## Compliance
- [ ] Hexagonal compliance ≥95%
- [ ] Zero infrastructure violations
- [ ] All ports have adapters
- [ ] All facades use ApplicationContext
- [ ] No circular dependencies
- [ ] No ignored tests
```

**Acceptance Criteria**:
- [ ] ALL checklist items complete
- [ ] Sign-off from Tech Lead, QA, Security, DevOps
- [ ] Stakeholder approval for production

### Sprint 13-14 Quality Gates

**All 6 gates must pass** ✅

---

## Sprint 15: Pre-Production Validation (Week 15)

**Goal**: Final validation before production deployment

### Day 1-2: Production Clone Testing

#### Task 15.1: Deploy to Production-Like Environment
```bash
# Clone production data to staging
./scripts/clone-prod-data.sh

# Deploy with production configs
./scripts/deploy-staging.sh --production-config

# Run full test suite
./scripts/full-test-suite.sh
```

**Acceptance Criteria**:
- [ ] Production data imported successfully
- [ ] All tests pass with production data
- [ ] No performance degradation

### Day 3-4: Soak Testing

#### Task 15.2: 48-Hour Soak Test
```bash
# Run load test for 48 hours
./scripts/soak-test.sh --duration 48h --users 50

# Monitor:
# - Memory leaks (heap growth)
# - Connection leaks (open connections)
# - File descriptor leaks
# - CPU usage trends
# - Error rates over time
```

**Acceptance Criteria**:
- [ ] No memory leaks (stable heap)
- [ ] No connection leaks
- [ ] CPU usage stable
- [ ] Error rate <0.1%

### Day 5: Go/No-Go Decision

#### Task 15.3: Production Deployment Approval
**Go/No-Go Criteria**:
- [ ] All quality gates passed (Sprints 1-15)
- [ ] Production readiness checklist 100% complete
- [ ] Soak test passed (48 hours)
- [ ] Security audit passed
- [ ] Stakeholder sign-off
- [ ] Rollback plan documented and tested
- [ ] On-call rotation staffed
- [ ] Communication plan ready

**If any criterion is NO-GO**: STOP and remediate.

**Acceptance Criteria**:
- [ ] Written approval from Tech Lead
- [ ] Written approval from Product Owner
- [ ] Written approval from CTO/VP Engineering
- [ ] Deployment scheduled

### Sprint 15 Quality Gates

**All 6 gates must pass** ✅

---

## Sprint 16: Production Deployment (Week 16)

**Goal**: Deploy to production and celebrate success!

### Day 1: Production Deployment

#### Task 16.1: Blue-Green Deployment
```bash
# Deploy to green environment (new-context)
./scripts/deploy-production.sh --environment green --version v2.0.0

# Smoke test green
./scripts/smoke-test.sh green

# Health check
curl https://green.riptide.com/health
# Expected: 200 OK

# Route 10% traffic to green
./scripts/traffic-split.sh --green 10

# Monitor for 1 hour
# - Error rates
# - Latency
# - Resource usage

# If healthy, route 50% traffic
./scripts/traffic-split.sh --green 50

# Monitor for 1 hour

# If healthy, route 100% traffic
./scripts/traffic-split.sh --green 100

# Decommission blue
./scripts/decommission.sh --environment blue
```

**Acceptance Criteria**:
- [ ] Green deployment successful
- [ ] Traffic gradually shifted (10% → 50% → 100%)
- [ ] No errors during shift
- [ ] Blue decommissioned after 24 hours

### Day 2-3: Post-Deployment Monitoring

#### Task 16.2: Intensive Monitoring
Monitor for 48 hours:
- [ ] Error rates (target: <0.1%)
- [ ] Latency (p95 < 500ms)
- [ ] Resource usage (CPU < 70%, memory < 80%)
- [ ] Database connections (no leaks)
- [ ] Circuit breaker states (mostly closed)
- [ ] Rate limit hit rate (acceptable)

**Acceptance Criteria**:
- [ ] All metrics within acceptable range
- [ ] No critical alerts
- [ ] No rollbacks required

### Day 4: Feature Flag Cleanup

#### Task 16.3: Remove Legacy Feature Flags
```toml
# crates/riptide-api/Cargo.toml

# BEFORE (kept for rollback safety)
[features]
default = ["new-context"]
legacy-appstate = []
new-context = []

# AFTER (legacy removed, no longer needed!)
[features]
default = []
# All feature flags removed - we're 100% new architecture!
```

```bash
# Delete legacy code paths
rm -rf crates/riptide-api/src/legacy/
git rm crates/riptide-api/src/state.rs  # Already deleted in Sprint 10

# Verify no conditional compilation
grep -r "#\[cfg(feature" crates/ --include="*.rs"

# Expected: 0 results (all feature flags removed)
```

**Acceptance Criteria**:
- [ ] All feature flags removed
- [ ] Legacy code paths deleted
- [ ] Compiles without warnings
- [ ] Tests pass

### Day 5: Retrospective & Celebration 🎉

#### Task 16.4: Project Retrospective

**Metrics Review**:

| Metric | Week 0 Baseline | Sprint 16 Final | Improvement |
|--------|-----------------|-----------------|-------------|
| Hexagonal Compliance | 24% | 95%+ | +296% |
| Test Coverage | 61% | 90%+ | +48% |
| Ignored Tests | 44 | 0 | -100% |
| Infrastructure Violations | 32 | 0 | -100% |
| Circular Dependencies | 8 | 0 | -100% |
| Missing Ports | 7 | 0 | -100% |
| AppState Fields | 40+ | 0 (deleted!) | -100% |

**Business Impact**:
- [ ] Deployment frequency: +2x
- [ ] MTTR: -40%
- [ ] Onboarding time: -60%
- [ ] Developer velocity: +35%
- [ ] Production incidents: -50%

**Lessons Learned**:
- What went well
- What could be improved
- What to do differently next time
- Knowledge sharing session

**Acceptance Criteria**:
- [ ] Retrospective documented
- [ ] Lessons learned shared with team
- [ ] Celebration event scheduled! 🎉

#### Task 16.5: Knowledge Transfer

**Documentation Handoff**:
- [ ] Architecture overview presentation
- [ ] Port-adapter training session
- [ ] Troubleshooting workshop
- [ ] On-call rotation training
- [ ] Q&A session for team

**Acceptance Criteria**:
- [ ] All team members trained
- [ ] Documentation complete
- [ ] On-call rotation operational

### Sprint 16 Quality Gates

**All 6 gates must pass** ✅

**Final Validation**:
- [ ] Production deployment successful
- [ ] Zero critical incidents
- [ ] All metrics green
- [ ] Feature flags removed
- [ ] Team trained
- [ ] PROJECT COMPLETE! 🎉

---

## Phase 4 Success Metrics

| Metric | Phase 3 End | Sprint 16 Target | Actual |
|--------|-------------|------------------|--------|
| AppState Removed | No | Yes | ___ |
| Feature Flag Default | legacy | new-context | ___ |
| Production Deployment | No | Yes | ___ |
| Hexagonal Compliance | 55% | 95%+ | ___ |
| Zero Violations | No | Yes | ___ |

---

## Phase 4 Deliverables

- [ ] AppState wrapper removed (Sprint 10)
- [ ] Feature flag flipped to new-context (Sprint 11)
- [ ] Production observability established (Sprint 12)
- [ ] Security audit passed (Sprint 13)
- [ ] Production readiness validated (Sprint 15)
- [ ] Production deployment successful (Sprint 16)
- [ ] All feature flags removed (Sprint 16)
- [ ] Hexagonal compliance: 95%+
- [ ] PROJECT COMPLETE! 🚀

---

## Final Checklist

Before marking project complete:

- [ ] All 16 sprints completed
- [ ] All quality gates passed
- [ ] Production running stable for 48+ hours
- [ ] All documentation updated
- [ ] Team trained
- [ ] Retrospective completed
- [ ] Celebration held 🎉

---

**Status**: Ready for Sprint 10 kickoff
**Owner**: Full Engineering Team
**Duration**: 7 weeks
**End State**: Production-ready hexagonal architecture with 95%+ compliance

🎯 **SUCCESS!**
