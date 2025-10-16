# RipTide EventMesh Refactoring Plan

**Status**: 🚧 In Progress
**Target**: Reduce complexity in 117 files with >600 LOC
**Goal**: Improve maintainability, testability, and compilation times
**Timeline**: 7-8 weeks

---

## 📊 Executive Summary

### Current State
- **117 files** exceed 600 lines of code
- **9 critical files** exceed 1,200 lines
- **Total LOC to refactor**: ~102,000 lines
- **Largest file**: `wasm/riptide-extractor-wasm/src/bindings.rs` (2,663 lines - auto-generated, skip)

### Success Criteria
- ✅ All files under 600 lines
- ✅ All tests passing
- ✅ Zero clippy warnings
- ✅ Documentation complete
- ✅ CI/CD green on all platforms
- ✅ No performance regressions

---

## 🎯 Quality Assurance Requirements

### Before Each Refactoring
```bash
# 1. Create feature branch
git checkout -b refactor/[component-name]

# 2. Ensure clean baseline
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features
```

### During Refactoring
```bash
# Run continuously during development
cargo check --all-features
cargo clippy --fix --allow-dirty --allow-staged
cargo fmt

# Run tests for affected modules
cargo test --package [crate-name] --lib
```

### After Each Refactoring
```bash
# 1. Full validation suite
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings -D clippy::all
cargo test --all-features --all-targets
cargo bench --no-run  # Ensure benchmarks compile
cargo doc --no-deps --all-features --document-private-items

# 2. Check for unused dependencies
cargo machete

# 3. Security audit
cargo audit

# 4. Run integration tests
cargo test --test '*' --all-features

# 5. Performance baseline
cargo build --release
# Run performance benchmarks and compare
```

### Commit Standards
```bash
# Commit message format
refactor([scope]): [description]

- Split [file] into [n] modules
- Extract [component] to separate file
- Improve [metric] (e.g., complexity, testability)
- All tests passing, zero clippy warnings

BREAKING CHANGE: [if applicable]
```

---

## 🔴 Phase 1: Critical Priority (Weeks 1-2)

### 1.1 Refactor `crates/riptide-api/src/metrics.rs` (1,375 LOC)

**Current Issues**:
- Single struct with 55+ metric fields
- Poor separation of concerns
- Difficult to maintain and test

**Refactoring Plan**:

#### Step 1: Create Module Structure
```bash
mkdir -p crates/riptide-api/src/metrics
touch crates/riptide-api/src/metrics/{mod.rs,http.rs,pipeline.rs,streaming.rs,spider.rs,pdf.rs,wasm.rs,workers.rs}
```

**Todo Checklist**:
- [ ] Create `metrics/mod.rs` with public exports
- [ ] Extract HTTP metrics to `metrics/http.rs` (~150 LOC)
  - `HttpMetrics` struct
  - Request/response counters
  - Error tracking
- [ ] Extract pipeline metrics to `metrics/pipeline.rs` (~200 LOC)
  - Phase timing histograms
  - Gate decision counters
  - Quality metrics
- [ ] Extract streaming metrics to `metrics/streaming.rs` (~180 LOC)
  - Connection metrics
  - Message counters
  - Memory tracking
- [ ] Extract spider metrics to `metrics/spider.rs` (~150 LOC)
  - Crawl counters
  - Frontier metrics
  - Performance tracking
- [ ] Extract PDF metrics to `metrics/pdf.rs` (~120 LOC)
  - Processing counters
  - Memory metrics
  - Error tracking
- [ ] Extract WASM metrics to `metrics/wasm.rs` (~100 LOC)
  - Memory page tracking
  - Cold start metrics
  - AOT cache metrics
- [ ] Extract worker metrics to `metrics/workers.rs` (~120 LOC)
  - Pool metrics
  - Job tracking
  - Queue depth
- [ ] Create unified `RipTideMetrics` facade in `mod.rs` (~200 LOC)
- [ ] Update all imports in dependent files
- [ ] Run full quality check suite

**Quality Gates**:
```bash
# After each module extraction
cargo test --package riptide-api --lib metrics
cargo clippy --package riptide-api -- -D warnings

# After completion
cargo test --package riptide-api --all-features
cargo doc --package riptide-api --no-deps
```

**Acceptance Criteria**:
- [ ] All modules <200 LOC
- [ ] Zero clippy warnings
- [ ] All existing tests pass
- [ ] Documentation complete
- [ ] No breaking API changes

---

