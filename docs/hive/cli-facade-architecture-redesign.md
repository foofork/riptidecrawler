# CLI-Facade Architecture Redesign: Separation of Concerns

**Status**: Architecture Design & Analysis
**Created**: 2025-10-21
**Purpose**: Design ideal separation between CLI user interface and library orchestration logic

---

## Executive Summary

**Problem**: The CLI crate contains **13,782 lines** of mixed concerns - CLI argument parsing, output formatting, business logic, orchestration, and execution code. The facade crate (**4,101 lines**) underutilizes its role as the high-level orchestration layer.

**Solution**: Establish clear boundaries where CLI handles only user interaction (arguments, output, progress) while facade handles all orchestration, coordination, and business logic.

**Impact**:
- **~8,000-10,000 lines** of orchestration logic should migrate from CLI → facade/new crates
- **~3,000-4,000 lines** remain in CLI for pure UI concerns
- Improved testability, reusability, and maintainability

---

## Current Architecture Assessment

### Crate Structure Overview

```
riptide-cli/                     13,782 LOC total
├── src/commands/                ~11,000 LOC (30 command files)
│   ├── extract.rs               972 LOC  ❌ Mixed CLI + orchestration
│   ├── render.rs                980 LOC  ❌ Mixed CLI + orchestration
│   ├── browser_pool_manager.rs  452 LOC  ❌ Should be library crate
│   ├── optimized_executor.rs    615 LOC  ❌ Should be facade/library
│   ├── engine_cache.rs          ~300 LOC ❌ Should be library crate
│   ├── wasm_aot_cache.rs        ~400 LOC ❌ Should be library crate
│   ├── adaptive_timeout.rs      ~350 LOC ❌ Should be library crate
│   ├── performance_monitor.rs   ~300 LOC ❌ Should be library crate
│   └── wasm_cache.rs            ~250 LOC ❌ Should be library crate
├── src/main.rs                  ~500 LOC  ✅ Appropriate
├── src/lib.rs                   ~200 LOC  ✅ Appropriate
├── src/output.rs                ~400 LOC  ✅ Appropriate (CLI concern)
└── src/metrics.rs               ~500 LOC  ⚠️  Should delegate to monitoring

riptide-facade/                  4,101 LOC total
├── src/facades/                 ~2,500 LOC
│   ├── scraper.rs               114 LOC   ✅ Good abstraction
│   ├── extractor.rs             715 LOC   ✅ Good abstraction
│   ├── browser.rs               ~400 LOC  ✅ Good abstraction
│   ├── spider.rs                ~300 LOC  ✅ Good abstraction
│   └── pipeline.rs              ~500 LOC  ✅ Good abstraction
├── src/builder.rs               237 LOC   ✅ Good pattern
├── src/config.rs                ~300 LOC  ✅ Appropriate
└── src/error.rs                 ~150 LOC  ✅ Appropriate
```

### Problem Categories

#### ❌ Category 1: Business Logic in CLI (High Priority)
**Files**: `extract.rs`, `render.rs`, `optimized_executor.rs`

```rust
// CURRENT BAD: CLI contains orchestration logic
// File: riptide-cli/src/commands/extract.rs

pub async fn execute_local_extraction(args: ExtractArgs, ...) -> Result<()> {
    // ❌ HTTP client creation (business logic)
    let client_builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(30));

    // ❌ Stealth controller logic (business logic)
    let mut stealth_controller = StealthController::from_preset(preset);

    // ❌ WASM extractor initialization (business logic)
    let extractor = WasmExtractor::new(&wasm_path).await?;

    // ❌ Headless browser launching (business logic)
    let launcher = HeadlessLauncher::with_config(config).await?;

    // ✅ Output formatting (appropriate for CLI)
    output::print_success("Content extracted");
}
```

**What's Wrong**:
- CLI directly instantiates `WasmExtractor`, `HeadlessLauncher`, `StealthController`
- CLI makes business logic decisions (engine selection, timeout calculation)
- CLI handles HTTP requests and browser automation
- CLI contains 500+ lines of extraction orchestration per command

