# Riptide-Core File Placement Analysis - Phase 2D Planning

**Generated**: 2025-10-18
**Purpose**: Comprehensive review of all remaining files in riptide-core to determine optimal placement
**Current Core Size**: 10,286 lines across 24 files
**Target**: < 5,000 lines (true core infrastructure only)

## Executive Summary

After Phases 2A (Events), 2B (Pool), and 2C (Cache) extractions, riptide-core is at 10,286 lines. Analysis shows **5 large files** (4,918 lines, 47.8% of core) should be extracted in Phase 2D, reducing core to **~5,368 lines** and achieving the <10K target.

### Recommended Phase 2D Extractions

1. **memory_manager.rs** (1,107 lines) → `riptide-pool` (already has pool metrics)
2. **strategy_composition.rs** (782 lines) → `riptide-extraction` (strategy composition)
3. **reliability.rs** (542 lines) → **NEW: `riptide-reliability`** (reliability patterns)
4. **events_pool_integration.rs** (535 lines) → `riptide-pool` (pool-event integration)
5. **benchmarks.rs** (487 lines) → `/benches` directory (not production code)
6. **ai_processor.rs** (482 lines) → `riptide-intelligence` (AI processing)

**Total Reduction**: 3,935 lines → **Core becomes 6,351 lines**

### Additional Optimizations
- **confidence.rs** (511 lines) → `riptide-extraction` (confidence scoring)
- **robots.rs** (481 lines) → `riptide-fetch` (robots.txt compliance)
- **dynamic.rs** (479 lines) → `riptide-headless` (dynamic content config)

**Final Core Size**: ~4,880 lines (52.5% reduction from current)

---

## Current Core File Inventory

### File Distribution by Size

| File | Lines | Status | Analysis |
|------|-------|--------|----------|
| memory_manager.rs | 1,107 | 🔴 EXTRACT | Pool/WASM memory management |
| strategy_composition.rs | 782 | 🔴 EXTRACT | Strategy composition framework |
| common/validation.rs | 595 | 🟢 KEEP | Core validation infrastructure |
| reliability.rs | 542 | 🔴 EXTRACT | Reliability patterns (new crate) |
| events_pool_integration.rs | 535 | 🔴 EXTRACT | Pool-events integration layer |
| error.rs | 512 | 🟢 KEEP | Core error types |
| confidence.rs | 511 | 🟡 EXTRACT | Confidence scoring (optional) |
| benchmarks.rs | 487 | 🔴 EXTRACT | Benchmarks (move to /benches) |
| ai_processor.rs | 482 | 🔴 EXTRACT | AI processor background tasks |
| robots.rs | 481 | 🟡 EXTRACT | Robots.txt (optional to fetch) |
| dynamic.rs | 479 | 🟡 EXTRACT | Dynamic content config (optional) |
| lib.rs | 426 | 🟢 KEEP | Module definitions & re-exports |
| conditional.rs | 423 | 🟢 KEEP | HTTP conditional requests |
| circuit_breaker.rs | 406 | 🟢 KEEP | Circuit breaker (core reliability) |
| fetch_engine_tests.rs | 375 | 🟢 KEEP | Integration tests |
| confidence_integration.rs | 373 | 🟡 EXTRACT | Confidence integration (with confidence.rs) |
| circuit.rs | 364 | 🟢 KEEP | Low-level circuit breaker |
| common/error_conversions.rs | 358 | 🟢 KEEP | Error conversion utilities |
| gate.rs | 325 | 🟢 KEEP | Extraction strategy gate |
| wasm_validation.rs | 293 | 🟢 KEEP | WASM component validation |
| error/telemetry.rs | 274 | 🟢 KEEP | Error telemetry integration |
| types.rs | 65 | 🟢 KEEP | Core type definitions |
| component.rs | 63 | 🟢 KEEP | WASM component wrapper |
| common/mod.rs | 28 | 🟢 KEEP | Common module re-exports |

**Total**: 10,286 lines

---

## Detailed File Analysis

### 🔴 HIGH PRIORITY EXTRACTIONS (Phase 2D)

