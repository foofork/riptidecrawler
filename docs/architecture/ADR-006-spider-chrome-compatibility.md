# ADR-006: Spider-Chrome API Compatibility Strategy

## Status
**Proposed** - Week 3 Day 1

## Context

### The Problem
Spider-chrome v2.37.128 is a fork of chromiumoxide 0.7.0 that uses incompatible type packages:
- **spider_chromiumoxide_cdp v0.7.4** vs chromiumoxide_cdp v0.7.0
- **spider_chromiumoxide_types v0.7.4** vs chromiumoxide_types v0.7.0

This creates type-level incompatibility despite API surface similarity. We cannot:
- Pass spider Page to chromiumoxide handlers
- Convert CDP events between versions
- Share connection handlers or pools
- Use spider Browser in chromiumoxide-based pools

### Requirements
1. **Support hybrid fallback** - Must enable 20% spider-chrome traffic with chromiumoxide fallback
2. **Resolve type conflicts** - Bridge the spider_chromiumoxide_cdp ↔ chromiumoxide_cdp gap
3. **Preserve stealth** - Maintain spider_chrome's fingerprinting countermeasures
4. **Minimize overhead** - <5% runtime performance cost
5. **Maintainable** - Easy to update when either library changes
6. **Testable** - Can test each engine independently

### Current State
- 20 files directly use `chromiumoxide::{Browser, Page, BrowserConfig}`
- Hybrid fallback in `crates/riptide-headless/src/hybrid_fallback.rs` blocked by type incompatibility
- Cannot dynamically switch between engines at runtime

## Decision

**We will use the "Trait Abstraction + Dynamic Dispatch" pattern.**

Create a new crate **`riptide-browser-abstraction`** that defines common traits and provides implementations for both chromiumoxide and spider_chrome through dynamic dispatch.

### Architecture

```rust
// New crate: crates/riptide-browser-abstraction

/// Common browser interface
#[async_trait]
pub trait BrowserEngine: Send + Sync {
    async fn new_page(&self, url: &str) -> Result<Box<dyn PageHandle>>;
    async fn close(&self) -> Result<()>;
    fn engine_type(&self) -> EngineType;
}

/// Common page interface
#[async_trait]
pub trait PageHandle: Send + Sync {
    async fn goto(&self, url: &str) -> Result<()>;
    async fn content(&self) -> Result<String>;
    async fn screenshot(&self, params: ScreenshotParams) -> Result<Vec<u8>>;
    async fn pdf(&self, params: PdfParams) -> Result<Vec<u8>>;
    async fn wait_for_navigation(&self) -> Result<()>;
    async fn evaluate(&self, script: &str) -> Result<serde_json::Value>;
    async fn close(&self) -> Result<()>;
    fn engine_type(&self) -> EngineType;
}

/// Engine type indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineType {
    SpiderChrome,
    Chromiumoxide,
}

/// Unified screenshot parameters (translates to both engines)
pub struct ScreenshotParams {
    pub format: ImageFormat,
    pub quality: Option<i64>,
    pub full_page: bool,
    pub clip: Option<Viewport>,
}

/// Implementations
pub struct SpiderChromeEngine {
    inner: spider_chrome::Browser,
}

pub struct ChromiumoxideEngine {
    inner: chromiumoxide::Browser,
}

pub struct SpiderChromePage {
    inner: spider_chrome::Page,
}

pub struct ChromiumoxidePage {
    inner: chromiumoxide::Page,
}
```

### Implementation Strategy

#### Phase 1: Core Abstraction (Day 2 - 4 hours)
1. Create `riptide-browser-abstraction` crate
2. Define `BrowserEngine` and `PageHandle` traits
3. Define common parameter types (ScreenshotParams, PdfParams, etc.)
4. Add `EngineType` enum

#### Phase 2: Chromiumoxide Wrapper (Day 2 - 2 hours)
1. Implement `ChromiumoxideEngine: BrowserEngine`
2. Implement `ChromiumoxidePage: PageHandle`
3. Add parameter translation (abstraction → chromiumoxide types)
4. Write unit tests

#### Phase 3: Spider-Chrome Wrapper (Day 2 - 2 hours)
1. Implement `SpiderChromeEngine: BrowserEngine`
2. Implement `SpiderChromePage: PageHandle`
3. Add parameter translation (abstraction → spider_chrome types)
4. Preserve stealth capabilities
5. Write unit tests

