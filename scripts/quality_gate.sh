#!/bin/bash
set -e

# Add cargo to PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Match GitHub Actions strictness - treat all warnings as errors (including deprecations)
export RUSTFLAGS="-Dwarnings"

echo "========================================="
echo "Running RipTide Quality Gate Checks"
echo "========================================="
echo ""

# Clean build artifacts for deterministic builds
cargo clean

# Store start time
start_time=$(date +%s)

# 1) Auto-fix formatting first, then verify
echo "1. Auto-fixing code formatting..."
cargo fmt --all
echo "✓ Code formatted"
echo ""

echo "2. Verifying formatting is clean..."
if ! cargo fmt --all -- --check; then
    echo "✗ Formatting verification failed (this shouldn't happen after auto-fix)"
    exit 1
fi
echo "✓ Formatting verification passed"
echo ""

# 3) Clippy check
echo "3. Running cargo clippy --workspace -- -D warnings"
if ! cargo clippy --workspace -- -D warnings 2>&1 | tee /tmp/clippy_output.log; then
    echo "✗ Clippy check failed - see /tmp/clippy_output.log"
    exit 1
fi
echo "✓ Clippy check passed"
echo ""

# 4) Build workspace
echo "4. Running cargo build --workspace"
if ! cargo build --workspace 2>&1 | tee /tmp/build_output.log; then
    echo "✗ Build failed - see /tmp/build_output.log"
    exit 1
fi
echo "✓ Build passed"
echo ""

# 5) Run tests
echo "5. Running cargo test --workspace --no-fail-fast"
echo "Note: Integration tests require external services (Redis, PostgreSQL, Chrome)"

if ! cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/test_output.log; then
    echo "✗ Tests failed - see /tmp/test_output.log"
    exit 1
fi
echo "✓ Tests passed"
echo ""

# Calculate duration
end_time=$(date +%s)
duration=$((end_time - start_time))
minutes=$((duration / 60))
seconds=$((duration % 60))

echo "========================================="
echo "All quality checks passed! ✓"
echo "Time taken: ${minutes}m ${seconds}s"
echo "========================================="
