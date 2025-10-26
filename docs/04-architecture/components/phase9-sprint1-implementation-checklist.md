# Phase 9 Sprint 1: Implementation Checklist
## Step-by-Step Execution Guide

**For**: Engineers implementing the migration
**Use**: Follow this checklist step-by-step, marking off items as completed
**Timeline**: 6 working days (4-6 hours per day)

---

## Pre-Flight Checklist (Before Starting)

### Environment Setup
- [ ] Working directory: `/workspaces/eventmesh`
- [ ] Git branch: `feature/phase9-sprint1-cli-migration`
- [ ] Clean build: `cargo clean && cargo build --workspace`
- [ ] All tests pass: `cargo test --workspace`
- [ ] No warnings: `cargo clippy --workspace -- -D warnings`

### Documentation Ready
- [ ] Read migration plan: `docs/architecture/phase9-sprint1-migration-plan.md`
- [ ] Read architecture diagrams: `docs/architecture/phase9-migration-architecture.md`
- [ ] Review executive summary: `docs/architecture/phase9-sprint1-executive-summary.md`

### Backup & Safety
- [ ] Create backup branch: `git checkout -b backup/cli-state-$(date +%Y%m%d)`
- [ ] Backup job storage (if used): `cp -r ~/.riptide/jobs ~/.riptide/backup-jobs-$(date +%Y%m%d)`
- [ ] Capture baseline: `cargo test --workspace > baseline-tests.txt 2>&1`

---

## Day 1: PDF Helpers Migration (4 hours)

**Risk**: 🟢 LOW | **Files**: 2 | **Tests**: 3

### Step 1.1: Extend riptide-pdf Library (2 hours)

#### File: `crates/riptide-pdf/src/utils.rs`
- [ ] Add to file:
  ```rust
  /// Load PDF from file path or URL
  pub async fn load_pdf_from_source(input: &str) -> Result<Vec<u8>>

  /// Parse page range string (e.g., "1-5,10,15-20")
  pub fn parse_page_range(range: &str) -> Result<Vec<u32>>

  /// Write output to file or stdout
  pub fn write_pdf_output(content: &str, output_path: Option<&str>) -> Result<()>
  ```

- [ ] Copy implementations from `crates/riptide-cli/src/commands/pdf_impl.rs`

