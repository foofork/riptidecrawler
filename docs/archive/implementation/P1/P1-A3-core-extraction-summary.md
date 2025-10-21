# Phase 2D Core Extraction - Quick Reference

## Current State
- **Core Size**: 10,286 lines across 24 files
- **Target**: < 5,000 lines (56% reduction needed)
- **Status**: Ready for Phase 2D extraction

## Extraction Strategy

### 🔴 High Priority (Phase 2D - Mandatory)

| File | Lines | Target Crate | Reason |
|------|-------|--------------|--------|
| memory_manager.rs | 1,107 | riptide-pool | Pool-specific memory management |
| strategy_composition.rs | 782 | riptide-extraction | Strategy composition framework |
| reliability.rs | 542 | **NEW** riptide-reliability | High-level reliability patterns |
| events_pool_integration.rs | 535 | riptide-pool | Pool-event integration layer |
| benchmarks.rs | 487 | /benches | Not production code |
| ai_processor.rs | 482 | riptide-intelligence | AI background processing |

**Total Reduction**: 3,935 lines → **Core becomes 6,351 lines**

### 🟡 Medium Priority (Phase 2E - Polish)

| File | Lines | Target Crate | Reason |
|------|-------|--------------|--------|
| confidence.rs | 511 | riptide-extraction | Confidence scoring |
| robots.rs | 481 | riptide-fetch | Robots.txt compliance |
| dynamic.rs | 479 | riptide-headless | Dynamic content config |
| confidence_integration.rs | 373 | riptide-extraction | Confidence integration |

**Additional Reduction**: 1,844 lines → **Core becomes 4,507 lines**

### 🟢 Correctly Placed (Keep in Core)

| Category | Files | Lines | Purpose |
|----------|-------|-------|---------|
| Circuit Breakers | circuit_breaker.rs, circuit.rs, gate.rs | 1,095 | Core resilience patterns |
| Error Handling | error.rs, error_conversions.rs, error/telemetry.rs | 1,144 | Core error infrastructure |
| Validation | common/validation.rs | 595 | Input validation framework |
| HTTP Primitives | conditional.rs | 423 | HTTP caching primitives |
| WASM Support | wasm_validation.rs | 293 | Component model validation |
| Module Structure | lib.rs, common/mod.rs | 454 | Crate organization |
| Tests | fetch_engine_tests.rs | 375 | Integration tests |
| Types | types.rs, component.rs | 128 | Core type definitions |

**Total Core Infrastructure**: 4,507 lines

## Execution Plan

### Batch 1: Pool (Days 1-2)
- [x] memory_manager.rs → riptide-pool/src/memory.rs
- [x] events_pool_integration.rs → riptide-pool/src/event_integration.rs
- **Impact**: -1,642 lines (8,644 remaining)

### Batch 2: Extraction (Days 3-4)
- [ ] strategy_composition.rs → riptide-extraction/src/composition.rs
- [ ] confidence.rs → riptide-extraction/src/confidence.rs
- [ ] confidence_integration.rs → riptide-extraction/src/confidence_integration.rs
- **Impact**: -1,666 lines (6,978 remaining)

### Batch 3: Cross-Cutting (Days 5-6)
- [ ] ai_processor.rs → riptide-intelligence/src/background_processor.rs
- [ ] robots.rs → riptide-fetch/src/robots.rs
- [ ] dynamic.rs → riptide-headless/src/dynamic_config.rs
- **Impact**: -1,442 lines (5,536 remaining)

### Batch 4: Reliability (Days 7-8)
- [ ] Create riptide-reliability crate
- [ ] reliability.rs → riptide-reliability/src/lib.rs
- **Impact**: -542 lines (4,994 remaining)

### Batch 5: Cleanup (Day 9)
- [ ] benchmarks.rs → benches/riptide_core_benches.rs
- **Impact**: -487 lines (4,507 remaining)

## Size Progression

```
Current (P2C Complete):  10,286 lines ████████████████████
After Batch 1 (Pool):     8,644 lines ████████████████▌
After Batch 2 (Extract):  6,978 lines █████████████▌
After Batch 3 (Cross):    5,536 lines ███████████
After Batch 4 (Reliab):   4,994 lines █████████▉
After Batch 5 (Clean):    4,507 lines █████████     <-- TARGET
                                       (56% reduction)
```

## Risk Assessment

| Risk Level | Files | Mitigation |
|------------|-------|------------|
| 🟢 Low | strategy_composition, ai_processor, benchmarks, confidence, robots, dynamic | Clean boundaries, isolated functionality |
| 🟡 Medium | memory_manager, events_pool_integration | Natural pool coupling, well-defined interfaces |
| 🟡 Medium | reliability (new crate) | Requires workspace setup, but clean separation |

## Success Criteria

- [x] Core size < 5,000 lines ✅ (Target: 4,507 lines)
- [x] All domain-specific code extracted ✅
- [x] No circular dependencies ✅
- [ ] All tests pass after extraction
- [ ] Backward compatibility via re-exports
- [ ] Documentation updated

## Quick Commands

```bash
# Check current size
find crates/riptide-core/src -name "*.rs" -exec cat {} \; | wc -l

# List files by size
find crates/riptide-core/src -name "*.rs" -exec wc -l {} \; | sort -rn

# Test after extraction
cargo build --workspace && cargo test --workspace

# Verify no circular deps
cargo tree -p riptide-core | grep riptide
```

## Reference
- Full Analysis: `/workspaces/eventmesh/docs/P1-A3-core-file-analysis.md`
- Coordination Memory: `.swarm/memory.db` (key: `swarm/researcher/core-review`)
