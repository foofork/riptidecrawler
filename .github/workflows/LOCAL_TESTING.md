# Local Testing Guide for Safety Audit Workflow

Before pushing your changes and triggering the CI workflow, you can run all safety checks locally for rapid feedback.

## Prerequisites

```bash
# Install Rust nightly (for Miri)
rustup toolchain install nightly

# Install ripgrep (for unsafe/WASM checks)
# On Ubuntu/Debian
sudo apt-get install ripgrep

# On macOS
brew install ripgrep

# Verify installations
cargo --version
cargo +nightly --version
rg --version
```

## Quick Test (1 minute)

Run these commands to get fast feedback:

```bash
# 1. Unsafe code audit (~5 seconds)
.github/workflows/scripts/check-unsafe.sh

# 2. WASM safety check (~2 seconds)
.github/workflows/scripts/check-wasm-safety.sh

# 3. Quick Clippy check on changed files (~30 seconds)
cargo clippy --lib --bins -- \
  -D clippy::unwrap_used \
  -D clippy::expect_used
```

## Full Local CI Simulation (5-10 minutes)

Replicate the exact CI workflow locally:

### Step 1: Unsafe Code Audit

```bash
echo "=== JOB 1: Unsafe Code Audit ==="
.github/workflows/scripts/check-unsafe.sh
```

Expected output:
```
🔍 Checking for unsafe blocks without SAFETY documentation...
✅ No unsafe blocks found in production code
```

### Step 2: Clippy Production Checks

```bash
echo "=== JOB 2: Clippy Production Checks ==="

# Production code (no unwrap/expect allowed)
cargo clippy \
  --workspace \
  --all-features \
  --lib \
  --bins \
  -- \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D warnings

# Test code (unwrap/expect allowed)
cargo clippy \
  --workspace \
  --all-features \
  --tests \
  -- \
  -A clippy::unwrap_used \
  -A clippy::expect_used \
  -D warnings
```

### Step 3: Miri Memory Safety (Optional)

**Note**: This is slow (~5-10 minutes). Consider running only on specific modules.

```bash
echo "=== JOB 3: Miri Memory Safety ==="

# Setup Miri (one-time)
cargo +nightly miri setup

# Run on memory_manager tests only (same as CI)
cargo +nightly miri test \
  --package riptide-core \
  --lib \
  memory_manager \
  -- \
  --nocapture

# Optional: Run on all tests (very slow)
# cargo +nightly miri test
```

### Step 4: WASM Safety Documentation

```bash
echo "=== JOB 4: WASM Safety Documentation ==="
.github/workflows/scripts/check-wasm-safety.sh
```

Expected output:
```
🔍 Checking WASM bindings for safety documentation...
📄 Checking ./wasm/riptide-extractor-wasm/src/bindings.rs
  ✅ Has required WASM FFI safety documentation
  📊 Contains 0 unsafe references

📊 Summary:
  Total bindings files: 1
  Violations: 0
✅ All WASM bindings are properly documented
```

## Fast Iteration Workflow

When actively developing and fixing issues:

```bash
#!/bin/bash
# save as: check-safety.sh

set -e

echo "🔍 Running safety checks..."
echo ""

# Fast checks first
echo "1️⃣ Unsafe code audit..."
.github/workflows/scripts/check-unsafe.sh
echo "✅ Passed"
echo ""

echo "2️⃣ WASM safety..."
.github/workflows/scripts/check-wasm-safety.sh
echo "✅ Passed"
echo ""

echo "3️⃣ Clippy checks..."
cargo clippy --workspace --lib --bins --quiet -- \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D warnings
echo "✅ Passed"
echo ""

echo "🎉 All safety checks passed! Safe to push."
```

Make executable and run:
```bash
chmod +x check-safety.sh
./check-safety.sh
```

## Fixing Common Issues

### Issue 1: Undocumented Unsafe Block

```bash
# Find the problematic file/line
.github/workflows/scripts/check-unsafe.sh

# Example output:
# ❌ src/memory.rs:42: Missing SAFETY documentation
```

Fix:
```rust
// BEFORE (wrong)
let value = unsafe { ptr.read() };

// AFTER (correct)
// SAFETY: ptr is valid because it was just returned from Box::into_raw
// and the Box allocation ensures proper alignment and initialization.
let value = unsafe { ptr.read() };
```

### Issue 2: Unwrap in Production Code

```bash
# Find the problematic code
cargo clippy --lib --bins -- \
  -D clippy::unwrap_used \
  -D clippy::expect_used
```

Fix:
```rust
// BEFORE (wrong - can panic)
fn get_config() -> Config {
    std::fs::read_to_string("config.toml")
        .unwrap()  // ❌ Panic if file doesn't exist
}

// AFTER (correct - returns Result)
fn get_config() -> Result<Config, Error> {
    std::fs::read_to_string("config.toml")
        .map_err(Error::ConfigNotFound)?
}
```

### Issue 3: Miri Detects Undefined Behavior

```bash
# Run Miri with detailed output
cargo +nightly miri test --package riptide-core --lib memory_manager -- --nocapture
```