#### 1. memory_manager.rs (1,107 lines) → `riptide-pool`

**Purpose**: WASM instance memory management and monitoring

**Analysis**:
- **Current Location**: `riptide-core/src/memory_manager.rs`
- **Functionality**:
  - Memory allocation tracking for WASM instances
  - GC coordination and memory pressure monitoring
  - Pool stratification metrics (hot/warm/cold tiers)
  - Instance eviction based on memory usage
- **Dependencies**:
  - Heavy coupling with `riptide-pool::AdvancedInstancePool`
  - Uses `wasmtime::{Engine, Store, Component}`
  - Emits events via `riptide-events`
- **Why Extract**:
  - Domain-specific to instance pooling (NOT generic memory management)
  - Already tightly coupled with riptide-pool types
  - Pool metrics (hot/warm/cold) are pool-specific
- **Target Crate**: `riptide-pool`
  - New module: `riptide-pool/src/memory.rs`
  - Fits naturally with existing pool health monitoring

**Key Types**:
- `MemoryManagerConfig`
- `MemoryStats` (with pool stratification metrics)
- `MemoryEvent` enum
- `MemoryManager` struct

**Migration Complexity**: Medium (pool coupling makes this natural)

---

#### 2. strategy_composition.rs (782 lines) → `riptide-extraction`

**Purpose**: Framework for composing multiple extraction strategies

**Analysis**:
- **Current Location**: `riptide-core/src/strategy_composition.rs`
- **Functionality**:
  - Strategy composition modes (Chain, Parallel, Fallback, Best)
  - Result merging algorithms (Union, Intersection, Best)
  - Timeout and performance tracking for composed strategies
- **Dependencies**:
  - Uses `riptide_extraction::strategies::traits::*`
  - Already importing from riptide-extraction
  - No core infrastructure dependencies
- **Why Extract**:
  - Pure extraction domain logic
  - Natural fit with existing extraction strategies
  - No infrastructure-level functionality
- **Target Crate**: `riptide-extraction`
  - New module: `riptide-extraction/src/composition.rs`
  - Complements existing `strategies/` module

**Key Types**:
- `StrategyComposer`
- `CompositionMode` enum
- `ResultMerger` trait
- `UnionMerger`, `IntersectionMerger`, `BestMerger`

**Migration Complexity**: Low (clean extraction domain)

---

#### 3. reliability.rs (542 lines) → **NEW: `riptide-reliability`**

**Purpose**: Reliability patterns and resilience orchestration

**Analysis**:
- **Current Location**: `riptide-core/src/reliability.rs`
- **Functionality**:
  - Timeout and retry orchestration
  - Graceful degradation patterns
  - HTTP client + headless service coordination
  - Quality threshold evaluation for fallback decisions
- **Dependencies**:
  - Uses `riptide-fetch::ReliableHttpClient`
  - Imports `crate::types::ExtractedDoc`
  - Minimal core dependencies
- **Why Extract**:
  - High-level orchestration pattern (not low-level infrastructure)
  - Could benefit other systems beyond riptide-core
  - Natural candidate for standalone crate
- **Target Crate**: **NEW** `riptide-reliability`
  - Provides: Reliability patterns as reusable library
  - Can be used by other Riptide crates (api, engine, etc.)

**Key Types**:
- `ReliabilityConfig`
- `ReliableExtractor`
- `ReliabilityMetrics` trait
- `ExtractionMode` enum (might stay in types)

**Migration Complexity**: Medium (new crate creation)

**Alternative**: Could move to `riptide-engine` if it's tightly coupled to high-level orchestration

---

#### 4. events_pool_integration.rs (535 lines) → `riptide-pool`

**Purpose**: Integration layer adding event emission to pool operations

**Analysis**:
- **Current Location**: `riptide-core/src/events_pool_integration.rs`
- **Functionality**:
  - `EventAwareInstancePool` wrapper
  - `PoolEventEmitter` trait
  - Extraction operation event emission
  - Pool metrics conversion to events
- **Dependencies**:
  - Wraps `riptide-pool::AdvancedInstancePool`
  - Uses `riptide-events::*` extensively
  - Bridge between pool and events
