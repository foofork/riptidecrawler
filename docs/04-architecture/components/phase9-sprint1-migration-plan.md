# Phase 9 Sprint 1: CLI Business Logic Migration Plan

**Goal**: Replace 3,700 LOC of CLI business logic with existing library crates
**Impact**: -45% codebase size, improved maintainability, reduced duplication
**Timeline**: 4-6 days (risk-sorted execution)

## Executive Summary

This plan migrates CLI business logic to proper library crates, eliminating duplication and improving architecture. Migrations are ordered by risk level (safest first) to maximize success probability.

### Target Migrations

| Component | Current LOC | Target Crate | Risk Level | Priority |
|-----------|-------------|--------------|------------|----------|
| PDF helpers | 134 | riptide-pdf | **LOW** ⚠️ | 1st (Day 1) |
| Browser pool | 456 | riptide-pool | **LOW** ⚠️ | 2nd (Day 2) |
| Cache mgmt | 262 | riptide-cache | **MEDIUM** ⚡ | 3rd (Day 3-4) |
| Job mgmt | 1,420 | riptide-workers | **HIGH** 🔥 | 4th (Day 5-6) |
| **TOTAL** | **2,272** | - | - | - |

**Note**: Original estimate was 3,700 LOC. Actual measured is 2,272 LOC (includes only actively used code).

---

## Migration Order (Risk-Sorted)

### 🟢 MIGRATION 1: PDF Helpers → riptide-pdf (134 LOC)
**Risk**: LOW | **Duration**: 4 hours | **Test Updates**: Minimal

#### Current State Analysis
- **Location**: `crates/riptide-cli/src/commands/pdf_impl.rs`
- **Functions**: 7 utility functions (load_pdf, extract_metadata, convert_to_markdown, etc.)
- **Dependencies**: Already uses `riptide-pdf::PdfExtractor`
- **Test Coverage**: Limited unit tests in module

#### Migration Strategy

**Step 1: Extend riptide-pdf Library (2 hours)**
```rust
// Add to riptide-pdf/src/utils.rs

/// Load PDF from file path or URL
pub async fn load_pdf_from_source(input: &str) -> Result<Vec<u8>> {
    if input.starts_with("http://") || input.starts_with("https://") {
        // Download PDF from URL
        let response = reqwest::get(input).await?;
        Ok(response.bytes().await?.to_vec())
    } else {
        // Read from local file
        std::fs::read(input).context("Failed to read PDF file")
    }
}

/// Parse page range string (e.g., "1-5,10,15-20")
pub fn parse_page_range(range: &str) -> Result<Vec<u32>> {
    // Move implementation from CLI
}

/// Write output to file or stdout
pub fn write_pdf_output(content: &str, output_path: Option<&str>) -> Result<()> {
    // Move implementation from CLI
}
```

**Step 2: Update CLI to Use Library (1 hour)**
```rust
// crates/riptide-cli/src/commands/pdf_impl.rs
// Replace implementations with re-exports

pub use riptide_pdf::utils::{
    load_pdf_from_source as load_pdf,
    parse_page_range,
    write_pdf_output as write_output,
};

// Keep CLI-specific wrappers thin
#[cfg(feature = "pdf")]
pub fn extract_metadata(pdf_data: &[u8]) -> Result<riptide_pdf::PdfDocMetadata> {
    let extractor = PdfExtractor::from_bytes(pdf_data)?;
    extractor.extract_metadata()
}
```

**Step 3: Update Tests (1 hour)**
- Move `test_parse_page_range()` to `riptide-pdf/src/utils.rs`
- Add integration test in `riptide-pdf/tests/utils_tests.rs`
- Update CLI test imports

#### Breaking Changes
**None** - Pure internal refactoring, all public APIs preserved.

#### Success Criteria
- [ ] All 7 functions migrated to riptide-pdf
- [ ] CLI tests pass without modification
- [ ] No new dependencies added
- [ ] Zero breaking changes to public API

---

### 🟢 MIGRATION 2: Browser Pool Manager → riptide-pool (456 LOC)
**Risk**: LOW | **Duration**: 6 hours | **Test Updates**: Moderate

