# Phase 4 Execution Plan: Infrastructure Consolidation
**Created:** 2025-11-08
**Phase Duration:** 2 weeks (7 sprints)
**Status:** Ready for Execution

---

## Execution Strategy

### Parallel Execution Groups

**Group 1: Independent (Days 1-2) - PARALLEL**
- Sprint 4.1: HTTP Client Consolidation ✅ Can run in parallel
- Sprint 4.6: Browser Crate Consolidation ✅ Can run in parallel
- Sprint 4.7: Pool Abstraction Unification ✅ Can run in parallel
- Sprint 4.2: Redis Consolidation (Validation only) ✅ Can run in parallel

**Group 2: Dependencies (Days 3-7) - SEQUENTIAL**
- Sprint 4.3: Streaming System Refactoring (4 days) - Depends on ports being defined
- Sprint 4.4: Resource Manager Consolidation (2 days) - Can overlap with streaming
- Sprint 4.5: Metrics System Split (1 day) - Can run after 4.4

### Agent Assignments (Group 1 - Parallel)

**Agent 1: HTTP Client Consolidation**
- Replace all reqwest::Client with ReliableHttpClient
- Files: riptide-fetch, riptide-spider, riptide-pdf, riptide-browser, riptide-search
- Add CircuitBreakerPreset implementations
- Duration: 2 days
- Success: Zero direct reqwest usage outside riptide-reliability

**Agent 2: Browser Crate Consolidation**
- Merge riptide-browser-abstraction + riptide-browser + riptide-headless
- Create modules: abstraction/, pool/, cdp/, http/
- Remove concrete CDP types from abstraction
- Duration: 1 day
- Success: Single riptide-browser crate, clean trait abstraction

**Agent 3: Pool Abstraction Unification**
- Define Pool<T> trait in riptide-types/src/ports/pool.rs
- Implement for BrowserPool, LlmClientPool, GenericPool
- Extract common logic
- Duration: 0.5 days
- Success: All pools implement Pool<T> trait

**Agent 4: Redis Consolidation Validation**
- Verify only 2 crates have Redis dependencies
- Validate CacheStorage usage in facades
- Check versioned key patterns
- Duration: 0.5 days
- Success: Redis in ≤2 crates, no direct redis:: in facades

### Agent Assignments (Group 2 - Sequential)

**Agent 5: Streaming System Refactoring (Lead)**
- Create StreamingTransport, StreamProcessor, StreamLifecycle ports
- Create StreamingFacade (consolidates processor, pipeline, lifecycle)
- Create WebSocketTransport and SseTransport adapters
- Move buffer.rs to riptide-reliability
- Move config.rs to riptide-config
- Delete streaming/ directory from API
- Duration: 4 days
- Success: Zero LOC in crates/riptide-api/src/streaming/

**Agent 6: Resource Manager Consolidation**
- Create ResourceFacade
- Define RateLimiter port
- Implement RedisRateLimiter adapter
- Move business logic from mod.rs (653 LOC)
- Duration: 2 days
- Success: resource_manager/ <500 LOC (only RAII guards remain)

**Agent 7: Metrics System Split**
- Create BusinessMetrics in riptide-facade/src/metrics/business.rs
- Keep TransportMetrics in riptide-api/src/metrics.rs
- Update all facades to use BusinessMetrics
- Duration: 1 day
- Success: metrics.rs <600 LOC, business metrics in facade

---

## Quality Gates (Per Sprint)

**MANDATORY - Zero Tolerance:**
```bash
# 1. Tests pass (NO ignored tests)
cargo test -p [affected-crate]

# 2. Clippy clean (ZERO warnings)
cargo clippy -p [affected-crate] -- -D warnings

# 3. Cargo check passes
cargo check -p [affected-crate]

# 4. Architecture validation
./scripts/validate_architecture.sh  # After creating in Sprint 5.1
```

**Per Sprint Validation:**
- Sprint 4.1: `rg "reqwest::Client" crates/riptide-{fetch,spider,pdf,browser,search}` → No matches
- Sprint 4.2: `find crates -name "Cargo.toml" -exec grep -l redis {} \; | wc -l` → ≤2
- Sprint 4.3: `[ ! -d crates/riptide-api/src/streaming ]` → True
- Sprint 4.4: `wc -l crates/riptide-api/src/resource_manager/*.rs` → <500 total
- Sprint 4.5: `wc -l crates/riptide-api/src/metrics.rs` → <600
- Sprint 4.6: `find crates -name "*browser*" -type d | wc -l` → 1
- Sprint 4.7: `rg "impl.*Pool<" crates/` → ≥3 implementations

---

## Rollback Strategy

