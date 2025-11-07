# RipTide Feature Gates Reference

**Generated**: 2025-11-07
**Status**: Comprehensive inventory of all feature gates across workspace crates

## Overview

This document provides a comprehensive reference for all feature gates (conditional compilation flags) used across the RipTide workspace. Feature gates allow fine-grained control over which functionality is compiled into each crate, enabling:

- **Modular builds**: Include only the features you need
- **Platform-specific optimizations**: Enable/disable features based on target platform
- **Development vs Production**: Different feature sets for different environments
- **Optional dependencies**: Reduce compilation time and binary size by excluding unused features

## Feature Gate System

### Default Features

Most crates define sensible defaults in their `default` feature set. To use a crate with default features:

```toml
riptide-api = { path = "crates/riptide-api" }
```

### Custom Features

To use custom features, disable defaults and specify features explicitly:

```toml
riptide-api = { path = "crates/riptide-api", default-features = false, features = ["spider", "extraction"] }
```

### Build with Features

```bash
# Build with default features
cargo build -p riptide-api

# Build with specific features
cargo build -p riptide-api --no-default-features --features "spider,extraction,native-parser"

# Build with all features
cargo build -p riptide-api --all-features
```

---

## Core Crates

### riptide-api

**Location**: `crates/riptide-api/Cargo.toml`

#### Default Features
```toml
default = ["spider", "extraction", "fetch", "native-parser"]
```

#### Core Feature Gates (Optional Dependencies)

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `spider` | Spider/crawler engine integration | `dep:riptide-spider` |
| `extraction` | HTML extraction capabilities | `dep:riptide-extraction` |
| `fetch` | HTTP/network layer | `dep:riptide-fetch` |
| `browser` | Browser automation support | `dep:riptide-browser`, `dep:riptide-headless` |
| `llm` | LLM intelligence features | `dep:riptide-intelligence` |
| `workers` | Worker pool management | `dep:riptide-workers` |
| `search` | Search functionality | `dep:riptide-search` |

#### WIP Feature Gates (Scaffolding)

| Feature | Description | Status |
|---------|-------------|--------|
| `events` | EventEmitter/ResultTransformer | Scaffolding |
| `sessions` | Session management system | Scaffolding |
| `streaming` | SSE/WebSocket/NDJSON streaming | Scaffolding |
| `telemetry` | Telemetry configuration | Scaffolding |
| `persistence` | Advanced caching and multi-tenancy | Scaffolding |

#### Performance & Profiling

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `jemalloc` | Real memory monitoring via jemalloc | `riptide-performance/jemalloc`, `tikv-jemallocator`, `tikv-jemalloc-ctl` |
| `profiling-full` | Full profiling with flamegraphs (dev only) | `jemalloc`, `riptide-performance/bottleneck-analysis-full` |

#### Extraction Strategy Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `native-parser` | Native Rust parser (default, fast) | `extraction`, `riptide-extraction/native-parser` |
| `wasm-extractor` | WASM-based extraction (opt-in) | `extraction`, `riptide-extraction/wasm-extractor` |

#### Full Feature Set

| Feature | Description |
|---------|-------------|
| `full` | All features for production (when ready) |

**Full includes**: `spider`, `extraction`, `fetch`, `browser`, `llm`, `workers`, `search`, `events`, `sessions`, `streaming`, `telemetry`, `persistence`, `jemalloc`

---

### riptide-extraction

**Location**: `crates/riptide-extraction/Cargo.toml`

#### Default Features
```toml
default = ["css-extraction", "regex-extraction", "dom-utils", "chunking", "native-parser"]
```

#### Extraction Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `css-extraction` | CSS selector-based extraction | Built-in |
| `regex-extraction` | Regular expression extraction | Built-in |
| `dom-utils` | DOM manipulation utilities | Built-in |
| `table-extraction` | HTML table extraction | Built-in |
| `chunking` | Text chunking for LLM processing | Built-in |

#### Strategy Features

| Feature | Description |
|---------|-------------|
| `strategy-traits` | Extensibility traits for custom strategies |

#### Optimization Features

| Feature | Description |
|---------|-------------|
| `jsonld-shortcircuit` | Early return for complete JSON-LD schemas (Phase 10) |

