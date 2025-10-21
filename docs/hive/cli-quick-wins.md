# CLI Quick Wins - Immediate Extraction Opportunities

**Generated:** 2025-10-21
**Priority:** Immediate implementation
**Effort:** 1-3 days per item
**Impact:** High maintainability and testability gains

---

## Overview

This document identifies **low-hanging fruit** in the CLI refactoring effort - modules that can be extracted immediately with minimal dependencies and maximum impact.

**Selection Criteria:**
- ✅ Low coupling to CLI framework
- ✅ High business logic density
- ✅ Clear architectural home
- ✅ No circular dependencies
- ✅ High reusability potential

---

## Quick Win #1: Extract Search Module → `riptide-operations`

**File:** `crates/riptide-cli/src/commands/search.rs` (82 LOC)
**Effort:** 4 hours
**Impact:** Enables search API endpoint

### Current State
```rust
// Minimal dependencies, pure logic
pub struct SearchArgs {
    pub query: String,
    pub limit: u32,
    pub domain: Option<String>,
}

pub async fn execute(args: SearchArgs) -> Result<()> {
    // 82 LOC of search implementation
}
```

### Extraction Plan
```rust
// New: riptide-operations/src/search/service.rs
pub struct SearchService {
    // Search implementation
}

impl SearchService {
    pub async fn search(&self, params: SearchParams) -> Result<SearchResults> {
        // Extracted logic
    }
}

// CLI: crates/riptide-cli/src/commands/search.rs (20 LOC after)
pub async fn execute(args: SearchArgs) -> Result<()> {
    let service = SearchService::new()?;
    let results = service.search(args.into()).await?;
    output::display_search_results(&results);
    Ok(())
}
```

### Benefits
- ✅ Search API endpoint ready
- ✅ Testable without CLI
- ✅ Reusable by web UI
- ✅ -76% CLI code (82 → 20 LOC)

### Migration Steps
1. Create `riptide-operations/src/search/service.rs`
2. Move search logic to service
3. Add unit tests (target: 8-10 tests)
4. Update CLI to use service
5. Delete old implementation

---

## Quick Win #2: Extract Health Check → `riptide-core`

**File:** `crates/riptide-cli/src/commands/health.rs` (59 LOC)
**Effort:** 3 hours
**Impact:** Enables health check endpoint for API server

### Current State
```rust
pub async fn execute() -> Result<()> {
    // Check system health
    // Print status
}
```

### Extraction Plan
```rust
// New: riptide-core/src/health/checker.rs
pub struct HealthChecker;

impl HealthChecker {
    pub async fn check(&self) -> HealthStatus {
        // Extracted logic
    }
}

// CLI: 15 LOC wrapper
pub async fn execute() -> Result<()> {
    let status = HealthChecker::new().check().await;
    output::display_health_status(&status);
    Ok(())
}
```

### Benefits
- ✅ Health endpoint for API server
- ✅ Monitoring integration ready
- ✅ Testable health checks
- ✅ -75% CLI code (59 → 15 LOC)

---

## Quick Win #3: Consolidate WASM Cache Modules

**Files:**
- `wasm_cache.rs` (282 LOC)
- `wasm_aot_cache.rs` (497 LOC)
- Total: 779 LOC

**Effort:** 1 day
**Impact:** -50% code through consolidation

### Problem
**Duplicate WASM cache implementations:**
1. `WasmModuleCache` (once_cell::OnceCell)
2. `WasmCache` (Lazy<Arc>)
3. `WasmAotCache` (tokio::sync::OnceCell)

**All do similar things:** Cache WASM modules for reuse

### Consolidation Plan
```rust
// New: riptide-extraction/src/wasm/cache.rs
pub struct UnifiedWasmCache {
    module_cache: HashMap<String, CachedModule>,
    aot_cache: HashMap<String, CompiledModule>,
}

impl UnifiedWasmCache {
    pub fn global() -> &'static Self { /* ... */ }

    pub async fn get_module(&self, path: &str) -> Result<WasmModule> {
        // Unified module caching
    }

    pub async fn get_compiled(&self, path: &str) -> Result<CompiledModule> {
        // Unified AOT caching
    }
}
```

### Benefits
- ✅ Single source of truth
- ✅ -50% code (779 → ~400 LOC)
- ✅ Easier to test
- ✅ Simpler singleton management
- ✅ Belongs in riptide-extraction (correct home)

### Migration Steps
1. Create `riptide-extraction/src/wasm/cache.rs`
2. Merge caching logic from 3 modules
3. Add comprehensive tests
4. Update CLI commands to use unified cache
5. Delete old cache modules

---

## Quick Win #4: Extract Validate Command → `riptide-core`

**File:** `crates/riptide-cli/src/commands/validate.rs` (182 LOC)
**Effort:** 6 hours
**Impact:** Validation service for API and worker processes