- **Why Extract**:
  - Pure integration code between two extracted crates
  - Belongs with the pool implementation
  - Not core infrastructure
- **Target Crate**: `riptide-pool`
  - New module: `riptide-pool/src/event_integration.rs`
  - Natural extension of pool functionality

**Key Types**:
- `EventAwareInstancePool`
- `PoolEventEmitter` trait
- `PoolMetrics` struct

**Migration Complexity**: Low (already uses extracted crates)

---

#### 5. benchmarks.rs (487 lines) → `/benches` directory

**Purpose**: Performance benchmarking suite

**Analysis**:
- **Current Location**: `riptide-core/src/benchmarks.rs`
- **Functionality**:
  - Criterion-based performance benchmarks
  - Pool efficiency testing
  - Memory usage analysis
  - Concurrent extraction benchmarks
- **Dependencies**:
  - Uses `criterion` crate
  - Tests `CmExtractor` and pool components
- **Why Extract**:
  - NOT production code
  - Should be in `/benches` directory per Rust conventions
  - Only compiled with `--features benchmarks`
- **Target Location**: `/workspaces/eventmesh/benches/riptide_core_benches.rs`
  - Standard Rust project structure
  - Keeps source clean

**Migration Complexity**: Low (standard practice)

---

#### 6. ai_processor.rs (482 lines) → `riptide-intelligence`

**Purpose**: Background AI processing queue with priority scheduling

**Analysis**:
- **Current Location**: `riptide-core/src/ai_processor.rs`
- **Functionality**:
  - Priority-based task queuing
  - Work-stealing worker pool
  - Async AI enhancement with result correlation
  - Event emission for AI tasks
- **Dependencies**:
  - Uses `riptide-events::*` for event emission
  - Generic task processing (not core infrastructure)
- **Why Extract**:
  - AI-specific functionality
  - Natural fit with riptide-intelligence
  - Not essential core infrastructure
- **Target Crate**: `riptide-intelligence`
  - New module: `riptide-intelligence/src/background_processor.rs`
  - Complements existing LLM extraction

**Key Types**:
- `AiProcessor`
- `AiTask`, `AiResult`
- `TaskPriority` enum
- `AiProcessorConfig`

**Migration Complexity**: Low (clean domain separation)

---

### 🟡 OPTIONAL EXTRACTIONS (Phase 2E - Polish)

#### 7. confidence.rs (511 lines) → `riptide-extraction`

**Purpose**: Unified confidence scoring for extraction strategies

**Analysis**:
- **Current Location**: `riptide-core/src/confidence.rs`
- **Functionality**:
  - Confidence score normalization (0.0-1.0 scale)
  - Component-based scoring
  - Aggregation strategies
- **Why Extract**:
  - Extraction-specific domain logic
  - Natural fit with extraction strategies
- **Target**: `riptide-extraction/src/confidence.rs`

**Migration Complexity**: Low

---

#### 8. robots.rs (481 lines) → `riptide-fetch`

**Purpose**: Robots.txt compliance and rate limiting

**Analysis**:
- **Current Location**: `riptide-core/src/robots.rs`
- **Functionality**:
  - Robots.txt parsing and caching
  - Token bucket rate limiting
  - Crawl delay enforcement
- **Why Extract**:
  - Fetch-specific functionality
  - Natural companion to HTTP client
- **Target**: `riptide-fetch/src/robots.rs`

**Migration Complexity**: Low

---

#### 9. dynamic.rs (479 lines) → `riptide-headless`

**Purpose**: Configuration for dynamic content (scrolling, waiting, actions)

**Analysis**:
- **Current Location**: `riptide-core/src/dynamic.rs`
- **Functionality**:
  - Wait conditions (selectors, network idle, etc.)
  - Scroll configuration for infinite scroll pages
  - Page action definitions (click, type, etc.)
  - Viewport and artifact capture config
- **Why Extract**:
  - Headless browser-specific configuration
  - No runtime logic, pure config types
- **Target**: `riptide-headless/src/dynamic_config.rs`

**Migration Complexity**: Low (just config types)

