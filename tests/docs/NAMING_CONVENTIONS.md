# Test Naming Conventions

**Version**: 1.0
**Date**: 2025-10-21
**Status**: Active Standard

## Overview

This document establishes consistent naming conventions for all tests in the EventMesh project. Following these conventions ensures:
- Tests are easily discoverable
- Purpose is clear from the name
- Tests are properly categorized
- IDE autocomplete is helpful
- Test reports are readable

## File Naming Conventions

### General Pattern
```
{component}_{test_type}_test{s}.rs
```

### Examples by Category

#### Unit Tests
```
tests/unit/
├── rate_limiter_tests.rs          # Multiple test cases for rate limiter
├── memory_manager_tests.rs         # Memory manager unit tests
├── circuit_breaker_test.rs         # Single component test
├── wasm_manager_tests.rs           # WASM manager tests
└── buffer_backpressure_tests.rs    # Buffer backpressure tests
```

**Pattern**: `{component}_tests.rs` or `{component}_{aspect}_tests.rs`

#### Integration Tests
```
tests/integration/
├── spider_integration_tests.rs     # Spider component integration
├── worker_integration_tests.rs     # Worker integration
├── contract_tests.rs                # API contract tests
├── session_persistence_tests.rs    # Session persistence integration
└── full_pipeline_tests.rs          # Full pipeline integration
```

**Pattern**: `{component}_integration_tests.rs` or `{concept}_tests.rs`

#### E2E Tests
```
tests/e2e/
├── end_to_end_workflow_tests.rs    # Complete workflow tests
├── real_world_tests.rs              # Real-world scenario tests
├── cli_e2e_tests.rs                 # CLI end-to-end tests
└── e2e_api.rs                       # API end-to-end tests
```

**Pattern**: `{scenario}_e2e_tests.rs` or `{component}_e2e_tests.rs` or `e2e_{aspect}.rs`

#### Performance Tests
```
tests/performance/
├── phase1_performance_tests.rs     # Phase 1 performance benchmarks
├── cli_performance_tests.rs         # CLI performance tests
└── benchmarks/
    └── extraction_benchmarks.rs     # Extraction benchmarks
```

**Pattern**: `{component}_performance_tests.rs` or `{aspect}_benchmarks.rs`

#### Chaos Tests
```
tests/chaos/
├── error_resilience_tests.rs       # Error resilience chaos tests
├── network_chaos_tests.rs          # Network failure tests
└── resource_chaos_tests.rs         # Resource exhaustion tests
```

**Pattern**: `{aspect}_chaos_tests.rs` or `{component}_resilience_tests.rs`

## Test Function Naming Conventions

### General Pattern
```rust
#[test]
fn test_{what}_{when}_{expected}()

// Or for async tests:
#[tokio::test]
async fn test_{what}_{when}_{expected}()
```

### Categories and Examples

#### Unit Tests - State Testing
```rust
#[test]
fn test_rate_limiter_allows_requests_under_limit() { }

#[test]
fn test_rate_limiter_blocks_requests_over_limit() { }

#[test]
fn test_circuit_breaker_opens_after_threshold() { }

#[test]
fn test_memory_manager_releases_on_drop() { }
```

**Pattern**: `test_{component}_{action}_{condition}`

#### Unit Tests - Edge Cases
```rust
#[test]
fn test_rate_limiter_handles_zero_limit() { }

#[test]
fn test_rate_limiter_handles_max_u64() { }

#[test]
fn test_circuit_breaker_with_empty_config() { }
```

**Pattern**: `test_{component}_handles_{edge_case}`

#### Integration Tests
```rust
#[tokio::test]
async fn test_spider_integrates_with_extraction_pipeline() { }

#[tokio::test]
async fn test_worker_coordinates_with_resource_manager() { }

#[tokio::test]
async fn test_session_persists_across_restarts() { }
```

**Pattern**: `test_{component}_integrates_with_{other_component}` or `test_{behavior}_across_{boundary}`

#### E2E Tests
```rust
#[tokio::test]
async fn test_complete_extraction_workflow_succeeds() { }

#[tokio::test]
async fn test_cli_extract_command_with_real_url() { }

#[tokio::test]
async fn test_end_to_end_streaming_with_backpressure() { }
```

**Pattern**: `test_{workflow}_{outcome}` or `test_end_to_end_{scenario}`

#### Performance Tests
```rust
#[bench]
fn bench_extraction_throughput_50_urls() { }

#[test]
fn test_ttfb_under_500ms_slo() { }

#[test]
fn test_p95_latency_meets_slo() { }
```

