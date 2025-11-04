# Unused Dependencies Analysis - Post riptide-core Elimination

**Analysis Date:** 2025-11-01
**Context:** P2-F1 Day 6 - riptide-core elimination completed
**Scope:** Workspace-wide dependency audit for potentially unused crates

## Executive Summary

Following the elimination of `riptide-core` and redistribution of functionality to specialized crates, this analysis identifies dependencies that may no longer be actively used in the codebase.

## Methodology

1. ✅ Reviewed all `Cargo.toml` files in workspace
2. ✅ Searched for actual usage patterns in source code
3. ✅ Analyzed dependency tree with `cargo tree`
4. ✅ Cross-referenced with feature flags
5. ✅ Examined dead code from refactoring patterns

## Findings

### HIGH CONFIDENCE - Likely Unused

#### 1. **spider** (workspace dependency)
- **Location:** Workspace `Cargo.toml` line 73
- **Declared Version:** `"2"`
- **Usage Found:** Only in 3 files:
  - `/workspaces/eventmesh/crates/riptide-facade/src/facades/mod.rs`
  - `/workspaces/eventmesh/crates/riptide-extraction/src/lib.rs`
  - `/workspaces/eventmesh/crates/riptide-config/src/lib.rs`
- **Assessment:** Workspace dependency declared but minimal/no actual usage
- **Confidence:** 85% - Need to verify if `spider` crate (not `spider_chrome`) is actually imported
- **Action:** Review actual imports vs. workspace declaration

#### 2. **governor** (workspace dependency)
- **Location:** Workspace `Cargo.toml` line 87
- **Declared Version:** `"0.6"`
- **Usage Found:** NO source files use `governor::`
- **Feature Flag:** Used in `riptide-performance` crate feature `resource-limits`
- **Assessment:** Feature-gated but potentially unused
- **Confidence:** 75% - Need to verify if feature is actively used
- **Action:** Check if `resource-limits` feature is ever enabled

#### 3. **tiktoken-rs** (riptide-extraction only)
- **Location:** `crates/riptide-extraction/Cargo.toml` line 33
- **Declared Version:** `"0.5"`
- **Usage Found:** NO source files contain `use tiktoken_rs`
- **Assessment:** Declared in chunking feature but no actual usage found
- **Confidence:** 90% - Strong candidate for removal
- **Action:** Verify chunking implementation doesn't require it

### MEDIUM CONFIDENCE - Conditionally Used

#### 4. **csv** (multiple crates)
- **Location:**
  - `crates/riptide-extraction/Cargo.toml` line 47
  - `crates/riptide-cli/Cargo.toml` line 43
- **Declared Version:** `"1.3"`
- **Usage Found:** Only in `crates/riptide-extraction/src/tables/parser.rs`
- **Assessment:** Used only in table extraction feature
- **Confidence:** 60% - Used but limited scope
- **Action:** Verify table-extraction feature flag dependency

#### 5. **xml** (riptide-spider and riptide-fetch)
- **Location:**
  - `crates/riptide-spider/Cargo.toml` line 50
  - `crates/riptide-fetch/Cargo.toml` line 53
- **Declared Version:** `"0.8"`
- **Usage Found:** Only in `crates/riptide-spider/src/sitemap.rs`
- **Assessment:** Used for sitemap parsing
- **Confidence:** 50% - Actively used but could be feature-gated
- **Action:** Consider making sitemap support optional

#### 6. **robotstxt** (riptide-spider and riptide-fetch)
- **Location:**
  - `crates/riptide-spider/Cargo.toml` line 32
  - `crates/riptide-fetch/Cargo.toml` line 52
- **Declared Version:** `"0.2"`
- **Usage Found:**
  - `crates/riptide-fetch/src/robots.rs`
  - `crates/riptide-spider/src/robots.rs`
- **Assessment:** Used in both crates for robots.txt handling
- **Confidence:** 40% - Duplicate dependency across crates
- **Action:** Consolidate to single crate or feature-gate

### LOW CONFIDENCE - Likely Still Used

#### 7. **wasmtime/wasmtime-wasi** (multiple crates)
- **Location:** Multiple crates with WASM features
- **Assessment:** Marked as optional in Phase 1 WASM refactoring
- **Usage Found:** 30+ files use `wasmtime::`
- **Confidence:** 20% - Actively used but now optional
- **Action:** Already properly feature-gated, no action needed

#### 8. **prometheus** (riptide-api, riptide-persistence, riptide-monitoring)
- **Usage Found:** 7 source files actively use `prometheus::`
- **Confidence:** 10% - Actively used
- **Action:** No action needed