### Current State
```rust
pub struct ValidateArgs {
    pub config: Option<String>,
    pub strict: bool,
}

pub async fn execute(args: ValidateArgs) -> Result<()> {
    // Config validation logic
}
```

### Extraction Plan
```rust
// New: riptide-core/src/validation/service.rs
pub struct ValidationService;

impl ValidationService {
    pub fn validate_config(&self, config: &Config) -> ValidationResult {
        // Extracted validation logic
    }
}

// CLI: 30 LOC wrapper
pub async fn execute(args: ValidateArgs) -> Result<()> {
    let result = ValidationService::new().validate_config(&config);
    output::display_validation_results(&result);
    Ok(())
}
```

### Benefits
- ✅ Config validation for API server startup
- ✅ Worker process validation
- ✅ Reusable validation rules
- ✅ -84% CLI code (182 → 30 LOC)

---

## Quick Win #5: Extract Engine Selection Logic

**Files:**
- `engine_fallback.rs` (471 LOC)
- `engine_cache.rs` (211 LOC)
- Total: 682 LOC

**Effort:** 1 day
**Impact:** Core engine selection service

### Current State
**Two separate modules:**
1. `EngineFallbackStrategy` - Fallback logic
2. `EngineSelectionCache` - Selection caching

**Both belong in core orchestration layer**

### Consolidation Plan
```rust
// New: riptide-core/src/engine/selector.rs
pub struct EngineSelector {
    cache: Cache<EngineSelection>,
    fallback_strategy: FallbackStrategy,
}

impl EngineSelector {
    pub fn global() -> &'static Self { /* ... */ }

    pub async fn select_engine(&self, context: &Context) -> Result<Engine> {
        // Unified selection logic with caching and fallback
    }
}
```

### Benefits
- ✅ Single engine selection API
- ✅ Correct architectural home (riptide-core)
- ✅ Reusable by all extraction paths
- ✅ -20% code through consolidation

---

## Quick Win #6: Extract Tables Module → `riptide-data`

**File:** `crates/riptide-cli/src/commands/tables.rs` (436 LOC)
**Effort:** 8 hours
**Impact:** Table extraction service for API

### Current State
```rust
pub struct TablesArgs {
    pub url: Option<String>,
    pub file: Option<String>,
    pub format: String,
    pub stdin: bool,
    pub output: Option<String>,
}

pub async fn execute(args: TablesArgs) -> Result<()> {
    // Table extraction and formatting logic (436 LOC)
}
```

### Extraction Plan
```rust
// New: riptide-data/src/tables/extractor.rs
pub struct TableExtractor;

impl TableExtractor {
    pub fn extract(&self, html: &str) -> Result<Vec<Table>> {
        // Table extraction logic
    }
}

// New: riptide-data/src/tables/formatter.rs
pub struct TableFormatter;

impl TableFormatter {
    pub fn format(&self, tables: &[Table], format: Format) -> Result<String> {
        // Formatting logic (markdown, csv, json)
    }
}

// CLI: 60 LOC wrapper
pub async fn execute(args: TablesArgs) -> Result<()> {
    let html = get_html(&args)?;
    let tables = TableExtractor::new().extract(&html)?;
    let output = TableFormatter::new().format(&tables, args.format.into())?;
    save_or_print(&output, &args.output)?;
    Ok(())
}
```

### Benefits
- ✅ Table extraction API endpoint
- ✅ Testable extraction logic
- ✅ Reusable formatters
- ✅ -86% CLI code (436 → 60 LOC)

---

## Quick Win #7: Migrate GLOBAL_POOL_MANAGER Singleton

**File:** `browser_pool_manager.rs` (452 LOC)
**Effort:** 4 hours
**Impact:** Unblocks headless crate refactoring

### Current State
```rust
// IN CLI (wrong place)
static GLOBAL_POOL_MANAGER: tokio::sync::OnceCell<Arc<BrowserPoolManager>> =
    tokio::sync::OnceCell::const_new();

impl BrowserPoolManager {
    pub async fn initialize_global() -> Result<Arc<Self>> {
        // ...
    }
}
```

### Migration Plan
```rust
// Move to: riptide-headless/src/pool/manager.rs
pub struct BrowserPoolManager {
    // Pool implementation
}

impl BrowserPoolManager {
    pub fn global() -> &'static Self {
        // Singleton in correct crate
    }
}

// CLI: Just use it
pub async fn execute(args: RenderArgs) -> Result<()> {
    let pool = BrowserPoolManager::global();
    // ...
}
```

### Benefits
- ✅ Singleton in correct architectural home
- ✅ Reusable by API server
- ✅ Unblocks other headless refactoring
- ✅ Cleaner CLI code

---

## Priority Matrix