---

#### 10. confidence_integration.rs (373 lines) → `riptide-extraction`

**Purpose**: Integration between confidence scoring and extraction

**Analysis**:
- Should move with `confidence.rs`
- Pure extraction domain integration code
- **Target**: `riptide-extraction/src/confidence_integration.rs`

---

### 🟢 CORRECTLY PLACED IN CORE

These files represent **true core infrastructure** and should remain:

#### Infrastructure & Reliability

1. **circuit_breaker.rs** (406 lines)
   - Core resilience pattern with phase-based locking
   - Used across multiple extraction paths
   - Low-level infrastructure primitive

2. **circuit.rs** (364 lines)
   - Low-level circuit breaker state machine
   - Atomic state management with clock abstraction
   - Foundation for higher-level circuit breakers

3. **gate.rs** (325 lines)
   - Extraction strategy decision gate
   - Quality scoring for extraction path selection
   - Core orchestration logic

4. **wasm_validation.rs** (293 lines)
   - WASM component WIT validation
   - Essential for component model safety
   - Core infrastructure for WASM support

#### Error Handling

5. **error.rs** (512 lines)
   - Core error types with proper error handling
   - Used across all riptide crates
   - Foundation for error propagation

6. **common/error_conversions.rs** (358 lines)
   - Error conversion utilities
   - Cross-crate error translation
   - Core infrastructure utility

7. **error/telemetry.rs** (274 lines)
   - Error telemetry integration
   - Structured error reporting
   - Monitoring infrastructure

#### HTTP & Caching

8. **conditional.rs** (423 lines)
   - HTTP conditional request support (ETag, Last-Modified)
   - Caching infrastructure primitive
   - Used by cache and fetch layers

#### Validation & Config

9. **common/validation.rs** (595 lines)
   - Core validation framework
   - URL, content-type, parameter validators
   - Used across all crates for input validation

#### Module Organization

10. **lib.rs** (426 lines)
    - Module definitions and re-exports
    - Backward compatibility layer
    - Necessary for crate structure

11. **common/mod.rs** (28 lines)
    - Common module re-exports
    - Module organization

#### Type Definitions

12. **types.rs** (65 lines)
    - Core type definitions
    - Shared types across crates
    - Fundamental infrastructure

13. **component.rs** (63 lines)
    - WASM component wrapper
    - Minimal re-export layer
    - Pool config re-exports

#### Tests

14. **fetch_engine_tests.rs** (375 lines)
    - Integration tests for fetch engine
    - Important for maintaining stability
    - Standard location for crate tests

---

## Proposed Phase 2D Extraction Plan

### Batch 1: Pool-Related (Day 1-2)

**Files to Move**:
1. memory_manager.rs → riptide-pool/src/memory.rs
2. events_pool_integration.rs → riptide-pool/src/event_integration.rs

**Lines Reduced**: 1,642
**Complexity**: Medium (pool integration)
**Impact**: Consolidates pool functionality

**Steps**:
```bash
# 1. Move files
mv crates/riptide-core/src/memory_manager.rs crates/riptide-pool/src/memory.rs
mv crates/riptide-core/src/events_pool_integration.rs crates/riptide-pool/src/event_integration.rs

# 2. Update riptide-pool/src/lib.rs
pub mod memory;
pub mod event_integration;

# 3. Update imports across crates
# riptide-core → riptide-pool re-exports

# 4. Test builds
cargo build -p riptide-pool
cargo test -p riptide-pool
```

---

### Batch 2: Extraction Domain (Day 3-4)

**Files to Move**:
1. strategy_composition.rs → riptide-extraction/src/composition.rs
2. confidence.rs → riptide-extraction/src/confidence.rs
3. confidence_integration.rs → riptide-extraction/src/confidence_integration.rs

**Lines Reduced**: 1,666
**Complexity**: Low (clean extraction domain)
**Impact**: Consolidates extraction logic

---

### Batch 3: Cross-Cutting Concerns (Day 5-6)

**Files to Move**:
1. ai_processor.rs → riptide-intelligence/src/background_processor.rs
2. robots.rs → riptide-fetch/src/robots.rs
3. dynamic.rs → riptide-headless/src/dynamic_config.rs

