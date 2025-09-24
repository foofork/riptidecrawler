# Test Configuration Guide for EventMesh RipTide

## Overview

This guide provides specific instructions for configuring and running tests in the EventMesh RipTide project. It focuses on getting existing tests to pass and establishing a solid foundation for test-driven development.

## Test Configuration Structure

### Workspace Configuration

The project uses a Cargo workspace with the following test-related configuration:

```toml
# Root Cargo.toml (workspace configuration)
[workspace]
members = [
  "crates/riptide-core",
  "crates/riptide-api",
  "crates/riptide-headless",
  "crates/riptide-workers",
  "wasm/riptide-extractor-wasm",
]
```

### Core Testing Dependencies

#### Workspace Dependencies (`Cargo.toml`)
```toml
[workspace.dependencies]
# Core async runtime
tokio = { version = "1", features = ["macros", "rt-multi-thread", "signal", "time"] }

# HTTP testing
axum = { version = "0.7", features = ["json", "ws"] }
hyper = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["gzip", "brotli", "json", "cookies", "http2", "rustls-tls"] }

# Serialization for test data
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Other essential dependencies
anyhow = "1"
uuid = { version = "1", features = ["v4", "serde"] }
url = "2"
```

#### Core Library Tests (`crates/riptide-core/Cargo.toml`)
```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
tokio-test = "0.4"
wasmtime = { version = "26", features = ["component-model"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

#### API Tests (`crates/riptide-api/Cargo.toml`)
```toml
[dev-dependencies]
tokio-test = "0.4"
mockall = "0.13"
tempfile = "3.8"
proptest = "1.4"
criterion = { version = "0.5", features = ["html_reports"] }
httpmock = "0.7"
wiremock = "0.6"
rstest = "0.22"
axum-extra = { version = "0.9" }
futures-util = "0.3"
tokio-tungstenite = "0.23"
```

#### Dedicated Test Crate (`tests/Cargo.toml`)
```toml
[dependencies]
# Core dependencies
tokio = { workspace = true, features = ["test-util", "macros", "rt-multi-thread", "time"] }
tokio-test = "0.4"
anyhow = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }

# Testing frameworks
mockall = "0.13"
proptest = "1.5"
criterion = { version = "0.5", features = ["html_reports"] }
wiremock = "0.6"

# HTTP and web testing
reqwest = { workspace = true }
axum = { workspace = true }
tower = { workspace = true, features = ["test-util"] }
hyper = { workspace = true }

# WASM testing
wasmtime = { workspace = true }
wasmtime-wasi = { workspace = true }

# EventMesh crates
riptide-core = { path = "../crates/riptide-core" }
riptide-api = { path = "../crates/riptide-api" }
riptide-headless = { path = "../crates/riptide-headless" }

# Utilities
uuid = { workspace = true }
tracing = { workspace = true }
tracing-test = "0.2"
tempfile = "3.10"
rand = { workspace = true }
```

## Test Execution Commands

### Basic Commands

```bash
# Check all workspace packages compile (including tests)
cargo test --no-run --workspace

# Run all tests in workspace
cargo test --workspace

# Run tests for specific package
cargo test -p riptide-core
cargo test -p riptide-api

# Run specific test types
cargo test --lib                    # Library unit tests only
cargo test --bins                   # Binary tests only
cargo test --tests                  # Integration tests only
cargo test --doc                    # Documentation tests
```

### Advanced Test Commands

```bash
# Run tests with output
cargo test --workspace -- --nocapture

# Run specific test by name
cargo test test_session_creation --workspace

# Run tests with thread limit (for resource-constrained environments)
cargo test --workspace -- --test-threads=1

# Run tests with timing information
cargo test --workspace -- --report-time

# Run only fast tests (custom filter)
cargo test --workspace unit

# Run integration tests only
cargo test --workspace integration
```

### Benchmark Commands

```bash
# Run all benchmarks
cargo bench --workspace

# Run benchmarks for specific package
cargo bench -p riptide-core

# Generate HTML benchmark reports (requires criterion feature)
cargo bench --workspace -- --output-format html
```

## Test Organization Pattern

### Unit Tests (In Source Files)
```rust
// src/some_module.rs
pub struct MyComponent;

impl MyComponent {
    pub fn process(&self, input: &str) -> Result<String, String> {
        if input.is_empty() {
            Err("Input cannot be empty".to_string())
        } else {
            Ok(format!("processed: {}", input))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_valid_input() {
        let component = MyComponent;
        let result = component.process("hello");
        assert_eq!(result, Ok("processed: hello".to_string()));
    }

    #[test]
    fn test_process_empty_input() {
        let component = MyComponent;
        let result = component.process("");
        assert_eq!(result, Err("Input cannot be empty".to_string()));
    }

    #[tokio::test]
    async fn test_async_behavior() {
        // Async test example
        let component = MyComponent;
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            async {
                // Test async operation
                Ok::<(), &str>(())
            }
        ).await.expect("Test timed out").expect("Test failed");
    }
}
```

### Integration Tests (In `tests/` Directory)
```rust
// tests/integration/api_tests.rs
use riptide_api::create_app;
use axum::{
    http::StatusCode,
    Router,
};
use tower::ServiceExt;
use hyper::{Body, Request};

#[tokio::test]
async fn test_api_health_endpoint() {
    let app = create_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

### Mock-based Testing
```rust
use mockall::{automock, predicate::*};

#[automock]
pub trait HttpClient {
    async fn get(&self, url: &str) -> Result<String, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_with_mock() {
        let mut mock_client = MockHttpClient::new();

        mock_client
            .expect_get()
            .with(eq("https://example.com"))
            .times(1)
            .returning(|_| Ok("response body".to_string()));

        // Test code that uses mock_client
        let result = mock_client.get("https://example.com").await;
        assert_eq!(result, Ok("response body".to_string()));
    }
}
```

### Property-based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_url_parsing_properties(
        scheme in "https?",
        host in "[a-z]{1,20}",
        path in "/[a-z/]*"
    ) {
        let url_str = format!("{}://{}.com{}", scheme, host, path);

        // Property: valid URLs should always parse successfully
        let parsed_url = url::Url::parse(&url_str);
        prop_assert!(parsed_url.is_ok());

        // Property: scheme should be preserved
        if let Ok(url) = parsed_url {
            prop_assert_eq!(url.scheme(), &scheme);
        }
    }
}
```

## Test Configuration Files

### Test Environment Variables
```bash
# .env.test
RUST_LOG=debug
RUST_BACKTRACE=1
TEST_TIMEOUT=60
DATABASE_URL=sqlite::memory:
REDIS_URL=redis://localhost:6379/1
```

### Test Feature Flags
```toml
# Cargo.toml
[features]
default = ["pdf"]
pdf = ["pdfium-render"]
benchmarks = ["criterion", "geohash"]
test-utils = ["mockall", "tempfile"]

