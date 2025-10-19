# CI Timeout Configuration

## Overview

This document describes the timeout configurations added to GitHub Actions workflows and test timeout constants for the EventMesh project.

## GitHub Actions Workflow Timeouts

All CI/CD workflows now include job-level timeout configurations to prevent runaway jobs and provide faster feedback on failures.

### Main CI Pipeline (.github/workflows/ci.yml)

| Job | Timeout | Rationale |
|-----|---------|-----------|
| check | 10 minutes | Fast preliminary checks (formatting) |
| build | 30 minutes | Parallel builds for native + WASM targets |
| test | 15 minutes | Unit and integration test suites |
| docker-build | 20 minutes | Docker image builds with layer caching |
| size-check | 5 minutes | Binary size analysis |
| quality | 20 minutes | Security audit and dependency checks |
| benchmark | 15 minutes | Performance benchmarking |
| validate | 5 minutes | Final validation checks |
| cleanup | 5 minutes | Build space cleanup |

**Total estimated pipeline time**: 30-40 minutes (with parallelization)

### API Validation Pipeline (.github/workflows/api-validation.yml)

| Job | Timeout | Rationale |
|-----|---------|-----------|
| static-analysis | 10 minutes | Code formatting and linting |
| contract-validation | 20 minutes | Dredd contract testing with API server startup |
| fuzzing-tests | 20 minutes | Schemathesis fuzzing with 100 examples |
| performance-tests | 15 minutes | k6 load testing (30s duration) |
| security-scan | 25 minutes | OWASP ZAP security scanning |
| test-coverage | 20 minutes | Code coverage generation with llvm-cov |
| api-benchmarks | 15 minutes | API-specific benchmarks |
| validation-complete | 5 minutes | Final validation and reporting |

**Total estimated pipeline time**: 20-30 minutes (with parallelization)

### Docker Build & Publish (.github/workflows/docker-build-publish.yml)

| Job | Timeout | Rationale |
|-----|---------|-----------|
| build-and-push | 30 minutes | Multi-stage Docker builds with caching |

## Test Timeout Constants Module

Location: `/workspaces/eventmesh/tests/common/timeouts.rs`

### Purpose

Provides standardized timeout durations for test operations with environment-based scaling support.

### Constants

| Constant | Default Duration | Use Cases |
|----------|-----------------|-----------|
| FAST_OP | 2 seconds | Simple unit tests, in-memory operations, health checks |
| MEDIUM_OP | 10 seconds | API requests, database queries, WASM initialization |
| SLOW_OP | 30 seconds | Complex workflows, multi-step integration tests |
| VERY_SLOW_OP | 60 seconds | Full system integration, heavy data processing |

### Usage Example

```rust
use std::time::Duration;
use tests::common::timeouts::{FAST_OP, MEDIUM_OP, SLOW_OP};

#[tokio::test]
async fn test_fast_operation() {
    let result = tokio::time::timeout(FAST_OP, fast_api_call()).await;
    assert!(result.is_ok(), "Fast operation timed out");
}

#[tokio::test]
async fn test_database_query() {
    let result = tokio::time::timeout(MEDIUM_OP, db_query()).await;
    assert!(result.is_ok(), "Medium operation timed out");
}
```

### Environment Variable Scaling

All timeouts can be scaled using the `TEST_TIMEOUT_MULTIPLIER` environment variable:

```bash
# Double all timeouts for slower CI environments
export TEST_TIMEOUT_MULTIPLIER=2.0
cargo test

# Half timeouts for faster local testing
export TEST_TIMEOUT_MULTIPLIER=0.5
cargo test

# Use default timeouts
unset TEST_TIMEOUT_MULTIPLIER
cargo test
```

**Valid range**: 0.1 to 10.0 (defaults to 1.0 if invalid)

### Scaled Duration Functions

For dynamic timeout scaling at runtime:

```rust
use tests::common::timeouts::{fast_op, medium_op, slow_op, very_slow_op};

#[tokio::test]
async fn test_with_scaled_timeout() {
    // Automatically respects TEST_TIMEOUT_MULTIPLIER
    let result = tokio::time::timeout(medium_op(), db_query()).await;
    assert!(result.is_ok());
}
```

## Benefits

1. **Faster Failure Detection**: Jobs fail fast instead of hanging for hours
2. **Resource Efficiency**: Prevents runaway jobs from consuming CI resources
3. **Consistent Test Behavior**: Standardized timeouts across all tests
4. **Environment Flexibility**: Easy scaling for different CI environments
5. **Clear Documentation**: Timeout rationale documented for each job

## CI Configuration Best Practices

1. **Job-level timeouts**: Set `timeout-minutes` on every job
2. **Step-level timeouts**: Use `timeout` command for long-running steps
3. **Test timeouts**: Always use timeout constants in async tests
4. **Scaling**: Set `TEST_TIMEOUT_MULTIPLIER=2.0` for slower CI runners
5. **Monitoring**: Review timeout failures to identify performance issues

## Future Improvements

1. Add step-level timeouts for long-running build steps
2. Implement automatic timeout adjustment based on historical data
3. Add timeout metrics to CI dashboards
4. Create timeout alerts for consistently slow jobs

## Related Files

- `/workspaces/eventmesh/.github/workflows/ci.yml`
- `/workspaces/eventmesh/.github/workflows/api-validation.yml`
- `/workspaces/eventmesh/.github/workflows/docker-build-publish.yml`
- `/workspaces/eventmesh/tests/common/timeouts.rs`

## References

- [GitHub Actions: timeout-minutes](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions#jobsjob_idtimeout-minutes)
- [Rust tokio::time::timeout](https://docs.rs/tokio/latest/tokio/time/fn.timeout.html)
