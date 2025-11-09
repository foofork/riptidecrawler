# Sprint 4.3: Streaming Refactoring - Quick Reference

## 📋 Implementation Checklist

### Pre-Flight

- [ ] Review all planning documents
- [ ] Set up performance baseline benchmarks
- [ ] Review Phase 3.1 patterns
- [ ] Ensure test environment ready (Redis, etc.)

---

## Phase 1: Foundation (4 hours)

### 1.1 Create Domain Ports
- [ ] Create `riptide-types/src/ports/streaming.rs`
  - [ ] Define `StreamingTransport` trait
  - [ ] Define `StreamProcessor` trait  
  - [ ] Define `StreamLifecycle` trait
  - [ ] Define `StreamEvent`, `StreamState`, `StreamMetrics`
  - [ ] Add 20+ unit tests

### 1.2 Move Error Types
- [ ] Move `streaming/error.rs` → `types/src/errors/streaming.rs`
  - [ ] Update error types to use ports
  - [ ] Add ApiError conversions
  - [ ] Add recovery logic

### 1.3 Move Configuration
- [ ] Move `streaming/config.rs` → `config/src/streaming.rs`
  - [ ] Add to riptide-config exports
  - [ ] Update env loading
  - [ ] Add validation methods

### 1.4 Verify
```bash
cargo check -p riptide-types
cargo check -p riptide-config
cargo test -p riptide-types -- streaming
cargo clippy --all -- -D warnings
```

**Commit:** `feat(ports): add streaming domain ports`

---

## Phase 2: StreamingFacade (8 hours)

### 2.1 Create Facade
- [ ] Create `facade/src/facades/streaming.rs`
  - [ ] Consolidate `processor.rs` logic (634 LOC)
  - [ ] Consolidate `pipeline.rs` logic (628 LOC)
  - [ ] Consolidate `lifecycle.rs` logic (622 LOC)
  - [ ] Implement 15+ facade methods
  - [ ] Add comprehensive error handling

### 2.2 Business Methods
- [ ] `create_crawl_stream()`
- [ ] `create_deepsearch_stream()`
- [ ] `process_urls_concurrent()`
- [ ] `execute_stream()`

### 2.3 Lifecycle Methods
- [ ] `start_stream()`, `pause_stream()`, `resume_stream()`
- [ ] `cancel_stream()`, `get_stream_status()`
- [ ] `get_stream_metrics()`, `list_active_streams()`

### 2.4 Testing
- [ ] Write 50+ unit tests
  - [ ] URL processing tests
  - [ ] Stream lifecycle tests
  - [ ] Error scenario tests
  - [ ] Backpressure tests
  - [ ] Metrics collection tests

### 2.5 Integration
- [ ] Add to `facade/src/facades/mod.rs`
- [ ] Export request/response types
- [ ] Update `riptide-facade/Cargo.toml`

### 2.6 Verify
```bash
cargo check -p riptide-facade
cargo test -p riptide-facade -- streaming
cargo clippy -p riptide-facade -- -D warnings
```

**Commit:** `feat(facade): add StreamingFacade with business logic`

---

## Phase 3: Transport Adapters (6 hours)

### 3.1 WebSocket Adapter
- [ ] Create `api/src/adapters/websocket_transport.rs` (350 LOC)
  - [ ] Extract transport logic from `websocket.rs`
  - [ ] Implement `StreamingTransport` trait
  - [ ] Add connection management
  - [ ] Handle ping/pong keepalive
  - [ ] Write 10+ tests

### 3.2 SSE Adapter
- [ ] Create `api/src/adapters/sse_transport.rs` (300 LOC)
  - [ ] Extract transport logic from `sse.rs`
  - [ ] Implement `StreamingTransport` trait
  - [ ] Add SSE event formatting
  - [ ] Handle keep-alive comments
  - [ ] Write 10+ tests

### 3.3 NDJSON Adapter
- [ ] Create `api/src/adapters/ndjson_transport.rs` (250 LOC)
  - [ ] Consolidate `ndjson/` directory logic
  - [ ] Implement `StreamingTransport` trait
  - [ ] Add NDJSON formatting
  - [ ] Handle streaming buffering
  - [ ] Write 10+ tests

### 3.4 Module Setup
- [ ] Create `api/src/adapters/mod.rs`
- [ ] Export all transport adapters
- [ ] Add documentation

### 3.5 Verify
```bash
cargo check -p riptide-api
cargo test -p riptide-api -- adapters::streaming
cargo clippy -p riptide-api -- -D warnings
```

**Commit:** `feat(adapters): add streaming transport adapters`

---

## Phase 4: Infrastructure Moves (4 hours)