### 1.2 Refactor `crates/riptide-api/src/state.rs` (1,222 LOC)

**Current Issues**:
- God object with 20+ dependencies
- Complex initialization logic (400+ LOC)
- Health checking mixed with state

**Refactoring Plan**:

#### Step 1: Create Module Structure
```bash
mkdir -p crates/riptide-api/src/state
touch crates/riptide-api/src/state/{mod.rs,app_state.rs,monitoring.rs,health.rs,config.rs}
```

**Todo Checklist**:
- [ ] Create `state/mod.rs` with public exports
- [ ] Extract core state to `state/app_state.rs` (~200 LOC)
  - `AppState` struct with core dependencies
  - Builder pattern for initialization
  - Basic accessors
- [ ] Extract monitoring to `state/monitoring.rs` (~250 LOC)
  - `MonitoringSystem` struct
  - Alert management
  - Performance tracking
  - Background task management
- [ ] Extract health checks to `state/health.rs` (~200 LOC)
  - `HealthChecker` implementation
  - Dependency health tracking
  - Health status types
  - Individual component checks
- [ ] Extract configuration to `state/config.rs` (~400 LOC)
  - `AppConfig` struct
  - Environment variable parsing
  - Validation logic
  - Default implementations
- [ ] Refactor `AppState::new()` to use builder pattern (~150 LOC)
- [ ] Update all imports in handlers and middleware
- [ ] Add integration tests for state initialization

**Quality Gates**:
```bash
# After each module
cargo test --package riptide-api --lib state::
cargo clippy --package riptide-api -- -D warnings

# Integration test
cargo test --package riptide-api --test state_integration
```

**Acceptance Criteria**:
- [ ] All modules <400 LOC
- [ ] Clean separation of concerns
- [ ] Builder pattern for complex initialization
- [ ] All tests passing
- [ ] Documentation complete

---

### 1.3 Refactor `crates/riptide-extraction/src/css_extraction.rs` (1,236 LOC)

**Current Issues**:
- 14 transformer implementations in one file
- Main extraction logic mixed with transformers
- Difficult to add new transformers

**Refactoring Plan**:

#### Step 1: Create Module Structure
```bash
mkdir -p crates/riptide-extraction/src/css_extraction/{transformers,selectors}
touch crates/riptide-extraction/src/css_extraction/{mod.rs,core.rs,builder.rs}
touch crates/riptide-extraction/src/css_extraction/transformers/{mod.rs,trim.rs,normalize.rs,number.rs,currency.rs,date.rs,url.rs,case.rs,text.rs,json.rs,html.rs}
touch crates/riptide-extraction/src/css_extraction/selectors/{mod.rs,config.rs,defaults.rs}
```

**Todo Checklist**:
- [ ] Create `css_extraction/mod.rs` with public API
- [ ] Extract core extractor to `core.rs` (~250 LOC)
  - `CssJsonExtractor` struct
  - Main extraction logic
  - Confidence scoring
  - Merge policy handling
- [ ] Extract builder to `builder.rs` (~150 LOC)
  - `CssConfigBuilder`
  - Fluent API
  - Validation
- [ ] Create transformer trait in `transformers/mod.rs` (~50 LOC)
- [ ] Extract individual transformers (each ~30-50 LOC):
  - [ ] `trim.rs` - TrimTransformer
  - [ ] `normalize.rs` - NormalizeWhitespaceTransformer
  - [ ] `number.rs` - NumberTransformer
  - [ ] `currency.rs` - CurrencyTransformer
  - [ ] `date.rs` - DateIsoTransformer
  - [ ] `url.rs` - UrlAbsoluteTransformer
  - [ ] `case.rs` - LowercaseTransformer, UppercaseTransformer
  - [ ] `text.rs` - SplitTransformer, JoinTransformer, RegexExtractTransformer, RegexReplaceTransformer
  - [ ] `json.rs` - JsonParseTransformer
  - [ ] `html.rs` - HtmlDecodeTransformer
- [ ] Extract selector configs to `selectors/config.rs` (~150 LOC)
- [ ] Extract default selectors to `selectors/defaults.rs` (~200 LOC)
- [ ] Update all tests
- [ ] Add transformer registration tests

**Quality Gates**:
```bash
# After each transformer extraction
cargo test --package riptide-extraction --lib css_extraction::transformers
cargo clippy --package riptide-extraction -- -D warnings

# Full suite
cargo test --package riptide-extraction --all-features
```

