# Code Coverage Guide - cargo-llvm-cov

This guide explains how to use cargo-llvm-cov for unified code coverage across the RipTide EventMesh 24-crate workspace.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
- [CI Integration](#ci-integration)
- [Coverage Reports](#coverage-reports)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Overview

RipTide EventMesh uses **cargo-llvm-cov** for code coverage analysis across all 24 workspace crates:

### Why cargo-llvm-cov?

- **Unified Coverage**: Single tool for all workspace crates
- **Accuracy**: LLVM-based instrumentation (more accurate than Tarpaulin)
- **Performance**: Faster execution with better caching
- **Multiple Formats**: HTML, LCOV, JSON output formats
- **CI-Friendly**: Optimized for GitHub Actions

### Coverage Baseline

- **Target**: 80% coverage across workspace
- **Minimum**: 75% per-crate coverage
- **Critical Crates**: 85% (core, infrastructure)

## Installation

### Local Development

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov --locked

# Install LLVM tools
rustup component add llvm-tools-preview

# Or install all tools at once
make install-tools
```

### Verify Installation

```bash
cargo llvm-cov --version
# Expected: cargo-llvm-cov 0.6.x
```

## Quick Start

### Generate Coverage (LCOV format)

```bash
make coverage
# Output: lcov.info
```

### Generate HTML Report

```bash
make coverage-html
# Output: target/llvm-cov/html/index.html
```

### Generate and Open HTML Report

```bash
make coverage-open
# Opens browser with coverage report
```

### Generate All Formats

```bash
make coverage-report
# Generates: HTML, LCOV, JSON
```

## Usage

### Command Reference

#### Makefile Targets

```bash
# Individual formats
make coverage          # LCOV format (for CI/Codecov)
make coverage-html     # HTML report
make coverage-lcov     # LCOV format (explicit)
make coverage-json     # JSON format
make coverage-open     # Generate and open HTML

# Comprehensive report (all formats)
make coverage-report
```

#### Direct cargo-llvm-cov Commands

```bash
# Basic coverage
cargo llvm-cov --workspace

# All features enabled
cargo llvm-cov --all-features --workspace

# Specific output formats
cargo llvm-cov --workspace --html
cargo llvm-cov --workspace --lcov --output-path lcov.info
cargo llvm-cov --workspace --json --output-path coverage.json

# Summary only (no files)
cargo llvm-cov --workspace --summary-only

# Exclude specific crates
cargo llvm-cov --workspace --exclude xtask --exclude riptide-test-utils

# Include ignored tests
cargo llvm-cov --workspace --include-ignored
```

### Cargo Aliases

The `.cargo/config.toml` defines convenient aliases:

```bash
# Using cargo aliases
cargo coverage           # Run coverage
cargo coverage-html      # HTML report
cargo coverage-json      # JSON report
cargo coverage-lcov      # LCOV report
cargo coverage-all       # Full coverage with lcov output
```

## CI Integration

### GitHub Actions Workflows

Coverage runs automatically on:
- Pull requests to `main`
- Pushes to `main`
- Refactoring changes to Rust files

### Workflow: baseline-check.yml

```yaml
- name: Install cargo-llvm-cov
  uses: taiki-e/install-action@cargo-llvm-cov

- name: Run coverage
  run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

- name: Upload to Codecov
  uses: codecov/codecov-action@v4
  with:
    files: lcov.info
    token: ${{ secrets.CODECOV_TOKEN }}
```

### Coverage Thresholds

The CI enforces coverage thresholds:

```bash
# Baseline threshold: 80%
THRESHOLD=80.0

# Extract coverage percentage
COVERAGE=$(cargo llvm-cov --workspace --summary-only | grep -oP 'TOTAL\s+\d+\s+\d+\s+\K\d+\.\d+')

# Fail if below threshold
if (( $(echo "$COVERAGE < $THRESHOLD" | bc -l) )); then
  exit 1
fi
```

## Coverage Reports

### HTML Report Structure

```
target/llvm-cov/html/
├── index.html              # Main coverage summary
├── crates/
│   ├── riptide-types/
│   │   ├── index.html
│   │   └── src/
│   │       └── lib.rs.html
│   ├── riptide-spider/
│   └── ...
└── coverage.css
```

### LCOV Format

```
lcov.info                   # Root of repository
```

Upload to:
- **Codecov**: Automatic via GitHub Actions
- **Coveralls**: Compatible format
- **SonarQube**: Can import LCOV

### JSON Format

```json
{
  "data": [
    {
      "files": [...],
      "totals": {
        "lines": { "count": 1000, "covered": 850, "percent": 85.0 },
        "functions": { "count": 200, "covered": 180, "percent": 90.0 },
        "regions": { "count": 500, "covered": 425, "percent": 85.0 }
      }
    }
  ]
}
```

## Best Practices

### 1. Run Coverage Locally Before Push

```bash
# Quick coverage check
make coverage-html

# Open report to identify gaps
make coverage-open
```

### 2. Focus on Critical Paths

Prioritize coverage for:
- Core business logic (85%+ target)
- Error handling paths
- Edge cases and boundaries
- Public API surfaces

### 3. Exclude Test Infrastructure

Test utilities and test-only code should be excluded:

```toml
# In Cargo.toml (per crate if needed)
[package.metadata.coverage]
exclude = [
    "tests/",
    "benches/",
    "**/test_*.rs"
]
```

### 4. Document Uncovered Code

Use `#[cfg(not(coverage))]` for legitimately uncovered code:

```rust
#[cfg(not(coverage))]
fn platform_specific_code() {
    // Code that can't be easily covered in CI
}
```

### 5. Monitor Coverage Trends

- Check Codecov dashboard regularly
- Review coverage in PR comments
- Address declining coverage immediately

## Troubleshooting

### Issue: "error: no such subcommand: `llvm-cov`"

**Solution**: Install cargo-llvm-cov

```bash
cargo install cargo-llvm-cov --locked
rustup component add llvm-tools-preview
```

### Issue: Coverage report shows 0%

**Solution**: Ensure tests are running

```bash
# Verify tests run successfully
cargo test --workspace

# Run coverage with verbose output
cargo llvm-cov --workspace --verbose
```

### Issue: Missing coverage for specific crate

**Solution**: Check workspace membership

```bash
# Verify crate is in workspace
grep -A 30 "\[workspace\]" Cargo.toml

# Run coverage for specific crate
cargo llvm-cov --package riptide-types
```

### Issue: Slow coverage generation

**Solution**: Use incremental builds and caching

```bash
# Use cached builds
export CARGO_INCREMENTAL=1

# Clean and rebuild if needed
cargo llvm-cov clean
make coverage
```

### Issue: Different results locally vs CI

**Solution**: Match CI environment

```bash
# Use same flags as CI
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Check for feature flag differences
cargo tree --all-features
```

### Issue: Coverage drops after refactoring

**Solution**: Ensure tests were moved with code

```bash
# Check test coverage
cargo llvm-cov --workspace --tests

# Verify test discovery
cargo test --workspace -- --list
```

## Advanced Usage

### Per-Crate Coverage

```bash
# Coverage for specific crate
cargo llvm-cov --package riptide-spider --open

# Multiple crates
cargo llvm-cov --package riptide-spider --package riptide-fetch
```

### Exclude Specific Files

```bash
# Exclude generated code
cargo llvm-cov --workspace --ignore-filename-regex '.*bindings\.rs'

# Exclude test utilities
cargo llvm-cov --workspace --exclude riptide-test-utils
```

### Integration with IDE

#### VS Code

Install **Coverage Gutters** extension:

```json
{
  "coverage-gutters.coverageFileNames": [
    "lcov.info",
    "target/llvm-cov/lcov.info"
  ]
}
```

Then:
1. Run `make coverage-lcov`
2. Press `Cmd+Shift+7` (or click "Watch" in status bar)
3. Coverage shows in gutter (green/red lines)

## Coverage by Component

### Current Coverage Targets

| Component | Crates | Target | Status |
|-----------|--------|--------|--------|
| Core | types, spider, fetch, security, monitoring, events, pool | 85% | ⚠️ In Progress |
| Extraction | extraction, search, intelligence | 80% | ⚠️ In Progress |
| Browser | browser, browser-abstraction, headless | 75% | ⚠️ In Progress |
| API | api, cli, facade | 80% | ⚠️ In Progress |
| Infrastructure | config, cache, reliability, persistence | 85% | ⚠️ In Progress |
| Workers | workers, streaming, pdf | 75% | ⚠️ In Progress |
| WASM | riptide-extractor-wasm | 70% | ⚠️ In Progress |

### Per-Crate Report

```bash
# Generate detailed per-crate report
cargo llvm-cov --workspace --summary-only
```

## Resources

- [cargo-llvm-cov GitHub](https://github.com/taiki-e/cargo-llvm-cov)
- [Codecov Dashboard](https://codecov.io)
- [LLVM Coverage Mapping](https://llvm.org/docs/CoverageMappingFormat.html)

## Support

For coverage-related questions:
1. Check this guide first
2. Review Codecov PR comments
3. Check GitHub Actions logs
4. Open an issue with `coverage` label

---

**Last Updated**: 2025-10-21
**Coverage Tool**: cargo-llvm-cov v0.6.21
**Target Coverage**: 80% workspace baseline