#### Current State Analysis
- **Location**: `crates/riptide-cli/src/commands/browser_pool_manager.rs`
- **Infrastructure**: Pre-warming, health checks, resource monitoring
- **Dependencies**: Uses `riptide-browser::pool::BrowserPool` (wrapper around it)
- **Issue**: Dead code warnings (`#[allow(dead_code)]` on most items)

#### Migration Strategy

**Step 1: Consolidate into riptide-pool (3 hours)**
```rust
// Move to riptide-pool/src/manager.rs

pub struct PoolManager {
    pool: Arc<BrowserPool>,
    config: PoolManagerConfig,
    stats: Arc<RwLock<ResourceStats>>,
    health_checker: Arc<Mutex<HealthChecker>>,
}

impl PoolManager {
    /// Create with pre-warming and health monitoring
    pub async fn new(config: PoolManagerConfig) -> Result<Self> {
        // Move implementation from CLI
    }

    /// Checkout with automatic monitoring
    pub async fn checkout(&self) -> Result<ManagedCheckout> {
        // Wraps BrowserPool::checkout with stats tracking
    }
}
```

**Step 2: CLI Adapter (2 hours)**
```rust
// crates/riptide-cli/src/commands/browser.rs (new thin adapter)

use riptide_pool::PoolManager;

pub async fn get_global_pool_manager() -> Result<Arc<PoolManager>> {
    GLOBAL_POOL.get_or_try_init(|| async {
        let config = PoolManagerConfig::from_env()?;
        PoolManager::new(config).await
    }).await
}
```

**Step 3: Update Tests (1 hour)**
- Move pool manager tests to `riptide-pool/tests/manager_tests.rs`
- Verify CLI integration tests still pass
- Test global pool initialization

#### Breaking Changes
**None** - CLI code currently marked as "planned infrastructure", not in active use.

#### Backwards Compatibility
- Global pool manager API preserved
- `get_global_pool_manager()` becomes thin wrapper
- All config options maintained

#### Success Criteria
- [ ] `browser_pool_manager.rs` reduced to <50 LOC adapter
- [ ] Pool manager tests moved to riptide-pool
- [ ] Health checking works in both CLI and library contexts
- [ ] Zero dead_code warnings

---

### 🟡 MIGRATION 3: Cache Management → riptide-cache (262 LOC)
**Risk**: MEDIUM | **Duration**: 8 hours | **Test Updates**: Significant

#### Current State Analysis
- **Location**: `crates/riptide-cli/src/commands/cache.rs`
- **Functions**: `status`, `clear`, `warm`, `validate`, `stats`
- **Issue**: CLI-specific output formatting mixed with business logic
- **Dependencies**: Uses `riptide-cache::CacheManager` (thin wrapper)

#### Migration Strategy

**Step 1: Extract Business Logic (4 hours)**
```rust
// Add to riptide-cache/src/operations.rs (new module)

pub struct CacheOperations {
    manager: CacheManager,
}

impl CacheOperations {
    /// Get cache statistics
    pub async fn get_stats(&self) -> Result<CacheStats> {
        // Extract from CLI show_status() - return data only
    }

    /// Clear cache by domain or all
    pub async fn clear(&self, domain: Option<String>) -> Result<ClearResult> {
        // Extract from CLI clear_cache() - return counts
    }

    /// Warm cache from URL list
    pub async fn warm(&self, urls: Vec<String>, options: WarmOptions) -> Result<WarmResult> {
        // Extract from CLI warm_cache() - return success/failure counts
    }

    /// Validate cache integrity
    pub async fn validate(&self) -> Result<ValidationResult> {
        // Extract from CLI validate_cache() - return validation data
    }
}

pub struct ClearResult {
    pub cleared_entries: usize,
    pub remaining_entries: usize,
    pub domain: Option<String>,
}

pub struct WarmResult {
    pub total_urls: usize,
    pub successful: usize,
    pub failed: usize,
    pub duration_ms: u64,
}

pub struct ValidationResult {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub invalid_entries: usize,
    pub expired_removed: usize,
}
```