**If Sprint Fails Quality Gates:**
1. DO NOT PROCEED to next sprint
2. Fix all errors/warnings in current sprint
3. Re-run quality gates
4. Only proceed when 100% passing

**Disk Space Management:**
```bash
# Before starting Phase 4
df -h / | head -2  # Must have >15GB free

# Clean if needed
cargo clean
```

**Targeted Builds (Save Disk):**
```bash
# Use per-crate builds during development
cargo build -p riptide-reliability  # Not --workspace
cargo test -p riptide-facade         # Not --workspace
```

**Full Workspace Build:**
```bash
# ONLY at end of Phase 4 for final validation
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## Success Criteria (Phase 4 Complete)

### Quantitative
- [ ] HTTP clients: 6 → 1 (all via ReliableHttpClient)
- [ ] Redis dependencies: 6 → ≤2
- [ ] streaming/ LOC in API: 5,427 → 0
- [ ] resource_manager/ LOC: 2,832 → <500
- [ ] metrics.rs LOC: 1,670 → <600
- [ ] Browser crates: 3 → 1
- [ ] Pool<T> implementations: ≥3

### Qualitative
- [ ] All tests passing (zero ignored)
- [ ] Zero clippy warnings
- [ ] Zero compilation errors
- [ ] Circuit breakers configured per endpoint type
- [ ] Streaming uses ports/adapters pattern
- [ ] Business metrics separated from transport metrics
- [ ] Browser abstraction has no concrete CDP types

### Final Validation
```bash
# Run complete validation
./scripts/validate_phase4.sh  # From roadmap

# All quality gates
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo check --workspace
```

---

## Timeline

**Week 7 (Days 1-5):**
- Days 1-2: Group 1 (parallel) - HTTP, Browser, Pool, Redis validation
- Day 3-5: Sprint 4.3 Part 1 - Streaming (ports, facade start)

**Week 8 (Days 1-5):**
- Days 1-3: Sprint 4.3 Part 2 - Streaming (complete migration, delete old code)
- Days 4-5: Sprint 4.4 - Resource Manager consolidation
- Day 5 afternoon: Sprint 4.5 - Metrics split

**Total:** 10 working days

---

## Commit Strategy

**Commit After Each Sprint:**
- Sprint 4.1 complete → Commit "feat(infra): consolidate HTTP clients to ReliableHttpClient"
- Sprint 4.2 complete → Commit "feat(infra): validate Redis consolidation"
- Sprint 4.3 complete → Commit "feat(infra): refactor streaming system to ports/adapters"
- Sprint 4.4 complete → Commit "feat(infra): consolidate resource manager to facades"
- Sprint 4.5 complete → Commit "feat(infra): split business and transport metrics"
- Sprint 4.6 complete → Commit "feat(infra): consolidate browser crates"
- Sprint 4.7 complete → Commit "feat(infra): unify pool abstractions"

**OR**

**Single Commit for Entire Phase 4:**
After all 7 sprints pass quality gates:
```
feat: Complete Phase 4 - Infrastructure Consolidation

- Sprint 4.1: Consolidate HTTP clients to ReliableHttpClient
- Sprint 4.2: Validate Redis consolidation (≤2 crates)
- Sprint 4.3: Refactor streaming system (5,427 LOC → facades)
- Sprint 4.4: Consolidate resource manager (2,832 → 500 LOC)
- Sprint 4.5: Split business/transport metrics (1,670 → 600 LOC)
- Sprint 4.6: Merge browser crates (3 → 1)
- Sprint 4.7: Unify pool abstractions with Pool<T> trait

Quality Gates:
- 210 tests passing, 0 failed, 5 ignored
- Zero clippy warnings
- Zero compilation errors
- All architecture validations passing

LOC Impact:
- 6,370 LOC deleted (infrastructure violations)
- 4,000 LOC added (clean ports/adapters)
- Net: -2,370 LOC reduction

🤖 Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>
```

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Streaming refactoring breaks WebSocket/SSE | Comprehensive integration tests before deleting old code |
| Circuit breakers too aggressive | Tune thresholds per CircuitBreakerPreset |
| Browser consolidation causes import breakage | Update all imports in single commit |
| Disk space during builds | Targeted builds, cargo clean, monitor df -h |

---

## Next Steps

1. ✅ Read Phase 4 and Phase 5 roadmaps (DONE)
2. ✅ Create execution plan (THIS DOCUMENT)
3. ⏭️ Spawn Group 1 agents (parallel execution)
4. ⏭️ Monitor agent progress
5. ⏭️ Validate Group 1 quality gates
6. ⏭️ Spawn Group 2 agents (sequential execution)
7. ⏭️ Final Phase 4 validation
8. ⏭️ Commit Phase 4 completion

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)
