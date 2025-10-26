# Local CI Testing Guide

> Catch GitHub Actions errors **before** pushing with these local checks

## Quick Reference

```bash
# ⚡ Fast check before commit (~30s)
make ci-quick

# 🔍 Full check before push (~3-5min)
make ci

# 🚀 Comprehensive pre-push check (NEW - catches everything)
./scripts/pre-push-check.sh
```

## Common GitHub Actions Errors & How to Catch Them Locally

### 1. **Clippy Warnings** ❌ Most Common!

**GitHub Error:**
```
error: used consecutive `str::replace` call
error: unused import
error: unused variable
```

**Local Check:**
```bash
cargo clippy --workspace --all-targets -- -D warnings
```

**Fix:**
```bash
# Auto-fix some issues
cargo clippy --workspace --all-targets --fix

# Or use make command
make lint
```

---

### 2. **Unused Imports** 🗑️

**GitHub Error:**
```
error: unused import: `axum::body::Body`
  --> crates/riptide-api/tests/cross_module_integration.rs:10:5
```

**Local Check:**
```bash
cargo build --workspace 2>&1 | grep "unused import"
```

**Fix:**
```rust
// Add #[allow(unused_imports)] if needed conditionally
#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use some_crate::SomeType;
}
```

---

### 3. **Invalid Feature Gates** ⚙️

**GitHub Error:**
```
error: unexpected `cfg` condition value: `tenants`
  = note: expected values for `feature` are: `default`, `events`, ...
```

**Local Check:**
```bash
cargo check --workspace --all-targets
```

**Fix:**
```rust
// Use #[ignore] instead of non-existent features
#[tokio::test]
#[ignore = "feature not implemented yet"]
async fn test_something() { }
```

---

### 4. **Test Failures** 🧪

**GitHub Error:**
```
test result: FAILED. 4 passed; 3 failed
```

**Local Check:**
```bash
# Run ALL tests (unit + integration)
cargo test --workspace -- --nocapture

# Or specific packages
cargo test --package riptide-api
cargo test --package riptide-spider
```

**Common Test Issues:**
- Assertions expecting wrong status codes
- Tests running too long (60s+ timeout)
- Race conditions in concurrent tests
- Feature-gated endpoints being tested without features

---

### 5. **Formatting Issues** 🎨

**GitHub Error:**
```
error: Diff in /path/to/file.rs
```

**Local Check:**
```bash
cargo fmt --all --check
```

**Fix:**
```bash
cargo fmt --all
```

---

### 6. **Build Failures** 🔨

**GitHub Error:**
```
error: could not compile `riptide-api`
```

**Local Check:**
```bash
cargo build --workspace --all-targets
```

---

### 7. **OpenAPI Schema Invalid** 📝

**GitHub Error:**
```
error: Missing required parameter 'format'
error: Undocumented HTTP status code 429
```

**Local Check:**
```bash
npx @apidevtools/swagger-cli validate docs/api/openapi.yaml
```

---

### 8. **Security Vulnerabilities** 🔒

**GitHub Error:**
```
error: 1 security advisory found
```

**Local Check:**
```bash
cargo audit
```

**Fix:**
```bash
cargo update
# Or update specific vulnerable dependencies
```

---

### 9. **License/Dependency Issues** 📜

**GitHub Error:**
```
error: banned dependency detected
```

**Local Check:**
```bash
cargo deny check
```

---

## Recommended Workflow

### Before Every Commit
```bash
make ci-quick
```
**Runs:** fmt check, clippy, unit tests (~30s)

### Before Every Push
```bash
./scripts/pre-push-check.sh
```
**Runs:** All checks + integration tests + security audits (~3-5min)

**Or use the traditional:**
```bash
make ci
```

### After Major Changes
```bash
# Run full test suite
make test

# Check specific packages
make test-api
make test-spider
make test-extraction
```

---

## What Gets Checked in GitHub Actions

### CI Workflow (`.github/workflows/ci.yml`)

1. **Quick Checks Job:**
   - ✅ Formatting (`cargo fmt --check`)
   - ✅ Clippy lints (`cargo clippy -- -D warnings`)
   - ✅ Unit tests (`cargo test --lib`)

2. **Build Job:**
   - ✅ Compilation (`cargo build --workspace`)
   - ✅ All targets build

3. **Test Job:**
   - ✅ Unit tests (`cargo test --lib --bins`)
   - ✅ Integration tests (`cargo test --tests`)
   - ✅ Doc tests (`cargo test --doc`)

4. **Security Job:**
   - ✅ Dependency audit (`cargo audit`)
   - ✅ License check (`cargo deny`)

### API Validation Workflow (`.github/workflows/api-validation.yml`)

- ✅ OpenAPI schema validation
- ✅ Schemathesis contract testing
- ✅ API endpoint compliance

---

## Advanced: Mimic Exact CI Environment

```bash
# Use same Rust flags as CI
export RUSTFLAGS="-D warnings"
export RUST_BACKTRACE=1
export CARGO_TERM_COLOR=always

# Run checks
cargo clippy --workspace --all-targets
cargo test --workspace
```

---

## Troubleshooting

### "My tests pass locally but fail in CI"

**Possible causes:**
1. **Parallel test execution** - CI may run tests in different order
   ```bash
   cargo test -- --test-threads=1
   ```

2. **Feature flags** - CI might enable different features
   ```bash
   cargo test --all-features
   ```

3. **Timing issues** - Tests with timeouts may behave differently
   - Add `#[timeout = Duration::from_secs(30)]` to flaky tests

### "Clippy passes locally but fails in CI"

**Cause:** Different Rust versions or clippy configuration

**Fix:**
```bash
# Update Rust to match CI
rustup update stable

# Run with exact CI flags
cargo clippy --workspace --all-targets -- -D warnings
```

---

## Installation

### Required Tools
```bash
# Core (required)
rustup component add rustfmt clippy

# Security auditing (recommended)
cargo install cargo-audit cargo-deny

# Coverage (optional)
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```

### Optional Tools
```bash
# OpenAPI validation
npm install -g @apidevtools/swagger-cli

# Benchmarking
cargo install cargo-criterion
```

---

## Quick Fixes Cheat Sheet

| Error Type | Quick Fix |
|------------|-----------|
| Formatting | `cargo fmt --all` |
| Clippy warnings | `cargo clippy --fix --allow-dirty` |
| Unused imports | Remove or add `#[allow(unused_imports)]` |
| Invalid features | Replace with `#[ignore = "reason"]` |
| Test failures | `cargo test --package <name> -- --nocapture` |
| OpenAPI invalid | Update `docs/api/openapi.yaml` |

---

## CI Performance Tips

### Make Tests Faster
```rust
// Use multi_thread for concurrent tests
#[tokio::test(flavor = "multi_thread")]
async fn test_concurrent() { }

// Add timeouts to prevent hanging
tokio::time::timeout(
    Duration::from_secs(30),
    test_future
).await.expect("Test timeout")
```

### Skip Slow Tests in Quick Checks
```rust
#[test]
#[ignore = "slow test"]
fn test_expensive_operation() { }
```

---

## Summary

**Golden Rule:** Run `./scripts/pre-push-check.sh` before every push!

This catches:
- ✅ Clippy warnings
- ✅ Unused imports
- ✅ Invalid feature gates
- ✅ Test failures
- ✅ Formatting issues
- ✅ Build errors
- ✅ Security issues
- ✅ OpenAPI schema problems

**Time investment:** 3-5 minutes locally vs 10-15 minutes waiting for CI to fail + fix + re-push cycle.