#### ❌ Category 2: Caching/Optimization Modules in CLI
**Files**: `engine_cache.rs`, `wasm_aot_cache.rs`, `wasm_cache.rs`, `adaptive_timeout.rs`

```rust
// CURRENT BAD: Performance modules in CLI crate
// File: riptide-cli/src/commands/engine_cache.rs

pub struct EngineSelectionCache {
    // ❌ This is library functionality, not CLI
    cache: Arc<RwLock<HashMap<String, CachedDecision>>>,
}

impl EngineSelectionCache {
    pub async fn store(&self, domain: &str, engine: Engine, ...) { ... }
    pub async fn get(&self, domain: &str) -> Option<Engine> { ... }
}
```

**What's Wrong**:
- Performance optimization modules belong in library crates
- CLI should consume these via facade, not implement them
- ~2,000 LOC of cache/optimization code pollutes CLI namespace

#### ❌ Category 3: Pool Management in CLI
**Files**: `browser_pool_manager.rs`

```rust
// CURRENT BAD: Infrastructure management in CLI
// File: riptide-cli/src/commands/browser_pool_manager.rs

pub struct BrowserPoolManager {
    pool: Arc<BrowserPool>,
    stats: Arc<RwLock<ResourceStats>>,
    health_checker: Arc<Mutex<HealthChecker>>,
    // ❌ 452 lines of pool management logic in CLI
}
```

**What's Wrong**:
- Browser pool management is library infrastructure
- CLI should use pool via facade, not manage it directly
- Health checking, pre-warming, statistics belong in library layer

#### ⚠️ Category 4: Weak Facade Utilization
**Current State**: Facade provides good abstractions but CLI bypasses them

```rust
// CURRENT: CLI bypasses facade for direct crate access
use riptide_extraction::wasm_extraction::WasmExtractor;  // ❌ Direct import
use riptide_browser::launcher::HeadlessLauncher;          // ❌ Direct import
use riptide_stealth::StealthController;                   // ❌ Direct import

// SHOULD BE: CLI uses only facade
use riptide_facade::prelude::*;                            // ✅ Facade only
```

---

## Target Architecture Design

### Principle: Three-Layer Separation

```
┌────────────────────────────────────────────────────────┐
│                    CLI LAYER                           │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Responsibilities:                                │  │
│  │  • Argument parsing (clap)                       │  │
│  │  • Output formatting (colored, tables)           │  │
│  │  • Progress indicators (indicatif)               │  │
│  │  • User interaction (prompts, confirmations)     │  │
│  │  • Error display to user                         │  │
│  └──────────────────────────────────────────────────┘  │
│         ↓ Calls facade methods only ↓                  │
└────────────────────────────────────────────────────────┘
                        ↓
┌────────────────────────────────────────────────────────┐
│                  FACADE LAYER                          │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Responsibilities:                                │  │
│  │  • High-level orchestration APIs                 │  │
│  │  • Coordination between crates                   │  │
│  │  • Business logic decisions                      │  │
│  │  • Strategy selection                            │  │
│  │  • Resource lifecycle management                 │  │
│  │  • Error translation to domain errors            │  │
│  └──────────────────────────────────────────────────┘  │
│         ↓ Uses specialized library crates ↓            │
└────────────────────────────────────────────────────────┘
                        ↓
┌────────────────────────────────────────────────────────┐
│               LIBRARY CRATES                           │
│  ┌──────────────────────────────────────────────────┐  │
│  │  riptide-extraction     (WASM, CSS, extraction)  │  │
│  │  riptide-browser        (pool, launcher, CDP)    │  │
│  │  riptide-cache          (caching strategies)     │  │
│  │  riptide-optimization   (NEW: perf modules)      │  │
│  │  riptide-stealth        (anti-detection)         │  │
│  │  riptide-fetch          (HTTP client)            │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
```

### Component Placement Matrix