**Step 2: Update CLI Commands (2 hours)**
```rust
// crates/riptide-cli/src/commands/cache.rs

use riptide_cache::{CacheOperations, ClearResult, WarmResult};

async fn show_status(output_format: &str) -> Result<()> {
    let ops = CacheOperations::new().await?;
    let stats = ops.get_stats().await?;

    // Keep ONLY formatting logic here
    match output_format {
        "json" => output::print_json(&stats),
        "table" => format_stats_table(&stats),
        _ => format_stats_text(&stats),
    }
}

async fn clear_cache(domain: Option<String>, output_format: &str) -> Result<()> {
    let ops = CacheOperations::new().await?;
    let result = ops.clear(domain).await?;

    // Format results
    format_clear_result(&result, output_format)
}
```

**Step 3: Update Tests (2 hours)**
- Split tests: business logic → `riptide-cache/tests/operations_tests.rs`
- Keep CLI tests focused on output formatting
- Add integration tests for cache operations

#### Breaking Changes
**Possible** - Depends on current API usage patterns

**Mitigation Strategy:**
1. Keep existing CLI command signatures identical
2. Add deprecation warnings if public APIs change
3. Provide migration guide for library users

#### Backwards Compatibility Plan
```rust
// riptide-cache/src/lib.rs - maintain old API

#[deprecated(since = "2.1.0", note = "Use CacheOperations::get_stats() instead")]
pub async fn get_cache_stats() -> Result<CacheStats> {
    CacheOperations::new().await?.get_stats().await
}
```

#### Success Criteria
- [ ] Business logic moved to riptide-cache
- [ ] CLI commands are <100 LOC (formatting only)
- [ ] All cache tests pass
- [ ] Output format unchanged
- [ ] Library API usable outside CLI

---

### 🔴 MIGRATION 4: Job Management → riptide-workers (1,420 LOC)
**Risk**: HIGH | **Duration**: 12-16 hours | **Test Updates**: Extensive

#### Current State Analysis
- **Location**:
  - `crates/riptide-cli/src/commands/job.rs` (784 LOC - API client)
  - `crates/riptide-cli/src/commands/job_local.rs` (636 LOC - local manager)
- **Complexity**: Two parallel implementations (API vs Local)
- **Issue**: `riptide-workers` crate exists but CLI has own job manager
- **Risk Factors**: State management, persistence, CLI-specific features

#### Key Risk Assessment

**Why HIGH Risk:**
1. **Dual implementations**: API-based vs local file-based
2. **State management**: In-memory tracking + filesystem persistence
3. **CLI-specific features**: Interactive watch mode, streaming logs, progress bars
4. **Test complexity**: 15 functions, multiple async workflows

**Risk Mitigation:**
- Migrate in 3 phases (API wrapper → local manager → CLI adapter)
- Keep both implementations during transition
- Feature flag for gradual rollout

#### Migration Strategy (3-Phase Approach)

**PHASE A: API Job Client → riptide-workers API module (4 hours)**

```rust
// Add to riptide-workers/src/api_client.rs (new module)

pub struct JobApiClient {
    client: RipTideClient,
    base_url: String,
}

impl JobApiClient {
    pub async fn submit_job(&self, request: SubmitJobRequest) -> Result<JobId> {
        // Move from job.rs submit_job()
    }

    pub async fn list_jobs(&self, filters: JobFilters) -> Result<Vec<Job>> {
        // Move from job.rs list_jobs()
    }

    pub async fn get_job_status(&self, job_id: &JobId) -> Result<Job> {
        // Move from job.rs job_status()
    }

    pub async fn cancel_job(&self, job_id: &JobId, force: bool) -> Result<()> {
        // Move from job.rs cancel_job()
    }

    pub async fn get_job_results(&self, job_id: &JobId) -> Result<JobResults> {
        // Move from job.rs job_results()
    }
}
```

**PHASE B: Local Job Manager → riptide-workers core (6 hours)**