**Acceptance Criteria**:
- [ ] Core extractor <250 LOC
- [ ] Each transformer <50 LOC
- [ ] Easy to add new transformers
- [ ] All tests passing
- [ ] Documentation with examples

---

### 1.4 Refactor `crates/riptide-core/src/spider/core.rs` (1,014 LOC)

**Current Issues**:
- Monolithic crawler implementation
- Request processing logic mixed with crawling
- Complex control flow

**Refactoring Plan**:

#### Step 1: Analyze and Split
```bash
# Already in module, need to split further
touch crates/riptide-core/src/spider/{processor.rs,fetcher.rs,extractor.rs,state.rs}
```

**Todo Checklist**:
- [ ] Extract request processor to `processor.rs` (~250 LOC)
  - `RequestProcessor` struct
  - Budget checking
  - Robots.txt validation
  - Semaphore management
  - Result recording
- [ ] Extract fetcher to `fetcher.rs` (~200 LOC)
  - `SpiderFetcher` struct
  - HTTP client management
  - Session handling
  - Circuit breaker integration
  - Fetch engine integration
- [ ] Extract URL/content extractor to `extractor.rs` (~150 LOC)
  - `ContentExtractor` struct
  - Link extraction
  - Text extraction
  - URL filtering
- [ ] Extract crawl state to `state.rs` (~100 LOC)
  - `CrawlState` struct
  - `PerformanceMetrics` struct
  - State management helpers
- [ ] Refactor main `Spider` to orchestrate (~350 LOC)
  - Delegate to specialized components
  - Main crawl loop
  - High-level coordination
- [ ] Update tests for each module
- [ ] Add integration tests

**Quality Gates**:
```bash
cargo test --package riptide-core --lib spider
cargo clippy --package riptide-core -- -D warnings
cargo test --package riptide-core --test spider_integration
```

**Acceptance Criteria**:
- [ ] Main `Spider` <400 LOC
- [ ] Each component <250 LOC
- [ ] Clear separation of concerns
- [ ] All tests passing
- [ ] Performance maintained

---

### 1.5 Refactor `crates/riptide-persistence/src/state.rs` (1,191 LOC)

**Current Issues**:
- Complex state management
- Mixed concerns (tenancy, caching, storage)

**Refactoring Plan**:

#### Step 1: Create Module Structure
```bash
mkdir -p crates/riptide-persistence/src/state
touch crates/riptide-persistence/src/state/{mod.rs,tenant.rs,cache.rs,storage.rs,operations.rs}
```

**Todo Checklist**:
- [ ] Extract tenant management to `tenant.rs` (~300 LOC)
  - Tenant operations
  - Multi-tenancy support
  - Isolation logic
- [ ] Extract cache operations to `cache.rs` (~250 LOC)
  - Cache layer implementation
  - TTL management
  - Invalidation strategies
- [ ] Extract storage layer to `storage.rs` (~250 LOC)
  - Storage backend abstraction
  - CRUD operations
  - Query builders
- [ ] Extract state operations to `operations.rs` (~200 LOC)
  - High-level operations
  - Transaction support
  - Batch operations
- [ ] Refactor main state coordinator (~150 LOC)
- [ ] Update all imports
- [ ] Add integration tests

**Quality Gates**:
```bash
cargo test --package riptide-persistence --lib
cargo clippy --package riptide-persistence -- -D warnings
```

**Acceptance Criteria**:
- [ ] All modules <300 LOC
- [ ] Clear responsibility boundaries
- [ ] All tests passing
- [ ] Documentation complete

---

## 🟡 Phase 2: High Priority (Weeks 3-4)

### 2.1 Refactor `crates/riptide-pdf/src/processor.rs` (1,136 LOC)

**Todo Checklist**:
- [ ] Create module structure
- [ ] Extract text processing to `processor/text.rs` (~300 LOC)
- [ ] Extract image processing to `processor/images.rs` (~250 LOC)
- [ ] Extract metadata handling to `processor/metadata.rs` (~200 LOC)
- [ ] Refactor core processor (~350 LOC)
- [ ] Quality checks

### 2.2 Refactor `crates/riptide-performance/src/monitoring/monitor.rs` (1,141 LOC)

**Todo Checklist**:
- [ ] Extract collector to `monitor/collector.rs` (~300 LOC)
- [ ] Extract alerting to `monitor/alerting.rs` (~300 LOC)
- [ ] Extract health calculation to `monitor/health.rs` (~250 LOC)
- [ ] Refactor main monitor (~250 LOC)
- [ ] Quality checks