- [ ] Add test module:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn test_parse_page_range() {
          // Move from CLI
      }
  }
  ```

- [ ] **Verify**: `cargo test -p riptide-pdf`
- [ ] **Verify**: `cargo clippy -p riptide-pdf -- -D warnings`

#### File: `crates/riptide-pdf/src/lib.rs`
- [ ] Add re-exports:
  ```rust
  pub use utils::{load_pdf_from_source, parse_page_range, write_pdf_output};
  ```

- [ ] **Verify**: `cargo doc -p riptide-pdf --no-deps`

### Step 1.2: Update CLI to Use Library (1 hour)

#### File: `crates/riptide-cli/src/commands/pdf_impl.rs`
- [ ] Replace implementations with:
  ```rust
  // Re-export library functions
  pub use riptide_pdf::{
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

  // ... other thin wrappers
  ```

- [ ] Remove duplicate implementations (keep only wrappers)
- [ ] **Verify LOC**: Should be ~30 lines (was 134)

- [ ] **Verify**: `cargo test -p riptide-cli`
- [ ] **Verify**: `cargo build -p riptide-cli`

### Step 1.3: Commit & Review (1 hour)
- [ ] Run full test suite: `cargo test --workspace`
- [ ] Check no dead code: `cargo build --workspace 2>&1 | grep dead_code`
- [ ] Format code: `cargo fmt --all`
- [ ] Git operations:
  ```bash
  git add crates/riptide-pdf/src/utils.rs
  git add crates/riptide-pdf/src/lib.rs
  git add crates/riptide-cli/src/commands/pdf_impl.rs
  git commit -m "refactor: migrate PDF helpers to riptide-pdf library

  - Move load_pdf_from_source, parse_page_range, write_pdf_output to riptide-pdf
  - Reduce CLI pdf_impl.rs from 134 LOC to ~30 LOC
  - Update tests to library location
  - Zero breaking changes

  Part of Phase 9 Sprint 1 (Day 1)
  "
  ```

- [ ] **Create PR**: Title: `[Phase9] Migrate PDF helpers to library`
- [ ] **Request review**: Tag 2+ reviewers

**Day 1 Complete**: ✅ 134 LOC migrated, 0 breaking changes

---

## Day 2: Browser Pool Manager Migration (6 hours)

**Risk**: 🟢 LOW | **Files**: 3 | **Tests**: 3

### Step 2.1: Create riptide-pool Manager (3 hours)

#### File: `crates/riptide-pool/src/manager.rs` (new file)
- [ ] Create file: `touch crates/riptide-pool/src/manager.rs`

- [ ] Copy entire implementation from `crates/riptide-cli/src/commands/browser_pool_manager.rs`

- [ ] Remove CLI-specific dependencies:
  ```rust
  // Remove: use crate::output
  // Add: use tracing::{info, warn, debug}
  ```

- [ ] Keep all functionality:
  - `PoolManagerConfig`
  - `BrowserPoolManager`
  - `PoolStats`, `ResourceStats`, `HealthStatus`
  - Health checking logic
  - Global pool manager

- [ ] Move tests from CLI file

- [ ] **Verify**: `cargo test -p riptide-pool`
- [ ] **Verify**: `cargo clippy -p riptide-pool -- -D warnings`

#### File: `crates/riptide-pool/src/lib.rs`
- [ ] Add module: `pub mod manager;`
- [ ] Add re-exports:
  ```rust
  pub use manager::{
      BrowserPoolManager, PoolManagerConfig, PoolStats,
      ResourceStats, HealthStatus, get_global_pool_manager,
      shutdown_global_pool_manager,
  };
  ```

### Step 2.2: Update CLI to Thin Adapter (2 hours)

#### File: `crates/riptide-cli/src/commands/browser_pool_manager.rs`
- [ ] Replace entire file with thin adapter:
  ```rust
  // Infrastructure for planned features
  #![allow(dead_code)]

  /// Re-export pool manager from library
  pub use riptide_pool::manager::{
      BrowserPoolManager,
      PoolManagerConfig,
      get_global_pool_manager,
      shutdown_global_pool_manager,
  };
  ```

- [ ] **Verify LOC**: Should be ~15 lines (was 456)

- [ ] Update imports in other CLI files (if any):
  ```bash
  grep -r "browser_pool_manager" crates/riptide-cli/src/
  ```

- [ ] **Verify**: `cargo test -p riptide-cli`
- [ ] **Verify**: `cargo build -p riptide-cli`

### Step 2.3: Update Tests (1 hour)

#### File: `crates/riptide-pool/tests/manager_tests.rs` (new file)
- [ ] Create file: `touch crates/riptide-pool/tests/manager_tests.rs`
- [ ] Move tests from CLI
- [ ] Add integration tests:
  ```rust
  #[tokio::test]
  async fn test_global_pool_manager_singleton() {
      let pool1 = get_global_pool_manager().await.unwrap();
      let pool2 = get_global_pool_manager().await.unwrap();
      // Verify same instance
  }
  ```

- [ ] **Verify**: `cargo test -p riptide-pool manager_tests`

### Step 2.4: Commit & Review (1 hour)
- [ ] Run full test suite: `cargo test --workspace`
- [ ] Format code: `cargo fmt --all`
- [ ] Git operations:
  ```bash
  git add crates/riptide-pool/src/manager.rs
  git add crates/riptide-pool/src/lib.rs
  git add crates/riptide-pool/tests/manager_tests.rs
  git add crates/riptide-cli/src/commands/browser_pool_manager.rs
  git commit -m "refactor: migrate browser pool manager to riptide-pool

  - Move BrowserPoolManager to riptide-pool/src/manager.rs
  - Reduce CLI browser_pool_manager.rs from 456 LOC to ~15 LOC
  - Move tests to library location
  - Eliminate dead_code warnings

  Part of Phase 9 Sprint 1 (Day 2)
  "
  ```

- [ ] **Create PR**: Title: `[Phase9] Migrate browser pool manager to library`
- [ ] **Request review**: Tag 2+ reviewers

**Day 2 Complete**: ✅ 456 LOC migrated, 0 breaking changes

---

## Day 3: Cache Operations Part 1 (4 hours)

**Risk**: 🟡 MEDIUM | **Files**: 2 | **Tests**: 2

### Step 3.1: Create Cache Operations Module (2 hours)

#### File: `crates/riptide-cache/src/operations.rs` (new file)
- [ ] Create file: `touch crates/riptide-cache/src/operations.rs`

- [ ] Define result types:
  ```rust
  #[derive(Debug, Serialize, Deserialize)]
  pub struct ClearResult {
      pub cleared_entries: usize,
      pub remaining_entries: usize,
      pub domain: Option<String>,
  }

  #[derive(Debug, Serialize, Deserialize)]
  pub struct ValidationResult {
      pub total_entries: usize,
      pub valid_entries: usize,
      pub invalid_entries: usize,
      pub expired_removed: usize,
  }
  ```

- [ ] Create operations struct:
  ```rust
  pub struct CacheOperations {
      manager: CacheManager,
  }

  impl CacheOperations {
      pub async fn new() -> Result<Self> {
          Ok(Self {
              manager: CacheManager::new().await?,
          })
      }

      pub async fn get_stats(&self) -> Result<CacheStats> {
          // Extract from CLI show_status()
      }

      pub async fn clear(&self, domain: Option<String>) -> Result<ClearResult> {
          // Extract from CLI clear_cache()
      }

      pub async fn validate(&self) -> Result<ValidationResult> {
          // Extract from CLI validate_cache()
      }
  }
  ```

- [ ] Extract business logic from `crates/riptide-cli/src/commands/cache.rs`

- [ ] **Verify**: `cargo test -p riptide-cache`
- [ ] **Verify**: `cargo clippy -p riptide-cache -- -D warnings`

#### File: `crates/riptide-cache/src/lib.rs`
- [ ] Add module: `pub mod operations;`
- [ ] Add re-exports:
  ```rust
  pub use operations::{
      CacheOperations, ClearResult, ValidationResult,
  };
  ```

### Step 3.2: Update CLI Commands (1.5 hours)

#### File: `crates/riptide-cli/src/commands/cache.rs`
- [ ] Update `show_status()`:
  ```rust
  async fn show_status(output_format: &str) -> Result<()> {
      let ops = CacheOperations::new().await?;
      let stats = ops.get_stats().await?;

      // Keep ONLY formatting logic
      match output_format {
          "json" => output::print_json(&stats),
          "table" => format_stats_table(&stats),
          _ => format_stats_text(&stats),
      }
  }
  ```

- [ ] Update `clear_cache()`:
  ```rust
  async fn clear_cache(domain: Option<String>, output_format: &str) -> Result<()> {
      let ops = CacheOperations::new().await?;
      let result = ops.clear(domain).await?;

      // Format results
      format_clear_result(&result, output_format)
  }
  ```

- [ ] Update `validate_cache()`:
  ```rust
  async fn validate_cache(output_format: &str) -> Result<()> {
      let ops = CacheOperations::new().await?;
      let result = ops.validate().await?;

      // Format results
      format_validation_result(&result, output_format)
  }
  ```

- [ ] **Verify output unchanged**: Compare with baseline
  ```bash
  cargo run --bin riptide -- cache status > new-output.txt
  diff baseline-cache-status.txt new-output.txt
  ```

### Step 3.3: Commit Checkpoint (0.5 hours)
- [ ] Run tests: `cargo test -p riptide-cache -p riptide-cli`
- [ ] Format: `cargo fmt --all`
- [ ] Git operations:
  ```bash
  git add crates/riptide-cache/src/operations.rs
  git add crates/riptide-cache/src/lib.rs
  git add crates/riptide-cli/src/commands/cache.rs
  git commit -m "refactor(cache): extract business logic to riptide-cache (Part 1)

  - Create CacheOperations in riptide-cache/src/operations.rs
  - Extract get_stats, clear, validate logic from CLI
  - CLI keeps only formatting code
  - Output format unchanged

  Part of Phase 9 Sprint 1 (Day 3 checkpoint)
  "
  ```

**Day 3 Complete**: ✅ Checkpoint committed, continue to Day 4

---

## Day 4: Cache Operations Part 2 (4 hours)

**Risk**: 🟡 MEDIUM | **Files**: 2 | **Tests**: 3

### Step 4.1: Complete Cache Operations (2 hours)

#### File: `crates/riptide-cache/src/operations.rs`
- [ ] Add remaining operations:
  ```rust
  impl CacheOperations {
      pub async fn warm(&self, urls: Vec<String>, options: WarmOptions) -> Result<WarmResult> {
          // Extract from CLI warm_cache()
      }
  }

  #[derive(Debug, Serialize, Deserialize)]
  pub struct WarmResult {
      pub total_urls: usize,
      pub successful: usize,
      pub failed: usize,
      pub duration_ms: u64,
  }
  ```

- [ ] **Verify**: `cargo test -p riptide-cache`

#### File: `crates/riptide-cli/src/commands/cache.rs`
- [ ] Update `warm_cache()`:
  ```rust
  async fn warm_cache(url_file: String, output_format: &str) -> Result<()> {
      // Read URLs from file (keep in CLI)
      let urls = read_url_file(&url_file)?;

      let ops = CacheOperations::new().await?;
      let result = ops.warm(urls, WarmOptions::default()).await?;

      // Format results
      format_warm_result(&result, output_format)
  }
  ```

- [ ] **Verify**: `cargo test -p riptide-cli cache`

### Step 4.2: Add Deprecation Warnings (0.5 hours)

#### File: `crates/riptide-cache/src/lib.rs`
- [ ] Add compatibility layer:
  ```rust
  #[deprecated(since = "2.1.0", note = "Use CacheOperations::get_stats() instead")]
  pub async fn get_cache_stats() -> Result<CacheStats> {
      CacheOperations::new().await?.get_stats().await
  }
  ```

### Step 4.3: Move/Update Tests (1 hour)

#### File: `crates/riptide-cache/tests/operations_tests.rs` (new file)
- [ ] Create file: `touch crates/riptide-cache/tests/operations_tests.rs`
- [ ] Move business logic tests from CLI
- [ ] Add new tests:
  ```rust
  #[tokio::test]
  async fn test_cache_operations_lifecycle() {
      let ops = CacheOperations::new().await.unwrap();

      // Warm cache
      let result = ops.warm(vec!["https://example.com".into()], ...).await.unwrap();
      assert!(result.successful > 0);

      // Get stats
      let stats = ops.get_stats().await.unwrap();
      assert!(stats.total_entries > 0);

      // Clear
      let result = ops.clear(None).await.unwrap();
      assert!(result.cleared_entries > 0);
  }
  ```

- [ ] **Verify**: `cargo test -p riptide-cache operations_tests`

### Step 4.4: Integration Testing (0.5 hours)
- [ ] Test all cache commands:
  ```bash
  cargo run --bin riptide -- cache status
  cargo run --bin riptide -- cache stats
  cargo run --bin riptide -- cache clear --domain example.com
  cargo run --bin riptide -- cache validate
  # (skip warm - requires URL file)
  ```

- [ ] Compare output with baseline

### Step 4.5: Commit & Review (1 hour)
- [ ] Run full test suite: `cargo test --workspace`
- [ ] Format: `cargo fmt --all`
- [ ] Git operations:
  ```bash
  git add crates/riptide-cache/src/operations.rs
  git add crates/riptide-cache/src/lib.rs
  git add crates/riptide-cache/tests/operations_tests.rs
  git add crates/riptide-cli/src/commands/cache.rs
  git commit -m "refactor(cache): complete cache operations migration (Part 2)

  - Add warm operation to CacheOperations
  - Move all business logic tests to library
  - Add deprecation warnings for old APIs
  - CLI cache.rs now <100 LOC (formatting only)

  Part of Phase 9 Sprint 1 (Day 4)
  "
  ```

- [ ] **Create PR**: Title: `[Phase9] Migrate cache operations to library`
- [ ] **Request review**: Tag 2+ reviewers

**Day 4 Complete**: ✅ 262 LOC migrated, backwards compatible

---

## Day 5: Job Management Part 1 (8 hours)

**Risk**: 🔴 HIGH | **Files**: 4 | **Tests**: 5+

### Step 5.1: Create API Client (4 hours)

#### File: `crates/riptide-workers/src/api_client.rs` (new file)
- [ ] Create file: `touch crates/riptide-workers/src/api_client.rs`

- [ ] Define types:
  ```rust
  pub struct JobApiClient {
      client: reqwest::Client,
      base_url: String,
  }

  #[derive(Debug, Serialize, Deserialize)]
  pub struct SubmitJobRequest {
      pub urls: Vec<String>,
      pub method: String,
      pub name: Option<String>,
      pub priority: String,
      // ... other fields
  }
  ```

- [ ] Extract methods from `crates/riptide-cli/src/commands/job.rs`:
  ```rust
  impl JobApiClient {
      pub fn new(base_url: String) -> Self {
          Self {
              client: reqwest::Client::new(),
              base_url,
          }
      }

      pub async fn submit_job(&self, request: SubmitJobRequest) -> Result<JobId> {
          // Extract from CLI submit_job()
      }

      pub async fn list_jobs(&self, filters: JobFilters) -> Result<Vec<Job>> {
          // Extract from CLI list_jobs()
      }

      pub async fn get_job_status(&self, job_id: &JobId) -> Result<Job> {
          // Extract from CLI job_status()
      }

      pub async fn cancel_job(&self, job_id: &JobId, force: bool) -> Result<()> {
          // Extract from CLI cancel_job()
      }

      pub async fn get_job_results(&self, job_id: &JobId) -> Result<serde_json::Value> {
          // Extract from CLI job_results()
      }

      pub async fn get_job_stats(&self, range: String, group_by: Option<String>) -> Result<JobStatsResponse> {
          // Extract from CLI job_stats()
      }
  }
  ```

- [ ] **Verify**: `cargo test -p riptide-workers api_client`
- [ ] **Verify**: `cargo clippy -p riptide-workers -- -D warnings`

#### File: `crates/riptide-workers/src/lib.rs`
- [ ] Add module: `pub mod api_client;`
- [ ] Add re-exports:
  ```rust
  pub use api_client::{JobApiClient, SubmitJobRequest};
  ```

### Step 5.2: Create Local Manager (4 hours)

#### File: `crates/riptide-workers/src/local_manager.rs` (new file)
- [ ] Create file: `touch crates/riptide-workers/src/local_manager.rs`

- [ ] Extract from `crates/riptide-cli/src/commands/job_local.rs`:
  ```rust
  pub struct LocalJobManager {
      base_dir: PathBuf,
      jobs: Arc<RwLock<HashMap<JobId, Job>>>,
  }

  impl LocalJobManager {
      pub fn new() -> Result<Self> {
          // Extract from CLI JobManager::new()
      }

      pub async fn submit_job(&self, params: JobSubmitParams) -> Result<JobId> {
          // Extract from CLI submit_job()
      }

      pub async fn list_jobs(&self, filters: JobFilters) -> Result<Vec<Job>> {
          // Extract from CLI list_jobs()
      }

      pub async fn get_job(&self, job_id: &JobId) -> Result<Job> {
          // Extract from CLI (status resolution)
      }

      pub async fn read_logs(&self, job_id: &JobId, options: LogOptions) -> Result<Vec<LogEntry>> {
          // Extract from CLI job_logs()
      }

      pub async fn cancel_job(&self, job_id: &JobId) -> Result<()> {
          // Extract from CLI cancel_job()
      }

      pub async fn load_results(&self, job_id: &JobId) -> Result<serde_json::Value> {
          // Extract from CLI job_results()
      }

      pub async fn get_stats(&self) -> Result<JobStats> {
          // Extract from CLI job_stats()
      }

      pub async fn cleanup_old_jobs(&self, days: u32) -> Result<Vec<JobId>> {
          // Extract from CLI cleanup_jobs()
      }

      pub fn get_storage_stats(&self) -> Result<StorageStats> {
          // Extract from CLI storage_info()
      }
  }
  ```

- [ ] **Verify**: `cargo test -p riptide-workers local_manager`

#### File: `crates/riptide-workers/src/lib.rs`
- [ ] Add module: `pub mod local_manager;`
- [ ] Add re-exports:
  ```rust
  pub use local_manager::{LocalJobManager, JobSubmitParams, LogOptions};
  ```

### Step 5.3: Commit Checkpoint
- [ ] Run tests: `cargo test -p riptide-workers`
- [ ] Format: `cargo fmt --all`
- [ ] Git operations:
  ```bash
  git add crates/riptide-workers/src/api_client.rs
  git add crates/riptide-workers/src/local_manager.rs
  git add crates/riptide-workers/src/lib.rs
  git commit -m "feat(workers): add job API client and local manager

  - Create JobApiClient for API-based job management
  - Create LocalJobManager for file-based job management
  - Extract business logic from CLI job commands
  - CLI adapters to be added in next commit

  Part of Phase 9 Sprint 1 (Day 5 checkpoint)
  "
  ```

**Day 5 Complete**: ✅ Checkpoint committed, continue to Day 6

---

## Day 6: Job Management Part 2 (8 hours)

**Risk**: 🔴 HIGH | **Files**: 3 | **Tests**: 10+

### Step 6.1: Update CLI to Use Library (4 hours)

#### File: `crates/riptide-cli/src/commands/job.rs`
- [ ] Replace implementations with thin adapters:
  ```rust
  use riptide_workers::{JobApiClient, SubmitJobRequest};

  async fn submit_job(...) -> Result<()> {
      let api_client = JobApiClient::new(client.base_url());
      let job_id = api_client.submit_job(request).await?;

      // CLI-specific: formatting, progress display
      display_submit_result(&job_id, output_format);
  }
  ```

- [ ] Update all job command functions (submit, list, status, logs, cancel, results, retry, stats)

- [ ] Keep display/formatting functions in CLI:
  - `display_job_status()`
  - `display_submit_result()`
  - Watch mode loop
  - Progress bars

- [ ] **Verify LOC**: Should be <400 LOC (was 784)

#### File: `crates/riptide-cli/src/commands/job_local.rs`
- [ ] Replace implementations with thin adapters:
  ```rust
  use riptide_workers::{LocalJobManager, JobSubmitParams};

  async fn submit_job(...) -> Result<()> {
      let manager = LocalJobManager::new()?;
      let job_id = manager.submit_job(params).await?;

      // CLI-specific: formatting
      display_submit_result(&job_id, output_format);
  }
  ```

- [ ] Update all job-local command functions

- [ ] Keep CLI-specific:
  - `display_job_status()`
  - `print_log_entry()`
  - Watch mode loop
  - Interactive features

- [ ] **Verify LOC**: Should be <300 LOC (was 636)

- [ ] **Verify**: `cargo test -p riptide-cli`

### Step 6.2: Add Compatibility Layer (2 hours)

#### File: `crates/riptide-workers/src/compat.rs` (new file)
- [ ] Create file: `touch crates/riptide-workers/src/compat.rs`

- [ ] Add compatibility functions:
  ```rust
  #[deprecated(since = "2.1.0", note = "Use LocalJobManager::new()")]
  pub fn create_job_manager() -> Result<LocalJobManager> {
      LocalJobManager::new()
  }

  /// Migration helper for job storage
  pub struct MigrationHelper;

  impl MigrationHelper {
      pub async fn migrate_job_storage(old_path: &Path, new_path: &Path) -> Result<()> {
          // Copy and convert job metadata
      }

      pub async fn validate_migration(new_path: &Path) -> Result<bool> {
          // Verify all jobs migrated successfully
      }
  }
  ```

#### File: `crates/riptide-workers/src/lib.rs`
- [ ] Add module: `pub mod compat;`

### Step 6.3: Create Migration Script (1 hour)

#### File: `scripts/migrate-job-storage.sh` (new file)
- [ ] Create script:
  ```bash
  #!/bin/bash
  set -euo pipefail

  OLD_DIR="${HOME}/.riptide/jobs"
  NEW_DIR="${HOME}/.riptide/workers/jobs"
  BACKUP_DIR="${HOME}/.riptide/backup-$(date +%Y%m%d)"

  echo "Backing up existing jobs..."
  cp -r "${OLD_DIR}" "${BACKUP_DIR}"

  echo "Migrating job storage format..."
  cargo run -p riptide-workers --example migrate-jobs -- \
      --source "${OLD_DIR}" \
      --target "${NEW_DIR}" \
      --validate

  echo "Migration complete."
  ```

- [ ] Make executable: `chmod +x scripts/migrate-job-storage.sh`

#### File: `scripts/rollback-job-migration.sh` (new file)
- [ ] Create rollback script:
  ```bash
  #!/bin/bash
  set -euo pipefail

  BACKUP_DIR="$1"

  echo "Rolling back job storage..."
  rm -rf "${HOME}/.riptide/workers/jobs"
  cp -r "${BACKUP_DIR}" "${HOME}/.riptide/jobs"

  echo "Rollback complete."
  ```

- [ ] Make executable: `chmod +x scripts/rollback-job-migration.sh`

### Step 6.4: Update/Move Tests (2 hours)

#### File: `crates/riptide-workers/tests/api_client_tests.rs` (new file)
- [ ] Move API client tests from CLI
- [ ] Add integration tests

#### File: `crates/riptide-workers/tests/local_manager_tests.rs` (new file)
- [ ] Move local manager tests from CLI
- [ ] Add job lifecycle tests

#### File: `crates/riptide-workers/tests/compat_tests.rs` (new file)
- [ ] Test migration helper
- [ ] Test backwards compatibility

- [ ] **Verify**: `cargo test -p riptide-workers`

### Step 6.5: Integration Testing (1 hour)
- [ ] Test job submission (API mode):
  ```bash
  cargo run --bin riptide -- job submit --url https://example.com
  ```

- [ ] Test job submission (local mode):
  ```bash
  cargo run --bin riptide -- job-local submit --url https://example.com
  ```

- [ ] Test all job commands (list, status, logs, cancel, results, stats)

- [ ] Verify watch mode works: `cargo run --bin riptide -- job-local status --id <job> -w`

### Step 6.6: Documentation (1 hour)

#### File: `docs/migration-guides/phase9-sprint1.md` (new file)
- [ ] Document breaking changes (if any)
- [ ] Provide migration examples
- [ ] Document rollback procedure

#### Update: `CHANGELOG.md`
- [ ] Add entry:
  ```markdown
  ## [2.1.0] - 2025-10-24

  ### Changed
  - **BREAKING (minor)**: Job storage location moved from `~/.riptide/jobs` to `~/.riptide/workers/jobs`
    - Migration script provided: `scripts/migrate-job-storage.sh`
    - Rollback script provided: `scripts/rollback-job-migration.sh`

  ### Improved
  - Migrated CLI business logic to library crates (-1,772 LOC from CLI)
  - PDF helpers now in riptide-pdf library
  - Browser pool manager now in riptide-pool library
  - Cache operations now in riptide-cache library
  - Job management now in riptide-workers library

  ### Internal
  - CLI reduced from 5,200 to 3,428 LOC (-34%)
  - Clear separation of presentation and business logic
  - Libraries are now independently testable and reusable
  ```

### Step 6.7: Commit & Review (1 hour)
- [ ] Run full test suite: `cargo test --workspace`
- [ ] Format: `cargo fmt --all`
- [ ] Git operations:
  ```bash
  git add crates/riptide-cli/src/commands/job.rs
  git add crates/riptide-cli/src/commands/job_local.rs
  git add crates/riptide-workers/src/compat.rs
  git add crates/riptide-workers/src/lib.rs
  git add crates/riptide-workers/tests/
  git add scripts/migrate-job-storage.sh
  git add scripts/rollback-job-migration.sh
  git add docs/migration-guides/phase9-sprint1.md
  git add CHANGELOG.md
  git commit -m "refactor(workers): complete job management migration

  - Update CLI to use JobApiClient and LocalJobManager
  - Add compatibility layer and migration helpers
  - Create migration and rollback scripts
  - Move all tests to library
  - CLI job commands now <700 LOC total (was 1,420)

  BREAKING: Job storage location changed
  - Old: ~/.riptide/jobs
  - New: ~/.riptide/workers/jobs
  - Migration: Run scripts/migrate-job-storage.sh
  - Rollback: Run scripts/rollback-job-migration.sh

  Part of Phase 9 Sprint 1 (Day 6 - COMPLETE)
  "
  ```

- [ ] **Create PR**: Title: `[Phase9] Complete job management migration to library`
- [ ] **Request review**: Tag 3+ reviewers (high-risk change)

**Day 6 Complete**: ✅ 1,420 LOC migrated, migration script provided

---

## Post-Migration Validation (Day 7+)

### Full Test Suite
- [ ] Run all tests: `cargo test --workspace`
- [ ] Run clippy: `cargo clippy --workspace -- -D warnings`
- [ ] Run format check: `cargo fmt --all --check`
- [ ] Build all targets: `cargo build --workspace --all-targets`

### Performance Validation
- [ ] Run benchmarks: `cargo bench --workspace`
- [ ] Compare with baseline (from pre-flight)
- [ ] Verify CLI startup time: `time cargo run --bin riptide -- --version`
- [ ] Verify job submission latency

### Output Format Validation
- [ ] Compare all command outputs with baseline:
  ```bash
  ./tests/golden-file-compare.sh
  ```
- [ ] Verify JSON output unchanged
- [ ] Verify table output unchanged
- [ ] Verify error messages unchanged

### Documentation Check
- [ ] Build docs: `cargo doc --workspace --no-deps`
- [ ] Review rustdoc output for all libraries
- [ ] Verify migration guide complete
- [ ] Verify changelog accurate

### LOC Verification
```bash
# Before (baseline)
tokei crates/riptide-cli/src/commands/ | grep "Total"
# Should show ~5,200 LOC

# After
tokei crates/riptide-cli/src/commands/ | grep "Total"
# Should show ~3,428 LOC (-34%)

# Libraries added
tokei crates/riptide-pdf/src/ | grep "Total"
tokei crates/riptide-pool/src/ | grep "Total"
tokei crates/riptide-cache/src/ | grep "Total"
tokei crates/riptide-workers/src/ | grep "Total"
```

### Integration Smoke Tests
```bash
# PDF commands
cargo run --bin riptide -- pdf extract sample.pdf

# Cache commands
cargo run --bin riptide -- cache status
cargo run --bin riptide -- cache stats

# Job commands (API mode)
cargo run --bin riptide -- job submit --url https://example.com

# Job commands (Local mode)
cargo run --bin riptide -- job-local submit --url https://example.com
cargo run --bin riptide -- job-local list
cargo run --bin riptide -- job-local stats

# Browser pool (if used)
cargo run --bin riptide -- (check if browser pool commands exist)
```

---

## Final Checklist

### Code Quality
- [ ] All tests pass
- [ ] Zero clippy warnings
- [ ] Code formatted (rustfmt)
- [ ] No dead code warnings
- [ ] No unused imports
- [ ] All TODOs addressed or documented

### Documentation
- [ ] Migration guide complete
- [ ] API docs updated (rustdoc)
- [ ] Changelog entry added
- [ ] Breaking changes documented
- [ ] Examples work

### PRs & Reviews
- [ ] PR #1 (PDF helpers) merged
- [ ] PR #2 (Browser pool) merged
- [ ] PR #3 (Cache operations) merged
- [ ] PR #4 (Job management) merged
- [ ] All review comments addressed

### Deployment Prep
- [ ] Migration script tested
- [ ] Rollback script tested
- [ ] Backup procedure documented
- [ ] Release notes drafted
- [ ] Team trained on migration

---

## Success Metrics

### Quantitative (Measure)
```bash
# LOC reduction
tokei crates/riptide-cli/src/ --output json | jq '.Rust.code'
# Target: ~3,428 (-34%)

# Test coverage
cargo tarpaulin --workspace --out Html
# Target: >80% for libraries

# Build time
hyperfine 'cargo build --workspace'
# Target: <150s total, <30s CLI

# Binary size
ls -lh target/release/riptide
# Target: <8.5 MB
```

### Qualitative (Verify)
- [ ] Code is more maintainable
- [ ] Libraries are independently testable
- [ ] CLI is thin presentation layer
- [ ] No code duplication between CLI and libraries
- [ ] Clear separation of concerns

---

## Troubleshooting

### Common Issues

**Issue: Tests failing after migration**
```bash
# Capture detailed error
cargo test --workspace -- --nocapture > test-errors.txt

# Check specific test
cargo test -p <crate> <test-name> -- --exact --nocapture

# Compare with baseline
diff baseline-tests.txt test-errors.txt
```

**Issue: Circular dependency**
```bash
# Check dependency graph
cargo tree -p riptide-cli -i riptide-workers
cargo tree -p riptide-workers -i riptide-cli

# If circular:
# - Move shared types to riptide-types
# - Use trait objects for abstraction
```

**Issue: Output format changed**
```bash
# Capture current output
cargo run --bin riptide -- cache status > new-output.txt

# Compare
diff baseline-cache-status.txt new-output.txt

# If different: Fix formatting in CLI adapter
```

**Issue: Performance regression**
```bash
# Benchmark specific command
hyperfine 'cargo run --release --bin riptide -- job-local submit --url https://example.com'

# Compare with baseline
# If >5ms slower: Profile with flamegraph
cargo flamegraph --bin riptide -- job-local submit --url https://example.com
```

### Emergency Rollback
```bash
# If critical issue detected:
1. git revert <commit-hash>  # Revert migration commit
2. cargo build --release     # Rebuild CLI
3. cargo test --workspace    # Verify tests pass
4. ./scripts/rollback-job-migration.sh ~/.riptide/backup-YYYYMMDD
```

---

## Notes & Observations

### Day 1 Notes:
- [ ] Record any issues encountered
- [ ] Note any deviations from plan
- [ ] Document learnings

### Day 2 Notes:
- [ ] Record any issues encountered
- [ ] Note any deviations from plan
- [ ] Document learnings

### Day 3 Notes:
- [ ] Record any issues encountered
- [ ] Note any deviations from plan
- [ ] Document learnings

### Day 4 Notes:
- [ ] Record any issues encountered
- [ ] Note any deviations from plan
- [ ] Document learnings

### Day 5 Notes:
- [ ] Record any issues encountered
- [ ] Note any deviations from plan
- [ ] Document learnings

### Day 6 Notes:
- [ ] Record any issues encountered
- [ ] Note any deviations from plan
- [ ] Document learnings

---

**Completion Date**: ___________________
**Engineer**: ___________________
**Reviewer**: ___________________
**Status**: ___________________