#### Parser Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `native-parser` | Native Rust parser (default, fast, always available) | Built-in |
| `wasm-extractor` | WASM-based extraction (opt-in) | `dep:wasmtime`, `dep:wasmtime-wasi` |

---

### riptide-pipeline

**Location**: `crates/riptide-pipeline/Cargo.toml`

#### Default Features
```toml
default = ["fetch", "extraction"]
```

#### Pipeline Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `fetch` | HTTP fetching capability | `dep:riptide-fetch` |
| `extraction` | HTML extraction capability | `dep:riptide-extraction` |
| `llm` | LLM intelligence integration | `dep:riptide-intelligence` |
| `full` | All pipeline features | `fetch`, `extraction`, `llm` |

---

### riptide-facade

**Location**: `crates/riptide-facade/Cargo.toml`

#### Default Features
```toml
default = []
```

#### Facade Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `wasm-extractor` | WASM extraction integration | `riptide-extraction/wasm-extractor`, `riptide-cache/wasm-extractor` |

---

### riptide-performance

**Location**: `crates/riptide-performance/Cargo.toml`

#### Default Features
```toml
default = ["memory-profiling", "cache-optimization", "resource-limits"]
```

**Note**: Default excludes `flamegraph` to avoid CDDL-1.0 license in CI

#### Core Feature Groups

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `memory-profiling` | Memory profiling capabilities | `tikv-jemalloc-ctl`, `pprof`, `memory-stats` |
| `bottleneck-analysis` | Performance bottleneck detection | `criterion` |
| `bottleneck-analysis-full` | Dev-only: includes flamegraph | `bottleneck-analysis`, `flamegraph` |
| `cache-optimization` | Cache optimization features | `moka`, `redis` |
| `resource-limits` | Resource limiting | `governor` |

#### Allocator Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `jemalloc` | jemalloc control interface | `tikv-jemalloc-ctl` |

#### Environment-Specific Feature Sets

| Feature | Description | Includes |
|---------|-------------|----------|
| `production` | Production-ready features (excludes flamegraph) | `jemalloc`, `memory-profiling`, `bottleneck-analysis`, `cache-optimization`, `resource-limits` |
| `development` | Development features (includes flamegraph) | `jemalloc`, `memory-profiling`, `bottleneck-analysis-full`, `cache-optimization` |

---

### riptide-streaming

**Location**: `crates/riptide-streaming/Cargo.toml`

#### Default Features
```toml
default = ["reports", "cli"]
```

#### Streaming Features

| Feature | Description |
|---------|-------------|
| `reports` | HTML report generation |
| `cli` | CLI interface support |

---

### riptide-cache

**Location**: `crates/riptide-cache/Cargo.toml`

#### Default Features
```toml
default = []
```

#### Cache Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `distributed` | Distributed caching support | Built-in |
| `warming` | Cache warming functionality | Built-in |
| `wasm-pool` | WASM pool integration | `riptide-pool/wasm-pool` |
| `wasm-extractor` | WASM extractor integration | `riptide-extraction/wasm-extractor` |

---

### riptide-pool

**Location**: `crates/riptide-pool/Cargo.toml`

#### Default Features
```toml
default = ["native-pool"]
```

#### Pool Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `native-pool` | Native extraction pooling (CSS/Regex extractors) | `dep:riptide-extraction` |
| `wasm-pool` | WASM pool management | `dep:wasmtime`, `dep:riptide-extraction`, `riptide-extraction/wasm-extractor` |

---

### riptide-monitoring

**Location**: `crates/riptide-monitoring/Cargo.toml`

#### Default Features
```toml
default = ["telemetry", "collector", "alerts", "health"]
```

#### Monitoring Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `telemetry` | OpenTelemetry integration | Built-in |
| `collector` | Metrics collection | Built-in |
| `alerts` | Alerting system | Built-in |
| `health` | Health check endpoints | Built-in |
| `reports` | Report generation | Built-in |
| `time-series` | Time-series metrics | Built-in |
| `prometheus` | Prometheus metrics export | `dep:prometheus`, `dep:lazy_static` |

---

### riptide-pdf

**Location**: `crates/riptide-pdf/Cargo.toml`

#### Default Features
```toml
default = ["pdf"]
```

