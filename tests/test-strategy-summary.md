# Test Strategy Summary - WASM-First Enhancements

**Agent:** Tester (Hive Mind Swarm)
**Date:** 2025-10-23
**Status:** ✅ Complete

---

## Quick Stats

- **Total Tests:** 147
  - Unit: 68
  - Integration: 42
  - Performance: 21
  - Chaos/Edge: 16
- **Execution Time:** ~27 minutes (full suite) | ~12 minutes (parallel)
- **Coverage Target:** 85% line coverage
- **Test Files:** 8 new test files + fixtures

---

## Test Breakdown by Feature

### 1. JSON-LD Short-Circuit (34 tests)
**Purpose:** Validate early-return optimization for complete Event/Article schemas

**Key Scenarios:**
- Complete vs incomplete Event schemas (9 tests)
- Complete vs incomplete Article schemas (9 tests)
- Integration with full extraction pipeline (8 tests)
- Performance: 80% time reduction validation (4 tests)
- Chaos: Malformed JSON, OOM, Unicode handling (4 tests)

**Critical Tests:**
- `test_complete_event_with_all_fields()` - Full event extraction
- `test_incomplete_event_missing_location()` - Fallback to full extraction
- `bench_jsonld_vs_full_extraction()` - 80% speedup validation

---

### 2. Probe-First Escalation (33 tests)
**Purpose:** Validate SPA detection → WASM probe → selective headless escalation

**Key Scenarios:**
- SPA framework detection (React, Vue, Angular, Next.js) (5 tests)
- Probe outcome handling (success, network needed, no growth, blocked) (9 tests)
- Real-world SPA integration tests (10 tests)
- Performance: 60-80% time savings validation (5 tests)
- Chaos: WASM OOM, infinite loops, DOM explosion (4 tests)

**Critical Tests:**
- `test_probe_success_no_network()` - Happy path
- `test_probe_success_network_needed()` - Trigger micro-network
- `test_escalation_chain()` - Full SPA → probe → network → headless
- `bench_probe_vs_direct_headless()` - Time savings validation

---

### 3. WASM Micro-Network (29 tests)
**Purpose:** Validate budgeted same-origin fetching within WASM

**Key Scenarios:**
- Budget enforcement (max requests, max bytes) (6 tests)
- Same-origin security checks (6 tests)
- Real API fetch scenarios (8 tests)
- Performance: 40-60% faster than headless (5 tests)
- Chaos: Request loops, timeouts, large responses (4 tests)

**Critical Tests:**
- `test_network_budget_max_requests()` - Request cap enforcement
- `test_same_origin_allowed()` - Origin validation
- `test_nextjs_api_route_fetch()` - Real Next.js API integration
- `test_budget_exceeded_escalation()` - Graceful escalation

---

### 4. Enhanced Content Signals (28 tests)
**Purpose:** Validate improved SPA detection via text density, placeholders, framework markers

**Key Scenarios:**
- Text density calculation (strip scripts, handle hidden elements) (6 tests)
- Placeholder detection (skeleton, shimmer, loading states) (5 tests)
- SPA framework detection (Next, Nuxt, Vite, Angular) (5 tests)
- Signal score thresholds (RawOK, WASM, Headless) (6 tests)
- Performance: <30ms total signal computation (4 tests)
- Chaos: All-whitespace, false positives (2 tests)

**Critical Tests:**
- `test_text_density_high_content()` - Static content detection
- `test_text_density_low_spa()` - SPA shell detection
- `test_placeholder_skeleton_classes()` - Loading state detection
- `test_signal_score_threshold_wasm()` - Correct engine selection

---

### 5. Domain Warm-Start (23 tests)
**Purpose:** Validate Redis-backed engine preference caching

**Key Scenarios:**
- Profile lifecycle (create, update, TTL expiry) (8 tests)
- Cache hit/miss behavior (10 tests)
- Performance: 50-100ms time savings (3 tests)
- Chaos: Redis unavailable, corrupted data (2 tests)

**Critical Tests:**
- `test_warm_start_cache_hit()` - Second extraction uses cached engine
- `test_warm_start_ttl_expiry()` - TTL enforcement
- `test_warm_start_stale_on_failure()` - Invalidate on failure
- `test_warm_start_redis_persistence()` - Cross-restart persistence

---

## Test Data Requirements

### Fixtures (50+ files)
**Location:** `/workspaces/eventmesh/tests/fixtures/`

1. **HTML Samples** (`html/`) - 30 files
   - Event/Article pages with complete/incomplete JSON-LD
   - SPA variations (React, Vue, Angular, Next.js)
   - Static sites, Cloudflare challenges, loading states
   - Malformed/huge JSON-LD edge cases

2. **API Mocks** (`api/`) - 20 files
   - Event lists, user profiles, settings
   - Large responses (500KB), chunked transfers
   - Various JSON structures for SPA data fetching

3. **Expected Results** (`expected/`) - 30 files
   - Ground truth extraction results for each HTML fixture
   - Used for assertion in integration tests