Common issues:
- **Use-after-free**: Check pointer lifetimes
- **Uninitialized memory**: Ensure proper initialization
- **Data races**: Add proper synchronization
- **Alignment**: Check pointer alignment requirements

### Issue 4: Missing WASM Safety Documentation

```bash
# Check which file needs documentation
.github/workflows/scripts/check-wasm-safety.sh
```

Fix:
```rust
// Add to top of bindings.rs file:
// SAFETY: Required for WASM component model FFI
```

## CI vs Local Differences

### Caching
- **CI**: Uses rust-cache for ~50% speedup on subsequent runs
- **Local**: Uses cargo's default cache in `~/.cargo`

To match CI behavior:
```bash
# Clear local cache (like fresh CI run)
cargo clean

# Run with cache (like subsequent CI run)
cargo clippy ...
```

### Environment
- **CI**: Ubuntu latest, fresh environment
- **Local**: Your environment, might have different tools

To match CI environment:
```bash
# Use same Rust toolchain as CI
rustup toolchain install stable
rustup default stable

# Check versions
cargo --version    # Should be latest stable
rustc --version    # Should match CI
```

## Performance Tips

### Parallel Execution
Run independent checks in parallel:

```bash
# Run all checks in parallel (GNU parallel)
parallel ::: \
  '.github/workflows/scripts/check-unsafe.sh' \
  '.github/workflows/scripts/check-wasm-safety.sh' \
  'cargo clippy --lib --bins --quiet -- -D clippy::unwrap_used -D clippy::expect_used'
```

### Incremental Checks
Only check modified files:

```bash
# Find modified Rust files
MODIFIED_FILES=$(git diff --name-only --diff-filter=ACMR | grep '\.rs$')

# Check only modified files for unsafe
for file in $MODIFIED_FILES; do
  if rg 'unsafe\s*\{' "$file" > /dev/null; then
    echo "Checking $file for SAFETY comments..."
    rg 'unsafe\s*\{' "$file" -B 1 | grep 'SAFETY:' || echo "⚠️ Missing SAFETY in $file"
  fi
done
```

### Watch Mode
Auto-run checks on file changes:

```bash
# Install cargo-watch
cargo install cargo-watch

# Watch for changes and run clippy
cargo watch -x 'clippy --lib --bins -- -D clippy::unwrap_used -D clippy::expect_used'
```

## Pre-commit Hook

Automatically run safety checks before each commit:

```bash
# Create .git/hooks/pre-commit
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash

echo "Running safety checks..."

# Run fast checks
.github/workflows/scripts/check-unsafe.sh || exit 1
.github/workflows/scripts/check-wasm-safety.sh || exit 1

# Quick Clippy check
cargo clippy --lib --bins --quiet -- \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D warnings || exit 1

echo "✅ Safety checks passed"
EOF

chmod +x .git/hooks/pre-commit
```

## Continuous Integration Tips

### Before Pushing
```bash
# Full local CI simulation
./check-safety.sh         # Your local script
cargo test                # Run all tests
cargo fmt --check         # Check formatting
```

### After PR Created
- Monitor GitHub Actions tab for workflow status
- Check "Safety Audit" check in PR
- Review summary in Actions tab for detailed results
- Fix any failures and push updates

### Debugging CI Failures

1. **Check CI logs**: Click "Details" next to failed check
2. **Run locally**: Use commands from CI logs
3. **Check environment**: Ensure same Rust version as CI
4. **Clear cache**: Try `cargo clean` and rerun

## Troubleshooting

### Scripts Not Executable
```bash
chmod +x .github/workflows/scripts/*.sh
```

### Ripgrep Not Found
```bash
# Ubuntu/Debian
sudo apt-get install ripgrep

# macOS
brew install ripgrep

# Or use cargo
cargo install ripgrep
```

### Miri Not Working
```bash
# Reinstall Miri
rustup toolchain install nightly --force
cargo +nightly miri setup --force
```

### Clippy Not Found
```bash
# Install Clippy
rustup component add clippy
```

## Related Documentation

- [Safety Audit Guide](../docs/development/safety-audit.md) - Complete workflow documentation
- [Quick Reference](../SAFETY_QUICK_REFERENCE.md) - Quick fix guide
- [Coding Standards](../../docs/development/coding-standards.md) - Project coding standards

## Summary

**Quick Pre-Push Check (1 min)**:
```bash
.github/workflows/scripts/check-unsafe.sh && \
.github/workflows/scripts/check-wasm-safety.sh && \
cargo clippy --lib --bins --quiet -- \
  -D clippy::unwrap_used -D clippy::expect_used -D warnings
```

**Full CI Simulation (5-10 min)**:
```bash
# All checks including Miri
.github/workflows/scripts/check-unsafe.sh && \
.github/workflows/scripts/check-wasm-safety.sh && \
cargo clippy --workspace --all-features --lib --bins -- \
  -D clippy::unwrap_used -D clippy::expect_used -D warnings && \
cargo +nightly miri test -p riptide-core memory_manager
```

Running these checks locally before pushing will catch 99% of CI failures and save significant time!