#### 9. **redis** (multiple crates)
- **Usage Found:** 13 source files actively use `use.*redis`
- **Confidence:** 5% - Core infrastructure dependency
- **Action:** No action needed

#### 10. **hdrhistogram** (workspace dependency)
- **Location:** Workspace `Cargo.toml` line 82
- **Usage Found:** 2 files in monitoring/telemetry
- **Confidence:** 15% - Used for metrics
- **Action:** No action needed

### VERY LOW CONFIDENCE - Enterprise Features (Removed)

These were intentionally removed in prior cleanup:
- ❌ `chromiumoxide` (replaced by `spider_chromiumoxide_cdp`)
- ❌ `riptide-core` references (106 files still have commented/doc references)

## Recommended Actions

### Immediate Actions (High Priority)

1. **Remove `tiktoken-rs` from riptide-extraction**
   ```bash
   # Remove from Cargo.toml line 33
   # Verify chunking still works without it
   ```

2. **Investigate `spider` workspace dependency**
   ```bash
   # Check if spider crate is different from spider_chrome
   # Remove if unused
   ```

3. **Review `governor` usage**
   ```bash
   # Check if resource-limits feature is ever enabled
   # Consider removing if unused
   ```

### Secondary Actions (Medium Priority)

4. **Consolidate `robotstxt` dependency**
   - Currently in both `riptide-spider` and `riptide-fetch`
   - Consider moving to single location

5. **Feature-gate `xml` for sitemap support**
   - Make sitemap parsing an optional feature
   - Reduce default dependency footprint

6. **Feature-gate `csv` for table extraction**
   - Already has table-extraction feature
   - Ensure csv is properly marked as optional

### Cleanup Actions (Low Priority)

7. **Remove stale `riptide-core` references**
   - 106 files still reference in comments/docs
   - Update documentation to reflect new architecture

8. **Audit feature flag dependencies**
   - Ensure all conditional features properly mark deps as optional
   - Review `riptide-api` full feature set

## Feature Flag Analysis

### riptide-extraction Features
```toml
default = ["css-extraction", "regex-extraction", "dom-utils", "chunking", "native-parser"]
native-parser = []           # ✅ Properly gated
wasm-extractor = ["dep:wasmtime", "dep:wasmtime-wasi"]  # ✅ Properly gated
table-extraction = []        # ⚠️ csv dependency not marked optional
```

### riptide-api Features
```toml
jemalloc = ["riptide-performance/jemalloc", "tikv-jemallocator", "tikv-jemalloc-ctl"]
profiling-full = ["jemalloc", "riptide-performance/bottleneck-analysis-full"]
# ⚠️ Check if these features are ever used in production
```

### riptide-performance Features
```toml
memory-profiling = ["tikv-jemalloc-ctl", "pprof", "memory-stats"]
resource-limits = ["governor"]  # ⚠️ Check if governor is actually used
```

## Dependencies by Confidence Level

### Remove (85-90% confidence)
1. tiktoken-rs (riptide-extraction)
2. spider workspace dependency (if confirmed unused)

### Investigate (60-75% confidence)
1. governor (riptide-performance)
2. csv table dependencies (feature-gate properly)

### Consolidate (40-50% confidence)
1. robotstxt (duplicate in two crates)
2. xml (consider feature-gating)

### Keep (0-20% confidence)
1. wasmtime/wasmtime-wasi (properly optional)
2. prometheus (actively used)
3. redis (core infrastructure)
4. hdrhistogram (monitoring)
5. All other workspace dependencies

## Testing Plan

Before removing any dependency:

1. ✅ Run full test suite: `cargo test --all-features`
2. ✅ Run without feature: `cargo test --no-default-features`
3. ✅ Check documentation builds: `cargo doc --all-features`
4. ✅ Run clippy: `cargo clippy --all-targets --all-features`
5. ✅ Verify CI pipeline passes

## Notes

- This analysis focuses on post-riptide-core elimination landscape
- Many dependencies are feature-gated, which is correct behavior
- Some "unused" dependencies may be required by transitive deps
- Always verify with `cargo tree` before removal
- Consider using `cargo-udeps` for automated detection:
  ```bash
  cargo install cargo-udeps
  cargo +nightly udeps --all-targets
  ```

## References

- P2-F1 Day 6 Completion Report: riptide-core eliminated
- WASM Phase 1 Refactoring: wasmtime made optional
- Phase 4 P0 Optimizations: spider_chrome migration complete