#### PDF Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `pdf` | PDF processing capabilities | `pdfium-render`, `lopdf` |
| `benchmarks` | Benchmark suite | Built-in |

---

## Cross-Cutting Features

### WASM Support

The WASM extraction system is **opt-in** across the codebase. By default, all crates use native Rust parsers for better performance.

#### WASM Feature Chain

To enable WASM extraction across the stack:

```toml
riptide-api = { features = ["wasm-extractor"] }
# This propagates to:
# - riptide-extraction/wasm-extractor
# - riptide-cache/wasm-extractor
# - riptide-pool/wasm-pool
```

#### WASM-Related Features by Crate

| Crate | Feature | Purpose |
|-------|---------|---------|
| `riptide-api` | `wasm-extractor` | Enable WASM extraction in API |
| `riptide-extraction` | `wasm-extractor` | WASM-based HTML extraction |
| `riptide-cache` | `wasm-extractor` | Cache WASM modules |
| `riptide-cache` | `wasm-pool` | WASM instance pooling |
| `riptide-pool` | `wasm-pool` | WASM instance management |
| `riptide-facade` | `wasm-extractor` | WASM extraction in facade |

### Parser Strategy

Two mutually compatible parser strategies:

1. **`native-parser`** (default): Fast, pure Rust implementation
2. **`wasm-extractor`** (opt-in): WASM-based extraction for extensibility

**Default configuration**: All crates use `native-parser` by default for optimal performance.

---

## Memory Profiling Features

### jemalloc Integration

The jemalloc allocator provides detailed memory statistics and profiling.

#### jemalloc Feature Chain

```toml
riptide-api = { features = ["jemalloc"] }
# This propagates to:
# - riptide-performance/jemalloc
# - tikv-jemallocator (MSVC-incompatible)
# - tikv-jemalloc-ctl
```

#### Platform Support

**Note**: jemalloc is **not available on MSVC** (Windows with MSVC toolchain). The feature is conditionally compiled:

```toml
[target.'cfg(not(target_env = "msvc"))'.dependencies]
tikv-jemallocator = { version = "0.5", optional = true }
tikv-jemalloc-ctl = { version = "0.5", optional = true }
```

### Profiling Levels

| Feature | Purpose | License Compatibility |
|---------|---------|----------------------|
| `jemalloc` | Basic memory profiling | ✅ CI-safe |
| `profiling-full` | Full profiling with flamegraphs | ⚠️ Dev-only (CDDL-1.0) |

**Note**: `profiling-full` includes the `flamegraph` crate which uses CDDL-1.0 license. Only use in local development.

---

## Build Profiles & Features

### Development

```bash
# Fast iteration with basic profiling
cargo build --profile fast-dev --features "jemalloc"

# Full development with flamegraphs (local only)
cargo build --profile development --features "jemalloc,profiling-full"
```

### Production

```bash
# Production build (no flamegraphs for license compliance)
cargo build --profile release --features "production"
```

### CI/CD

```bash
# CI-optimized build
cargo build --profile ci --no-default-features --features "spider,extraction,fetch"
```

---

## Feature Compatibility Matrix

### Compatible Features

| Feature | Compatible With | Incompatible With |
|---------|----------------|-------------------|
| `native-parser` | All features | None |
| `wasm-extractor` | All features | None (can coexist with `native-parser`) |
| `jemalloc` | All non-Windows MSVC | Windows MSVC builds |
| `profiling-full` | Development only | CI builds (license) |

---

## Best Practices

### 1. Use Defaults for Most Cases

The default feature sets are carefully chosen for common use cases:

```toml
# Simple and effective
riptide-api = { path = "crates/riptide-api" }
```

### 2. Disable Defaults When Customizing

When you need specific features, disable defaults first:

```toml
riptide-api = {
  path = "crates/riptide-api",
  default-features = false,
  features = ["spider", "native-parser"]
}
```

### 3. Check Platform Compatibility

Some features are platform-specific:

```toml
# jemalloc not available on Windows MSVC
[target.'cfg(not(target_env = "msvc"))'.dependencies]
riptide-api = { features = ["jemalloc"] }
```

### 4. Separate Dev and Production Features

Use different features for development and production:

```toml
# Development - Cargo.toml
[dependencies]
riptide-performance = { features = ["development"] }

# Production - via CLI
cargo build --release --features "production"
```