### 4.1 Move Buffer Manager
- [ ] Move `streaming/buffer.rs` → `reliability/src/streaming/buffer.rs` (554 LOC)
- [ ] Update imports in facade
- [ ] Add to riptide-reliability exports
- [ ] Update `riptide-reliability/Cargo.toml`
- [ ] Run tests: `cargo test -p riptide-reliability -- buffer`

### 4.2 Integrate Metrics
- [ ] Integrate `streaming/metrics.rs` → `api/src/metrics.rs` (329 LOC)
- [ ] Add streaming metrics to RipTideMetrics
- [ ] Remove duplicate metric definitions
- [ ] Update facade to use unified metrics
- [ ] Run tests: `cargo test -p riptide-api -- metrics`

### 4.3 Cross-Crate Updates
- [ ] Update riptide-facade imports
- [ ] Update riptide-api imports
- [ ] Update re-exports
- [ ] Update all Cargo.toml dependencies

### 4.4 Verify
```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

**Commit:** `refactor(infra): move streaming infrastructure`

---

## Phase 5: Handler Refactoring (3 hours)

### 5.1 Refactor handlers/streaming.rs
- [ ] Use `StreamingFacade` for business logic
- [ ] Create protocol-specific handlers:
  - [ ] `crawl_stream_ndjson()` (<50 LOC)
  - [ ] `crawl_stream_sse()` (<50 LOC)
  - [ ] `crawl_stream_websocket()` (<50 LOC)
- [ ] Remove direct pipeline orchestration
- [ ] Update route registration

### 5.2 Update handlers/pdf.rs
- [ ] Replace `response_helpers` with facade
- [ ] Use transport adapters
- [ ] Maintain existing functionality
- [ ] Update tests

### 5.3 Update handlers/mod.rs
- [ ] Remove streaming re-exports
- [ ] Export new handlers
- [ ] Update public API

### 5.4 Verify
```bash
cargo check -p riptide-api
cargo test -p riptide-api -- handlers::streaming
# Run integration tests
cargo test --test streaming_integration
```

**Commit:** `refactor(handlers): use StreamingFacade`

---

## Phase 6: Cleanup & Testing (3 hours)

### 6.1 Delete Old streaming/ Directory
- [ ] Delete `streaming/processor.rs`
- [ ] Delete `streaming/pipeline.rs`
- [ ] Delete `streaming/lifecycle.rs`
- [ ] Delete `streaming/websocket.rs`
- [ ] Delete `streaming/sse.rs`
- [ ] Delete `streaming/ndjson/` (entire directory)
- [ ] Delete `streaming/response_helpers.rs`
- [ ] Delete `streaming/buffer.rs` (moved)
- [ ] Delete `streaming/config.rs` (moved)
- [ ] Delete `streaming/error.rs` (moved)
- [ ] Delete `streaming/metrics.rs` (integrated)
- [ ] Delete `streaming/mod.rs`
- [ ] Delete `streaming/tests.rs`
- [ ] **Delete entire `streaming/` directory**

### 6.2 Test Migration
- [ ] Migrate tests to new locations:
  - [ ] Domain port tests → `riptide-types/src/ports/streaming.rs`
  - [ ] Facade tests → `riptide-facade/tests/streaming_facade_tests.rs`
  - [ ] Adapter tests → `riptide-api/tests/adapters/`
  - [ ] Integration tests → `riptide-api/tests/integration/`
- [ ] Update test imports
- [ ] Add missing test coverage

### 6.3 Documentation
- [ ] Update module documentation
- [ ] Add architecture diagrams to code
- [ ] Update API examples
- [ ] Create migration guide

### 6.4 Final Verification
```bash
# Build everything
cargo build --workspace

# Run ALL tests
cargo test --workspace

# Check for warnings
RUSTFLAGS="-D warnings" cargo clippy --workspace -- -D warnings

# Run benchmarks
cargo bench --bench streaming_throughput
cargo bench --bench streaming_latency
cargo bench --bench streaming_memory
```

### 6.5 Quality Gates
- [ ] ✅ 0 compiler warnings
- [ ] ✅ 0 clippy warnings
- [ ] ✅ All 200+ tests pass
- [ ] ✅ Test coverage > 80%
- [ ] ✅ Documentation complete
- [ ] ✅ Performance within 5% of baseline

**Commit:** `refactor(streaming): complete hexagonal architecture migration`

---

## Post-Migration

### Deployment Checklist
- [ ] Review all changes
- [ ] Run full test suite on CI
- [ ] Run performance benchmarks
- [ ] Compare metrics with baseline
- [ ] Update deployment documentation
- [ ] Create rollback plan
- [ ] Deploy to staging
- [ ] Monitor staging metrics (24 hours)
- [ ] Deploy to production (canary)
- [ ] Monitor production metrics (48 hours)
- [ ] Full production rollout

### Monitoring
- [ ] Set up alerts for:
  - [ ] P99 latency > 200ms
  - [ ] Error rate > 5%
  - [ ] Memory usage > 2x baseline
  - [ ] Connection failures > 1%

---

## Quick Commands

### Build & Test
```bash
# Phase-by-phase verification
cargo check -p riptide-types && \
cargo check -p riptide-config && \
cargo check -p riptide-facade && \
cargo check -p riptide-api

