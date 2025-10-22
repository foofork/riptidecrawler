# CLI Testing Research - Phase 6 Infrastructure

**Research Date:** 2025-10-21
**Agent:** Researcher (Hive Mind Phase 6)
**Purpose:** Document best practices for CLI integration testing and chaos engineering

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [CLI Command Inventory](#cli-command-inventory)
3. [assert_cmd & assert_fs Best Practices](#assert_cmd--assert_fs-best-practices)
4. [Chaos Engineering Patterns](#chaos-engineering-patterns)
5. [Critical Failure Paths](#critical-failure-paths)
6. [Implementation Recommendations](#implementation-recommendations)
7. [References](#references)

---

## Executive Summary

This research document provides comprehensive guidance for implementing CLI integration tests and chaos engineering for the RipTide CLI (`riptide-cli`). The RipTide CLI is a complex application with 19 top-level commands, 60+ subcommands, and multiple execution modes (API, direct/offline, API-only).

### Key Findings

1. **Testing Framework**: `assert_cmd` (v2.0.17) + `assert_fs` (v1.1.3) are industry-standard for Rust CLI testing
2. **Command Coverage**: 19 main commands require integration testing across 3 execution modes
3. **Critical Paths**: 552 error-returning operations and 197 panic-prone operations identified
4. **Chaos Engineering**: File system, network, and resource exhaustion are primary failure injection targets
5. **Existing Gaps**: No CLI integration tests exist; only unit tests for cache and metrics modules

### Immediate Actions Required

- Add `assert_cmd` and `assert_fs` to dev-dependencies
- Implement CLI integration test harness
- Create chaos engineering test suite
- Establish baseline test coverage metrics

---

## CLI Command Inventory

### Analysis Methodology

Analyzed `/workspaces/eventmesh/crates/riptide-cli/src/main.rs` and `/workspaces/eventmesh/crates/riptide-cli/src/commands/mod.rs` to identify all CLI commands and their execution patterns.

### Top-Level Commands (19)

| Command | Subcommands | Priority | Complexity | Test Strategy |
|---------|-------------|----------|------------|---------------|
| `extract` | - | **CRITICAL** | High | Full coverage (API, direct, offline) |
| `render` | - | **CRITICAL** | High | Full coverage + resource limits |
| `crawl` | - | **CRITICAL** | High | Full coverage + timeout handling |
| `search` | - | High | Medium | API + error scenarios |
| `cache` | 5 (status, clear, warm, validate, stats) | **CRITICAL** | Medium | State management + persistence |
| `wasm` | 3 (info, benchmark, health) | High | Medium | Resource monitoring |
| `stealth` | 4 (configure, test, info, generate) | Medium | Medium | Configuration validation |
| `domain` | Multiple | Medium | Medium | Profile management |
| `health` | - | **CRITICAL** | Low | API connectivity |
| `metrics` | 3 (show, tail, export) | **CRITICAL** | Medium | Data collection + export |
| `validate` | - | High | Low | Configuration checks |
| `system-check` | - | High | Medium | System resource validation |
| `tables` | - | Medium | Medium | HTML parsing |
| `schema` | Multiple | Medium | Medium | API schema operations |
| `pdf` | Multiple | Medium | High | PDF processing (feature-gated) |
| `job` | 7 (submit, list, status, logs, cancel, retry, delete) | **CRITICAL** | High | Job lifecycle management |
| `job-local` | Similar to job | High | High | Offline job management |
| `session` | Multiple | High | Medium | Session state management |

### Execution Modes

All commands support three execution modes that must be tested independently:

1. **API Mode** (default): `--api-url http://localhost:8080`
2. **Direct/Offline Mode**: `--direct` flag (no API server required)
3. **API-Only Mode**: `--api-only` flag (fails if API unavailable)

### Subcommand Details

#### Cache Subcommands
```
cache status              # Show cache statistics
cache clear [--domain]    # Clear cache entries
cache warm --url-file     # Preload URLs
cache validate            # Integrity checks
cache stats               # Detailed statistics
```

#### Metrics Subcommands
```
metrics show              # Current metrics summary
metrics tail --interval   # Live monitoring
metrics export --format   # Export to prom/json/csv
```

#### Job Management
```
job submit --url --method [--batch] [--priority]
job list [--status] [--priority] [--tag]
job status --job-id [--watch]
job logs --job-id [--follow]
job cancel --job-id
job retry --job-id
job delete --job-id
```

#### PDF Commands (feature: pdf)
```
pdf extract --input [--pages]
pdf metadata --input
pdf convert --input --output
pdf search --input --query
```

#### Stealth Commands
```
stealth configure --preset [--ua-file]
stealth test --url --preset
stealth info
stealth generate --level [--output]
```

---

## assert_cmd & assert_fs Best Practices

### Overview

`assert_cmd` provides fluent assertions for testing command-line interfaces, while `assert_fs` offers filesystem fixtures and assertions.

### Installation

```toml
# crates/riptide-cli/Cargo.toml
[dev-dependencies]
assert_cmd = "2.0.17"
assert_fs = "1.1.3"
predicates = "3.1"      # For advanced assertions
tempfile = "3.13"       # Already present
```

### Core Patterns

#### 1. Basic Command Execution

```rust
use assert_cmd::Command;

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains(env!("CARGO_PKG_VERSION")));
}
```

#### 2. Testing Exit Codes

```rust
#[test]
fn test_extract_missing_url() {
    Command::cargo_bin("riptide").unwrap()
        .args(&["extract"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("required"));
}
```

#### 3. Filesystem Fixtures with assert_fs

```rust
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_extract_to_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let output_file = temp.child("output.json");

    Command::cargo_bin("riptide").unwrap()
        .args(&["extract", "--url", "https://example.com",
                "--file", output_file.path().to_str().unwrap(),
                "--direct"])
        .assert()
        .success();

    output_file.assert(predicate::path::exists());
    output_file.assert(predicate::path::is_file());

    let content = std::fs::read_to_string(output_file.path()).unwrap();
    assert!(content.contains("content"));
}
```

#### 4. Environment Variable Testing

```rust
#[test]
fn test_api_url_from_env() {
    Command::cargo_bin("riptide").unwrap()
        .env("RIPTIDE_API_URL", "http://custom:8080")
        .env("RIPTIDE_API_KEY", "test-key")
        .args(&["health"])
        .assert()
        .code(predicate::in_iter([0, 1])); // May fail if server unavailable
}
```

#### 5. stdin/stdout Testing

```rust
#[test]
fn test_extract_from_stdin() {
    let html = r#"<html><body><h1>Test</h1></body></html>"#;

    Command::cargo_bin("riptide").unwrap()
        .args(&["extract", "--stdin", "--local"])
        .write_stdin(html)
        .assert()
        .success()
        .stdout(predicate::str::contains("Test"));
}
```

#### 6. JSON Output Validation

```rust
use serde_json::Value;

#[test]
fn test_json_output() {
    let output = Command::cargo_bin("riptide").unwrap()
        .args(&["cache", "stats", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).unwrap();
    assert!(json["total_entries"].is_number());
}
```

#### 7. Timeout Testing

```rust
use std::time::Duration;

#[test]
fn test_command_timeout() {
    Command::cargo_bin("riptide").unwrap()
        .args(&["crawl", "--url", "https://slow-site.com", "--direct"])
        .timeout(Duration::from_secs(5))
        .assert()
        .failure(); // Should timeout
}
```

#### 8. Multi-Execution Testing

```rust
#[test]
fn test_cache_persistence() {
    let temp = assert_fs::TempDir::new().unwrap();
    let cache_dir = temp.child("cache");

    // First execution - populate cache
    Command::cargo_bin("riptide").unwrap()
        .env("RIPTIDE_CACHE_DIR", cache_dir.path())
        .args(&["extract", "--url", "https://example.com", "--direct"])
        .assert()
        .success();

    // Second execution - verify cache hit
    Command::cargo_bin("riptide").unwrap()
        .env("RIPTIDE_CACHE_DIR", cache_dir.path())
        .args(&["cache", "stats"])
        .assert()
        .success()
        .stdout(predicate::str::contains("total_entries: 1"));
}
```

### Best Practices Summary

1. **Use `cargo_bin()`**: Always use `Command::cargo_bin()` to test the actual binary
2. **Test All Output Formats**: Verify text, JSON, and table outputs independently
3. **Validate Exit Codes**: Check success (0) and failure (non-zero) scenarios
4. **Test Environment**: Cover `RIPTIDE_API_URL`, `RIPTIDE_API_KEY`, `RIPTIDE_DIRECT`, etc.
5. **Temporary Directories**: Always use `assert_fs::TempDir` for file operations
6. **Predicates**: Use `predicates` crate for flexible assertions
7. **Isolation**: Each test should be independent and not affect others
8. **Timeout Protection**: Set timeouts for network-dependent tests

### Advanced Patterns

#### Property-Based Testing with proptest

```toml
[dev-dependencies]
proptest = "1.8.0"
```

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_extract_url_variations(url in "https?://[a-z]+\\.com(/[a-z]*)?") {
        Command::cargo_bin("riptide").unwrap()
            .args(&["extract", "--url", &url, "--direct"])
            .assert()
            .code(predicate::in_iter([0, 1]));
    }
}
```

---

## Chaos Engineering Patterns

### Overview

Chaos engineering involves deliberately injecting failures to test system resilience. For CLI applications, this includes filesystem failures, network issues, resource exhaustion, and signal handling.

### Failure Injection Categories

#### 1. Filesystem Failures

**Scenarios:**
- Read-only filesystems
- No disk space
- Permission denied
- Corrupted files
- Concurrent access conflicts

**Implementation:**

```rust
use std::fs;
use std::os::unix::fs::PermissionsExt;

#[test]
fn test_readonly_cache_dir() {
    let temp = assert_fs::TempDir::new().unwrap();
    let cache_dir = temp.child("cache");
    cache_dir.create_dir_all().unwrap();

    // Make directory read-only
    let mut perms = fs::metadata(cache_dir.path()).unwrap().permissions();
    perms.set_mode(0o444);
    fs::set_permissions(cache_dir.path(), perms).unwrap();

    Command::cargo_bin("riptide").unwrap()
        .env("RIPTIDE_CACHE_DIR", cache_dir.path())
        .args(&["extract", "--url", "https://example.com", "--direct"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("permission denied"));
}

#[test]
fn test_corrupted_cache_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let cache_file = temp.child("cache.db");

    // Write corrupted data
    fs::write(cache_file.path(), b"corrupted binary data \x00\xFF").unwrap();

    Command::cargo_bin("riptide").unwrap()
        .env("RIPTIDE_CACHE_FILE", cache_file.path())
        .args(&["cache", "validate"])
        .assert()
        .failure();
}
```

#### 2. Network Failures

**Scenarios:**
- Connection timeout
- DNS resolution failure
- HTTP 500 errors
- Connection reset
- Partial responses

**Implementation:**

```rust
#[test]
fn test_api_unavailable() {
    Command::cargo_bin("riptide").unwrap()
        .env("RIPTIDE_API_URL", "http://localhost:9999") // Non-existent
        .args(&["health", "--api-only"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unavailable"));
}

#[test]
fn test_api_timeout() {
    Command::cargo_bin("riptide").unwrap()
        .env("RIPTIDE_API_URL", "http://httpbin.org/delay/60")
        .env("RIPTIDE_TIMEOUT", "1")
        .args(&["extract", "--url", "https://example.com"])
        .timeout(Duration::from_secs(5))
        .assert()
        .failure();
}
```

#### 3. Resource Exhaustion

**Scenarios:**
- Out of memory
- CPU saturation
- File descriptor exhaustion
- Thread pool exhaustion

**Implementation:**

```rust
#[test]
fn test_memory_limit() {
    // Requires ulimit or cgroups
    Command::cargo_bin("riptide").unwrap()
        .args(&["crawl", "--url", "https://large-site.com",
                "--max-pages", "10000", "--direct"])
        .env("RIPTIDE_MAX_MEMORY_MB", "100")
        .assert()
        .code(predicate::in_iter([0, 1, 137])); // 137 = SIGKILL
}

#[test]
fn test_concurrent_job_limit() {
    // Submit many jobs concurrently
    let handles: Vec<_> = (0..100)
        .map(|i| {
            std::thread::spawn(move || {
                Command::cargo_bin("riptide").unwrap()
                    .args(&["job", "submit", "--url", &format!("https://example.com/{}", i)])
                    .assert()
                    .code(predicate::in_iter([0, 1]));
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
```

#### 4. Signal Handling

**Scenarios:**
- SIGINT (Ctrl+C)
- SIGTERM (graceful shutdown)
- SIGKILL (forced termination)

**Implementation:**

```rust
use std::process::{Command as StdCommand, Stdio};
use std::time::Duration;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

#[test]
fn test_sigint_graceful_shutdown() {
    let mut child = StdCommand::new(env!("CARGO_BIN_EXE_riptide"))
        .args(&["crawl", "--url", "https://example.com", "--max-pages", "1000"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // Let it run for a bit
    std::thread::sleep(Duration::from_secs(2));

    // Send SIGINT
    kill(Pid::from_raw(child.id() as i32), Signal::SIGINT).unwrap();

    // Should exit gracefully
    let exit_status = child.wait_timeout(Duration::from_secs(5)).unwrap();
    assert!(exit_status.is_some());
}
```

#### 5. Race Conditions

**Scenarios:**
- Concurrent cache access
- Simultaneous config updates
- Job state transitions

**Implementation:**

```rust
#[test]
fn test_concurrent_cache_writes() {
    let temp = assert_fs::TempDir::new().unwrap();
    let cache_dir = temp.path().to_str().unwrap();

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let cache_dir = cache_dir.to_string();
            std::thread::spawn(move || {
                Command::cargo_bin("riptide").unwrap()
                    .env("RIPTIDE_CACHE_DIR", cache_dir)
                    .args(&["extract", "--url", &format!("https://example.com/{}", i), "--direct"])
                    .assert()
                    .success();
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify cache integrity
    Command::cargo_bin("riptide").unwrap()
        .env("RIPTIDE_CACHE_DIR", cache_dir)
        .args(&["cache", "validate"])
        .assert()
        .success();
}
```

### Chaos Engineering Testing Harness

```rust
/// Chaos testing configuration
struct ChaosConfig {
    inject_network_failures: bool,
    inject_fs_failures: bool,
    inject_memory_pressure: bool,
    failure_rate: f64, // 0.0 to 1.0
}

impl ChaosConfig {
    fn apply_to_command(&self, cmd: &mut Command) {
        if self.inject_network_failures && rand::random::<f64>() < self.failure_rate {
            cmd.env("RIPTIDE_API_URL", "http://localhost:9999");
        }

        if self.inject_memory_pressure {
            cmd.env("RIPTIDE_MAX_MEMORY_MB", "50");
        }
    }
}

#[test]
fn test_chaos_extract() {
    let config = ChaosConfig {
        inject_network_failures: true,
        inject_fs_failures: true,
        inject_memory_pressure: true,
        failure_rate: 0.3,
    };

    for _ in 0..100 {
        let mut cmd = Command::cargo_bin("riptide").unwrap();
        config.apply_to_command(&mut cmd);

        cmd.args(&["extract", "--url", "https://example.com", "--direct"])
            .assert()
            .code(predicate::in_iter([0, 1])); // Accept success or graceful failure
    }
}
```

### Fault Injection Library Integration

Consider using existing Rust chaos engineering tools:

```toml
[dev-dependencies]
# Potential chaos engineering libraries
fault-injection = "0.1"  # If available
chaos-mesh-client = "0.1" # For orchestrated chaos
```

---

## Critical Failure Paths

### Analysis Results

**Methodology:** Analyzed codebase using grep patterns to identify error-prone code paths.

#### Error-Returning Operations (552 total)

Files with most `Result<` and `.await?` patterns:
- `job/manager.rs`: 37 occurrences
- `wasm_aot_cache.rs`: 28 occurrences
- `job_local.rs`: 28 occurrences
- `metrics/mod.rs`: 23 occurrences
- `commands/extract.rs`: 20 occurrences

#### Panic-Prone Operations (197 total)

Files with most `unwrap()`, `expect()`, `panic!`, `anyhow::bail!`:
- `cache/manager.rs`: 22 occurrences
- `cache/storage.rs`: 19 occurrences
- `metrics/storage.rs`: 18 occurrences
- `metrics/collector.rs`: 15 occurrences
- `adaptive_timeout.rs`: 14 occurrences

### Critical Path Categories

#### 1. **Cache Operations** (PRIORITY: CRITICAL)

**Failure Scenarios:**
- Cache directory doesn't exist
- No write permissions
- Disk full
- Corrupted cache database
- Concurrent access conflicts

**Test Coverage Required:**
```rust
#[test] fn test_cache_dir_not_found()
#[test] fn test_cache_permission_denied()
#[test] fn test_cache_disk_full()
#[test] fn test_cache_corruption_recovery()
#[test] fn test_cache_concurrent_writes()
#[test] fn test_cache_lru_eviction_edge_cases()
```

#### 2. **WASM Module Loading** (PRIORITY: CRITICAL)

**Failure Scenarios:**
- WASM module not found
- Invalid WASM binary
- Compilation failure
- Initialization timeout
- Runtime crash

**Test Coverage Required:**
```rust
#[test] fn test_wasm_module_not_found()
#[test] fn test_wasm_invalid_binary()
#[test] fn test_wasm_compilation_failure()
#[test] fn test_wasm_init_timeout()
#[test] fn test_wasm_fallback_to_api()
```

#### 3. **API Client** (PRIORITY: CRITICAL)

**Failure Scenarios:**
- Connection timeout
- Authentication failure
- Rate limiting
- API server down
- Malformed responses

**Test Coverage Required:**
```rust
#[test] fn test_api_connection_timeout()
#[test] fn test_api_auth_failure()
#[test] fn test_api_rate_limit()
#[test] fn test_api_server_unavailable()
#[test] fn test_api_malformed_response()
#[test] fn test_api_fallback_to_direct()
```

#### 4. **Job Management** (PRIORITY: CRITICAL)

**Failure Scenarios:**
- Job storage corruption
- Job state inconsistency
- Concurrent job modifications
- Job timeout
- Job cancellation race

**Test Coverage Required:**
```rust
#[test] fn test_job_storage_corruption()
#[test] fn test_job_state_consistency()
#[test] fn test_concurrent_job_cancel()
#[test] fn test_job_timeout_handling()
#[test] fn test_job_retry_backoff()
```

#### 5. **Browser Pool** (PRIORITY: HIGH)

**Failure Scenarios:**
- Browser launch failure
- Browser crash
- CDP connection lost
- Page load timeout
- Resource exhaustion

**Test Coverage Required:**
```rust
#[test] fn test_browser_launch_failure()
#[test] fn test_browser_crash_recovery()
#[test] fn test_cdp_connection_lost()
#[test] fn test_page_load_timeout()
#[test] fn test_browser_pool_exhaustion()
```

#### 6. **Metrics Collection** (PRIORITY: HIGH)

**Failure Scenarios:**
- Metrics storage full
- Metrics file corruption
- Export format errors
- Aggregation overflow

**Test Coverage Required:**
```rust
#[test] fn test_metrics_storage_full()
#[test] fn test_metrics_corruption_recovery()
#[test] fn test_metrics_export_formats()
#[test] fn test_metrics_aggregation_edge_cases()
```

#### 7. **Session Management** (PRIORITY: MEDIUM)

**Failure Scenarios:**
- Session file corruption
- Cookie expiration
- Session timeout
- Concurrent session access

**Test Coverage Required:**
```rust
#[test] fn test_session_file_corruption()
#[test] fn test_session_cookie_expiration()
#[test] fn test_session_timeout()
#[test] fn test_concurrent_session_use()
```

### Critical Path Priority Matrix

| Component | Failure Impact | Frequency | Test Priority |
|-----------|----------------|-----------|---------------|
| Cache Operations | High | High | **P0** |
| API Client | High | High | **P0** |
| WASM Loading | High | Medium | **P0** |
| Job Management | High | Medium | **P0** |
| Browser Pool | Medium | Medium | **P1** |
| Metrics Collection | Low | High | **P1** |
| Session Management | Medium | Low | **P2** |

---

## Implementation Recommendations

### Phase 1: Foundation (Week 1)

1. **Add Dependencies**
   ```toml
   [dev-dependencies]
   assert_cmd = "2.0.17"
   assert_fs = "1.1.3"
   predicates = "3.1"
   proptest = "1.8.0"
   ```

2. **Create Test Infrastructure**
   - `/workspaces/eventmesh/crates/riptide-cli/tests/integration/`
   - `/workspaces/eventmesh/crates/riptide-cli/tests/integration/common/mod.rs` (test utilities)
   - `/workspaces/eventmesh/crates/riptide-cli/tests/integration/chaos/mod.rs` (chaos framework)

3. **Baseline Tests**
   - Version output
   - Help text
   - Invalid command detection
   - Environment variable handling

### Phase 2: Core Command Tests (Week 2)

1. **Extract Command** (P0)
   - All execution modes (API, direct, API-only)
   - All engines (auto, raw, wasm, headless)
   - All methods (wasm, css, llm, regex, auto)
   - Error scenarios (404, timeout, invalid HTML)

2. **Cache Command** (P0)
   - All subcommands
   - Persistence tests
   - Corruption recovery
   - Concurrent access

3. **Health Command** (P0)
   - API connectivity
   - Timeout handling
   - Offline mode

### Phase 3: Job Management Tests (Week 3)

1. **Job Submit/List/Status** (P0)
   - Lifecycle testing
   - State transitions
   - Concurrent operations
   - Error recovery

2. **Job Logs** (P1)
   - Follow mode
   - Filtering
   - Large log files

### Phase 4: Chaos Engineering (Week 4)

1. **Filesystem Chaos**
   - Read-only scenarios
   - Disk full
   - Permission denied
   - Corrupted files

2. **Network Chaos**
   - Connection timeouts
   - DNS failures
   - HTTP errors
   - Partial responses

3. **Resource Chaos**
   - Memory limits
   - CPU throttling
   - File descriptor limits
   - Thread exhaustion

### Phase 5: Advanced Testing (Week 5+)

1. **Property-Based Testing**
   - URL validation
   - Configuration combinations
   - Edge case discovery

2. **Performance Benchmarks**
   - Command startup time
   - Throughput testing
   - Memory profiling
   - Regression detection

3. **Fuzzing**
   - Input validation
   - Malformed data handling
   - Security testing

### Test Organization

```
crates/riptide-cli/tests/
├── integration/
│   ├── common/
│   │   ├── mod.rs              # Shared utilities
│   │   ├── fixtures.rs         # Test data
│   │   └── assertions.rs       # Custom assertions
│   ├── commands/
│   │   ├── extract_tests.rs    # Extract command tests
│   │   ├── cache_tests.rs      # Cache command tests
│   │   ├── job_tests.rs        # Job command tests
│   │   └── ...
│   ├── chaos/
│   │   ├── mod.rs              # Chaos framework
│   │   ├── filesystem.rs       # FS chaos tests
│   │   ├── network.rs          # Network chaos tests
│   │   └── resources.rs        # Resource chaos tests
│   └── scenarios/
│       ├── offline_mode.rs     # Full offline scenarios
│       └── api_fallback.rs     # Fallback scenarios
└── property/
    └── url_validation.rs       # Property-based tests
```

### CI/CD Integration

```yaml
# .github/workflows/cli-integration-tests.yml
name: CLI Integration Tests

on: [push, pull_request]

jobs:
  cli-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      - name: Run integration tests
        run: cargo test --package riptide-cli --test '*' -- --test-threads=1

      - name: Run chaos tests
        run: cargo test --package riptide-cli --test 'chaos*' -- --test-threads=1

      - name: Generate coverage
        run: cargo llvm-cov --package riptide-cli --html
```

### Metrics & Monitoring

Track the following metrics:
- Test coverage percentage (target: 80%+)
- Number of commands tested
- Number of failure scenarios covered
- Test execution time
- Flaky test rate

---

## References

### Rust CLI Testing

1. **assert_cmd Documentation**: https://docs.rs/assert_cmd/
2. **assert_fs Documentation**: https://docs.rs/assert_fs/
3. **Predicates Documentation**: https://docs.rs/predicates/
4. **CLI Testing Guide**: https://rust-cli.github.io/book/tutorial/testing.html

### Chaos Engineering

1. **Chaos Engineering Principles**: https://principlesofchaos.org/
2. **Chaos Mesh**: https://chaos-mesh.org/
3. **Fault Injection Patterns**: https://www.microsoft.com/en-us/research/publication/chaos-engineering/

### Rust Testing Best Practices

1. **The Rust Book - Testing**: https://doc.rust-lang.org/book/ch11-00-testing.html
2. **Rust By Example - Testing**: https://doc.rust-lang.org/rust-by-example/testing.html
3. **Property Testing with proptest**: https://altsysrq.github.io/proptest-book/

### RipTide-Specific

1. **Existing Test Patterns**: `/workspaces/eventmesh/crates/riptide-cli/tests/cache_tests.rs`
2. **Metrics Integration Tests**: `/workspaces/eventmesh/crates/riptide-cli/tests/metrics_integration_test.rs`
3. **Command Structure**: `/workspaces/eventmesh/crates/riptide-cli/src/commands/mod.rs`

---

## Appendix A: Command Execution Matrix

| Command | API Mode | Direct Mode | API-Only | Priority |
|---------|----------|-------------|----------|----------|
| extract | ✅ | ✅ | ✅ | P0 |
| render | ✅ | ✅ | ✅ | P0 |
| crawl | ✅ | ✅ | ✅ | P0 |
| search | ✅ | ❌ | ✅ | P1 |
| cache * | ✅ | ✅ | ✅ | P0 |
| wasm * | ✅ | ✅ | ✅ | P1 |
| stealth * | ✅ | ✅ | ❌ | P2 |
| domain * | ✅ | ❌ | ✅ | P2 |
| health | ✅ | ❌ | ✅ | P0 |
| metrics * | ✅ | ✅ | ❌ | P0 |
| validate | ✅ | ✅ | ✅ | P1 |
| system-check | ✅ | ✅ | ✅ | P1 |
| tables | ✅ | ✅ | ✅ | P2 |
| schema * | ✅ | ❌ | ✅ | P2 |
| pdf * | ✅ | ✅ | ✅ | P2 |
| job * | ✅ | ❌ | ✅ | P0 |
| job-local * | ❌ | ✅ | ❌ | P1 |
| session * | ✅ | ✅ | ❌ | P1 |

*Note: Commands marked with * have subcommands*

---

## Appendix B: Error Code Inventory

Identified error handling patterns across the codebase:

| Pattern | Occurrences | Files | Risk Level |
|---------|-------------|-------|------------|
| `Result<T>` returns | 552 | 45 | Medium |
| `unwrap()` | 132 | 28 | High |
| `expect()` | 53 | 22 | High |
| `panic!()` | 8 | 6 | Critical |
| `anyhow::bail!()` | 4 | 3 | Medium |

**Recommendation**: Prioritize testing code paths with `unwrap()` and `panic!()` calls.

---

## Appendix C: Sample Test Template

```rust
//! Integration tests for [COMMAND] command
//!
//! Tests cover:
//! - Basic functionality
//! - Error scenarios
//! - Execution modes (API, direct, API-only)
//! - Output formats (text, json, table)
//! - Edge cases

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Helper to create a test command
fn riptide_cmd() -> Command {
    Command::cargo_bin("riptide").unwrap()
}

#[test]
fn test_[command]_success() {
    riptide_cmd()
        .args(&["[command]", "--arg", "value"])
        .assert()
        .success()
        .stdout(predicate::str::contains("expected output"));
}

#[test]
fn test_[command]_missing_required_arg() {
    riptide_cmd()
        .args(&["[command]"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_[command]_json_output() {
    let output = riptide_cmd()
        .args(&["[command]", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert!(json.is_object());
}

#[test]
fn test_[command]_direct_mode() {
    riptide_cmd()
        .args(&["[command]", "--direct"])
        .assert()
        .success();
}

#[test]
fn test_[command]_chaos_readonly_fs() {
    let temp = assert_fs::TempDir::new().unwrap();
    let output = temp.child("output.json");

    // Make read-only
    let mut perms = std::fs::metadata(temp.path()).unwrap().permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o444);
    std::fs::set_permissions(temp.path(), perms).unwrap();

    riptide_cmd()
        .args(&["[command]", "--file", output.path().to_str().unwrap()])
        .assert()
        .failure();
}
```

---

**Document Status:** COMPLETE
**Next Steps:** Implement Phase 1 (Foundation) tests
**Owner:** Coder Agent (Phase 6 Implementation)