4. **Baselines** (`benchmarks/baseline.json`)
   - Performance baselines for regression detection
   - Updated quarterly

---

## Execution Plan

### Local Development
```bash
# Fast feedback loop (< 1 min)
cargo test --lib

# Before commit (< 10 min)
cargo test --test '*_integration_test'

# Performance check (weekly)
cargo bench

# Full suite (before PR)
cargo test --all
```

### CI/CD Pipeline
```yaml
1. Fast Feedback (2 min)  → Unit tests
2. Integration (10 min)   → Real scenarios
3. Performance (15 min)   → Benchmarks + regression check
4. Chaos (5 min)          → Edge cases
5. E2E (20 min)           → Full system validation
```

**Total CI time:** ~52 minutes sequential | ~20 minutes parallel

---

## Success Criteria

### Functional ✅
- JSON-LD short-circuit: >80% time reduction
- Probe-first: 60-80% savings vs old direct-headless
- Micro-network: 90% of API SPAs avoid headless
- Signal accuracy: <5% false positive rate
- Warm-start: >70% cache hit rate

### Performance ✅
- No benchmark regression >10%
- Memory growth <50MB per extraction
- Linear scaling to 10x load

### Reliability ✅
- 0 panics in chaos tests
- 100% graceful error handling
- Full telemetry coverage

---

## Implementation Priority

### Phase 1: Core Unit Tests (Week 1)
- JSON-LD parsing and validation
- Probe outcome handling
- Network budget enforcement
- Signal computation logic
- Domain profile lifecycle

**Deliverable:** 68 unit tests passing

### Phase 2: Integration Tests (Week 2)
- Real HTML fixture extraction
- Engine escalation flows
- API mock integrations
- Cache hit/miss scenarios

**Deliverable:** 42 integration tests passing

### Phase 3: Performance & Chaos (Week 3)
- Benchmark suite setup
- Baseline establishment
- Chaos scenario validation
- Regression detection

**Deliverable:** 37 perf + chaos tests passing

### Phase 4: CI/CD Integration (Week 4)
- GitHub Actions workflow
- Automated regression checks
- Coverage reporting
- Flaky test monitoring

**Deliverable:** Full CI pipeline operational

---

## Key Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Fixture staleness | False negatives | Quarterly refresh with real-world samples |
| Flaky integration tests | CI instability | Retry logic + 1-week fix SLA |
| Performance baseline drift | False regressions | Weekly baseline updates |
| Test data bloat | Slow CI | Fixture compression + lazy loading |
| Incomplete coverage | Bugs in prod | 85% coverage gate + mandatory edge cases |

---

## Next Steps

1. **Review this strategy** with architect and coder agents
2. **Generate fixtures** (30 HTML samples from real sites)
3. **Implement unit tests** (Phase 1: 68 tests)
4. **Set up benchmark harness** (criterion.rs integration)
5. **Create CI workflow** (GitHub Actions template)

---

## Test File Structure

```
tests/
├── unit/
│   ├── jsonld_shortcircuit_test.rs      (18 tests)
│   ├── probe_escalation_test.rs         (14 tests)
│   ├── wasm_micronetwork_test.rs        (12 tests)
│   ├── content_signals_test.rs          (16 tests)
│   └── domain_warmstart_test.rs         (8 tests)
├── integration/
│   ├── jsonld_extraction_test.rs        (8 tests)
│   ├── probe_escalation_test.rs         (10 tests)
│   ├── wasm_micronetwork_test.rs        (8 tests)
│   ├── content_signals_test.rs          (6 tests)
│   └── domain_warmstart_test.rs         (10 tests)
├── benchmarks/
│   ├── enhancement_perf_test.rs         (21 benchmarks)
│   └── baseline.json
├── chaos/
│   └── edge_cases_test.rs               (16 tests)
├── fixtures/
│   ├── html/                            (30 files)
│   ├── api/                             (20 files)
│   └── expected/                        (30 files)
└── enhancement-test-strategy.md         (this document)
```

---

## Appendix: Command Reference

### Test Execution
```bash
# Run specific test file
cargo test --test jsonld_extraction_test

# Run tests matching pattern
cargo test probe_first

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench

# Generate coverage report
cargo tarpaulin --out Html
```

### Fixture Management
```bash
# Generate fixture from live URL
cargo run --bin fixture-gen -- https://example.com/event

# Validate fixture completeness
cargo run --bin fixture-check -- tests/fixtures/html/

# Update expected results
cargo test --test '*_test' -- --update-golden
```

### Performance Analysis
```bash
# Compare against baseline
cargo bench -- --save-baseline current
cargo bench -- --baseline current

# Profile specific benchmark
cargo flamegraph --bench enhancement_perf_test -- probe_success
```

---

**Test strategy stored in memory:** `hive/tester/plan`
**Full documentation:** `/workspaces/eventmesh/tests/enhancement-test-strategy.md`