| Component | Current Location | Target Location | Reasoning |
|-----------|-----------------|-----------------|-----------|
| **Argument Parsing** | CLI ✅ | CLI | User interface concern |
| **Output Formatting** | CLI ✅ | CLI | User interface concern |
| **Progress Bars** | CLI ✅ | CLI | User interface concern |
| **Engine Selection Logic** | CLI ❌ | Facade | Business logic |
| **WASM Orchestration** | CLI ❌ | Facade | Orchestration |
| **Browser Launching** | CLI ❌ | Facade | Orchestration |
| **Stealth Configuration** | CLI ❌ | Facade | Business logic |
| **BrowserPoolManager** | CLI ❌ | riptide-browser | Infrastructure |
| **EngineCache** | CLI ❌ | riptide-optimization (NEW) | Performance |
| **WasmAotCache** | CLI ❌ | riptide-optimization (NEW) | Performance |
| **AdaptiveTimeout** | CLI ❌ | riptide-optimization (NEW) | Performance |
| **PerformanceMonitor** | CLI ❌ | riptide-monitoring ✅ | Already exists |
| **OptimizedExecutor** | CLI ❌ | Facade | Orchestration hub |

---

## Detailed Migration Plan

### Phase 1: Create New Optimization Crate

**New Crate**: `riptide-optimization`

**Purpose**: Centralize performance optimization modules currently scattered in CLI

```rust
// crates/riptide-optimization/src/lib.rs

pub mod engine_cache;        // From CLI
pub mod wasm_aot_cache;      // From CLI
pub mod wasm_cache;          // From CLI
pub mod adaptive_timeout;    // From CLI

pub use engine_cache::EngineSelectionCache;
pub use wasm_aot_cache::WasmAotCache;
pub use wasm_cache::WasmCache;
pub use adaptive_timeout::AdaptiveTimeoutManager;

// Unified optimization facade
pub struct OptimizationManager {
    pub engine_cache: Arc<EngineSelectionCache>,
    pub wasm_aot: Arc<WasmAotCache>,
    pub wasm_cache: Arc<WasmCache>,
    pub timeout_mgr: Arc<AdaptiveTimeoutManager>,
}
```

**Migration**:
```bash
# Move files
mv cli/src/commands/engine_cache.rs optimization/src/
mv cli/src/commands/wasm_aot_cache.rs optimization/src/
mv cli/src/commands/wasm_cache.rs optimization/src/
mv cli/src/commands/adaptive_timeout.rs optimization/src/
```

### Phase 2: Enhance Facade with Orchestration

**Add to Facade**: `riptide-facade/src/facades/executor.rs`

```rust
// NEW FILE: riptide-facade/src/facades/executor.rs
//! Unified execution facade - replaces OptimizedExecutor from CLI

use crate::config::RiptideConfig;
use riptide_browser::launcher::HeadlessLauncher;
use riptide_extraction::wasm_extraction::WasmExtractor;
use riptide_optimization::OptimizationManager;
use riptide_stealth::StealthController;

/// High-level execution facade for content extraction
pub struct ExecutorFacade {
    config: RiptideConfig,
    optimization: OptimizationManager,
    browser_launcher: Option<HeadlessLauncher>,
    wasm_extractor: Option<WasmExtractor>,
}

impl ExecutorFacade {
    /// Create new executor facade
    pub async fn new(config: RiptideConfig) -> Result<Self> {
        let optimization = OptimizationManager::new().await?;
        Ok(Self {
            config,
            optimization,
            browser_launcher: None,
            wasm_extractor: None,
        })
    }

    /// Extract content from URL with automatic engine selection
    pub async fn extract(&self, url: &str, options: ExtractionOptions)
        -> Result<ExtractionResult>
    {
        // 🎯 ALL orchestration logic here (not in CLI)
        let engine = self.select_engine(url, options).await?;

        match engine {
            Engine::Wasm => self.extract_wasm(url, options).await,
            Engine::Headless => self.extract_headless(url, options).await,
            Engine::Raw => self.extract_raw(url, options).await,
        }
    }

    /// Render page with browser
    pub async fn render(&self, url: &str, options: RenderOptions)
        -> Result<RenderResult>
    {
        // 🎯 ALL rendering logic here (not in CLI)
        let launcher = self.get_or_init_launcher().await?;
        let session = launcher.launch_page(url, options.stealth).await?;

        // Capture outputs based on options
        let mut result = RenderResult::new();

        if options.html {
            result.html = Some(session.page().content().await?);
        }

        if options.screenshot {
            result.screenshot = Some(session.page().screenshot().await?);
        }

        Ok(result)
    }

    // 🎯 Internal orchestration methods (hidden from CLI)
    async fn select_engine(&self, url: &str, options: ExtractionOptions)
        -> Result<Engine>
    {
        // Check cache
        if let Some(cached) = self.optimization.engine_cache.get(url).await {
            return Ok(cached);
        }

        // Auto-detect or use explicit
        let engine = if let Some(explicit) = options.engine {
            explicit
        } else {
            self.auto_detect_engine(url).await?
        };

        // Cache decision
        self.optimization.engine_cache.store(url, engine, 0.9).await?;
        Ok(engine)
    }
}
```