```rust
// Add to riptide-workers/src/local_manager.rs (new module)

pub struct LocalJobManager {
    base_dir: PathBuf,
    jobs: Arc<RwLock<HashMap<JobId, Job>>>,
}

impl LocalJobManager {
    /// Create job manager with filesystem persistence
    pub fn new() -> Result<Self> {
        // Move from job_local.rs JobManager
    }

    /// Submit job for local execution
    pub async fn submit_job(&self, params: JobSubmitParams) -> Result<JobId> {
        // Move business logic from job_local.rs submit_job()
        // Keep CLI-specific features (streaming) separate
    }

    /// List jobs with filters
    pub async fn list_jobs(&self, filters: JobFilters) -> Result<Vec<Job>> {
        // Move from job_local.rs list_jobs()
    }

    /// Get job status
    pub async fn get_job(&self, job_id: &JobId) -> Result<Job> {
        // Move from job_local.rs (status resolution)
    }

    /// Read job logs
    pub async fn read_logs(&self, job_id: &JobId, options: LogOptions) -> Result<Vec<LogEntry>> {
        // Move from job_local.rs job_logs()
    }

    /// Cancel running job
    pub async fn cancel_job(&self, job_id: &JobId) -> Result<()> {
        // Move from job_local.rs cancel_job()
    }
}
```

**PHASE C: CLI Adapter Layer (2-4 hours)**

```rust
// crates/riptide-cli/src/commands/job.rs (keep thin adapter)

use riptide_workers::{JobApiClient, LocalJobManager};

pub async fn execute(client: RipTideClient, command: JobCommands) -> Result<()> {
    match command {
        JobCommands::Submit { .. } => {
            let api_client = JobApiClient::new(client);
            let job_id = api_client.submit_job(request).await?;

            // CLI-specific: formatting, progress display
            display_submit_result(&job_id, output_format);
        }
        // ... other commands
    }
}

// Keep CLI-specific functions:
// - display_job_status() - formatting logic
// - watch mode loop - interactive UI
// - progress bars and spinners
```

#### Breaking Changes Analysis

**HIGH IMPACT CHANGES:**
1. **Job ID resolution**: Short ID matching moved to library
2. **Log streaming**: Watch mode (`-f` flag) stays in CLI
3. **Storage location**: Filesystem paths may change

**Mitigation:**
```rust
// riptide-workers/src/compat.rs - compatibility layer

#[deprecated(since = "2.1.0", note = "Use LocalJobManager::new()")]
pub fn create_job_manager() -> Result<LocalJobManager> {
    LocalJobManager::new()
}

// Provide migration helper
pub struct MigrationHelper;

impl MigrationHelper {
    /// Migrate old CLI job storage to new format
    pub async fn migrate_job_storage(old_path: &Path, new_path: &Path) -> Result<()> {
        // Copy and convert job metadata
    }
}
```

#### Backwards Compatibility Strategy

**Option 1: Feature Flag (Recommended)**
```toml
# Cargo.toml
[features]
default = ["workers-migration"]
workers-migration = []  # Enable new riptide-workers integration
legacy-jobs = []        # Keep old implementation
```

**Option 2: Gradual Rollout**
```rust
// Week 1: Add new implementation alongside old
// Week 2: Test in production with --experimental-workers flag
// Week 3: Make new implementation default
// Week 4: Remove old implementation
```

#### Test Migration Plan

**Test Categories:**
1. **Unit tests** → Move to `riptide-workers/tests/`
   - Job submission logic
   - Status tracking
   - Progress calculations

2. **Integration tests** → Keep in CLI
   - Command parsing
   - Output formatting
   - Interactive features (watch mode)

3. **New tests needed:**
   - Storage migration tests
   - Compatibility layer tests
   - Concurrent job execution

**Test Update Estimate:**
- Unit test migration: 4 hours
- Integration test updates: 3 hours
- New compatibility tests: 3 hours
- **Total**: 10 hours

#### Success Criteria
- [ ] Job submission works via riptide-workers
- [ ] Local job manager handles persistence
- [ ] CLI commands are <500 LOC total
- [ ] All job commands work identically
- [ ] Watch mode (`-f`) still functional
- [ ] Progress bars/spinners unchanged
- [ ] Migration path for existing jobs
- [ ] Zero data loss during migration

#### Rollback Plan
1. Keep old implementation behind feature flag for 1 release
2. Document migration issues
3. Provide rollback script for job storage
4. Monitor error rates in production

---

## Order of Operations (Day-by-Day Plan)