| Quick Win | Effort | Impact | Priority | Dependencies |
|-----------|--------|--------|----------|--------------|
| **#1 Search** | 4h | High | **P0** | None |
| **#2 Health** | 3h | High | **P0** | None |
| **#4 Validate** | 6h | High | **P0** | None |
| **#7 GLOBAL_POOL_MANAGER** | 4h | Critical | **P0** | None |
| **#3 WASM Cache Consolidation** | 8h | High | **P1** | None |
| **#5 Engine Selection** | 8h | Medium | **P1** | None |
| **#6 Tables** | 8h | Medium | **P2** | None |

### Recommended Order

**Week 1 (P0):**
1. Health check (3h) - Monday AM
2. Search (4h) - Monday PM
3. Validate (6h) - Tuesday
4. GLOBAL_POOL_MANAGER (4h) - Wednesday AM

**Total Week 1:** 17 hours, 4 modules extracted

**Week 2 (P1):**
5. WASM Cache Consolidation (8h) - Thursday-Friday
6. Engine Selection (8h) - Monday-Tuesday

**Total Week 2:** 16 hours, 3 more modules consolidated

**Week 3 (P2):**
7. Tables extraction (8h) - Wednesday-Thursday

**Total:** 41 hours (1 week of focused work)

---

## Expected Outcomes

After completing all quick wins:

### Code Reduction
- **CLI LOC:** -2,913 LOC (21% of total CLI code)
- **Library LOC:** +1,800 LOC (new reusable services)
- **Net reduction:** -1,113 LOC (38% reduction in target modules)

### Architectural Improvements
- **Singletons migrated:** 1 (GLOBAL_POOL_MANAGER)
- **Cache consolidation:** 3 → 1 unified cache
- **Engine selection:** 2 modules → 1 service
- **New library services:** 5 (search, health, validate, tables, engine)

### Testing Improvements
- **New test coverage:** ~50 unit tests for extracted services
- **Testable code:** +1,800 LOC (from 0% to 80%+ coverage)
- **Integration tests:** 7 new CLI integration tests

### Reusability Gains
- **API endpoints enabled:** 4 (search, health, validate, tables)
- **Services available to workers:** 5
- **Shared logic:** Engine selection, WASM caching, validation

---

## Success Criteria

Each quick win should achieve:
- ✅ CLI command <100 LOC (thin layer)
- ✅ Library service with 80%+ test coverage
- ✅ No breaking changes to CLI interface
- ✅ Documentation updated
- ✅ Benchmarks show no performance regression

---

## Risk Mitigation

### For Each Extraction:

**Before:**
1. Add integration test for current CLI behavior
2. Document current API contract
3. Measure performance baseline

**During:**
4. Implement library service with tests
5. Update CLI to use service
6. Verify integration test still passes

**After:**
7. Run performance benchmarks
8. Code review for architectural violations
9. Update documentation

### Rollback Plan
Keep old implementation as `_legacy` suffix until new implementation is proven in production.

---

## Tracking Progress

Use this checklist to track quick wins:

- [ ] **Quick Win #1:** Search extraction
  - [ ] Library service created
  - [ ] Tests added (8+ tests)
  - [ ] CLI updated
  - [ ] Integration test passes

- [ ] **Quick Win #2:** Health check extraction
  - [ ] Library service created
  - [ ] Tests added (5+ tests)
  - [ ] CLI updated
  - [ ] Health endpoint enabled

- [ ] **Quick Win #3:** WASM cache consolidation
  - [ ] Unified cache implemented
  - [ ] Migration script created
  - [ ] All 3 old caches removed
  - [ ] Tests pass

- [ ] **Quick Win #4:** Validate extraction
  - [ ] Validation service created
  - [ ] Tests added (10+ tests)
  - [ ] CLI updated
  - [ ] API validation enabled

- [ ] **Quick Win #5:** Engine selection consolidation
  - [ ] Unified selector implemented
  - [ ] Cache + fallback merged
  - [ ] Tests added
  - [ ] Performance validated

- [ ] **Quick Win #6:** Tables extraction
  - [ ] Extractor service created
  - [ ] Formatter service created
  - [ ] Tests added (12+ tests)
  - [ ] CLI updated

- [ ] **Quick Win #7:** GLOBAL_POOL_MANAGER migration
  - [ ] Singleton moved to riptide-headless
  - [ ] CLI updated
  - [ ] Integration tests pass
  - [ ] No initialization order issues

---

## Next Steps

After completing quick wins:
1. Review CLI complexity metrics (target: -21% LOC)
2. Measure test coverage improvement (target: +1,800 testable LOC)
3. Proceed to Phase 1 P0 extractions (Core Operations)
4. Continue with singleton migration plan

---

**Start with Quick Win #1 (Search) - lowest effort, highest learning value!**
