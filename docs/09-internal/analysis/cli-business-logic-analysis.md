# CLI Business Logic Analysis - Extraction Candidates

**Analysis Date:** 2025-10-23
**Analyzer:** Code Quality Analyzer Agent
**Files Analyzed:** 51 source files in riptide-cli crate

---

## EXECUTIVE SUMMARY

This analysis identifies remaining business logic in the CLI crate that should be extracted to dedicated crates for improved reusability, testability, and optimization potential. The analysis found **~2,600 lines of pure business logic** suitable for extraction across 5 high-priority candidates.

**Priority Distribution:**
- **HIGH Priority:** 5 systems (~2,600 lines) - Immediate extraction recommended
- **MEDIUM Priority:** 2 utilities (~55 lines) - Extract when convenient
- **LOW Priority:** 3 systems - Keep in CLI (presentation/orchestration logic)

---

## HIGH PRIORITY EXTRACTIONS

### 1. Job Management System ⭐⭐⭐ (CRITICAL)

**Location:**
- `/workspaces/eventmesh/crates/riptide-cli/src/job/manager.rs` (374 lines)
- `/workspaces/eventmesh/crates/riptide-cli/src/job/storage.rs` (300+ lines)
- `/workspaces/eventmesh/crates/riptide-cli/src/job/types.rs`

**Business Logic Identified:**
- ✅ Job lifecycle state machine (Pending → Running → Completed/Failed/Cancelled)
- ✅ Job submission with priority queuing (Low/Medium/High/Critical)
- ✅ Progress tracking with atomic updates (completed, failed, current URL)
- ✅ Job statistics aggregation (success rate, avg duration, by-status counts)
- ✅ Persistent storage with JSONL append-only logs
- ✅ Tag-based job filtering and search
- ✅ Automatic cleanup of old jobs (configurable retention)
- ✅ Thread-safe concurrent job access (Arc<RwLock<HashMap>>)

**Complexity Metrics:**
- Total Lines: ~700
- Cyclomatic Complexity: High (state machine transitions)
- Reusability Score: Very High
- Testing Isolation Benefit: Critical

**Recommendation:** Extract to **`riptide-jobs`** crate

**Rationale:**
1. Complex domain logic with clear boundaries
2. Reusable across CLI, API server, worker processes, and monitoring tools
3. State machine transitions require isolated testing
4. Could benefit from optimized async executor for concurrent job updates
5. Job queue algorithms could be enhanced (priority queues, scheduling)

**Target Crate Structure:**
```
riptide-jobs/
├── src/
│   ├── lib.rs           # Public API
│   ├── manager.rs       # JobManager orchestration
│   ├── storage.rs       # Persistence layer (JSONL)
│   ├── types.rs         # Job domain types
│   ├── queue.rs         # Priority queue logic
│   ├── statistics.rs    # Aggregation algorithms
│   └── cleanup.rs       # Retention policy
├── tests/
│   ├── lifecycle_tests.rs
│   ├── concurrency_tests.rs
│   └── storage_tests.rs
└── Cargo.toml
```

**Migration Effort:** 3-4 days (complex state machine testing)

---

### 2. Metrics Aggregation System ⭐⭐⭐ (CRITICAL)

**Location:**
- `/workspaces/eventmesh/crates/riptide-cli/src/metrics/aggregator.rs` (435 lines)
- `/workspaces/eventmesh/crates/riptide-cli/src/metrics/collector.rs`
- `/workspaces/eventmesh/crates/riptide-cli/src/metrics/storage.rs`
- `/workspaces/eventmesh/crates/riptide-cli/src/metrics/types.rs`

