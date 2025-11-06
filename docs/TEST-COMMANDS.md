# Test Verification Commands Reference

## Quick Start

### Compile All Tests
```bash
# Check compilation only (fast)
cargo test -p riptide-api --lib --no-run
cargo test -p riptide-pipeline --lib --no-run
```

### Run All Tests
```bash
# Run tests (excluding ignored)
cargo test -p riptide-api --lib -- --test-threads=2
cargo test -p riptide-pipeline --lib -- --test-threads=2
```

### Run Specific Test Categories
```bash
# Validation tests only
cargo test -p riptide-api --lib validation::tests

# Resource manager tests
cargo test -p riptide-api --lib resource_manager

# Middleware tests
cargo test -p riptide-api --lib middleware
```

## Advanced Usage

### Run Ignored Tests (Requires Resources)
```bash
# Setup: Install Chrome and start Redis
sudo apt-get install chromium-browser
docker run -d -p 6379:6379 redis

# Run ignored tests
cargo test -p riptide-api --lib -- --ignored --test-threads=1
```

### Fix Warnings
```bash
# Auto-fix unused imports and variables
cargo fix --lib -p riptide-api --tests
```

### Check for Errors with Strict Mode
```bash
# Fail on any warnings
RUSTFLAGS="-D warnings" cargo test -p riptide-api --lib --no-run
```

### Generate Coverage Report
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin -p riptide-api --lib --out Html
```

## Troubleshooting

### Recompile from Clean State
```bash
cargo clean
cargo test -p riptide-api --lib --no-run
```

### View Test Output
```bash
# Show stdout/stderr for passing tests
cargo test -p riptide-api --lib -- --nocapture

# Show output for specific test
cargo test -p riptide-api --lib test_name -- --nocapture
```

### Debug Failing Tests
```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test -p riptide-api --lib test_name

# Run with full backtrace
RUST_BACKTRACE=full cargo test -p riptide-api --lib test_name
```

## Continuous Integration

### CI-Friendly Commands
```bash
# Fast check (no test execution)
cargo test --workspace --lib --no-run

# Run tests with proper threading
cargo test --workspace --lib -- --test-threads=2

# Check for compilation errors
cargo check --workspace --tests
```

## Performance Benchmarking

### Run Benchmarks (if available)
```bash
cargo bench -p riptide-api
```

### Profile Test Execution
```bash
cargo test -p riptide-api --lib -- --test-threads=1 --nocapture
```

## Quality Gates

### Pre-Commit Checklist
```bash
# 1. Format code
cargo fmt --all

# 2. Check compilation
cargo check --workspace

# 3. Run clippy
cargo clippy --all -- -D warnings

# 4. Run tests
cargo test --workspace --lib

# 5. Fix warnings
cargo fix --workspace --tests --allow-dirty
```

## Current Status

### Test Results (2025-11-06)
```
✅ riptide-api:      234 passed, 2 failed, 31 ignored
✅ riptide-pipeline: 2 passed, 0 failed, 0 ignored
✅ Compilation:      0 errors, 4 warnings
```

### Known Issues
1. `test_check_memory_pressure_with_real_metrics` - Environment-specific
2. `test_session_store_cleanup` - Timing issue

### Files Modified
- See `/workspaces/eventmesh/docs/TEST-HEALTH-REPORT-2025-11-06.md` for details