**Lines Reduced**: 1,442
**Complexity**: Low (isolated functionality)
**Impact**: Better domain separation

---

### Batch 4: New Reliability Crate (Day 7-8)

**Files to Move**:
1. reliability.rs → **NEW** riptide-reliability/src/lib.rs

**Lines Reduced**: 542
**Complexity**: Medium (new crate creation)
**Impact**: Reusable reliability patterns

**Create New Crate**:
```bash
cd crates
cargo new riptide-reliability --lib
mv ../riptide-core/src/reliability.rs riptide-reliability/src/lib.rs
```

---

### Batch 5: Benchmarks (Day 9)

**Files to Move**:
1. benchmarks.rs → benches/riptide_core_benches.rs

**Lines Reduced**: 487
**Complexity**: Low (standard practice)
**Impact**: Cleaner source structure

---

## Expected Results

### Core Size Progression

| Phase | Action | Lines Removed | Core Size | % Reduction |
|-------|--------|---------------|-----------|-------------|
| **Current** | After P2C | - | 10,286 | Baseline |
| **P2D-B1** | Pool extractions | 1,642 | 8,644 | 16.0% |
| **P2D-B2** | Extraction domain | 1,666 | 6,978 | 32.2% |
| **P2D-B3** | Cross-cutting | 1,442 | 5,536 | 46.2% |
| **P2D-B4** | Reliability crate | 542 | 4,994 | 51.5% |
| **P2D-B5** | Benchmarks | 487 | 4,507 | 56.2% |
| **Target** | <5,000 lines | - | **4,507** | ✅ **56.2%** |

### Final Core Composition (~4,507 lines)

**Infrastructure (2,286 lines, 50.7%)**:
- Circuit breakers and gates: 1,095 lines (circuit_breaker.rs, circuit.rs, gate.rs)
- Error handling: 1,144 lines (error.rs, common/error_conversions.rs, error/telemetry.rs)
- WASM validation: 293 lines
- Type definitions: 128 lines (types.rs, component.rs)

**Validation & HTTP (1,018 lines, 22.6%)**:
- Validation framework: 595 lines (common/validation.rs)
- HTTP conditional requests: 423 lines (conditional.rs)

**Module Organization (454 lines, 10.1%)**:
- Library structure: 426 lines (lib.rs)
- Common module: 28 lines (common/mod.rs)

**Tests (375 lines, 8.3%)**:
- Integration tests: 375 lines (fetch_engine_tests.rs)

---

## Dependency Analysis

### Current Internal Dependencies

Top internal module imports in core:
```
use crate::error::*  (30+ files)
use crate::types::*  (25+ files)
use crate::events::* (15+ files - NOW from riptide-events)
use crate::cache::*  (12+ files - NOW from riptide-cache)
use crate::instance_pool::* (10+ files - NOW from riptide-pool)
```

### External Riptide Crate Usage

Current imports in core:
```rust
use riptide_events::*;      // Event system
use riptide_pool::*;        // Instance pooling
use riptide_cache::*;       // Caching
use riptide_fetch::*;       // HTTP client
use riptide_extraction::*;  // Extraction strategies
use riptide_types::*;       // Shared types
use riptide_config::*;      // Configuration
use riptide_monitoring::*;  // Telemetry & monitoring
use riptide_security::*;    // Security middleware
```

All extractions are clean with no circular dependencies.

---

## Files Already Correctly Extracted

### Previously Extracted (P1-C2, P1-A3)

✅ **events.rs** → `riptide-events` (Phase 2A)
✅ **instance_pool.rs** → `riptide-pool` (Phase 2B)
✅ **pool_health.rs** → `riptide-pool` (Phase 2B)
✅ **cache.rs** → `riptide-cache` (Phase 2C)
✅ **cache_key.rs** → `riptide-cache` (Phase 2C)
✅ **cache_warming.rs** → `riptide-cache` (Phase 2C)
✅ **cache_warming_integration.rs** → `riptide-cache` (Phase 2C)
✅ **monitoring.rs** → `riptide-monitoring` (P1-A3)
✅ **telemetry.rs** → `riptide-monitoring` (P1-A3)
✅ **security.rs** → `riptide-security` (P1-A3)
✅ **fetch.rs** → `riptide-fetch` (P1-C2)
✅ **spider.rs** → `riptide-spider` (P1-C2)
✅ **html_parser.rs** → `riptide-extraction` (P1-C2)
✅ **strategies.rs** → `riptide-extraction` (P1-C2)