**Pattern**: `bench_{operation}_{scale}` or `test_{metric}_meets_slo`

#### Chaos Tests
```rust
#[tokio::test]
async fn test_resilience_to_network_timeout() { }

#[tokio::test]
async fn test_recovery_from_component_crash() { }

#[tokio::test]
async fn test_graceful_degradation_under_load() { }
```

**Pattern**: `test_{resilience|recovery|degradation}_{failure_scenario}`

#### Security Tests
```rust
#[test]
fn test_stealth_mode_masks_user_agent() { }

#[test]
fn test_sensitive_data_not_leaked_in_logs() { }

#[test]
fn test_api_rejects_malformed_input() { }
```

**Pattern**: `test_{security_aspect}_{protection}`

### Behavior-Driven Test Names (London School TDD)

For mock-based tests, use descriptive behavior names:

```rust
#[tokio::test]
async fn should_call_extractor_when_url_provided() { }

#[tokio::test]
async fn should_verify_contract_when_response_received() { }

#[tokio::test]
async fn should_retry_on_temporary_failure() { }
```

**Pattern**: `should_{action}_when_{condition}` or `should_{behavior}_on_{event}`

## Module and Directory Naming

### Module Files
```
mod.rs           # Module organization and re-exports
test_utils.rs    # Test utilities specific to this module
fixtures.rs      # Test fixtures for this module
```

### Directory Names
- Use `snake_case` for all directories
- Be descriptive but concise
- Group related tests together
- Avoid abbreviations unless standard (e.g., `e2e`, `api`)

```
tests/
├── unit/                    # Not "u" or "unit_tests"
├── integration/             # Not "int" or "integration_tests"
├── e2e/                     # Standard abbreviation OK
├── performance/             # Not "perf" unless industry standard
└── component/
    ├── cli/                 # Standard abbreviation OK
    ├── wasm/                # Standard abbreviation OK
    └── api/                 # Standard abbreviation OK
```

## Test Suite Naming

For grouped test suites, use descriptive names:

```rust
mod rate_limiter_tests {
    use super::*;

    mod under_load {
        #[test]
        fn test_maintains_limit() { }

        #[test]
        fn test_fair_distribution() { }
    }

    mod configuration {
        #[test]
        fn test_accepts_valid_config() { }

        #[test]
        fn test_rejects_invalid_config() { }
    }
}
```

**Pattern**: Group by aspect/scenario using nested modules

## Fixture and Mock Naming

### Mock Types
```rust
pub struct MockHttpClient { }       # Mock prefix for London School mocks
pub struct FakeDatabase { }         # Fake prefix for in-memory test doubles
pub struct StubService { }          # Stub prefix for minimal implementations
pub struct SpyLogger { }            # Spy prefix for observation
```

### Test Data
```rust
fn sample_valid_url() -> String { }
fn sample_malformed_html() -> String { }
fn fixture_spa_with_actions() -> SpaFixture { }
fn example_extraction_result() -> ExtractedDoc { }
```

**Pattern**:
- `sample_{description}` for simple data
- `fixture_{description}` for complex setups
- `example_{description}` for representative instances

## Common Prefixes and Their Meanings

| Prefix | Meaning | Example |
|--------|---------|---------|
| `test_` | Standard test function | `test_rate_limiter_blocks_overflow` |
| `bench_` | Benchmark test | `bench_extraction_1000_urls` |
| `should_` | Behavior specification | `should_retry_on_timeout` |
| `verify_` | Contract verification | `verify_api_contract_compliance` |
| `assert_` | Assertion helper | `assert_extraction_valid` |
| `mock_` | Mock implementation | `mock_http_client` |
| `fake_` | Fake implementation | `fake_database` |
| `stub_` | Stub implementation | `stub_service` |
| `fixture_` | Test fixture | `fixture_spa_scenario` |
| `sample_` | Sample data | `sample_valid_config` |

## Anti-Patterns to Avoid

### ❌ Bad Examples
```rust
// Too vague
#[test]
fn test1() { }
#[test]
fn test_stuff() { }
#[test]
fn it_works() { }

// Too verbose
#[test]
fn test_that_the_rate_limiter_component_correctly_blocks_requests_when_the_configured_limit_is_exceeded() { }

// Unclear purpose
#[test]
fn test_foo() { }
#[test]
fn test_bar_baz() { }

// Mixed naming styles
#[test]
fn TestCamelCase() { }  // Use snake_case
#[test]
fn test-kebab-case() { }  // Use snake_case
```