### Day 1: PDF Helpers (4 hours)
**Morning:**
- ✅ Add utility functions to `riptide-pdf/src/utils.rs`
- ✅ Update CLI to use library functions
- ✅ Run tests

**Afternoon:**
- ✅ Move unit tests to library
- ✅ Update documentation
- ✅ Create PR, get review

**Risk**: LOW - No dependencies on other migrations

---

### Day 2: Browser Pool Manager (6 hours)
**Morning:**
- ✅ Create `riptide-pool/src/manager.rs`
- ✅ Move `PoolManager` implementation
- ✅ Add health monitoring

**Afternoon:**
- ✅ Update CLI to thin adapter
- ✅ Move tests to library
- ✅ Integration testing
- ✅ Create PR

**Risk**: LOW - Code currently unused (dead_code warnings)

---

### Day 3: Cache Management Part 1 (4 hours)
**Morning:**
- ✅ Create `riptide-cache/src/operations.rs`
- ✅ Extract business logic (status, stats)
- ✅ Define result types

**Afternoon:**
- ✅ Update CLI commands (status, stats)
- ✅ Test output formatting unchanged
- ✅ Commit checkpoint

**Risk**: MEDIUM - Active code, output must match exactly

---

### Day 4: Cache Management Part 2 (4 hours)
**Morning:**
- ✅ Extract remaining operations (clear, warm, validate)
- ✅ Update CLI commands
- ✅ Add deprecation warnings

**Afternoon:**
- ✅ Move/update tests
- ✅ Integration testing
- ✅ Create PR
- ✅ Update docs

**Risk**: MEDIUM - Warm cache has external dependencies

---

### Day 5: Job Management Part 1 (8 hours)
**Morning (4 hours):**
- ✅ Create `riptide-workers/src/api_client.rs`
- ✅ Move API job commands
- ✅ Test API client independently

**Afternoon (4 hours):**
- ✅ Create `riptide-workers/src/local_manager.rs`
- ✅ Move core job submission logic
- ✅ Test job creation/tracking
- ✅ Commit checkpoint

**Risk**: HIGH - Complex state management

---

### Day 6: Job Management Part 2 (8 hours)
**Morning (4 hours):**
- ✅ Move remaining local manager functions
- ✅ Add compatibility layer
- ✅ Update CLI to use library

**Afternoon (4 hours):**
- ✅ Update/move tests (extensive)
- ✅ Integration testing
- ✅ Create migration script
- ✅ Documentation
- ✅ Create PR

**Risk**: HIGH - Must preserve all CLI functionality

---

## Cross-Cutting Concerns

### Dependency Management

**Add to riptide-workers/Cargo.toml:**
```toml
[dependencies]
riptide-types = { path = "../riptide-types" }
riptide-monitoring = { path = "../riptide-monitoring" }
serde = { workspace = true }
tokio = { workspace = true, features = ["full"] }
anyhow = { workspace = true }
```

**Add to riptide-cache/Cargo.toml:**
```toml
[dependencies]
reqwest = { workspace = true }  # For URL loading (cache warm)
```

**Update riptide-cli/Cargo.toml:**
```toml
[dependencies]
riptide-workers = { path = "../riptide-workers" }
riptide-cache = { path = "../riptide-cache" }
riptide-pool = { path = "../riptide-pool" }
```

### Testing Strategy

**Test Distribution:**
| Test Type | Current Location | Target Location | Count |
|-----------|------------------|-----------------|-------|
| PDF unit tests | CLI | riptide-pdf | 3 |
| Pool manager tests | CLI | riptide-pool | 3 |
| Cache operation tests | CLI | riptide-cache | 5 |
| Job logic tests | CLI | riptide-workers | 10+ |
| CLI integration tests | CLI | CLI (keep) | 20+ |

**Total Test Migration:** ~21 test functions moved

**Test Validation Checklist:**
- [ ] All library unit tests pass in isolation
- [ ] CLI integration tests pass with new libraries
- [ ] Output format unchanged (golden file testing)
- [ ] Error messages preserved
- [ ] Performance benchmarks stable

### Documentation Updates

