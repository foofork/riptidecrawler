# EventMesh Testing Guide

**Version**: 1.0
**Date**: 2025-10-21
**For**: Developers contributing to EventMesh

## Table of Contents
1. [Introduction](#introduction)
2. [Test Philosophy](#test-philosophy)
3. [Getting Started](#getting-started)
4. [Writing Tests](#writing-tests)
5. [Test Categories](#test-categories)
6. [Best Practices](#best-practices)
7. [Common Patterns](#common-patterns)
8. [Debugging Tests](#debugging-tests)
9. [CI/CD Integration](#cicd-integration)

## Introduction

This guide helps you write effective tests for EventMesh following our London School TDD approach. Whether you're fixing a bug, adding a feature, or improving performance, tests are your safety net and documentation.

### Our Testing Goals

- **≥80% code coverage** across all components
- **Zero panic guarantee** under tested conditions
- **Fast feedback** for developers (< 5 minutes full suite)
- **Clear documentation** through test names and structure
- **Reliable CI/CD** with minimal flaky tests

## Test Philosophy

### London School TDD (Mockist Approach)

We follow the London School TDD methodology, which emphasizes:

1. **Outside-In Development**: Start with acceptance tests, drive inward
2. **Behavior Over State**: Test how objects collaborate, not internal state
3. **Mock Collaborators**: Isolate the system under test
4. **Interface-First**: Design interfaces through testing
5. **Contract Testing**: Verify interactions, not implementations

### Test Pyramid

```
         /\
        /E2E\         Few (< 20) - Slow (seconds) - Comprehensive
       /------\
      /Integration\ Moderate (50-100) - Medium (< 1s) - Contracts
     /----------\
    /    Unit    \  Many (200+) - Fast (< 10ms) - Focused
   /--------------\
```

**Aim for**:
- 70% Unit Tests
- 20% Integration Tests
- 10% E2E Tests

## Getting Started

### Prerequisites

```bash
# Install Rust and Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install test dependencies
cargo build --workspace

# Install coverage tools
cargo install cargo-tarpaulin

# Install benchmarking tools
cargo install criterion
```

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run specific category
cargo test --test 'unit*'           # Unit tests only
cargo test --test 'integration*'    # Integration tests only
cargo test --test 'e2e*'            # E2E tests only

# Run with output
cargo test -- --nocapture

# Run single test
cargo test test_rate_limiter_blocks_overflow

# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Run tests in parallel
cargo test -- --test-threads=8
```

### Coverage Analysis

```bash
# Generate coverage report
cargo tarpaulin --workspace --out Html --output-dir coverage/

# View coverage
open coverage/tarpaulin-report.html

# Coverage with exclusions
cargo tarpaulin --workspace --exclude-files 'tests/*' 'benches/*'
```

## Writing Tests

### Test-Driven Development Workflow

**Red-Green-Refactor Cycle**:

```
1. RED: Write a failing test
2. GREEN: Write minimal code to pass
3. REFACTOR: Improve design while keeping tests green
```

### Example: Adding Rate Limiting Feature

#### Step 1: Write the Test (RED)

```rust
// tests/unit/rate_limiter_tests.rs

#[test]
fn test_rate_limiter_blocks_requests_over_limit() {
    // Arrange
    let config = RateLimiterConfig {
        limit: 10,
        window: Duration::from_secs(1),
    };
    let limiter = RateLimiter::new(config);

    // Act - Send 15 requests
    let mut results = Vec::new();
    for _ in 0..15 {
        results.push(limiter.allow());
    }

    // Assert
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 10);
    assert_eq!(results.iter().filter(|r| r.is_err()).count(), 5);
}
```

#### Step 2: Implement (GREEN)

```rust
// src/rate_limiter.rs

pub struct RateLimiter {
    config: RateLimiterConfig,
    state: Arc<Mutex<State>>,
}

impl RateLimiter {
    pub fn new(config: RateLimiterConfig) -> Self {
        // Minimal implementation to pass test
        Self {
            config,
            state: Arc::new(Mutex::new(State::default())),
        }
    }

    pub fn allow(&self) -> Result<(), RateLimitError> {
        let mut state = self.state.lock().unwrap();
        if state.count < self.config.limit {
            state.count += 1;
            Ok(())
        } else {
            Err(RateLimitError::LimitExceeded)
        }
    }
}
```

#### Step 3: Refactor (REFACTOR)

```rust
// Improve design - add time-based windows, better error messages, etc.
```

### Mock-Based Testing

```rust
use mockall::predicate::*;
use mockall::*;

// Define mock trait
#[automock]
pub trait HttpClient {
    async fn get(&self, url: &str) -> Result<String, Error>;
}

#[tokio::test]
async fn test_spider_uses_http_client() {
    // Arrange - Create mock with expectations
    let mut mock_client = MockHttpClient::new();
    mock_client
        .expect_get()
        .with(eq("https://example.com"))
        .times(1)
        .returning(|_| Ok("<html>content</html>".to_string()));

    let spider = Spider::new(mock_client);

    // Act
    let result = spider.fetch("https://example.com").await;

    // Assert
    assert!(result.is_ok());
    // Mock automatically verifies expectations were met
}
```

## Test Categories

### Unit Tests

**Purpose**: Test single units in isolation

**Template**:
```rust
// tests/unit/component_name_tests.rs

use super::*;

#[cfg(test)]
mod component_tests {
    use super::*;

    fn setup() -> Component {
        Component::new(default_config())
    }

    #[test]
    fn test_component_behavior_under_condition() {
        // Arrange
        let component = setup();

        // Act
        let result = component.method(input);

        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    fn test_component_handles_error_case() {
        let component = setup();

        let result = component.method(invalid_input);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Specific));
    }
}
```

### Integration Tests

**Purpose**: Test component interactions

**Template**:
```rust
// tests/integration/component_integration_tests.rs

use super::*;

#[tokio::test]
async fn test_components_integrate_correctly() {
    // Arrange
    let component_a = ComponentA::new();
    let component_b = ComponentB::new();

    // Act
    let result_a = component_a.process(input).await?;
    let result_b = component_b.consume(result_a).await?;

    // Assert
    assert!(result_b.is_valid());
}
```

### E2E Tests

**Purpose**: Test complete user workflows

**Template**:
```rust
// tests/e2e/workflow_tests.rs

#[tokio::test]
async fn test_complete_extraction_workflow() {
    // Arrange - Setup test environment
    let app = TestApp::spawn().await;

    // Act - Execute workflow
    let response = app
        .client
        .post("/api/extract")
        .json(&json!({ "url": "https://example.com" }))
        .send()
        .await?;

    // Assert - Verify end result
    assert_eq!(response.status(), 200);
    let doc: ExtractedDoc = response.json().await?;
    assert!(doc.title.is_some());
    assert!(doc.content.len() > 100);
}
```

## Best Practices

### DO ✅

1. **Write tests first** (TDD)
2. **Test behavior, not implementation**
3. **Use descriptive test names** that explain what and why
4. **Follow AAA pattern**: Arrange, Act, Assert
5. **Keep tests independent** - no shared state
6. **Use fixtures and builders** for test data
7. **Mock external dependencies** in unit tests
8. **Test edge cases and error paths**
9. **Document non-obvious test scenarios**
10. **Keep tests fast** - especially unit tests

### DON'T ❌

1. **Don't test implementation details**
2. **Don't share state between tests**
3. **Don't use random data** (use deterministic test data)
4. **Don't ignore flaky tests** (fix or remove them)
5. **Don't write tests after code** (defeats TDD purpose)
6. **Don't test framework code** (trust your dependencies)
7. **Don't create god tests** (test one thing per test)
8. **Don't skip error path testing**
9. **Don't use real external services** in unit/integration tests
10. **Don't commit commented-out tests**

## Common Patterns

### Pattern 1: Test Data Builders

```rust
// tests/fixtures/builders.rs

pub struct ExtractionRequestBuilder {
    url: String,
    options: ExtractionOptions,
}

impl ExtractionRequestBuilder {
    pub fn new() -> Self {
        Self {
            url: "https://example.com".to_string(),
            options: ExtractionOptions::default(),
        }
    }

    pub fn with_url(mut self, url: &str) -> Self {
        self.url = url.to_string();
        self
    }

    pub fn with_options(mut self, options: ExtractionOptions) -> Self {
        self.options = options;
        self
    }

    pub fn build(self) -> ExtractionRequest {
        ExtractionRequest {
            url: self.url,
            options: self.options,
        }
    }
}

// Usage in tests
#[test]
fn test_extraction() {
    let request = ExtractionRequestBuilder::new()
        .with_url("https://test.com")
        .with_options(custom_options())
        .build();
}
```

### Pattern 2: Assertion Helpers

```rust
// tests/common/assertions.rs

pub fn assert_extraction_valid(doc: &ExtractedDoc) {
    assert!(doc.title.is_some(), "Document should have a title");
    assert!(!doc.content.is_empty(), "Document should have content");
    assert!(doc.url.is_some(), "Document should have URL");
}

pub fn assert_error_contains(error: &Error, substring: &str) {
    let error_msg = error.to_string();
    assert!(
        error_msg.contains(substring),
        "Expected error to contain '{}', got: {}",
        substring,
        error_msg
    );
}
```

### Pattern 3: Test Fixtures

```rust
// tests/fixtures/mod.rs

pub fn sample_html_article() -> String {
    include_str!("../fixtures/data/article.html").to_string()
}

pub fn sample_spa_page() -> String {
    include_str!("../fixtures/data/spa.html").to_string()
}

pub fn mock_http_client() -> MockHttpClient {
    let mut client = MockHttpClient::new();
    client
        .expect_get()
        .returning(|_| Ok(sample_html_article()));
    client
}
```

### Pattern 4: Async Test Utilities

```rust
// tests/common/async_utils.rs

pub async fn with_timeout<F, T>(duration: Duration, future: F) -> Result<T, TimeoutError>
where
    F: Future<Output = T>,
{
    tokio::time::timeout(duration, future)
        .await
        .map_err(|_| TimeoutError)
}

// Usage
#[tokio::test]
async fn test_operation_completes_quickly() {
    let result = with_timeout(
        Duration::from_secs(1),
        slow_operation()
    ).await;

    assert!(result.is_ok(), "Operation should complete within 1 second");
}
```

## Debugging Tests

### Debug Output

```rust
#[test]
fn test_with_debug_output() {
    let value = complex_calculation();

    // Temporary debug output
    dbg!(&value);

    // Or use println for structured output
    println!("Calculated value: {:?}", value);

    assert_eq!(value, expected);
}

// Run with: cargo test -- --nocapture
```

### Focused Testing

```rust
// Focus on single test
#[test]
#[ignore = "Debugging in progress"]
fn test_specific_scenario() {
    // ... test code
}

// Run only this test
// cargo test test_specific_scenario -- --ignored
```

### Test Logging

```rust
#[tokio::test]
async fn test_with_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Your test code here
    // Now you'll see all debug! logs
}

// Run with: RUST_LOG=debug cargo test -- --nocapture
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v2

      - name: Run tests
        run: |
          cargo test --workspace --all-features

      - name: Run coverage
        run: |
          cargo tarpaulin --workspace --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v1
```

### Performance Test Gates

```rust
#[test]
fn test_performance_regression() {
    let start = Instant::now();
    perform_operation();
    let duration = start.elapsed();

    // Fail if performance regresses more than 10%
    let baseline = Duration::from_millis(100);
    let threshold = baseline.mul_f32(1.1);

    assert!(
        duration < threshold,
        "Performance regression: {:?} > {:?}",
        duration,
        threshold
    );
}
```

## Summary

- **Write tests first** using TDD
- **Follow the test pyramid** (70% unit, 20% integration, 10% E2E)
- **Use London School TDD** with mocks and behavior testing
- **Keep tests fast, focused, and independent**
- **Document through clear test names and structure**
- **Maintain ≥80% coverage** across the codebase

For detailed guidelines, see:
- [Naming Conventions](NAMING_CONVENTIONS.md)
- [Category Matrix](CATEGORY_MATRIX.md)
- [Test Organization Plan](TEST_ORGANIZATION_PLAN.md)

---

**Questions?** Check existing tests for examples or ask the team!