### Phase 3: Simplify CLI Commands

**NEW CLI Pattern**: Thin wrappers around facade

```rust
// crates/riptide-cli/src/commands/extract.rs (AFTER refactor)
// FROM: 972 lines → TO: ~150 lines

use riptide_facade::prelude::*;  // ✅ Only facade import
use crate::output;                // ✅ CLI concern

pub async fn execute(args: ExtractArgs, output_format: &str) -> Result<()> {
    // ✅ Parse CLI args into facade options
    let options = ExtractionOptions {
        url: args.url.as_ref(),
        engine: parse_engine(&args.engine)?,
        show_confidence: args.show_confidence,
        metadata: args.metadata,
        stealth: parse_stealth(&args.stealth_level),
        timeout: Duration::from_millis(args.init_timeout_ms),
    };

    // ✅ Create facade (or use global instance)
    let config = RiptideConfig::from_env()?;
    let executor = ExecutorFacade::new(config).await?;

    // ✅ Call facade - ALL logic is in facade now
    let result = executor.extract(&args.url.unwrap(), options).await?;

    // ✅ Format output (CLI responsibility)
    output_extraction_result(&result, output_format, &args)?;

    // ✅ Save to file if requested (CLI responsibility)
    if let Some(ref file_path) = args.file {
        fs::write(file_path, &result.content)?;
        output::print_success(&format!("Saved to: {}", file_path));
    }

    Ok(())
}

// ✅ Small helper to map CLI args to facade types
fn parse_engine(s: &str) -> Result<Option<Engine>> {
    match s {
        "auto" => Ok(None),
        "wasm" => Ok(Some(Engine::Wasm)),
        "headless" => Ok(Some(Engine::Headless)),
        "raw" => Ok(Some(Engine::Raw)),
        _ => Err(anyhow!("Invalid engine: {}", s))
    }
}

// ✅ Format output (pure UI logic, stays in CLI)
fn output_extraction_result(
    result: &ExtractionResult,
    format: &str,
    args: &ExtractArgs
) -> Result<()> {
    match format {
        "json" => output::print_json(result),
        "text" => {
            output::print_success("Extraction complete");
            if args.show_confidence {
                output::print_key_value("Confidence",
                    &format!("{:.1}%", result.confidence * 100.0));
            }
            println!("\n{}", result.content);
        }
        "table" => {
            let table = output::create_table(vec!["Field", "Value"]);
            table.add_row(vec!["URL", &result.url]);
            table.add_row(vec!["Method", &result.method]);
            println!("{table}");
        }
        _ => output::print_json(result),
    }
    Ok(())
}
```

### Phase 4: Move Browser Pool to Library

**Target**: `riptide-browser` crate should own pool management