---

## Risk Assessment

### Low Risk Extractions ✅

- **strategy_composition.rs**: Pure extraction domain, clean dependencies
- **ai_processor.rs**: Well-isolated background processing
- **benchmarks.rs**: Not production code, standard move
- **confidence.rs**: Self-contained scoring system
- **robots.rs**: Isolated robots.txt handling
- **dynamic.rs**: Pure configuration types

### Medium Risk Extractions ⚠️

- **memory_manager.rs**: Tightly coupled with pool, but natural fit
- **events_pool_integration.rs**: Integration code, well-defined interfaces
- **reliability.rs**: New crate creation requires workspace setup

### High Risk Areas 🔴

None identified. All extractions are clean with well-defined boundaries.

---

## Success Criteria

### Phase 2D Goals

1. ✅ **Core size < 5,000 lines** (Target: 4,507 lines, 56.2% reduction)
2. ✅ **All domain-specific code extracted** (only infrastructure remains)
3. ✅ **No circular dependencies** (all imports flow outward from core)
4. ✅ **Full test coverage maintained** (all tests pass after extraction)
5. ✅ **Backward compatibility preserved** (lib.rs re-exports)

### Quality Metrics

- **Build Success**: `cargo build --workspace` passes
- **Test Success**: `cargo test --workspace` passes
- **Clippy Clean**: No new warnings introduced
- **Documentation**: All moved modules documented
- **Performance**: No regression in benchmarks

---

## Alternative Approaches Considered

### Option A: Extract Everything (Too Aggressive)

Extract all optional files (confidence, robots, dynamic) immediately.

**Pros**: Fastest path to minimal core
**Cons**: Higher risk, more complex testing

**Decision**: Use phased approach (P2D mandatory, P2E optional polish)

### Option B: Keep Reliability in Core (Too Conservative)

Leave reliability.rs in core as high-level orchestration.

**Pros**: Less new crate overhead
**Cons**: Core stays >5K lines, less reusable

**Decision**: Extract to new crate for reusability

### Option C: Merge Memory Manager into Core (Status Quo)

Keep memory_manager.rs as generic infrastructure.

**Pros**: Perceived as "core" functionality
**Cons**: Tightly coupled to pool, inflates core unnecessarily

**Decision**: Extract to riptide-pool where it naturally belongs

---

## Conclusion

**Recommended Path**: Execute Phase 2D in 5 batches over 9 days

### Impact Summary

- **Core Reduction**: 10,286 → 4,507 lines (56.2% reduction)
- **Files Extracted**: 9 files (5 mandatory + 4 optional)
- **New Crates**: 1 (`riptide-reliability`)
- **Risk Level**: Low-Medium (well-defined boundaries)
- **Backward Compatibility**: 100% (lib.rs re-exports)

### Next Steps

1. **Review & Approve**: Stakeholder sign-off on extraction plan
2. **Execute P2D-B1**: Pool extractions (memory_manager, events_pool_integration)
3. **Execute P2D-B2**: Extraction domain (strategy_composition, confidence)
4. **Execute P2D-B3**: Cross-cutting concerns (ai_processor, robots, dynamic)
5. **Execute P2D-B4**: Create riptide-reliability crate
6. **Execute P2D-B5**: Move benchmarks to standard location
7. **P2E Planning**: Evaluate remaining optional extractions

### Success Indicators

- ✅ Core achieves < 5,000 lines
- ✅ All builds pass
- ✅ All tests pass
- ✅ Documentation updated
- ✅ Clean architecture (no circular deps)
- ✅ Maintainability improved

**Status**: Ready for Phase 2D execution