### 5. Document Custom Feature Combinations

When using non-default features, document why:

```toml
# Using WASM for extensibility despite performance cost
riptide-api = {
  default-features = false,
  features = ["spider", "wasm-extractor"]  # Custom extractors
}
```

---

## Feature Gate Usage in Code

### Conditional Compilation

Feature gates are used throughout the codebase with `#[cfg(feature = "...")]`:

```rust
// Conditionally compile based on feature
#[cfg(feature = "wasm-extractor")]
use wasmtime::Engine;

// Feature-gated module
#[cfg(feature = "profiling-full")]
pub mod flamegraph;

// Feature-gated function
#[cfg(feature = "jemalloc")]
pub fn get_memory_stats() -> MemoryStats {
    // jemalloc-specific implementation
}

// Default implementation
#[cfg(not(feature = "jemalloc"))]
pub fn get_memory_stats() -> MemoryStats {
    // Fallback implementation
}
```

### Feature Detection

Check which features are enabled at runtime:

```rust
pub fn available_features() -> Vec<&'static str> {
    let mut features = vec![];

    #[cfg(feature = "wasm-extractor")]
    features.push("wasm-extractor");

    #[cfg(feature = "jemalloc")]
    features.push("jemalloc");

    #[cfg(feature = "native-parser")]
    features.push("native-parser");

    features
}
```

---

## Common Feature Combinations

### Minimal Build (fastest compilation)

```bash
cargo build -p riptide-api \
  --no-default-features \
  --features "fetch,native-parser"
```

### Standard Web Scraping

```bash
cargo build -p riptide-api \
  --features "spider,extraction,fetch,native-parser"
```

### Full-Featured Development

```bash
cargo build -p riptide-api \
  --features "full,profiling-full"
```

### Production API Server

```bash
cargo build --release -p riptide-api \
  --features "spider,extraction,fetch,browser,llm,jemalloc"
```

### WASM-Based Extraction

```bash
cargo build -p riptide-api \
  --no-default-features \
  --features "spider,wasm-extractor"
```

---

## Troubleshooting

### Feature Not Available

**Error**: `feature X is not available in this configuration`

**Solution**: Check if the feature is:
1. Defined in the crate's Cargo.toml `[features]` section
2. Compatible with your platform (e.g., jemalloc on non-MSVC)
3. Spelled correctly (case-sensitive)

### Compilation Errors with Features

**Error**: `cannot find type Y in this scope`

**Solution**: Ensure required features are enabled:
```bash
# Check what features are currently enabled
cargo metadata --format-version 1 | jq '.packages[] | select(.name == "riptide-api") | .features'

# Enable missing feature
cargo build --features "missing-feature"
```

### License Conflicts

**Error**: License incompatibility in CI

**Solution**: Avoid `profiling-full` in CI builds:
```yaml
# .github/workflows/ci.yml
- run: cargo build --features "production"  # excludes flamegraph
```

---

## Future Feature Gates

These features are planned but not yet implemented:

| Feature | Status | Target Crate |
|---------|--------|--------------|
| `distributed-spider` | Planned | riptide-spider |
| `advanced-caching` | WIP | riptide-cache |
| `gpu-acceleration` | Planned | riptide-extraction |
| `real-time-streaming` | WIP | riptide-streaming |

---

## Summary

### Total Feature Gates by Category

- **Core Features**: 15 (spider, extraction, fetch, etc.)
- **Performance Features**: 8 (jemalloc, profiling, etc.)
- **Parser Features**: 2 (native-parser, wasm-extractor)
- **Monitoring Features**: 7 (telemetry, collector, etc.)
- **Cache Features**: 4 (distributed, warming, etc.)
- **WIP/Scaffolding**: 5 (events, sessions, etc.)

### Recommended Configurations

| Use Case | Features |
|----------|----------|
| Development | `default + profiling-full` |
| Production | `spider,extraction,fetch,browser,jemalloc` |
| CI/Testing | `default` (exclude profiling-full) |
| Minimal API | `fetch,native-parser` |
| WASM Research | `wasm-extractor` |

---

**Maintained by**: RipTide Team
**Last Updated**: 2025-11-07
**Next Review**: Phase 3 completion
