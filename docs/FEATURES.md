# Feature Flags

RipTide uses Cargo feature flags to make WASM extraction **truly optional**, allowing you to choose between native-only (fast, small) or WASM-enabled (sandboxed) builds.

---

## Extraction Engines

### `native-parser` (Default, Recommended)

Pure Rust HTML parser using the `scraper` crate with comprehensive extraction capabilities.

**Characteristics:**
- **Performance**: 2-5ms per page extraction
- **Binary Size**: ~45 MB (50% smaller than WASM)
- **Build Time**: ~5 minutes (40% faster than WASM)
- **Memory**: Minimal overhead, no runtime sandboxing
- **Architecture**: Direct HTML parsing with DOM traversal

**Pros:**
- ✅ Fast extraction with minimal overhead
- ✅ Smaller binary size
- ✅ Faster compilation
- ✅ No WASM runtime dependency
- ✅ Lower memory footprint

**Cons:**
- ❌ No sandboxing (trusts HTML sources)
- ❌ Runs in same process as API

**Use When:**
- Processing trusted HTML sources (99% of use cases)
- Performance is critical
- Deploying in resource-constrained environments
- Building for embedded systems
- You don't need process isolation

---

### `wasm-extractor` (Opt-in, Specialized)

WASM-based extraction with sandboxing via Wasmtime Component Model.

**Characteristics:**
- **Performance**: 10-20ms per page extraction (4x slower)
- **Binary Size**: ~95 MB (+50 MB overhead)
- **Build Time**: ~8 minutes (+60% longer)
- **Memory**: Additional overhead for WASM runtime
- **Architecture**: WebAssembly with memory isolation

**Pros:**
- ✅ Sandboxed execution environment
- ✅ Memory isolation
- ✅ Resource limits enforced
- ✅ Process separation

**Cons:**
- ❌ Slower extraction (4x overhead)
- ❌ Larger binary size (+50 MB)
- ❌ Longer build time (+60%)
- ❌ More complex deployment

**Use When:**
- Processing untrusted HTML from unknown sources
- Strict resource limits required
- Plugin architecture with untrusted extractors
- Compliance requires sandboxing
- Security-critical applications

---

## Building with Feature Flags

### Default Build (Native Parser Only)

**Fastest, smallest, recommended for most users:**

```bash
# Default build - native parser only
cargo build --release

# Or explicitly specify no WASM
cargo build --release --no-default-features --features native-parser
```

**Binary Output:**
- `riptide-api`: ~45 MB
- Build time: ~5 minutes
- No WASM file needed

**Run:**
```bash
# No WASM_EXTRACTOR_PATH needed
./target/release/riptide-api
```

---

### WASM-Enabled Build

**Full functionality with optional sandboxing:**

```bash
# 1. Build with WASM feature
cargo build --release --features wasm-extractor

# 2. Build WASM component
rustup target add wasm32-wasip2
cargo build --release --target wasm32-wasip2 -p riptide-extractor-wasm

# 3. Optimize WASM (optional)
wasm-opt -Oz \
  target/wasm32-wasip2/release/riptide_extractor_wasm.wasm \
  -o extractor.wasm
```

**Binary Output:**
- `riptide-api`: ~95 MB
- `extractor.wasm`: ~2-5 MB
- Build time: ~8 minutes

**Run:**
```bash
# WASM_EXTRACTOR_PATH required
export WASM_EXTRACTOR_PATH=/path/to/extractor.wasm
./target/release/riptide-api
```

---

### Build Specific Packages

```bash
# API with native parser only (default)
cargo build --release -p riptide-api

# API with WASM support
cargo build --release -p riptide-api --features wasm-extractor

# Headless browser service (no WASM dependency)
cargo build --release -p riptide-headless

# Workers with WASM support
cargo build --release -p riptide-workers --features wasm-extractor
```

---

## Runtime Behavior

### Three-Tier Fallback System

RipTide implements a **three-tier fallback** for maximum reliability:

```
┌─────────────────────────────────────────────────────────┐
│          Level 1: Compile-Time (Feature Flags)          │
│  Is wasm-extractor feature enabled?                     │
│    Yes → WasmExtractor available                        │
│    No  → Only NativeExtractor available                 │
└──────────────────────┬──────────────────────────────────┘
                       │
                       v
┌─────────────────────────────────────────────────────────┐
│          Level 2: Runtime (File Availability)           │
│  Is WASM file at WASM_EXTRACTOR_PATH?                   │
│    Yes → Use WASM                                       │
│    No  → Fall back to native                            │
└──────────────────────┬──────────────────────────────────┘
                       │
                       v
┌─────────────────────────────────────────────────────────┐
│          Level 3: Execution (Error Recovery)            │
│  Did extraction succeed?                                │
│    Yes → Return result                                  │
│    No  → Try fallback strategy                          │
└─────────────────────────────────────────────────────────┘
```

---

### Native-Only Build (Default)

```bash
# Run with native parser (default, no env var needed)
./target/release/riptide-api

# Logs show:
# INFO riptide_api::state: Content extractor initialized
#   extractor_type="native" wasm_available=false
```

**Behavior:**
- ✅ Uses native Rust parser
- ✅ No WASM_EXTRACTOR_PATH needed
- ✅ Fastest performance (2-5ms)
- ✅ Smallest memory footprint

---

### WASM-Enabled Build with WASM File

```bash
# Export WASM path
export WASM_EXTRACTOR_PATH=/opt/riptide/extractor/extractor.wasm

# Run API
./target/release/riptide-api

# Logs show:
# INFO riptide_extraction::unified_extractor: Using WASM extractor
# INFO riptide_api::state: Content extractor initialized
#   extractor_type="wasm" wasm_available=true
```

**Behavior:**
- ✅ Uses WASM extractor (sandboxed)
- ✅ Falls back to native if WASM fails
- ⏱️ Slower performance (10-20ms)
- 📦 Larger memory footprint

---

### WASM-Enabled Build WITHOUT WASM File

```bash
# WASM_EXTRACTOR_PATH not set or file missing
./target/release/riptide-api

# Logs show:
# WARN riptide_extraction::unified_extractor:
#   WASM file not found at /path/to/extractor.wasm, using native
# INFO riptide_api::state: Content extractor initialized
#   extractor_type="native" wasm_available=true
```

**Behavior:**
- ✅ Falls back to native parser (Level 2 fallback)
- ✅ Still works (graceful degradation)
- ⚠️ Warning logged about missing WASM

---

### WASM Path Set but Feature Disabled

```bash
# Set WASM path
export WASM_EXTRACTOR_PATH=/path/to/extractor.wasm

# Run native-only build
./target/release/riptide-api  # Built without --features wasm-extractor

# Logs show:
# WARN riptide_extraction::unified_extractor:
#   WASM_EXTRACTOR_PATH set but wasm-extractor feature not enabled.
#   Rebuild with --features wasm-extractor to use WASM.
# INFO riptide_api::state: Content extractor initialized
#   extractor_type="native" wasm_available=false
```

**Behavior:**
- ✅ Uses native parser (only option available)
- ⚠️ Warning about feature mismatch
- 💡 Clear message to rebuild with feature

---

## Performance Comparison

### Build Time

| Configuration | Time | Relative |
|---------------|------|----------|
| Native only   | ~5 min | Baseline |
| With WASM     | ~8 min | +60% |

**Recommendation:** Use native-only for faster CI/CD pipelines.

---

### Binary Size

| Configuration | API Binary | WASM File | Total |
|---------------|-----------|-----------|-------|
| Native only   | ~45 MB    | N/A       | ~45 MB |
| With WASM     | ~95 MB    | ~2-5 MB   | ~100 MB |

**Recommendation:** Use native-only for embedded/edge deployments.

---

### Runtime Performance

Typical HTML extraction (5KB page):

| Extractor | Time (avg) | Relative | Throughput |
|-----------|-----------|----------|------------|
| Native    | 2-5ms     | 1x       | 200-500 req/s |
| WASM      | 10-20ms   | 4x       | 50-100 req/s |

**Recommendation:** Use native for high-throughput applications.

---

### Memory Usage

Per-request memory overhead:

| Extractor | Memory | Relative |
|-----------|--------|----------|
| Native    | ~50 KB | 1x       |
| WASM      | ~200 KB | 4x      |

**Recommendation:** Use native for memory-constrained environments.

---

## Feature Selection Guide

### Use Native Parser (Default) When:

