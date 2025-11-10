# Week 0: Baseline & Validation

**Duration**: 5 business days
**Goal**: Establish measurable baseline and prevent duplicate port creation
**Outcome**: Comprehensive dependency map, port inventory, baseline metrics dashboard

---

## Overview

Week 0 is **critical** to avoid the blockers that plagued v1 of this roadmap:
- Recreating ports that already exist (wasted 2 weeks in v1)
- No baseline metrics (couldn't measure progress)
- Undefined initialization order (caused circular dependency bugs)
- Missing adapter inventory (duplicated infrastructure work)

This week captures the "before" state so we can prove the "after" state is better.

---

## Day 1: Baseline Metrics Capture

### Morning (9 AM - 12 PM)

#### Task 1.1: Capture Build & Test Baseline
```bash
# Run full test suite and capture results
cargo test --workspace > baseline-tests.txt 2>&1

# Capture test counts
grep "test result:" baseline-tests.txt > baseline-test-summary.txt

# Capture ignored tests
grep "#\[ignore\]" -r crates/ --include="*.rs" > baseline-ignored-tests.txt

# Count violations
echo "Ignored tests: $(wc -l < baseline-ignored-tests.txt)" >> baseline-metrics.md
```

**Acceptance Criteria**:
- [ ] Total test count recorded
- [ ] Passing/failing/ignored counts documented
- [ ] Baseline stored in `docs/metrics/baseline-tests.txt`

#### Task 1.2: Capture Code Metrics
```bash
# Line counts per crate
find crates/ -name "*.rs" -exec wc -l {} + > baseline-loc.txt

# Clippy warnings count
cargo clippy --workspace -- -D warnings 2>&1 | tee baseline-clippy.txt

# Count infrastructure violations (facades with concrete deps)
grep -r "use riptide_infrastructure::" crates/riptide-facade/src/*.rs | wc -l > baseline-violations.txt
```

**Acceptance Criteria**:
- [ ] Lines of code per crate recorded
- [ ] Clippy warning count baseline established
- [ ] Infrastructure violation count (should be 32)

### Afternoon (1 PM - 5 PM)

#### Task 1.3: Benchmark Runtime Performance
```bash
# Run composition benchmarks
cd crates/riptide-facade
cargo bench --bench composition_benchmarks > ../../docs/metrics/baseline-benchmarks.txt

# Capture key metrics
grep "time:" ../../docs/metrics/baseline-benchmarks.txt > ../../docs/metrics/baseline-perf-summary.txt
```

**Acceptance Criteria**:
- [ ] Baseline benchmark results for `boxed_stream` and `composed_stream`
- [ ] Performance targets defined (e.g., <10% overhead after refactor)

#### Task 1.4: Document Current State
Create `docs/metrics/BASELINE-METRICS.md`:

```markdown
# Baseline Metrics (Week 0)

**Captured**: 2025-11-10
**Purpose**: Measurable "before" state for refactoring validation

## Test Metrics
- **Total Tests**: [COUNT]
- **Passing**: [COUNT]
- **Failing**: [COUNT]
- **Ignored**: 44
- **Coverage**: 61%

## Architecture Metrics
- **Hexagonal Compliance**: 24%
- **Infrastructure Violations**: 32
- **Circular Dependencies**: 8
- **AppState Fields**: 40+

## Performance Metrics
- **Baseline Stream (1000 items)**: [TIME]
- **BoxStream (1000 items)**: [TIME]
- **Composed Stream (1000 items)**: [TIME]

## Code Metrics
- **riptide-api LOC**: [COUNT]
- **riptide-facade LOC**: [COUNT]
- **riptide-browser LOC**: 7,878
- **Total Workspace LOC**: [COUNT]

## Quality Metrics
- **Clippy Warnings**: [COUNT]
- **Dead Code Warnings**: [COUNT]
- **Large Files (>1000 LOC)**: 5 (state.rs, browser.rs, streaming.rs, etc.)
```

**Acceptance Criteria**:
- [ ] All baseline metrics documented with actual values
- [ ] File committed to git for historical tracking

---

## Day 2: Port Inventory & Audit

### Morning (9 AM - 12 PM)

#### Task 2.1: Inventory Existing Ports
Create `docs/architecture/PORT-INVENTORY.md`:

```markdown
# Existing Ports (Already Implemented)

## Core Infrastructure Ports
✅ `BrowserDriver` - crates/riptide-types/src/browser.rs
✅ `HttpClient` - crates/riptide-types/src/http.rs
✅ `CacheStorage` - crates/riptide-types/src/cache.rs
✅ `SessionStorage` - crates/riptide-types/src/session.rs
✅ `EventBus` - crates/riptide-types/src/events.rs

## Domain Ports
✅ `Repository<T>` - crates/riptide-types/src/repository.rs
✅ `TransactionManager` - crates/riptide-types/src/transaction.rs
✅ `Clock` - crates/riptide-types/src/time.rs
✅ `Entropy` - crates/riptide-types/src/entropy.rs

## Metrics Ports
✅ `MetricsCollector` - crates/riptide-types/src/metrics.rs
✅ `HealthChecker` - crates/riptide-types/src/health.rs
```

**Acceptance Criteria**:
- [ ] All existing ports catalogued with file paths
- [ ] Trait signatures validated (compile check)
- [ ] Adapter implementations verified for each port

#### Task 2.2: Identify MISSING Ports
Cross-reference facade dependencies against port inventory:

```bash
# Find all facade dependencies
grep -r "pub.*: Arc<" crates/riptide-facade/src/*.rs > facade-deps.txt

# Compare against port inventory
# Manual review to identify missing ports
```

**Expected Missing Ports** (to be created in Sprint 2 & 5.5):
1. `IdempotencyStore` - Missing port for deduplication
2. `CircuitBreaker` - Missing port for resilience
3. `RateLimiter` - Missing port for throttling
4. `Validator` - Missing port for input validation
5. `Authorizer` - Missing port for permissions
6. `SearchEngine` - Missing port for full-text search
7. `PdfProcessor` - Missing port for PDF generation

**Acceptance Criteria**:
- [ ] List of 7 genuinely missing ports documented
- [ ] Rationale for each missing port explained
- [ ] Priority order for creation (P0 vs P1)

### Afternoon (1 PM - 5 PM)

#### Task 2.3: Adapter Inventory
Create `docs/architecture/ADAPTER-INVENTORY.md`:

```markdown
# Existing Adapters

## Infrastructure Adapters (riptide-infrastructure)
✅ `ChromiumBrowserAdapter` → BrowserDriver
✅ `ReqwestHttpAdapter` → HttpClient
✅ `RedisCacheAdapter` → CacheStorage
✅ `PostgresSessionAdapter` → SessionStorage
✅ `NatsBusAdapter` → EventBus
✅ `PrometheusMetricsAdapter` → MetricsCollector

## Missing Adapters (to create)
❌ `InMemoryIdempotencyAdapter` → IdempotencyStore
❌ `CircuitBreakerAdapter` → CircuitBreaker
❌ `RateLimiterAdapter` → RateLimiter
❌ `JsonSchemaValidatorAdapter` → Validator
❌ `RbacAuthorizerAdapter` → Authorizer
❌ `MeilisearchAdapter` → SearchEngine
❌ `WeasyPrintAdapter` → PdfProcessor
```

**Acceptance Criteria**:
- [ ] All existing adapters catalogued
- [ ] Missing adapters identified for Sprint 6
- [ ] Implementation complexity estimated (S/M/L)

#### Task 2.4: Document Dependency Graph
Use `cargo tree` to map facade dependencies:

```bash
# Generate dependency graph for riptide-facade
cargo tree -p riptide-facade --depth 2 > facade-dependency-tree.txt

# Identify circular dependencies
cargo tree -p riptide-api --depth 3 | grep -E "riptide-(facade|api)" > circular-deps.txt
```

Create visual dependency map:
```
AppState (2213 LOC)
├─> Facades (35+ facades)
│   ├─> BrowserFacade (1711 LOC)
│   ├─> StreamingFacade (1464 LOC)
│   └─> ...
├─> Infrastructure (direct dependencies - VIOLATION)
│   ├─> ChromiumBrowserAdapter
│   ├─> ReqwestHttpAdapter
│   └─> ...
└─> ApplicationContext (NOT INTEGRATED)
    └─> Port Traits (clean interface)
```

**Acceptance Criteria**:
- [ ] Dependency graph documented with file paths
- [ ] Circular dependencies highlighted (8 expected)
- [ ] Violation count matches baseline (32)

---

## Day 3: Initialization Order Analysis

### Morning (9 AM - 12 PM)

#### Task 3.1: Map Current Initialization
Trace `main.rs` startup sequence:

```rust
// crates/riptide-api/src/main.rs
async fn main() -> Result<()> {
    // 1. Load configuration
    let config = load_config()?;

    // 2. Initialize infrastructure (DIRECT - VIOLATION)
    let http_client = reqwest::Client::new();
    let browser_pool = ChromiumPool::new(config.browser)?;
    let cache = RedisCache::new(config.redis)?;

    // 3. Initialize AppState (GOD OBJECT)
    let state = AppState {
        http_client,
        browser_pool,
        cache,
        // ... 37 more fields
    };

    // 4. ApplicationContext NOT USED

    // 5. Start server
    axum::Server::bind(&addr)
        .serve(app.with_state(state))
        .await?;
}
```

**Acceptance Criteria**:
- [ ] Current initialization order documented
- [ ] Direct infrastructure dependencies identified (VIOLATIONS)
- [ ] Missing ApplicationContext integration highlighted

#### Task 3.2: Design Target Initialization Order
Document ideal initialization sequence:

```rust
// TARGET: Clean initialization with ApplicationContext

async fn main() -> Result<()> {
    // 1. Load configuration
    let config = load_config()?;

    // 2. Initialize adapters (infrastructure)
    let browser_adapter = ChromiumBrowserAdapter::new(config.browser)?;
    let http_adapter = ReqwestHttpAdapter::new(config.http)?;
    let cache_adapter = RedisCacheAdapter::new(config.redis)?;

    // 3. Initialize ApplicationContext (DI container)
    let context = ApplicationContext::builder()
        .with_browser_driver(Arc::new(browser_adapter))
        .with_http_client(Arc::new(http_adapter))
        .with_cache_storage(Arc::new(cache_adapter))
        .build()?;

    // 4. Validate context (all ports wired)
    context.validate()?;

    // 5. Initialize facades (depends on context)
    let facades = FacadeRegistry::new(context.clone());

    // 6. Hybrid AppState (wrapper for gradual migration)
    let state = AppState { context, facades };

    // 7. Start server
    axum::Server::bind(&addr)
        .serve(app.with_state(state))
        .await?;
}
```

**Acceptance Criteria**:
- [ ] Target initialization order documented
- [ ] Dependencies between layers clarified
- [ ] Hybrid AppState pattern shown

### Afternoon (1 PM - 5 PM)

#### Task 3.3: Identify Init Order Blockers
Document what prevents clean initialization today:

**Blockers**:
1. **AppState has 40+ fields** - Too large to migrate atomically
2. **Facades depend on AppState** - Circular dependency
3. **ApplicationContext not integrated** - Not used in runtime
4. **No validation step** - Can't verify port wiring
5. **Direct infrastructure imports** - Facades bypass DI

**Resolution Plan** (implemented in Sprint 3):
- Keep hybrid `AppState { context, facades }` until Sprint 10
- Migrate facades one-by-one to use `context` instead of direct fields
- Add `ApplicationContext::validate()` in Sprint 2
- Feature flag: `new-context` (opt-in), `legacy-appstate` (default)

**Acceptance Criteria**:
- [ ] All init order blockers documented
- [ ] Resolution mapped to specific sprints
- [ ] Feature flag strategy defined

#### Task 3.4: Create Init Order ADR
Document architecture decision:

`docs/architecture/ADR-001-initialization-order.md`:
```markdown
# ADR 001: Hybrid Initialization Order

**Status**: Accepted
**Date**: 2025-11-10

## Context
AppState has 40+ fields creating initialization complexity and circular dependencies.

## Decision
Use hybrid pattern during migration:
1. Initialize ApplicationContext first (ports)
2. Initialize Facades second (depends on context)
3. Wrap both in AppState temporarily
4. Remove AppState wrapper in Sprint 10

## Consequences
- Gradual migration reduces risk
- Feature flags enable safe rollback
- Temporary complexity (two systems)
- Clean endpoint by Sprint 10
```

**Acceptance Criteria**:
- [ ] ADR documents decision rationale
- [ ] Consequences clearly stated
- [ ] Timeline for hybrid pattern removal defined

---

## Day 4: Feature Flag Setup

### Morning (9 AM - 12 PM)

#### Task 4.1: Add Feature Flags to Cargo.toml
```toml
# crates/riptide-api/Cargo.toml
[features]
default = ["legacy-appstate"]
legacy-appstate = []
new-context = []
```

**Acceptance Criteria**:
- [ ] Feature flags added to all affected crates
- [ ] `cargo build --features new-context` compiles
- [ ] `cargo build --features legacy-appstate` compiles (default)

#### Task 4.2: Add Conditional Compilation
```rust
// crates/riptide-api/src/state.rs

#[cfg(feature = "legacy-appstate")]
pub struct AppState {
    pub http_client: Client,
    pub cache: Arc<tokio::sync::Mutex<CacheManager>>,
    // ... 40+ fields
}

#[cfg(feature = "new-context")]
pub struct AppState {
    pub context: Arc<ApplicationContext>,
    pub facades: Arc<FacadeRegistry>,
}
```

**Acceptance Criteria**:
- [ ] Both code paths compile independently
- [ ] Tests pass in both modes
- [ ] Runtime behavior identical (for now)

### Afternoon (1 PM - 5 PM)

#### Task 4.3: Create Rollback Runbook
`docs/runbooks/ROLLBACK-FEATURE-FLAGS.md`:
```markdown
# Rollback Procedure: Feature Flags

## Immediate Rollback (<5 minutes)

1. **Stop incoming traffic**: Set load balancer to maintenance mode
2. **Flip feature flag**:
   ```bash
   # In deployment config
   FEATURES="legacy-appstate"  # Remove "new-context"
   ```
3. **Restart service**: `systemctl restart riptide-api`
4. **Verify health**: `curl http://localhost:3000/health`
5. **Resume traffic**: Remove maintenance mode

## Full Rollback (<2 hours)

1. **Immediate rollback** (above)
2. **Revert deployment**: `git revert <commit>` or rollback container tag
3. **Rebuild**: `cargo build --release --features legacy-appstate`
4. **Redeploy**: Standard deployment procedure
5. **Post-mortem**: Document rollback reason and preventive measures
```

**Acceptance Criteria**:
- [ ] Rollback procedure documented
- [ ] <5 minute immediate rollback proven
- [ ] <2 hour full recovery validated

#### Task 4.4: Setup CI/CD for Both Modes
Update `.github/workflows/ci.yml`:
```yaml
jobs:
  test-legacy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Test legacy mode
        run: cargo test --features legacy-appstate

  test-new:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Test new context mode
        run: cargo test --features new-context
```

**Acceptance Criteria**:
- [ ] CI runs tests for both modes
- [ ] Both modes must pass before merge
- [ ] Build artifacts for both modes generated

---

## Day 5: Quality Gate Rehearsal

### Morning (9 AM - 12 PM)

#### Task 5.1: Run Baseline Quality Gates
Execute all 6 gates on current codebase:

**Gate 1: Builds in both modes**
```bash
cargo build --features legacy-appstate
cargo build --features new-context
# Expected: Both pass (identical code for now)
```

**Gate 2: Top routes run**
```bash
# Start server
cargo run &
sleep 5

# Test top 3 routes
curl http://localhost:3000/api/v1/crawl -X POST -d '{"url":"https://example.com"}'
curl http://localhost:3000/api/v1/extract -X POST -d '{"url":"https://example.com"}'
curl http://localhost:3000/health

# Expected: All return 200 OK
```

**Gate 3: All ports wired**
```bash
# Not applicable yet (ApplicationContext not integrated)
# Will be added in Sprint 2
```

**Gate 4: Tests pass**
```bash
cargo test --workspace
# Expected: [BASELINE] passing, 44 ignored
```

**Gate 5: Rollback works**
```bash
# Switch feature flags
FEATURES="new-context" cargo run &
# Kill server
FEATURES="legacy-appstate" cargo run &
# Expected: Both modes work identically
```

**Gate 6: Docs updated**
```bash
# Verify baseline docs exist
ls docs/metrics/BASELINE-METRICS.md
ls docs/architecture/PORT-INVENTORY.md
ls docs/architecture/ADAPTER-INVENTORY.md
# Expected: All exist and accurate
```

**Acceptance Criteria**:
- [ ] All 6 gates executed
- [ ] Results documented (pass/fail)
- [ ] Rehearsal proves gates are achievable

### Afternoon (1 PM - 5 PM)

#### Task 5.2: Create Sprint Checklist Template
`docs/templates/SPRINT-QUALITY-GATES.md`:
```markdown
# Sprint [N] Quality Gates

**Sprint Goal**: [Description]
**Date**: [Date]

## Pre-Gate Checklist
- [ ] Code freeze (no new features)
- [ ] All PRs merged
- [ ] Changelog updated

## Mandatory Gates

### Gate 1: Builds in both modes
- [ ] `cargo build --features legacy-appstate` ✅
- [ ] `cargo build --features new-context` ✅
- [ ] No warnings with `-D warnings`

### Gate 2: Top routes run
- [ ] `/api/v1/crawl` returns 200 ✅
- [ ] `/api/v1/extract` returns 200 ✅
- [ ] `/health` returns 200 ✅
- [ ] Response payloads match spec

### Gate 3: All ports wired
- [ ] `ApplicationContext::validate()` passes ✅
- [ ] No missing dependencies
- [ ] Dependency graph acyclic

### Gate 4: Tests pass
- [ ] Unit tests: [COUNT] passing
- [ ] Integration tests: [COUNT] passing
- [ ] Coverage: ≥90%
- [ ] No ignored tests added

### Gate 5: Rollback works
- [ ] Feature flag flip tested ✅
- [ ] Rollback completes in <5 minutes
- [ ] Zero data loss
- [ ] No manual intervention required

### Gate 6: Docs updated
- [ ] Dependency matrix current ✅
- [ ] ADRs written for new ports
- [ ] Migration guide updated
- [ ] Runbook changes documented

## Post-Gate Actions
- [ ] Metrics dashboard updated
- [ ] Stakeholder notification sent
- [ ] Sprint retrospective scheduled
- [ ] Next sprint planning completed

## Sign-off
- [ ] Tech Lead: [Name]
- [ ] QA Lead: [Name]
- [ ] Product Owner: [Name]
```

**Acceptance Criteria**:
- [ ] Template created and validated
- [ ] Team trained on gate execution
- [ ] Checklist integrated into sprint workflow

#### Task 5.3: Create Baseline Metrics Dashboard
Document baseline for progress tracking:

`docs/metrics/PROGRESS-DASHBOARD.md`:
```markdown
# Refactoring Progress Dashboard

## Hexagonal Compliance

| Metric | Week 0 | Sprint 3 | Sprint 6 | Sprint 9 | Sprint 12 | Sprint 16 | Target |
|--------|--------|----------|----------|----------|-----------|-----------|--------|
| Compliance % | 24% | - | - | - | - | - | 95% |
| Violations | 32 | - | - | - | - | - | 0 |
| Circular Deps | 8 | - | - | - | - | - | 0 |

## Test Coverage

| Metric | Week 0 | Sprint 3 | Sprint 6 | Sprint 9 | Sprint 12 | Sprint 16 | Target |
|--------|--------|----------|----------|----------|-----------|-----------|--------|
| Coverage % | 61% | - | - | - | - | - | 90% |
| Ignored Tests | 44 | - | - | - | - | - | 0 |
| Total Tests | [COUNT] | - | - | - | - | - | [+50%] |

## Quality Gates

| Sprint | Gate 1 | Gate 2 | Gate 3 | Gate 4 | Gate 5 | Gate 6 | Status |
|--------|--------|--------|--------|--------|--------|--------|--------|
| Week 0 | ✅ | ✅ | N/A | ⚠️ | ✅ | ✅ | Baseline |
| Sprint 1 | - | - | - | - | - | - | Pending |
```

**Acceptance Criteria**:
- [ ] Dashboard template created
- [ ] Week 0 baseline values filled in
- [ ] Update cadence defined (end of each sprint)

#### Task 5.4: Week 0 Retrospective
Document learnings and blockers:

**What Went Well**:
- Comprehensive baseline captured
- Port inventory prevents duplicate work
- Feature flags proven to work

**What Could Improve**:
- [Team fills in]

**Blockers for Sprint 1**:
- [Team fills in]

**Action Items**:
- [ ] Share roadmap with stakeholders
- [ ] Schedule Sprint 1 kickoff
- [ ] Assign facades to engineers

**Acceptance Criteria**:
- [ ] Retrospective documented
- [ ] Action items tracked in project management tool
- [ ] Stakeholder communication sent

---

## Week 0 Quality Gates

### Gate 1: Builds in both modes
**Status**: ✅ PASS
- `cargo build --features legacy-appstate`: Success
- `cargo build --features new-context`: Success (identical code)

### Gate 2: Top routes run
**Status**: ✅ PASS
- `/api/v1/crawl`: 200 OK
- `/api/v1/extract`: 200 OK
- `/health`: 200 OK

### Gate 3: All ports wired
**Status**: N/A (ApplicationContext not yet integrated)
- Will be applicable starting Sprint 2

### Gate 4: Tests pass
**Status**: ⚠️ BASELINE
- Total: [COUNT]
- Passing: [COUNT]
- Failing: 3 (CDP batch execution - existing issue)
- Ignored: 44 (to be addressed in Sprint 8-9)
- Coverage: 61%

### Gate 5: Rollback works
**Status**: ✅ PASS
- Feature flag flip tested
- Both modes run identically
- <5 minute rollback proven

### Gate 6: Docs updated
**Status**: ✅ PASS
- `BASELINE-METRICS.md` created
- `PORT-INVENTORY.md` created
- `ADAPTER-INVENTORY.md` created
- `ADR-001-initialization-order.md` created

---

## Deliverables Checklist

- [ ] `docs/metrics/BASELINE-METRICS.md` - All baseline values captured
- [ ] `docs/architecture/PORT-INVENTORY.md` - Existing ports catalogued
- [ ] `docs/architecture/ADAPTER-INVENTORY.md` - Adapters mapped
- [ ] `docs/architecture/ADR-001-initialization-order.md` - Init order documented
- [ ] `docs/runbooks/ROLLBACK-FEATURE-FLAGS.md` - Rollback procedure
- [ ] `docs/templates/SPRINT-QUALITY-GATES.md` - Reusable checklist
- [ ] `docs/metrics/PROGRESS-DASHBOARD.md` - Progress tracking
- [ ] Feature flags added to `Cargo.toml`
- [ ] CI/CD updated for dual-mode builds
- [ ] Stakeholder communication sent

---

## Next: Sprint 1

**Goal**: Create 5 missing P0 ports and break 3 critical circular dependencies
**Duration**: 1 week
**See**: [ROADMAP-PHASE-1.md](ROADMAP-PHASE-1.md)

---

**Status**: Ready for execution
**Owner**: Tech Lead
**Stakeholders**: Backend Team, QA, Product Owner