```rust
// crates/riptide-browser/src/pool_manager.rs (NEW location)
// Move from: cli/src/commands/browser_pool_manager.rs

pub struct BrowserPoolManager {
    pool: Arc<BrowserPool>,
    config: PoolManagerConfig,
    // ... all pool management logic
}

// Expose via riptide-browser/src/lib.rs
pub use pool_manager::BrowserPoolManager;
```

**Facade Integration**:
```rust
// riptide-facade/src/facades/browser.rs

use riptide_browser::BrowserPoolManager;  // ✅ From library

impl BrowserFacade {
    async fn new(config: RiptideConfig) -> Result<Self> {
        let pool_config = PoolManagerConfig::from_config(&config);
        let pool_manager = BrowserPoolManager::new(pool_config).await?;

        Ok(Self {
            pool_manager: Arc::new(pool_manager),
            // ...
        })
    }
}
```

---

## Dependency Architecture

### BEFORE (Current - Problematic)

```
riptide-cli
    ├─> clap, colored, indicatif        ✅ CLI dependencies
    ├─> riptide-extraction              ❌ Direct library access
    ├─> riptide-browser                 ❌ Direct library access
    ├─> riptide-stealth                 ❌ Direct library access
    ├─> riptide-pdf                     ❌ Direct library access
    └─> riptide-monitoring              ❌ Direct library access

riptide-facade
    ├─> riptide-types                   ✅ Appropriate
    ├─> riptide-fetch                   ✅ Appropriate
    └─> riptide-extraction              ✅ Appropriate
```

**Problems**:
- CLI has 8+ direct library dependencies
- Facade is bypassed for most operations
- No clear separation of concerns

### AFTER (Target - Clean)

```
riptide-cli
    ├─> clap, colored, indicatif        ✅ CLI dependencies only
    └─> riptide-facade                  ✅ ONLY facade dependency

riptide-facade
    ├─> riptide-extraction              ✅ Orchestrates extraction
    ├─> riptide-browser                 ✅ Orchestrates browsing
    ├─> riptide-optimization (NEW)      ✅ Orchestrates optimization
    ├─> riptide-stealth                 ✅ Orchestrates stealth
    ├─> riptide-cache                   ✅ Orchestrates caching
    └─> riptide-monitoring              ✅ Orchestrates monitoring

riptide-optimization (NEW)
    ├─> riptide-cache                   ✅ Uses caching
    └─> riptide-types                   ✅ Shared types

riptide-browser
    ├─> spider_chrome                   ✅ CDP implementation
    └─> riptide-stealth                 ✅ Browser stealth
```

**Benefits**:
- CLI has exactly 1 library dependency (facade)
- All orchestration through facade
- Clear layered architecture

---

## Benefits Analysis

### Code Quality Benefits

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| **CLI LoC** | 13,782 | ~4,000 | -71% |
| **CLI Dependencies** | 8 crates | 1 crate | -87% |
| **CLI Test Complexity** | High (mocking 8 crates) | Low (mocking facade) | 8x easier |
| **Facade LoC** | 4,101 | ~10,000 | +143% (appropriate) |
| **Facade Responsibilities** | Limited | Comprehensive | Better design |
| **Reusability** | CLI-locked | Library-first | ∞ improvement |

### Maintainability Benefits

```diff
# BEFORE: Testing CLI command requires complex setup
#[tokio::test]
async fn test_extract_command() {
    // ❌ Need to mock 8 different subsystems
    let mock_wasm = MockWasm::new();
    let mock_browser = MockBrowser::new();
    let mock_stealth = MockStealth::new();
    let mock_cache = MockCache::new();
    // ... 4 more mocks

    // ❌ Complex test setup
    let result = execute_local_extraction(args, ...).await;
}

# AFTER: Testing CLI is simple
#[tokio::test]
async fn test_extract_command() {
    // ✅ Mock only the facade
    let mock_facade = MockExecutorFacade::new()
        .with_result(ExtractionResult { ... });

    // ✅ Simple, focused test
    let result = execute(args, mock_facade).await;
    assert_eq!(result.content, "expected");
}
```