### 2.3 Refactor `crates/riptide-streaming/src/reports.rs` (1,130 LOC)

**Todo Checklist**:
- [ ] Extract generator to `reports/generator.rs` (~300 LOC)
- [ ] Extract formatters to `reports/formatters.rs` (~300 LOC)
- [ ] Extract metrics to `reports/metrics.rs` (~250 LOC)
- [ ] Refactor main reports (~250 LOC)
- [ ] Quality checks

### 2.4 Refactor `crates/riptide-core/src/instance_pool/pool.rs` (964 LOC)

**Todo Checklist**:
- [ ] Extract manager to `pool/manager.rs` (~300 LOC)
- [ ] Extract lifecycle to `pool/lifecycle.rs` (~250 LOC)
- [ ] Extract health to `pool/health.rs` (~200 LOC)
- [ ] Refactor main pool (~200 LOC)
- [ ] Quality checks

### 2.5 Refactor `crates/riptide-workers/src/processors.rs` (906 LOC)

**Todo Checklist**:
- [ ] Create `processors/` directory
- [ ] Extract individual processors (each ~80-100 LOC)
- [ ] Create processor registry
- [ ] Update processor factory
- [ ] Quality checks

---

## 🟢 Phase 3: Medium Priority (Weeks 5-6)

### 3.1-3.10 Refactor Files 700-900 LOC

**Todo Template** (Apply to each file):
- [ ] Analyze file structure
- [ ] Identify logical boundaries
- [ ] Create module structure
- [ ] Extract components (aim for <300 LOC each)
- [ ] Update imports
- [ ] Run quality checks
- [ ] Update documentation

**Files in this phase**:
1. `cache_warming.rs` (881 LOC)
2. `fetch.rs` (827 LOC)
3. `endpoints.js` (818 LOC)
4. `html_extraction_tests.rs` (817 LOC)
5. `adaptive_stop.rs` (813 LOC)
6. `streaming_tests.rs` (835 LOC)
7. `ndjson_stream_tests.rs` (775 LOC)
8. `dashboard.rs` (774 LOC)
9. `runtime_switch.rs` (763 LOC)
10. `events/types.rs` (767 LOC)

---

## ⚪ Phase 4: Low Priority (Week 7)

### 4.1 Files 600-700 LOC (63 files)

**Strategy**: Monitor and refactor opportunistically
- Refactor when making changes to these files
- No dedicated refactoring effort unless needed
- Focus on keeping them from growing

---

## 🔵 Special Cases

### Skip These Files
- `wasm/riptide-extractor-wasm/src/bindings.rs` (2,663 LOC)
  - **Reason**: Auto-generated by `wit-bindgen`
  - **Action**: Add to `.clippy.toml` allow list

### Test Files (Lower Priority)
- Integration tests (20+ files)
  - Can be large but isolated
  - Refactor only if causing maintenance issues

---

## 🛠️ Tooling and Automation

### Pre-commit Hook Setup
```bash
# .git/hooks/pre-commit
#!/bin/bash
set -e

echo "Running pre-commit checks..."

# Format check
cargo fmt --all --check

# Clippy check
cargo clippy --all-targets --all-features -- -D warnings

# Quick test
cargo test --lib --all-features

echo "Pre-commit checks passed!"
```

### CI/CD Quality Gates

**GitHub Actions Workflow** (`.github/workflows/refactoring-quality.yml`):
```yaml
name: Refactoring Quality Checks

on:
  pull_request:
    branches: [main, develop]
    paths:
      - 'crates/**/*.rs'

jobs:
  quality-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy

      - name: Check formatting
        run: cargo fmt --all --check

      - name: Clippy (zero warnings)
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Run tests
        run: cargo test --all-features

      - name: Check documentation
        run: cargo doc --no-deps --all-features --document-private-items

      - name: Audit dependencies
        run: cargo audit

      - name: Line count check
        run: |
          ./scripts/check_file_lengths.sh
```

### Line Count Enforcement Script
```bash
#!/bin/bash
# scripts/check_file_lengths.sh

MAX_LINES=600
VIOLATIONS=0

echo "Checking for files exceeding $MAX_LINES lines..."

while IFS= read -r file; do
    LINES=$(wc -l < "$file")
    if [ "$LINES" -gt "$MAX_LINES" ]; then
        echo "❌ $file: $LINES lines (exceeds $MAX_LINES)"
        VIOLATIONS=$((VIOLATIONS + 1))
    fi
done < <(find crates -name "*.rs" -not -path "*/target/*" -not -name "bindings.rs")

if [ "$VIOLATIONS" -gt 0 ]; then
    echo "Found $VIOLATIONS files exceeding $MAX_LINES lines"
    exit 1
fi

echo "✅ All files within $MAX_LINES line limit"
```

