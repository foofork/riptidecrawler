# Integration Tests with Testcontainers

This document describes how to run and maintain integration tests for the RipTide persistence and cache layers.

## Overview

Integration tests use [testcontainers](https://github.com/testcontainers/testcontainers-rs) to spin up isolated Redis and PostgreSQL instances for testing. This ensures:

- **Isolation**: Each test run uses fresh containers
- **No Manual Setup**: No need to install/configure databases manually
- **CI/CD Ready**: Works in any environment with Docker
- **Reproducibility**: Consistent test environment across machines

## Prerequisites

1. **Docker**: Must have Docker installed and running
   ```bash
   docker --version  # Verify Docker is installed
   ```

2. **Rust Toolchain**: Ensure you have the latest stable Rust
   ```bash
   rustc --version
   ```

## Running Tests

### Quick Start

Run all integration tests for a specific crate:

```bash
# Persistence layer tests
cargo test -p riptide-persistence --test '*' -- --test-threads=1

# Cache layer tests
cargo test -p riptide-cache --test '*' -- --test-threads=1
```

**Note**: Use `--test-threads=1` to avoid container port conflicts.

### Running Specific Test Files

```bash
# Redis integration tests (persistence)
cargo test -p riptide-persistence --test redis_testcontainer_integration

# Integration module tests
cargo test -p riptide-persistence --test integration

# Cache Redis tests
cargo test -p riptide-cache --test integration
```

### Running Individual Tests

```bash
# Run a specific test by name
cargo test -p riptide-persistence test_redis_connection_with_testcontainer

# Run tests matching a pattern
cargo test -p riptide-persistence test_cache_ -- --test-threads=1
```

## Test Structure

### Persistence Layer (`riptide-persistence`)

```
crates/riptide-persistence/tests/
├── helpers/
│   ├── mod.rs                     # Helper module exports
│   ├── postgres_helpers.rs        # PostgreSQL testcontainer setup
│   └── redis_helpers.rs            # Redis testcontainer setup
├── integration/
│   ├── mod.rs                     # Main integration tests
│   ├── cache_integration_tests.rs
│   ├── state_integration_tests.rs
│   ├── performance_tests.rs
│   └── spillover_tests.rs
├── redis_testcontainer_integration.rs  # Comprehensive Redis tests
└── redis_integration_tests.rs     # Legacy tests (being migrated)
```

### Cache Layer (`riptide-cache`)

```
crates/riptide-cache/tests/
├── helpers/
│   ├── mod.rs                     # Helper module exports
│   └── redis_helpers.rs           # Redis testcontainer setup
└── integration/
    └── redis_tests.rs             # Redis adapter tests
```

## Test Helpers

### Redis Helper Usage

```rust
use testcontainers::clients::Cli;
use helpers::RedisTestContainer;

#[tokio::test]
async fn my_redis_test() -> Result<()> {
    // Start Redis container
    let docker = Cli::default();
    let redis_container = RedisTestContainer::new(&docker).await?;

    // Get connection string
    let redis_url = redis_container.get_connection_string();

    // Use in your tests...
    let cache = PersistentCacheManager::new(redis_url, config).await?;

    // Container automatically cleans up when dropped
    Ok(())
}
```

### PostgreSQL Helper Usage

```rust
use testcontainers::clients::Cli;
use helpers::PostgresTestContainer;

#[tokio::test]
#[cfg(feature = "postgres")]
async fn my_postgres_test() -> Result<()> {
    // Start PostgreSQL container
    let docker = Cli::default();
    let pg_container = PostgresTestContainer::new(&docker).await?;

    // Initialize schema
    pg_container.init_session_schema().await?;

    // Get connection pool
    let pool = pg_container.get_pool();

    // Use in your tests...

    // Cleanup test data
    pg_container.cleanup().await?;

    Ok(())
}
```

## Test Coverage

### Redis Integration Tests (~1,120 LOC)

**File**: `redis_testcontainer_integration.rs`

- ✅ Connection establishment
- ✅ Cache set/get/delete operations
- ✅ TTL expiration
- ✅ Multi-tenant isolation
- ✅ Batch operations
- ✅ Large value storage
- ✅ Metadata support
- ✅ Concurrent operations
- ✅ Performance benchmarks
- ✅ Error handling
- ✅ Cache statistics

### Integration Module Tests

**Files**: `integration/mod.rs` and sub-modules

- ✅ Basic integration workflow
- ✅ Performance targets (<50ms cache get)
- ✅ Error handling
- ✅ Compression functionality
- ✅ TTL functionality
- ✅ State management
- ✅ Checkpoint creation
- ✅ Session operations

## Troubleshooting

### Docker Not Running

```
Error: Cannot connect to Docker daemon
```

**Solution**: Start Docker Desktop or Docker daemon:
```bash
sudo systemctl start docker  # Linux
# OR open Docker Desktop on Mac/Windows
```

### Port Conflicts

```
Error: Address already in use
```

**Solution**: Use `--test-threads=1` to run tests sequentially:
```bash
cargo test -p riptide-persistence --test '*' -- --test-threads=1
```

### Slow Tests

Test containers can take 5-10 seconds to start. This is normal.

**Tips**:
- Use `--test-threads=1` to avoid multiple container startups
- Run specific test files instead of all tests
- Containers are reused within the same test execution

### Permission Denied

```
Error: Permission denied while trying to connect to Docker
```

**Solution**: Add user to docker group (Linux):
```bash
sudo usermod -aG docker $USER
newgrp docker
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  integration-tests:
    runs-on: ubuntu-latest

    services:
      docker:
        image: docker:dind
        options: --privileged

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run Integration Tests
        run: |
          cargo test -p riptide-persistence --test '*' -- --test-threads=1
          cargo test -p riptide-cache --test '*' -- --test-threads=1
```

## Performance Benchmarks

Integration tests include performance assertions:

- **Cache GET**: < 50ms
- **Cache SET**: < 100ms
- **100 operations**: < 2 seconds
- **Concurrent operations**: Properly isolated

## Writing New Tests

### Template for Redis Tests

```rust
#[tokio::test]
async fn test_my_feature() -> Result<()> {
    // 1. Setup container
    let docker = Cli::default();
    let redis_container = RedisTestContainer::new(&docker).await?;

    // 2. Create config
    let config = test_cache_config();
    let cache = PersistentCacheManager::new(
        redis_container.get_connection_string(),
        config
    ).await?;

    // 3. Test your feature
    cache.set("test_key", &"value", None, None, None).await?;
    let result: Option<String> = cache.get("test_key", None).await?;

    // 4. Assert
    assert_eq!(result, Some("value".to_string()));

    // 5. Cleanup (automatic when container drops)
    Ok(())
}
```

### Best Practices

1. **Use unique keys**: Avoid key collisions between tests
2. **Cleanup data**: Use container cleanup methods
3. **Test isolation**: Don't depend on test execution order
4. **Clear assertions**: Make expectations explicit
5. **Document edge cases**: Comment on non-obvious behavior

## Migration from Manual Setup

The original 81 `#[ignore]` tests have been migrated to use testcontainers:

- ✅ No more manual Redis/PostgreSQL setup
- ✅ All tests can run in CI/CD
- ✅ Improved test isolation
- ✅ Faster developer onboarding

## Feature Flags

Some tests require optional features:

```bash
# Run with PostgreSQL support
cargo test -p riptide-persistence --features postgres --test '*'

# Run with compression
cargo test -p riptide-persistence --features compression --test '*'

# Run with all features
cargo test -p riptide-persistence --all-features --test '*'
```

## Metrics and Coverage

Check test coverage:

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --workspace --exclude-files 'crates/riptide-api/*' \
  --out Html --output-dir coverage
```

## Support

For issues or questions:

1. Check this documentation
2. Review test examples in `redis_testcontainer_integration.rs`
3. Check testcontainers-rs documentation: https://github.com/testcontainers/testcontainers-rs
4. File an issue in the repository

---

**Last Updated**: 2025-11-08
**Maintainer**: RipTide Team