**Required Documentation:**
1. **Migration Guide** (`docs/migration-guides/phase9-sprint1.md`)
   - Breaking changes (if any)
   - Code examples before/after
   - Migration script usage

2. **Architecture Decision Record** (`docs/adr/0XX-cli-library-split.md`)
   - Rationale for library extraction
   - Design decisions
   - Alternatives considered

3. **API Documentation**
   - Update rustdoc comments
   - Add usage examples
   - Document deprecations

4. **Changelog Entry**
   - List all changes
   - Highlight breaking changes
   - Credit contributors

---

## Risk Mitigation & Rollback Plan

### Risk Matrix

| Migration | Impact | Probability | Severity | Mitigation |
|-----------|--------|-------------|----------|------------|
| PDF | LOW | 10% | Minor | Pure refactor, no API changes |
| Browser Pool | LOW | 15% | Minor | Currently unused code |
| Cache | MEDIUM | 30% | Moderate | Feature flag, gradual rollout |
| Job Mgmt | HIGH | 50% | Major | Multi-phase, compat layer, feature flag |

### Rollback Procedures

**Immediate Rollback (< 1 hour):**
```bash
# Revert commit
git revert <migration-commit-hash>

# Rebuild CLI
cargo build --release -p riptide-cli

# Verify tests
cargo test -p riptide-cli
```

**Partial Rollback (Job Management):**
```toml
# Cargo.toml - disable new implementation
[features]
default = ["legacy-jobs"]  # Switch back to old code
```

**Data Migration Rollback:**
```bash
# Restore old job storage format
./scripts/rollback-job-migration.sh --backup-dir ~/.riptide/backup
```

---

## Success Metrics

### Code Quality Metrics
- **LOC Reduction**: Target 2,272 LOC removed from CLI
- **Duplication**: Eliminate duplicate job/cache logic
- **Test Coverage**: Maintain >80% coverage in libraries
- **Clippy Warnings**: Zero new warnings introduced

### Performance Metrics
- **CLI Startup Time**: No regression (currently ~50ms)
- **Job Submission Latency**: <10ms overhead from library calls
- **Cache Operations**: Match current performance ±5%

### User Impact Metrics
- **Breaking Changes**: Minimize to job management only (if any)
- **Documentation**: 100% of public APIs documented
- **Migration Success**: 95% of users migrate smoothly

---

## Dependencies & Blockers

### Required Before Starting
- [ ] Phase 8 (testing infrastructure) complete
- [ ] CI pipeline stable
- [ ] Backup of production job data (if applicable)

### External Dependencies
- None - All code is internal to monorepo

### Potential Blockers
1. **Circular dependencies**: riptide-cache might depend on CLI types
   - **Mitigation**: Use riptide-types for shared types
2. **Feature flags**: Old code might depend on new features
   - **Mitigation**: Clear dependency graph, feature isolation
3. **Integration test failures**: Output format changes
   - **Mitigation**: Golden file testing, format validation

---

## Appendix A: Detailed LOC Breakdown

### Current CLI Command Files
```
crates/riptide-cli/src/commands/
├── job.rs                    784 LOC (API client)
├── job_local.rs              636 LOC (Local manager)
├── cache.rs                  262 LOC (Cache operations)
├── browser_pool_manager.rs   456 LOC (Pool management)
├── pdf_impl.rs               134 LOC (PDF utilities)
└── TOTAL                   2,272 LOC
```

### Target Library Additions
```
crates/riptide-workers/src/
├── api_client.rs           ~400 LOC (from job.rs)
├── local_manager.rs        ~450 LOC (from job_local.rs)
└── compat.rs               ~100 LOC (compatibility layer)

crates/riptide-cache/src/
├── operations.rs           ~180 LOC (from cache.rs)
└── (formatting stays in CLI)

crates/riptide-pool/src/
├── manager.rs              ~350 LOC (from browser_pool_manager.rs)
└── (CLI adapter <50 LOC)

crates/riptide-pdf/src/
├── utils.rs additions       ~80 LOC (from pdf_impl.rs)
└── (CLI wrappers <30 LOC)
```

### Expected CLI Reduction
- **Before**: 2,272 LOC of business logic in CLI
- **After**: ~500 LOC of CLI adapters + formatting
- **Reduction**: 1,772 LOC (-78%)

