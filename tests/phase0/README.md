# Phase 0 Test Suite

Comprehensive test suite for Phase 0 components following TDD London School approach.

## Test Organization

- **unit/**: Unit tests with mocked dependencies
- **integration/**: Integration tests with real components
- **fixtures/**: Recorded HTTP responses and test data

## TDD London School Principles

### RED-GREEN-REFACTOR Cycle

1. **RED**: Write failing test first (nothing implemented)
2. **GREEN**: Make test pass with minimal code
3. **REFACTOR**: Improve code quality while keeping tests green

### Key Principles

- **Mock all dependencies**: Tests should be isolated
- **Test behavior, not implementation**: Focus on interfaces
- **One assertion per test**: Each test verifies one behavior
- **Descriptive names**: Test names explain what and why

## Test Coverage Requirements

- Statements: >80%
- Branches: >75%
- Functions: >80%
- Lines: >80%

## Running Tests

```bash
# All Phase 0 tests
cargo test --test phase0_*

# Specific component
cargo test --test phase0_redis_pool
cargo test --test phase0_retry_policy
cargo test --test phase0_rate_limiter
cargo test --test phase0_http_client

# With coverage
cargo tarpaulin --test phase0_* --out Html

# Fast tests only (unit tests)
cargo test --test 'phase0_*' --lib
```

## Test Components

1. **RedisPool**: Connection reuse, health checks, retry logic
2. **HTTP Client Factory**: Default and custom client creation
3. **RetryPolicy**: Exponential backoff verification
4. **SimpleRateLimiter**: Governor-based rate limiting
5. **Config Secrets Redaction**: Debug and serialization safety
6. **HTTP Fixtures**: Wiremock-based recorded responses

## Memory Coordination

Test status and results are stored in memory for swarm coordination:

- `tests/phase0/coverage` - Coverage metrics
- `tests/phase0/fixtures` - Fixture metadata
- `tests/phase0/status` - Test execution status