### Reusability Benefits

**BEFORE**: Cannot reuse extraction logic outside CLI
```rust
// ❌ Tight coupling to CLI args and output
fn execute_local_extraction(args: ExtractArgs, output_format: &str) -> Result<()>
```

**AFTER**: Library users can use facade directly
```rust
// ✅ Python bindings can use facade
#[pyfunction]
fn extract_content(url: &str, options: PyDict) -> PyResult<String> {
    let executor = ExecutorFacade::new(config)?;
    let result = executor.extract(url, options.into())?;
    Ok(result.content)
}

// ✅ WebAssembly can use facade
#[wasm_bindgen]
pub async fn extract(url: &str) -> JsValue {
    let executor = ExecutorFacade::new(default_config())?;
    let result = executor.extract(url, Default::default()).await?;
    serde_wasm_bindgen::to_value(&result)?
}

// ✅ API server can use facade
async fn api_extract(Json(req): Json<ExtractRequest>) -> Json<ExtractResponse> {
    let executor = ExecutorFacade::new(config).await?;
    let result = executor.extract(&req.url, req.options).await?;
    Json(result)
}
```

---

## Migration Strategy

### Step-by-Step Migration Path

#### **Step 1: Create riptide-optimization crate** ⏱️ 2-3 hours

```bash
# 1. Create new crate
cargo new --lib crates/riptide-optimization

# 2. Move optimization modules
mv crates/riptide-cli/src/commands/engine_cache.rs crates/riptide-optimization/src/
mv crates/riptide-cli/src/commands/wasm_aot_cache.rs crates/riptide-optimization/src/
mv crates/riptide-cli/src/commands/wasm_cache.rs crates/riptide-optimization/src/
mv crates/riptide-cli/src/commands/adaptive_timeout.rs crates/riptide-optimization/src/

# 3. Create unified module
cat > crates/riptide-optimization/src/lib.rs <<EOF
pub mod engine_cache;
pub mod wasm_aot_cache;
pub mod wasm_cache;
pub mod adaptive_timeout;

pub use engine_cache::EngineSelectionCache;
pub use wasm_aot_cache::WasmAotCache;
pub use wasm_cache::WasmCache;
pub use adaptive_timeout::AdaptiveTimeoutManager;
EOF

# 4. Update workspace
# Add to Cargo.toml: "crates/riptide-optimization"
```

#### **Step 2: Move BrowserPoolManager to riptide-browser** ⏱️ 1-2 hours

```bash
# 1. Move pool manager
mv crates/riptide-cli/src/commands/browser_pool_manager.rs \
   crates/riptide-browser/src/pool_manager.rs

# 2. Expose in riptide-browser
# Add to riptide-browser/src/lib.rs:
# pub mod pool_manager;
# pub use pool_manager::BrowserPoolManager;
```

#### **Step 3: Create ExecutorFacade** ⏱️ 4-6 hours

```bash
# 1. Create new facade
touch crates/riptide-facade/src/facades/executor.rs

# 2. Move OptimizedExecutor logic from CLI to ExecutorFacade
# Transform CLI-specific code into library API

# 3. Export from facade
# Add to riptide-facade/src/facades/mod.rs
```

#### **Step 4: Refactor extract command** ⏱️ 3-4 hours

```bash
# 1. Backup current implementation
cp crates/riptide-cli/src/commands/extract.rs \
   crates/riptide-cli/src/commands/extract.rs.backup

# 2. Rewrite to use ExecutorFacade
# Reduce from 972 lines to ~150 lines
# Focus on: arg parsing → facade call → output formatting

# 3. Remove direct library imports
# Keep only: riptide_facade, output, clap, anyhow
```

#### **Step 5: Refactor render command** ⏱️ 3-4 hours

Similar process to extract command

#### **Step 6: Update CLI dependencies** ⏱️ 1 hour