---

## Appendix B: Testing Checklist

### Pre-Migration Tests (Baseline)
```bash
# Capture current behavior
cargo test -p riptide-cli --all-features > baseline-tests.txt

# Capture output formats
./tests/golden-file-capture.sh
```

### Per-Migration Testing
```bash
# After each migration:
cargo test -p riptide-cli          # CLI tests
cargo test -p riptide-<target>     # Library tests
cargo clippy -p riptide-cli        # No new warnings
cargo doc -p riptide-<target>      # Docs build

# Integration validation
./tests/integration-smoke-test.sh
```

### Post-Migration Validation
```bash
# Full test suite
cargo test --workspace

# Output format validation
diff baseline-tests.txt current-tests.txt

# Performance regression check
cargo bench -p riptide-cli

# Documentation coverage
cargo doc --workspace --no-deps
```

---

## Appendix C: Command Reference

### Build Commands
```bash
# Build specific migrations
cargo build -p riptide-pdf          # After migration 1
cargo build -p riptide-pool         # After migration 2
cargo build -p riptide-cache        # After migration 3
cargo build -p riptide-workers      # After migration 4
cargo build -p riptide-cli          # After each migration

# Clean build to catch issues
cargo clean && cargo build --workspace
```

### Test Commands
```bash
# Run specific migration tests
cargo test -p riptide-pdf utils_tests
cargo test -p riptide-pool manager_tests
cargo test -p riptide-cache operations_tests
cargo test -p riptide-workers local_manager_tests

# Integration tests
cargo test -p riptide-cli --test integration
```

### Verification Commands
```bash
# Check for dead code
cargo build -p riptide-cli 2>&1 | grep "dead_code"

# Check for duplicate code
cargo duplicate -p riptide-cli -p riptide-workers

# Check LOC reduction
tokei crates/riptide-cli/src/commands/
```

---

## Appendix D: Migration Script Templates

### Job Storage Migration Script
```bash
#!/bin/bash
# scripts/migrate-job-storage.sh

set -euo pipefail

OLD_DIR="${HOME}/.riptide/jobs"
NEW_DIR="${HOME}/.riptide/workers/jobs"
BACKUP_DIR="${HOME}/.riptide/backup-$(date +%Y%m%d)"

echo "Backing up existing jobs to ${BACKUP_DIR}..."
cp -r "${OLD_DIR}" "${BACKUP_DIR}"

echo "Migrating job storage format..."
cargo run -p riptide-workers --bin migrate-jobs -- \
    --source "${OLD_DIR}" \
    --target "${NEW_DIR}" \
    --validate

echo "Migration complete. Backup stored at ${BACKUP_DIR}"
```

### Rollback Script
```bash
#!/bin/bash
# scripts/rollback-job-migration.sh

set -euo pipefail

BACKUP_DIR="$1"
CURRENT_DIR="${HOME}/.riptide/workers/jobs"

echo "Rolling back from ${BACKUP_DIR}..."
rm -rf "${CURRENT_DIR}"
cp -r "${BACKUP_DIR}" "${HOME}/.riptide/jobs"

echo "Rollback complete. Restart riptide-cli."
```

---

## Sign-off Checklist

### Before Starting Sprint
- [ ] Review complete migration plan
- [ ] Confirm risk assessment acceptable
- [ ] Backup production data (if applicable)
- [ ] CI pipeline green
- [ ] Team availability confirmed

### After Each Migration
- [ ] Tests pass (library + CLI)
- [ ] Documentation updated
- [ ] PR created and reviewed
- [ ] Changelog entry added
- [ ] No new clippy warnings

### Sprint Completion Criteria
- [ ] All 4 migrations complete
- [ ] 2,272+ LOC removed from CLI
- [ ] All tests passing
- [ ] Zero breaking changes (or documented migration path)
- [ ] Performance benchmarks stable
- [ ] Documentation complete
- [ ] Phase 9 Sprint 2 ready to start

---

**Document Version**: 1.0
**Created**: 2025-10-23
**Last Updated**: 2025-10-23
**Author**: System Architect
**Status**: Ready for Implementation