### ✅ Good Examples
```rust
// Clear and concise
#[test]
fn test_rate_limiter_blocks_overflow() { }

#[test]
fn test_circuit_breaker_opens_after_threshold() { }

#[tokio::test]
async fn test_extraction_succeeds_with_valid_html() { }

// Behavior-driven
#[test]
fn should_return_error_when_url_malformed() { }

#[test]
fn should_cache_result_when_extraction_succeeds() { }
```

## File Organization Within Test Files

### Standard Structure
```rust
// 1. Imports
use super::*;
use crate::fixtures::*;
use mockall::predicate::*;

// 2. Test fixtures and helpers
fn setup_test_environment() -> TestEnv { }
fn sample_test_data() -> TestData { }

// 3. Test modules (grouped by scenario)
mod happy_path {
    #[test]
    fn test_scenario_1() { }

    #[test]
    fn test_scenario_2() { }
}

mod error_handling {
    #[test]
    fn test_error_case_1() { }

    #[test]
    fn test_error_case_2() { }
}

mod edge_cases {
    #[test]
    fn test_boundary_condition_1() { }

    #[test]
    fn test_boundary_condition_2() { }
}

// 4. Integration/E2E tests (if mixed in same file)
mod integration {
    #[tokio::test]
    async fn test_full_workflow() { }
}
```

## Documentation Conventions

### Test Function Documentation
```rust
/// Tests that the rate limiter correctly blocks requests when the limit is exceeded.
///
/// # Test Scenario
/// - Configure rate limiter with 10 requests/second
/// - Send 15 requests rapidly
/// - Verify first 10 succeed, last 5 are blocked
///
/// # Acceptance Criteria
/// - First 10 requests return Ok(())
/// - Requests 11-15 return Err(RateLimitExceeded)
/// - Rate limiter state is consistent
#[test]
fn test_rate_limiter_blocks_overflow() {
    // Test implementation
}
```

### File-Level Documentation
```rust
//! Unit tests for the rate limiter component.
//!
//! These tests verify the rate limiting behavior under various conditions:
//! - Normal operation under load
//! - Edge cases (zero limit, max limit)
//! - Error handling
//! - Concurrent access
//!
//! # Test Strategy
//! Uses mock time provider for deterministic testing.
```

## Special Cases

### Regression Tests
```rust
// Reference the issue/bug number
#[test]
fn test_regression_issue_123_memory_leak() { }

#[test]
fn test_regression_pr_456_timeout_handling() { }
```

### Platform-Specific Tests
```rust
#[test]
#[cfg(target_os = "linux")]
fn test_linux_specific_feature() { }

#[test]
#[cfg(feature = "experimental")]
fn test_experimental_feature() { }
```

### Ignored/Flaky Tests
```rust
#[test]
#[ignore = "Flaky on CI - Issue #789"]
fn test_timing_sensitive_operation() { }

#[test]
#[ignore = "Requires external service"]
fn test_real_api_integration() { }
```

## Checklist for Test Names

Before committing a test, verify:

- [ ] Name clearly describes what is being tested
- [ ] Name indicates the expected outcome
- [ ] Follows the project's naming pattern
- [ ] Uses `snake_case` (not camelCase or kebab-case)
- [ ] Is discoverable (someone can find it by searching for the component)
- [ ] Avoids abbreviations unless standard
- [ ] Includes test type suffix where appropriate (`_tests`, `_test`)
- [ ] Groups logically with related tests
- [ ] Has clear documentation if the name alone isn't sufficient

## Examples by Test Pyramid Level

### Unit Level (Many, Fast, Focused)
```rust
test_parser_handles_empty_string()
test_parser_handles_unicode()
test_parser_returns_error_on_invalid_syntax()
test_cache_evicts_lru_entry()
test_cache_updates_access_time()
```

### Integration Level (Moderate, Real Components)
```rust
test_spider_extraction_pipeline_integration()
test_worker_resource_manager_coordination()
test_session_persistence_across_components()
test_streaming_backpressure_integration()
```

### E2E Level (Few, Comprehensive, Slow)
```rust
test_complete_url_extraction_workflow()
test_cli_extract_with_real_website()
test_end_to_end_spa_rendering_and_extraction()
```

## Summary

Good test names are:
- **Descriptive**: Tell what is being tested
- **Specific**: Indicate the scenario or condition
- **Consistent**: Follow established patterns
- **Discoverable**: Easy to find via search
- **Maintainable**: Easy to understand months later

Bad test names are:
- **Vague**: "test1", "test_stuff"
- **Overly verbose**: Entire sentences
- **Inconsistent**: Mix of different styles
- **Cryptic**: Abbreviations or jargon

---

**Remember**: Test names are documentation. They should tell the story of what the system does and how it behaves.