#### Phase 4: Integration (Day 3 - 2 hours)
1. Update `hybrid_fallback.rs` to use `Box<dyn BrowserEngine>`
2. Update launcher to return `Box<dyn BrowserEngine>`
3. Update pools to accept `Box<dyn PageHandle>`
4. Run integration tests

## Consequences

### Positive

1. **Runtime Engine Selection** ✅
   - Can dynamically choose engine at runtime
   - Perfect for 20% hybrid fallback strategy
   - No compile-time feature flag complexity

2. **Type Safety** ✅
   - All incompatibilities hidden behind trait boundary
   - Compiler enforces common interface
   - Safe to mix both engines in same binary

3. **Maintainability** ✅
   - Clear separation between engines
   - Easy to add new engines later
   - Update isolation (changes to one don't affect other)

4. **Testability** ✅
   - Can mock `BrowserEngine` trait for testing
   - Test each implementation independently
   - Integration tests work with both engines

5. **Minimal Changes** ✅
   - Most existing code unchanged (still uses methods)
   - Only launcher/factory code changes
   - Gradual migration path

6. **Preserves Features** ✅
   - Spider-chrome stealth retained
   - Chromiumoxide stability retained
   - Each engine's unique features accessible via trait extensions

### Negative

1. **Dynamic Dispatch Overhead** ⚠️
   - Virtual function call cost (~1-3ns per call)
   - Heap allocation for trait objects
   - **Mitigation:** Negligible compared to network/browser latency
   - **Impact:** <0.01% of total request time

2. **Trait Object Limitations** ⚠️
   - Cannot use generics (except with where clauses)
   - No sized types in return positions (must use Box)
   - **Mitigation:** Use associated types and Box smartly

3. **Parameter Translation** ⚠️
   - Need to convert abstraction params → engine params
   - Small amount of boilerplate per method
   - **Mitigation:** Use From/Into traits, keep params simple

4. **No Direct CDP Access** ⚠️
   - Cannot access underlying CDP commands directly
   - Must add methods to trait for special cases
   - **Mitigation:** Add `as_spider_chrome()` / `as_chromiumoxide()` escape hatches

### Trade-offs

| Criterion | Wrapper | Adapter | Bridge | Trait (Chosen) |
|-----------|---------|---------|--------|----------------|
| Flexibility | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| Performance | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| Code Size | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
| Maintainability | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| Testing | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| Runtime Switch | ❌ | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |

## Implementation Plan

### Day 2: Core Implementation (8 hours)

**Morning (4 hours):**
1. Create `crates/riptide-browser-abstraction/` structure
2. Define core traits (`BrowserEngine`, `PageHandle`)
3. Define parameter types (ScreenshotParams, PdfParams, Viewport, etc.)
4. Write trait documentation and examples

**Afternoon (4 hours):**
5. Implement `ChromiumoxideEngine` and `ChromiumoxidePage`
6. Implement `SpiderChromeEngine` and `SpiderChromePage`
7. Add parameter translation helpers
8. Write unit tests for both implementations

### Day 3: Integration (6 hours)

**Morning (3 hours):**
1. Update `hybrid_fallback.rs` to use trait objects
2. Update `launcher.rs` to return `Box<dyn BrowserEngine>`
3. Update `pool.rs` to accept `Box<dyn PageHandle>`

**Afternoon (3 hours):**
4. Run and fix integration tests
5. Add new tests for hybrid switching
6. Update documentation
7. Performance benchmarks

### Day 4: Validation (4 hours)

**Morning (2 hours):**
1. Run full test suite
2. Fix any remaining issues
3. Performance validation (<5% overhead)

**Afternoon (2 hours):**
4. Update Week 2 progress document
5. Prepare Week 3 status report
6. Document any issues found

**Total Estimated Time:** 18 hours (2.25 days)

## Complexity Estimates

### Lines of Code (Estimated)

```
crates/riptide-browser-abstraction/
├── src/
│   ├── lib.rs                    # 150 lines (exports, docs)
│   ├── traits.rs                 # 200 lines (BrowserEngine, PageHandle)
│   ├── params.rs                 # 300 lines (ScreenshotParams, PdfParams, etc.)
│   ├── chromiumoxide_impl.rs     # 400 lines (Engine + Page impl)
│   ├── spider_chrome_impl.rs     # 500 lines (Engine + Page impl, stealth)
│   ├── error.rs                  # 100 lines (Error types)
│   └── tests.rs                  # 400 lines (Unit tests)
├── Cargo.toml                    # 50 lines
└── README.md                     # 100 lines

Total: ~2,200 lines
```

### Modified Files

```
crates/riptide-headless/src/hybrid_fallback.rs    # ~50 lines changed
crates/riptide-headless/src/launcher.rs           # ~30 lines changed
crates/riptide-headless/src/pool.rs               # ~40 lines changed
crates/riptide-engine/src/hybrid_fallback.rs      # ~50 lines changed
crates/riptide-engine/src/launcher.rs             # ~30 lines changed
crates/riptide-engine/src/pool.rs                 # ~40 lines changed

Total modifications: ~240 lines
```

**Grand Total:** ~2,440 lines of code

## Alternatives Considered

### 1. Wrapper Pattern (Rejected)
**Why rejected:** Too tightly coupled to spider_chrome internals. Hard to maintain when spider_chrome updates. Cannot support fallback without rewrite.

```rust
// Problem: Wraps spider_chrome, no fallback possible
pub struct SpiderChromeWrapper {
    inner: spider_chrome::Browser,
}
```

### 2. Adapter Pattern (Rejected)
**Why rejected:** Still requires type conversion between incompatible CDP types. Cannot convert `spider_chromiumoxide_cdp::Page` to `chromiumoxide_cdp::Page` without serialization.

```rust
// Problem: Cannot convert types
impl From<spider_chrome::Page> for chromiumoxide::Page {
    fn from(page: spider_chrome::Page) -> Self {
        // Impossible! Different types
    }
}
```

### 3. Bridge Pattern with Concrete Types (Rejected)
**Why rejected:** Similar to trait abstraction but more verbose. Requires implementing bridge for each operation manually. Trait abstraction is cleaner.

### 4. Feature Flags (Rejected)
**Why rejected:** Cannot do runtime switching. Requires recompiling for different engines. No hybrid fallback possible.

```toml
[features]
engine-spider = ["spider_chrome"]
engine-chromium = ["chromiumoxide"]
```

### 5. Type Erasure with Arc<dyn Any> (Rejected)
**Why rejected:** Unsafe, loses type safety, requires downcasting everywhere. Error-prone and unergonomic.

## Risk Assessment

### High Risks
- **None identified** - This is a proven pattern in Rust ecosystem

### Medium Risks
1. **Parameter Mismatch** - Risk: 30%
   - Mitigation: Comprehensive parameter tests for both engines
   - Impact: Minor (caught by tests)

2. **Performance Regression** - Risk: 20%
   - Mitigation: Benchmarks before/after
   - Impact: Low (can optimize if needed)

### Low Risks
1. **API Changes in Upstream** - Risk: 10%
   - Mitigation: Update wrappers when libraries update
   - Impact: Isolated to wrapper implementations

## Validation Criteria

✅ **Success Criteria:**
1. Can instantiate either engine at runtime
2. Can switch between engines per request
3. 20% spider-chrome, 80% chromiumoxide routing works
4. All existing tests pass
5. Performance overhead <5%
6. No unsafe code required
7. New tests for hybrid switching pass

❌ **Failure Criteria:**
1. Cannot support runtime switching
2. >5% performance regression
3. Requires unsafe code
4. Cannot preserve spider_chrome stealth features

## Next Steps (Day 2)

1. Create `riptide-browser-abstraction` crate structure
2. Implement core traits and parameter types
3. Add chromiumoxide implementation
4. Add spider_chrome implementation
5. Write comprehensive tests
6. Integrate with hybrid_fallback.rs
7. Run integration tests
8. Performance benchmarks

## References

- **API Analysis:** `/workspaces/eventmesh/docs/integration/SPIDER-CHROME-API-ANALYSIS.md`
- **Hybrid Fallback:** `/workspaces/eventmesh/crates/riptide-headless/src/hybrid_fallback.rs`
- **Week 2 Progress:** `/workspaces/eventmesh/docs/hive-mind-todos.md`

## Approval

- **Architect:** Research Agent (2025-10-17)
- **Status:** Awaiting coder implementation
- **Timeline:** Day 2-4 (18 hours total)