```toml
# crates/riptide-cli/Cargo.toml

[dependencies]
# ✅ CLI-specific only
clap = { workspace = true }
colored = "2.1"
indicatif = "0.17"
comfy-table = "7.1"
dialoguer = "0.11"
anyhow = { workspace = true }
tokio = { workspace = true }
serde_json = { workspace = true }

# ✅ ONE library dependency
riptide-facade = { path = "../riptide-facade" }

# ❌ REMOVE these (now accessed via facade):
# riptide-extraction = ...
# riptide-browser = ...
# riptide-stealth = ...
# riptide-pdf = ...
# spider_chrome = ...
```

#### **Step 7: Test migration** ⏱️ 2-3 hours

```bash
# 1. Run tests
cargo test --package riptide-cli
cargo test --package riptide-facade
cargo test --package riptide-optimization

# 2. Build CLI
cargo build --release --bin riptide

# 3. Smoke tests
./target/release/riptide extract --url https://example.com --local
./target/release/riptide render --url https://example.com --html
```

**Total Migration Time**: ~16-23 hours of focused development

---

## Risk Assessment

### High Risk ⚠️

| Risk | Impact | Mitigation |
|------|--------|-----------|
| **Breaking CLI commands** | High | Maintain CLI arg compatibility; extensive testing |
| **Performance regression** | Medium | Benchmark before/after; optimize facade overhead |
| **Facade API design** | High | Review with team; allow iteration before stabilization |

### Medium Risk ⚡

| Risk | Impact | Mitigation |
|------|--------|-----------|
| **Increased facade complexity** | Medium | Good documentation; clear module boundaries |
| **Dependency cycles** | Medium | Careful crate graph design; use cargo tree |
| **Migration time** | Medium | Phased approach; parallel work on commands |

### Low Risk ✅

| Risk | Impact | Mitigation |
|------|--------|-----------|
| **User-facing changes** | Low | CLI args remain identical |
| **Build time increase** | Low | Parallel compilation helps |
| **Documentation burden** | Low | Auto-generate docs; examples in facade |

---

## Acceptance Criteria

### Must Have ✅

- [ ] CLI crate has exactly 1 library dependency (facade)
- [ ] CLI crate is <5,000 LoC
- [ ] All orchestration logic is in facade or library crates
- [ ] All CLI commands work identically to before
- [ ] All tests pass
- [ ] `cargo clippy` passes with no warnings
- [ ] Performance is within 5% of baseline

### Should Have 🎯

- [ ] Facade provides Python bindings example
- [ ] Facade provides WASM bindings example
- [ ] API server example using facade directly
- [ ] Documentation for facade usage patterns
- [ ] Migration guide for users (if API changes)

### Nice to Have 💎

- [ ] Facade benchmarks
- [ ] Facade integration tests
- [ ] CLI smoke test suite
- [ ] Performance comparison report

---

## Conclusion

### Key Takeaways

1. **Clear Separation**: CLI = UI, Facade = Orchestration, Libraries = Implementation
2. **~70% Reduction** in CLI complexity (13,782 → 4,000 LoC)
3. **New Crate Needed**: `riptide-optimization` for performance modules
4. **Facade Growth**: From 4,101 → ~10,000 LoC (appropriate expansion)
5. **Better Reusability**: Facade becomes primary library API

### Next Steps

1. **Review** this design with team
2. **Approve** crate structure and migration phases
3. **Execute** Step 1 (create riptide-optimization)
4. **Validate** with one command (extract) as prototype
5. **Scale** to all commands once pattern proven

### Success Metrics

- CLI crate LOC: **<5,000** ✅
- CLI library dependencies: **1** (facade only) ✅
- Test coverage: **>80%** for facade ✅
- Performance: **Within 5%** of current ✅
- Reusability: **3+ use cases** (CLI, Python, WASM) ✅

---

**Document Status**: Ready for Review
**Next Action**: Team review and approval of architecture
**Estimated Total Effort**: 16-23 hours focused development