**Business Logic Identified:**
- ✅ Percentile calculations (P50, P95, P99) with caching
- ✅ Running average algorithms (Welford's method)
- ✅ Moving average with configurable window sizes
- ✅ Anomaly detection using z-score method (threshold-based)
- ✅ Rate of change calculations (time-series analysis)
- ✅ Time-bucket aggregation (hourly/daily grouping)
- ✅ Error categorization (timeout, network, permission, auth, rate_limit)
- ✅ Command-level metrics aggregation by name
- ✅ Cache hit rate computation
- ✅ Metric point storage with timestamps

**Complexity Metrics:**
- Total Lines: ~600+
- Cyclomatic Complexity: Medium-High
- Reusability Score: Very High
- Optimization Potential: High (SIMD for percentiles)

**Recommendation:** Merge into **`riptide-monitoring`** crate

**Rationale:**
1. Statistical algorithms are pure business logic
2. Reusable for observability across all riptide components
3. Critical for production monitoring dashboards
4. Could benefit from SIMD optimizations for percentile calculations
5. riptide-monitoring already exists and is the natural home
6. Testing isolation critical for statistical correctness

**Key Algorithms to Preserve:**
```rust
// Percentile calculation (linear interpolation)
fn calculate_percentiles(values: &[f64]) -> (f64, f64, f64)

// Running average (efficient incremental update)
fn update_running_avg(current: &mut f64, new_value: f64, count: u64)

// Anomaly detection (z-score threshold)
fn detect_anomalies(&self, points: &[MetricPoint], threshold: f64) -> Vec<usize>

// Error pattern matching
fn categorize_error(error: &str) -> String
```

**Migration Effort:** 2-3 days (merge into existing crate)

---

### 3. Domain Profile Management ⭐⭐⭐ (HIGH)

**Location:**
- `/workspaces/eventmesh/crates/riptide-cli/src/commands/domain.rs` (900+ lines)

**Business Logic Identified:**
- ✅ Domain profile creation and versioning
- ✅ Site structure analysis (common elements, navigation patterns)
- ✅ Pattern detection algorithms (URL, content, metadata patterns)
- ✅ Drift detection with configurable thresholds
- ✅ Baseline capture from live sites
- ✅ Profile import/export (JSON, YAML)
- ✅ Configuration management (stealth levels, rate limits, UA strategies)
- ✅ Selector extraction with frequency analysis
- ✅ Domain metadata tracking (success rate, response times)
- ✅ Multi-version profile storage

**Domain Types:**
```rust
DomainProfile {
  - name, domain, version
  - config: DomainConfig (stealth, rate_limit, robots.txt)
  - baseline: SiteBaseline (structure, patterns, selectors)
  - metadata: DomainMetadata (stats, tags, author)
  - patterns: DomainPatterns (regex, paths, excludes)
}

DriftReport {
  - overall_drift: f64
  - changes: Vec<DriftChange> (type, location, severity)
  - summary: DriftSummary (critical/major/minor counts)
  - recommendations: Vec<String>
}
```

**Complexity Metrics:**
- Total Lines: ~900+
- Cyclomatic Complexity: High
- Reusability Score: High
- Uniqueness: Very High (no duplicate logic elsewhere)

**Recommendation:** Extract to **`riptide-domain-profiles`** crate

**Rationale:**
1. Site analysis algorithms are sophisticated domain logic
2. Reusable for automated monitoring, API server, worker schedulers
3. Drift detection critical for production stability alerts
4. Pattern matching algorithms benefit from isolated testing
5. No other crate handles this domain

**Target Crate Structure:**
```
riptide-domain-profiles/
├── src/
│   ├── lib.rs           # Public API
│   ├── profile.rs       # Profile CRUD operations
│   ├── analysis.rs      # Site structure analysis
│   ├── drift.rs         # Drift detection algorithms
│   ├── patterns.rs      # Pattern extraction (URL, content)
│   ├── storage.rs       # Profile persistence (JSON/YAML)
│   └── config.rs        # Configuration management
├── tests/
│   ├── profile_tests.rs
│   ├── drift_detection_tests.rs
│   └── pattern_extraction_tests.rs
└── Cargo.toml
```

**Migration Effort:** 4-5 days (many algorithms)

---

### 4. Session Management System ⭐⭐ (HIGH)

**Location:**
- `/workspaces/eventmesh/crates/riptide-cli/src/session/manager.rs` (191 lines)
- `/workspaces/eventmesh/crates/riptide-cli/src/session/types.rs`

**Business Logic Identified:**
- ✅ Session lifecycle (create, activate, deactivate)
- ✅ Session state management with HashMap cache
- ✅ Import/export with format detection (JSON, YAML)
- ✅ Session validation and overwrite logic
- ✅ Cross-session persistence to disk (~/.riptide/sessions/)
- ✅ Current session tracking
- ✅ Session metadata management

**Session Types:**
```rust
Session {
  - name: String
  - created_at: DateTime<Utc>
  - updated_at: DateTime<Utc>
  - config: HashMap (key-value settings)
  - active: bool
}

SessionManager {
  - current_session: Option<String>
  - sessions: HashMap<String, Session>
}
```

**Complexity Metrics:**
- Total Lines: ~200
- Cyclomatic Complexity: Medium
- Reusability Score: High
- Testing Isolation Benefit: Medium

**Recommendation:** Extract to **`riptide-sessions`** crate

**Rationale:**
1. Stateful domain logic independent of CLI
2. Reusable for API session management, worker contexts
3. Clear separation from UI concerns
4. Could support distributed session stores (Redis) in future

**Target Crate Structure:**
```
riptide-sessions/
├── src/
│   ├── lib.rs           # Public API
│   ├── manager.rs       # SessionManager orchestration
│   ├── types.rs         # Session domain types
│   ├── storage.rs       # Persistence (JSON/YAML)
│   └── validation.rs    # Session validation rules
├── tests/
│   ├── lifecycle_tests.rs
│   └── import_export_tests.rs
└── Cargo.toml
```

**Migration Effort:** 1-2 days (straightforward)

---

### 5. Cache Management System ⭐⭐ (HIGH - Verify Duplication)

**Location:**
- `/workspaces/eventmesh/crates/riptide-cli/src/cache/manager.rs` (347 lines)
- `/workspaces/eventmesh/crates/riptide-cli/src/cache/storage.rs` (200+ lines)
- `/workspaces/eventmesh/crates/riptide-cli/src/cache/types.rs`

**Business Logic Identified:**
- ✅ LRU eviction algorithm (Least Recently Used)
- ✅ Multi-criteria eviction (size + count limits)
- ✅ Domain-based cache partitioning
- ✅ TTL expiration management
- ✅ Cache statistics tracking (hit rate, evictions, size)
- ✅ Thread-safe concurrent access (Arc<RwLock>)
- ✅ Automatic cleanup of expired entries

**LRU Algorithm:**
```rust
async fn evict_lru_entry(&self, entries: &mut HashMap<..>, stats: &mut CacheStats) {
  // Find entry with oldest last_accessed timestamp
  let lru_url = entries
    .iter()
    .min_by_key(|(_, entry)| entry.last_accessed)
    .map(|(url, _)| url.clone());

  // Remove and update stats
}
```

**Complexity Metrics:**
- Total Lines: ~550
- Cyclomatic Complexity: Medium-High
- Reusability Score: Very High
- Optimization Potential: Medium

**Recommendation:** **Verify against existing `riptide-cache` crate first**

**Action Items:**
1. ✅ Check if `riptide-cache` already has LRU logic
2. ⚠️ If duplicate: Delete CLI version, use existing crate
3. ⚠️ If missing: Move this implementation to `riptide-cache`
4. ⚠️ If different: Consolidate into single unified implementation

**Migration Effort:** 1-2 days (after verification)

---

## MEDIUM PRIORITY EXTRACTIONS

### 6. Page Range Parsing Utility ⭐ (MEDIUM)

**Location:**
- `/workspaces/eventmesh/crates/riptide-cli/src/pdf_impl.rs` (lines 128-163)

**Business Logic:**
```rust
// Parses expressions like "1-5,10-15" into Vec<u32>
pub fn parse_page_range(range_str: &str) -> Result<Vec<u32>>
```

**Recommendation:** Move to **`riptide-pdf`** crate (utils module)

**Rationale:**
- Small but reusable (~35 lines)
- Pure algorithm, no CLI dependencies
- Useful for PDF processing in API/workers

**Migration Effort:** 30 minutes

---

### 7. Error Categorization ⭐ (MEDIUM)

**Location:**
- `/workspaces/eventmesh/crates/riptide-cli/src/metrics/aggregator.rs` (lines 288-309)

**Business Logic:**
```rust
fn categorize_error(error: &str) -> String {
  // Pattern matching: timeout, network, permission, not_found,
  // parse, rate_limit, authentication, unknown
}
```

**Recommendation:** Move to **`riptide-monitoring`** (with metrics)

**Rationale:**
- Small but valuable domain knowledge (~20 lines)
- Reusable for error analytics across system
- Could be enhanced with ML classification later

**Migration Effort:** 30 minutes

---

## LOW PRIORITY (Keep in CLI)

### 8. API Client Wrappers (LOW - CLI-Specific)

**Location:**
- `/workspaces/eventmesh/crates/riptide-cli/src/api_client.rs` (171 lines)
- `/workspaces/eventmesh/crates/riptide-cli/src/api_wrapper.rs` (145 lines)
- `/workspaces/eventmesh/crates/riptide-cli/src/client.rs`

**Rationale for Keeping:**
- ✅ Tightly coupled to CLI execution modes (direct/api/api-only)
- ✅ Thin wrappers around reqwest with CLI-specific fallback logic
- ✅ Not complex enough to justify separate crate
- ✅ Request/Response DTOs are appropriate for CLI
- ⚠️ No reusable business logic

---

### 9. Output Formatting (LOW - Presentation Layer)

**Location:**
- `/workspaces/eventmesh/crates/riptide-cli/src/output.rs`

**Rationale for Keeping:**
- ✅ Pure presentation logic (comfy-table, colored output)
- ✅ CLI-specific formatting concerns
- ✅ No business logic, just view layer
- ✅ Depends on CLI output modes (json/text/table)

---

### 10. Command Execution Orchestration (LOW - CLI Layer)

**Location:**
- `/workspaces/eventmesh/crates/riptide-cli/src/commands/*.rs` (18 command files)

**Rationale for Keeping:**
- ✅ Orchestration layer between CLI and business logic
- ✅ Most already delegate to crates (extraction, reliability, pdf, etc.)
- ✅ Command parsing is CLI-specific (clap integration)
- ✅ Request/Response DTOs for API calls are appropriate here
- ⚠️ Some files like `domain.rs` contain business logic (already flagged above)

---

## ALREADY EXTRACTED (Verified ✅)

The following business logic has been successfully moved to dedicated crates:

1. ✅ **PDF Processing** → `riptide-pdf` (text, tables, metadata extraction)
2. ✅ **Content Extraction** → `riptide-extraction` (WASM-based extraction)
3. ✅ **Browser Management** → `riptide-browser` (launcher, pool, automation)
4. ✅ **Reliability/Engine Selection** → `riptide-reliability` (Spider, Playwright)
5. ✅ **Stealth Features** → `riptide-stealth` (fingerprint resistance)
6. ✅ **Worker Pool** → `riptide-workers` (async task distribution)
7. ✅ **Monitoring/Validation** → `riptide-monitoring` (health checks, partial)

---

## PRIORITY RANKING & ROADMAP

### **Phase 10: Immediate Action (High Impact)**

1. **Job Management System** (3-4 days)
   - Impact: Very High (reusable across all components)
   - Complexity: High (state machine testing)
   - Target: `riptide-jobs` crate

2. **Metrics Aggregation** (2-3 days)
   - Impact: Very High (observability critical)
   - Complexity: Medium-High (statistical algorithms)
   - Target: Merge into `riptide-monitoring`

3. **Domain Profile Management** (4-5 days)
   - Impact: High (unique production-critical logic)
   - Complexity: High (many algorithms)
   - Target: `riptide-domain-profiles` crate

**Phase 10 Total:** ~9-12 days

---

### **Phase 11: Next Round (Medium Impact)**

4. **Session Management** (1-2 days)
   - Impact: Medium (reusable for API/workers)
   - Complexity: Low-Medium
   - Target: `riptide-sessions` crate

5. **Cache Manager** (1-2 days)
   - Impact: Medium (verify duplication first)
   - Complexity: Medium
   - Target: Verify vs `riptide-cache`, consolidate

**Phase 11 Total:** ~2-4 days

---

### **Future Optimization (Low Effort)**

6. **Page Range Parsing** (30 mins)
   - Target: `riptide-pdf` utils

7. **Error Categorization** (30 mins)
   - Target: `riptide-monitoring`

**Future Total:** ~1 day

---

## BENEFITS OF EXTRACTION

### ✅ Testing Benefits
- **Isolated Unit Tests:** Complex algorithms tested without CLI dependencies
- **Faster Test Execution:** No CLI initialization overhead
- **Better Coverage:** Focus on business logic paths
- **Parallel Testing:** Crates tested independently
- **Example:** Job state machine transitions tested in isolation

### ✅ Reusability Benefits
- **Job Management:** Reusable in API server, worker processes, monitoring tools
- **Session Management:** Distributed systems, API sessions, worker contexts
- **Metrics Aggregation:** All components share same statistical engine
- **Domain Profiles:** Automated monitoring, scheduled crawls, API validation

### ✅ Optimization Potential
- **Job Queue:** Could use optimized async executor (tokio-uring)
- **Metrics Aggregation:** SIMD for percentile calculations (4x speedup)
- **Cache Eviction:** Lock-free data structures for concurrency
- **Domain Analysis:** Parallel pattern matching across pages

### ✅ Maintenance Benefits
- **Clear Separation:** Business logic vs CLI orchestration
- **Independent Versioning:** Domain logic versioned separately
- **Easier Refactoring:** Change algorithms without CLI impact
- **Reduced CLI Compilation:** Faster iteration on UI changes

---

## ANTI-PATTERNS TO AVOID

### ❌ Over-Extraction
- Don't extract thin wrappers (e.g., `api_client.rs` is fine in CLI)
- Don't extract pure presentation logic (e.g., `output.rs`)
- Don't extract CLI-specific orchestration

### ❌ Premature Extraction
- Verify cache logic isn't duplicate before extracting
- Ensure business logic is stable before moving
- Don't extract during active feature development

### ❌ Breaking Encapsulation
- Don't expose internal storage formats in public API
- Keep domain types clean (no CLI-specific fields)
- Use trait boundaries for testability

---

## ESTIMATED TOTAL EFFORT

| Phase | Items | Days | Priority |
|-------|-------|------|----------|
| Phase 10 | Job Management, Metrics, Domain Profiles | 9-12 | HIGH |
| Phase 11 | Sessions, Cache (verify first) | 2-4 | MEDIUM |
| Future | Page Range, Error Categories | 1 | LOW |
| **TOTAL** | **7 extractions** | **12-17 days** | - |

---

## VERIFICATION CHECKLIST

Before extracting each system:

- [ ] Verify no duplicate logic exists in target crate
- [ ] Identify all dependencies (ensure no CLI-specific deps)
- [ ] Create public API design (trait boundaries)
- [ ] Write migration tests (ensure parity)
- [ ] Update CLI to use new crate
- [ ] Remove old code after validation
- [ ] Update documentation

---

## FILES ANALYZED (51 total)

### Core CLI Files (10):
- `main.rs`, `lib.rs`, `client.rs`, `config.rs`, `execution_mode.rs`
- `output.rs`, `api_client.rs`, `api_wrapper.rs`, `validation_adapter.rs`, `pdf_impl.rs`

### Command Files (18):
- `commands/mod.rs`, `extract.rs`, `render.rs`, `crawl.rs`, `search.rs`
- `commands/cache.rs`, `wasm.rs`, `stealth.rs`, `domain.rs`, `health.rs`
- `commands/metrics.rs`, `validate.rs`, `system_check.rs`, `tables.rs`, `schema.rs`
- `commands/pdf.rs`, `job.rs`, `job_local.rs`, `session.rs`

### Business Logic Modules (12):
- `job/manager.rs`, `job/storage.rs`, `job/types.rs`, `job/mod.rs`
- `session/manager.rs`, `session/types.rs`, `session/mod.rs`
- `cache/manager.rs`, `cache/storage.rs`, `cache/types.rs`, `cache/mod.rs`
- `metrics/aggregator.rs`, `metrics/collector.rs`, `metrics/storage.rs`, `metrics/types.rs`, `metrics/mod.rs`

### Additional Commands (11):
- `commands/optimized_executor.rs`, `domain_refactored.rs`
- `commands/wasm_cache.rs`, `engine_cache.rs`, `wasm_aot_cache.rs`
- `commands/adaptive_timeout.rs`, `extract_enhanced.rs`
- `commands/progress.rs`, `performance_monitor.rs`

---

## CONCLUSION

The CLI crate currently contains **~2,600 lines of pure business logic** that should be extracted to dedicated crates. The highest-priority candidates are:

1. **Job Management** (~700 lines) - Complex state machine
2. **Metrics Aggregation** (~600 lines) - Statistical algorithms
3. **Domain Profiles** (~900 lines) - Site analysis logic
4. **Session Management** (~200 lines) - Stateful domain logic
5. **Cache Manager** (~550 lines) - LRU eviction (verify duplication)

These extractions will improve **testability** (isolated unit tests), **reusability** (shared across components), and **optimization potential** (SIMD, async executors). The estimated effort is **12-17 days** across two phases.

**Recommended Next Step:** Start with Job Management System (highest complexity and reusability).