# Full workspace test
cargo test --workspace

# Clippy all
cargo clippy --workspace -- -D warnings

# Build with warnings as errors
RUSTFLAGS="-D warnings" cargo build --workspace
```

### Performance Benchmarks
```bash
# Establish baseline (before migration)
cargo bench --bench streaming_throughput > baseline_throughput.txt
cargo bench --bench streaming_latency > baseline_latency.txt
cargo bench --bench streaming_memory > baseline_memory.txt

# Compare after migration
cargo bench --bench streaming_throughput
cargo bench --bench streaming_latency
cargo bench --bench streaming_memory
```

### Git Workflow
```bash
# Create feature branch
git checkout -b feat/sprint-4.3-streaming-refactoring

# Commit after each phase
git add .
git commit -m "feat(ports): add streaming domain ports"
git commit -m "feat(facade): add StreamingFacade with business logic"
git commit -m "feat(adapters): add streaming transport adapters"
git commit -m "refactor(infra): move streaming infrastructure"
git commit -m "refactor(handlers): use StreamingFacade"
git commit -m "refactor(streaming): complete hexagonal architecture migration"

# Push and create PR
git push origin feat/sprint-4.3-streaming-refactoring
gh pr create --title "Sprint 4.3: Streaming System Refactoring" --body "$(cat docs/execution/SPRINT_4.3_SUMMARY.md)"
```

---

## Rollback Strategy

### If Issues Occur:

**Immediate Rollback Triggers:**
- P99 latency > 200ms for 5 minutes
- Error rate > 5% for 2 minutes
- Memory usage > 2x baseline
- Connection failures > 1%

**Rollback Steps:**
```bash
# 1. Identify last known good commit
git log --oneline

# 2. Revert to previous version
git revert HEAD~6..HEAD  # Revert all 6 phase commits

# OR hard reset (if not pushed)
git reset --hard <last-good-commit>

# 3. Redeploy
cargo build --release
./deploy.sh

# 4. Monitor metrics
watch -n 5 'curl http://localhost:8080/metrics | grep streaming'

# 5. Investigate root cause
# 6. Fix and re-deploy
```

---

## File Count Summary

| Phase | Files Created | Files Moved | Files Deleted | Net Change |
|-------|---------------|-------------|---------------|------------|
| 1 | 1 | 2 | 0 | -1 |
| 2 | 1 | 0 | 0 | +1 |
| 3 | 4 | 0 | 0 | +4 |
| 4 | 0 | 1 | 0 | 0 |
| 5 | 0 | 0 | 0 | 0 |
| 6 | 0 | 0 | 15 | -15 |
| **Total** | **6** | **3** | **15** | **-6** |

**LOC Summary:**
- Before: 7,986 LOC (15 files)
- After: ~3,500 LOC (9 files)
- Reduction: 4,486 LOC (56%)

---

## Success Criteria

### ✅ Functional
- All streaming protocols work (NDJSON, SSE, WebSocket)
- Backpressure handling maintains stability
- Metrics collection is comprehensive
- Error handling is robust
- Lifecycle events are tracked

### ✅ Performance
- P99 latency < 100ms
- Throughput > 10,000 msg/sec
- Memory per connection < 5MB
- Max concurrent connections > 1,000
- Performance within 5% of baseline

### ✅ Code Quality
- 0 clippy warnings
- 0 compiler warnings
- Test coverage > 80%
- Documentation coverage > 90%
- All handlers < 50 LOC

### ✅ Architecture
- Clean hexagonal architecture
- No circular dependencies
- Proper separation of concerns
- Protocol-agnostic business logic
- Infrastructure in correct layers

---

## Resources

- **Full Plan:** `docs/execution/SPRINT_4.3_STREAMING_PLAN.md` (40KB)
- **Summary:** `docs/execution/SPRINT_4.3_SUMMARY.md` (6KB)
- **Architecture:** `docs/execution/SPRINT_4.3_ARCHITECTURE_DIAGRAM.md` (28KB)
- **This Checklist:** `docs/execution/SPRINT_4.3_QUICK_REFERENCE.md`

**Total Documentation:** 74KB across 4 files

---

**Ready to implement?** Start with Phase 1! 🚀