- ✅ Processing trusted HTML sources (your sites, known publishers)
- ✅ Performance is critical (high throughput)
- ✅ Resource constraints (memory, CPU, disk)
- ✅ Fast build times required (CI/CD)
- ✅ Smaller binary size needed (embedded, edge)
- ✅ Internal content processing

**This is 99% of use cases.**

---

### Use WASM Extractor When:

- ✅ Processing untrusted HTML from unknown sources
- ✅ Strict resource limits required (memory caps)
- ✅ Compliance requires sandboxing
- ✅ Plugin architecture with untrusted code
- ✅ Security-critical applications
- ✅ Need process isolation

**This is <1% of use cases.**

---

## Environment Variables

### WASM Configuration

```bash
# WASM extractor file path (only if wasm-extractor feature enabled)
export WASM_EXTRACTOR_PATH=/opt/riptide/extractor/extractor.wasm

# WASM runtime configuration (optional)
export WASM_INSTANCES_PER_WORKER=1
export WASM_MODULE_TIMEOUT_SECS=5
export WASM_MAX_MEMORY_MB=128
```

### Logging

```bash
# See extractor selection
export RUST_LOG=info,riptide_extraction=debug

# Logs will show:
# - Which extractor was selected (native/wasm)
# - Fallback events (if WASM unavailable)
# - Extraction failures and retries
```

---

## Health Check

The `/healthz` endpoint reports extractor status:

```bash
curl http://localhost:8080/healthz | jq
```

**Native-only build:**
```json
{
  "status": "healthy",
  "version": "0.9.0",
  "extractor": {
    "type": "native",
    "wasm_available": false,
    "fallback_enabled": true
  }
}
```

**WASM-enabled build with WASM active:**
```json
{
  "status": "healthy",
  "version": "0.9.0",
  "extractor": {
    "type": "wasm",
    "wasm_available": true,
    "fallback_enabled": true
  }
}
```

---

## Migration Guide

### From Always-WASM to Optional-WASM

**Before (v0.8.x):**
```bash
# WASM always compiled and required
cargo build --release
cargo build --release --target wasm32-wasip2 -p riptide-extractor-wasm
export WASM_EXTRACTOR_PATH=target/wasm32-wasip2/release/riptide_extractor_wasm.wasm
./target/release/riptide-api
```

**After (v0.9.0+ - Native by default):**
```bash
# Default: Native only (faster, smaller)
cargo build --release
./target/release/riptide-api
# No WASM needed!
```

**After (v0.9.0+ - WASM opt-in):**
```bash
# Explicit opt-in for WASM
cargo build --release --features wasm-extractor
cargo build --release --target wasm32-wasip2 -p riptide-extractor-wasm
export WASM_EXTRACTOR_PATH=target/wasm32-wasip2/release/riptide_extractor_wasm.wasm
./target/release/riptide-api
```

---

## Troubleshooting

### Build Errors

**Error:** `wasm_extraction` module not found
```
Solution: Add --features wasm-extractor to cargo build command
```

**Error:** `WasmExtractor` type not found
```
Solution: Code was written for WASM-enabled build. Rebuild with --features wasm-extractor
```

---

### Runtime Warnings

**Warning:** WASM_EXTRACTOR_PATH set but feature not enabled
```
Solution: Either rebuild with --features wasm-extractor or unset WASM_EXTRACTOR_PATH
```

**Warning:** WASM file not found
```
Solution: Ensure WASM_EXTRACTOR_PATH points to valid .wasm file, or unset to use native
```

---

### Performance Issues

**Problem:** Extraction is slow (>10ms per page)
```
Check: Are you using WASM extractor? Switch to native for 4x speedup.
Solution: Remove WASM_EXTRACTOR_PATH or rebuild without --features wasm-extractor
```

**Problem:** High memory usage
```
Check: Is WASM extractor enabled? It uses 4x more memory than native.
Solution: Use native extractor for lower memory footprint.
```

---

## See Also

- **[Docker Deployment](DOCKER.md)** - Container configuration for both variants
- **[Architecture](04-architecture/ARCHITECTURE.md)** - System design and extraction pipeline
- **[API Reference](02-api-reference/ENDPOINT_CATALOG.md)** - REST API documentation
- **[Performance](performance-monitoring.md)** - Monitoring and optimization

---

**Built with 🚀 by the RipTide Team**

*Choose the right extractor for your use case: Native for speed, WASM for security.*