---

## 📈 Progress Tracking

### Metrics Dashboard

Track these KPIs weekly:
- [ ] Files >600 LOC: **117** → Target: **0**
- [ ] Average file size: **~800 LOC** → Target: **<400 LOC**
- [ ] Clippy warnings: **TBD** → Target: **0**
- [ ] Test coverage: **TBD** → Target: **>80%**
- [ ] Build time: **TBD** → Target: **<30% improvement**

### Weekly Review Checklist
```markdown
## Week [N] Progress Report

### Completed
- [ ] File 1: [name] - Refactored into [n] modules
- [ ] File 2: [name] - Refactored into [n] modules

### Quality Metrics
- [ ] All tests passing: ✅/❌
- [ ] Clippy warnings: [count]
- [ ] Files remaining: [count]
- [ ] Build time change: [+/- X%]

### Blockers
- [List any blockers]

### Next Week
- [ ] [Planned work]
```

---

## 🎓 Best Practices During Refactoring

### Code Organization
1. **One responsibility per file** - Clear, focused purpose
2. **Consistent module structure** - Same patterns across crates
3. **Public API stability** - Minimize breaking changes
4. **Documentation first** - Write docs before implementation
5. **Test coverage** - Add tests for new boundaries

### Refactoring Safety
1. **Small, atomic commits** - Each commit compiles and passes tests
2. **Feature flags** - Use for large refactorings
3. **Backward compatibility** - Deprecate before removing
4. **Performance validation** - Benchmark before/after
5. **Code review** - All refactorings require review

### Common Patterns
```rust
// Before: Large struct
pub struct LargeComponent {
    // 50+ fields
}

// After: Composed components
pub struct MainComponent {
    http: HttpComponent,
    processing: ProcessingComponent,
    storage: StorageComponent,
}

// Before: Large impl block
impl LargeComponent {
    // 30+ methods
}

// After: Focused trait implementations
impl MainComponent {
    // 5-10 core methods
}

impl HttpHandler for HttpComponent {
    // HTTP-specific methods
}

impl DataProcessor for ProcessingComponent {
    // Processing methods
}
```

---

## ✅ Completion Checklist

### Phase 1 (Weeks 1-2)
- [ ] metrics.rs refactored and validated
- [ ] state.rs refactored and validated
- [ ] css_extraction.rs refactored and validated
- [ ] spider/core.rs refactored and validated
- [ ] persistence/state.rs refactored and validated

### Phase 2 (Weeks 3-4)
- [ ] pdf/processor.rs refactored
- [ ] monitor.rs refactored
- [ ] reports.rs refactored
- [ ] instance_pool/pool.rs refactored
- [ ] workers/processors.rs refactored

### Phase 3 (Weeks 5-6)
- [ ] 10 medium-priority files refactored

### Phase 4 (Week 7)
- [ ] CI/CD quality gates implemented
- [ ] Documentation updated
- [ ] Performance validated
- [ ] All tests passing

### Final Validation
- [ ] Zero files >600 LOC (excluding auto-generated)
- [ ] Zero clippy warnings
- [ ] All tests passing
- [ ] Documentation complete
- [ ] Performance benchmarks green
- [ ] Code review completed

---

## 📚 Resources

### Documentation
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Clippy Lint List](https://rust-lang.github.io/rust-clippy/master/)
- [Rust Module System](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)

### Tools
- `cargo-modules` - Visualize module structure
- `cargo-bloat` - Analyze binary size
- `cargo-expand` - View macro expansions
- `cargo-watch` - Auto-run checks on file changes

### Commands Reference
```bash
# Development workflow
cargo watch -x check -x test -x clippy

# Release validation
cargo build --release
cargo test --release
cargo bench

# Documentation
cargo doc --open --no-deps

# Coverage (requires cargo-tarpaulin)
cargo tarpaulin --all-features --workspace --timeout 300 --out Html
```

---

## 🚀 Getting Started

1. **Review this plan** with the team
2. **Set up tooling** (pre-commit hooks, CI/CD)
3. **Start with Phase 1.1** (metrics.rs)
4. **Follow the quality gates** religiously
5. **Track progress** weekly
6. **Adjust timeline** based on learnings

**Let's build better, more maintainable code!** 🎯