# Use in tests
[dev-dependencies]
my-crate = { path = ".", features = ["test-utils"] }
```

## Debugging Test Failures

### Common Test Debugging Commands
```bash
# Run specific failing test with full output
cargo test failing_test_name -- --nocapture

# Run with environment variables for debugging
RUST_LOG=debug cargo test failing_test_name

# Show test output even for passing tests
cargo test -- --nocapture

# Run with backtrace for panics
RUST_BACKTRACE=1 cargo test failing_test_name

# Run single-threaded to avoid race conditions
cargo test -- --test-threads=1
```

### Test Timeout Configuration
```rust
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_with_timeout() {
    timeout(Duration::from_secs(5), async {
        // Test operation that might hang
        slow_operation().await
    })
    .await
    .expect("Test timed out after 5 seconds")
    .expect("Test failed");
}
```

## Performance Testing Configuration

### Benchmark Setup
```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            black_box(my_function(black_box("input")))
        })
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

### Memory and Performance Monitoring
```rust
#[cfg(test)]
mod performance_tests {
    use std::alloc::{GlobalAlloc, System, Layout};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TrackingAllocator;

    static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

    unsafe impl GlobalAlloc for TrackingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let size = layout.size();
            ALLOCATED.fetch_add(size, Ordering::SeqCst);
            System.alloc(layout)
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            let size = layout.size();
            ALLOCATED.fetch_sub(size, Ordering::SeqCst);
            System.dealloc(ptr, layout)
        }
    }

    #[tokio::test]
    async fn test_memory_usage() {
        let initial = ALLOCATED.load(Ordering::SeqCst);

        // Perform operation
        expensive_operation().await;

        let final_usage = ALLOCATED.load(Ordering::SeqCst);
        let used = final_usage - initial;

        assert!(used < 10_000_000, "Used too much memory: {} bytes", used);
    }
}
```

## Continuous Integration Configuration

### GitHub Actions Test Configuration
```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [stable, beta, nightly]

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: dtolnay/rust-toolchain@master
      with:
        toolchain: ${{ matrix.rust }}

    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

    - name: Run tests
      run: |
        cargo test --workspace --verbose
        cargo test --workspace --release --verbose

    - name: Run benchmarks
      if: matrix.rust == 'stable'
      run: cargo bench --workspace

    - name: Check documentation
      if: matrix.rust == 'stable'
      run: cargo doc --workspace --no-deps
```

## Test Data Management

### Test Fixtures Organization
```
tests/
├── fixtures/
│   ├── mod.rs              # Fixture loading utilities
│   ├── test_data.rs        # Test data structures
│   ├── sample_data/        # Sample files
│   │   ├── html/
│   │   ├── json/
│   │   └── pdf/
│   └── golden/             # Golden test files
│       ├── expected_outputs/
│       └── test_inputs/
```

### Fixture Loading Utilities
```rust
// tests/fixtures/mod.rs
use std::path::{Path, PathBuf};

pub struct TestFixtures;

impl TestFixtures {
    pub fn fixture_path(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    pub fn load_html(name: &str) -> String {
        let path = Self::fixture_path(&format!("sample_data/html/{}.html", name));
        std::fs::read_to_string(path).expect("Failed to load HTML fixture")
    }

    pub fn load_json<T>(name: &str) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        let path = Self::fixture_path(&format!("sample_data/json/{}.json", name));
        let content = std::fs::read_to_string(path).expect("Failed to load JSON fixture");
        serde_json::from_str(&content).expect("Failed to parse JSON fixture")
    }
}
```

## Test Maintenance Guidelines

### 1. Test Naming Conventions
- Unit tests: `test_[component]_[behavior]_[expected_outcome]`
- Integration tests: `test_[feature]_[scenario]_[integration]`
- Property tests: `prop_[property_being_tested]`
- Benchmark tests: `bench_[operation]_[scenario]`

### 2. Test Organization Rules
- Keep unit tests in the same file as the code they test
- Use integration test crates for cross-component testing
- Group related tests in modules
- Use `#[cfg(test)]` for test-only code

### 3. Test Data Best Practices
- Use factories for complex test data creation
- Avoid hardcoded values; use constants or generators
- Clean up test resources in teardown methods
- Use temporary files/directories for file system tests

### 4. Performance Test Guidelines
- Run performance tests separately from unit tests
- Establish baseline measurements
- Use statistical significance testing
- Monitor for performance regressions in CI

This configuration guide provides a comprehensive framework for testing in the EventMesh RipTide project, focusing on reliability, maintainability, and comprehensive coverage while ensuring existing tests can pass successfully.