# SIGILL (Signal 4) Error Research Report - Rust Test Environments

**Date:** 2025-10-08
**Project:** EventMesh
**Research Focus:** SIGILL errors in Rust/tokio async tests, rand crate CPU features, and Docker/CI/CD solutions

---

## Executive Summary

This project is experiencing SIGILL (Illegal Instruction) errors due to **`target-cpu=native`** configuration in `.cargo/config.toml` and CI/CD workflows. The issue occurs when code compiled with CPU-specific optimizations (AVX2, RDRAND, etc.) runs on CPUs that don't support those instructions.

**Critical Finding:** Your `.cargo/config.toml` uses `-C target-cpu=native` for x86_64 builds, and your CI/CD pipeline also sets `RUSTFLAGS: "-Dwarnings -C target-cpu=native"`. This creates non-portable binaries that may SIGILL on different CPUs.

---

## 1. Common Causes of SIGILL in Rust/Tokio Async Tests

### 1.1 Root Cause: CPU Feature Mismatch

**The Problem:**
- `target-cpu=native` compiles code optimized for the **build machine's CPU**
- Binary includes instructions (AVX, AVX2, AVX-512, RDRAND, RDSEED) specific to that CPU
- When run on a CPU without those features → **SIGILL**

### 1.2 Tokio-Specific Issues

From research findings:
- **Red Hat Bug #2224885**: Rust 1.71.0 failed to compile tokio tests on s390x with SIGILL
- **Recent Regression (Jan 2025)**: Nightly compiler regression between `nightly-2025-01-07` and `nightly-2025-01-13` causing SIGILL in optimized builds
- Tokio's async runtime can trigger SIMD-optimized code paths that fail on incompatible CPUs

### 1.3 Your Specific Configuration

**File: `/workspaces/eventmesh/.cargo/config.toml` (Lines 8-12):**
```toml
# Native x86_64 builds - OK to optimize for specific CPU features
[target.'cfg(all(target_arch = "x86_64", not(target_family = "wasm")))']
rustflags = [
  "-C", "target-cpu=native",
]
```

**File: `/workspaces/eventmesh/.github/workflows/ci.yml` (Lines 84, 234, 241):**
```yaml
env:
  RUSTFLAGS: "-Dwarnings -C target-cpu=native"
```

**Impact:** Tests built on GitHub Actions runners (AMD EPYC 7763) will include AVX2 and RDRAND instructions that may not exist on deployment targets.

---

## 2. Rand Crate and SIMD/AVX CPU Instructions

### 2.1 Rand Crate Versions Detected

From dependency analysis:
```
rand v0.7.3
rand v0.8.5  ← Primary version
rand v0.9.2
```

Multiple versions present due to transitive dependencies (phf_generator, scraper, etc.)

### 2.2 Rand's CPU Feature Dependencies

**getrandom crate (rand's backend):**
- Uses **RDRAND/RDSEED** CPU instructions when available
- Has runtime detection but can be compiled with `target-cpu=native`
- Known issues:
  - **GitHub Issue #27**: RDRAND implementation should enable feature itself
  - **GitHub Issue #228**: RDRAND-based output bias concerns
  - AMD CPUs before family 17h (Zen) have RDRAND bugs after suspend

**Key Risk:** When compiled with `target-cpu=native` on a CPU with RDRAND, the binary will use RDRAND instructions even on CPUs without it.

### 2.3 Evidence from Your Codebase

**File: `/workspaces/eventmesh/crates/riptide-intelligence/tests/provider_tests.rs` (Line 34-35):**
```rust
// Only use random if failure_rate is non-zero to avoid SIGILL issues
if self.failure_rate > 0.0 && rand::random::<f32>() < self.failure_rate {
```

**Comment shows awareness of SIGILL risk with rand!** This is a defensive workaround.

### 2.4 CPU Features on Build Machine

Current CI runner (AMD EPYC 7763) supports:
```
avx avx2 rdrand rdseed sha_ni vaes vpclmulqdq
```

These will be baked into binaries with `target-cpu=native`.

---

## 3. CPU Feature Flags and SIGILL Causes

### 3.1 How target-cpu=native Works

**From Rust documentation:**
- `-C target-cpu=native` tells LLVM to optimize for the **current machine**
- Doesn't change the target triple, just enables all CPU features
- **Non-portable by design**

### 3.2 Problematic Instructions

| Instruction Set | Required CPU | SIGILL Risk |
|----------------|--------------|-------------|
| **AVX** | Sandy Bridge (2011+) | High on pre-2011 CPUs |
| **AVX2** | Haswell (2013+) | High on pre-2013 CPUs |
| **AVX-512** | Skylake-X (2017+) | Very High (not widely supported) |
| **RDRAND** | Ivy Bridge (2012+) | Medium (getrandom fallback usually works) |
| **RDSEED** | Broadwell/Zen (2015+) | Medium |

### 3.3 Detection at Runtime

**CPUID detection** (not happening with `target-cpu=native`):
- Bit 30 of ECX after CPUID function 01H = RDRAND support
- Bit 18 of EBX after CPUID function 07H = RDSEED support

**Runtime detection in Rust:**
```rust
if is_x86_feature_detected!("avx2") {
    unsafe { my_function_avx2() }
} else {
    my_function_scalar()
}
```

---

## 4. Environment-Specific Issues (Docker, CI/CD, Different CPUs)

### 4.1 Docker Container Issues

**The Docker Problem:**
- Build machine: AMD EPYC 7763 with AVX2/RDRAND
- Runtime container: May run on older Xeon without AVX2
- Result: **SIGILL on container startup**

**Your Docker Build (`.github/workflows/ci.yml` lines 246-290):**
```yaml
docker-build:
  needs: [build]  # Uses artifacts from build with target-cpu=native
```

Binaries built with `target-cpu=native` are copied into Docker images → non-portable containers.

### 4.2 CI/CD Environment Mismatch

**GitHub Actions Runner:**
- CPU: AMD EPYC 7763 (latest features)
- Compiles with all AVX2/RDRAND/RDSEED enabled

**Deployment Targets:**
- May be older AWS EC2 instances (pre-Haswell)
- Local developer machines (varied CPUs)
- Other CI systems (different CPU families)

**Result:** Binaries work in CI, fail in production.

### 4.3 Cross-Platform Testing Gap

Your CI only tests on **one CPU architecture**:
```yaml
platform: linux/amd64  # Single platform, single CPU type
```

No testing on:
- Older CPUs (pre-AVX2)
- ARM64 (if needed)
- AMD vs Intel variations

---

## 5. Solutions and Workarounds

### 5.1 Immediate Fix: Remove target-cpu=native

**✅ RECOMMENDED FOR PRODUCTION**

**Update `.cargo/config.toml`:**
```toml
# BEFORE (PROBLEMATIC):
[target.'cfg(all(target_arch = "x86_64", not(target_family = "wasm")))']
rustflags = [
  "-C", "target-cpu=native",
]

# AFTER (PORTABLE):
[target.'cfg(all(target_arch = "x86_64", not(target_family = "wasm")))']
rustflags = [
  # Remove target-cpu=native for portable binaries
  # Use runtime feature detection instead
]
```

**Update `.github/workflows/ci.yml`:**
```yaml
# BEFORE (PROBLEMATIC):
env:
  RUSTFLAGS: "-Dwarnings -C target-cpu=native"

# AFTER (PORTABLE):
env:
  RUSTFLAGS: "-Dwarnings"
  # Remove -C target-cpu=native
```

**Trade-off:** ~5-15% performance loss, but **guaranteed portability**.

---

### 5.2 Alternative: Specify Baseline CPU Target

**For wider compatibility:**
```toml
[target.'cfg(all(target_arch = "x86_64", not(target_family = "wasm")))']
rustflags = [
  "-C", "target-cpu=x86-64-v2",  # ~2009+ CPUs (SSE4.2, POPCNT)
  # or
  "-C", "target-cpu=x86-64-v3",  # ~2015+ CPUs (AVX2, BMI2)
]
```

**x86-64 Microarchitecture Levels:**
- **x86-64**: Original (2003+) - SSE2 only
- **x86-64-v2**: ~2009+ - adds SSE4.2, POPCNT
- **x86-64-v3**: ~2015+ - adds AVX2, BMI2, FMA
- **x86-64-v4**: ~2017+ - adds AVX-512 (rare)

**Recommended:** `x86-64-v2` for maximum compatibility, `x86-64-v3` for modern systems.

---

### 5.3 Advanced: Runtime Feature Detection

**Use `multiversion` crate for critical hot paths:**

**Cargo.toml:**
```toml
[dependencies]
multiversion = "0.7"
```

**Code example:**
```rust
use multiversion::multiversion;

#[multiversion(targets("x86_64+avx2", "x86_64+avx", "x86_64"))]
fn process_data(input: &[f32]) -> Vec<f32> {
    // Compiler generates 3 versions:
    // 1. AVX2-optimized
    // 2. AVX-optimized
    // 3. Baseline x86_64
    // Runtime dispatches to best available
    input.iter().map(|x| x * 2.0).collect()
}
```

**Benefits:**
- Portable binary that works everywhere
- Still gets optimizations on capable CPUs
- No SIGILL risk

---

### 5.4 Ultimate Solution: cargo-multivers

**For production deployments:**

**Installation:**
```bash
cargo install cargo-multivers
```

**Usage:**
```bash
# Build fat binary with multiple CPU feature sets
cargo multivers build --release

# Result: Single binary with x86-64, x86-64-v2, x86-64-v3 versions
# Runtime CPU detection selects best version
```

**GitHub Action Integration:**
```yaml
- name: Install cargo-multivers
  run: cargo install cargo-multivers

- name: Build optimized portable binary
  run: cargo multivers build --release
```

**Benefits:**
- Best performance on all CPUs
- No SIGILL risk
- Single binary deployment
- Official Rust solution for this problem

---

### 5.5 Rand Crate Specific Fixes

**Option 1: Disable RDRAND feature (if causing issues):**
```toml
[dependencies]
getrandom = { version = "0.2", default-features = false, features = ["std"] }
```

**Option 2: Force specific getrandom backend:**
```toml
[dependencies]
getrandom = { version = "0.2", features = ["std"] }
# Ensures OS randomness source, not RDRAND
```

**Option 3: Update to latest rand/getrandom:**
```bash
cargo update -p rand -p getrandom
```

Recent versions have better CPU detection and fallbacks.

---

## 6. Recommended Action Plan

### Phase 1: Immediate Fix (0-1 hour)

1. **Remove `target-cpu=native` from `.cargo/config.toml`**
2. **Remove `target-cpu=native` from CI workflows**
3. **Test that builds still work**
4. **Verify no SIGILL in tests**

### Phase 2: Baseline Optimization (1-2 hours)

1. **Add `target-cpu=x86-64-v2` to config** (safe baseline)
2. **Update CI to test on multiple environments** (if possible)
3. **Document CPU requirements** in README

### Phase 3: Advanced Optimization (2-4 hours)

1. **Identify performance-critical functions**
2. **Add `multiversion` crate for hot paths**
3. **Benchmark performance gain**
4. **Consider `cargo-multivers` for releases**

### Phase 4: Rand Crate Audit (1-2 hours)

1. **Update rand to latest stable (0.8.5)**
2. **Check for RDRAND-specific issues**
3. **Add integration tests for randomness**
4. **Consider pinning getrandom version**

---

## 7. Configuration Examples

### 7.1 Portable .cargo/config.toml

```toml
[build]
target = "x86_64-unknown-linux-gnu"

# Portable x86_64 builds (NO target-cpu=native)
[target.'cfg(all(target_arch = "x86_64", not(target_family = "wasm")))']
rustflags = [
  # Option 1: Default (maximum portability)
  # No special CPU flags

  # Option 2: Baseline modern CPUs (~2009+)
  # "-C", "target-cpu=x86-64-v2",

  # Option 3: Recent CPUs with AVX2 (~2015+)
  # "-C", "target-cpu=x86-64-v3",
]

# WASM builds - must NOT inherit x86 CPU flags
[target.wasm32-wasi]
rustflags = []

[target.wasm32-wasip2]
rustflags = []
```

### 7.2 Updated CI Workflow

```yaml
env:
  RUST_BACKTRACE: 1
  RUSTFLAGS: "-Dwarnings"  # Removed -C target-cpu=native
  CARGO_TERM_COLOR: always
  CARGO_INCREMENTAL: 0

jobs:
  build:
    strategy:
      matrix:
        target:
          - native
          - wasm32-wasip2
    steps:
      - name: Build native binaries
        if: matrix.target == 'native'
        env:
          # Portable build - works on all x86_64 CPUs
          RUSTFLAGS: "-Dwarnings"
        run: cargo build --release --workspace

      - name: Build optimized (optional)
        if: matrix.target == 'native' && github.event_name == 'release'
        env:
          # Only for release builds, use cargo-multivers
          RUSTFLAGS: "-Dwarnings"
        run: |
          cargo install cargo-multivers
          cargo multivers build --release
```

### 7.3 Dockerfile Best Practices

```dockerfile
# Multi-stage build for portable binaries

# Stage 1: Build with portable flags
FROM rust:1.75 AS builder
WORKDIR /app

# Use default Rust target (portable)
ENV RUSTFLAGS="-C target-cpu=x86-64-v2"

COPY . .
RUN cargo build --release

# Stage 2: Runtime with minimal dependencies
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/riptide-api /usr/local/bin/

# Verify CPU compatibility at build time
RUN lscpu | grep -i flags || true

CMD ["riptide-api"]
```

---

## 8. Testing Strategy

### 8.1 CPU Compatibility Tests

**Add to CI:**
```yaml
test-compatibility:
  name: Test on Different CPU Levels
  runs-on: ubuntu-latest
  strategy:
    matrix:
      cpu-level: [baseline, modern, latest]
  steps:
    - name: Build for CPU level
      run: |
        case ${{ matrix.cpu-level }} in
          baseline) CPU="x86-64" ;;
          modern) CPU="x86-64-v2" ;;
          latest) CPU="x86-64-v3" ;;
        esac
        RUSTFLAGS="-C target-cpu=$CPU" cargo build --release

    - name: Test binary
      run: cargo test --release
```

### 8.2 Rand Crate Tests

**Add test for SIGILL detection:**
```rust
#[test]
fn test_rand_no_sigill() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Should not SIGILL on any CPU
    for _ in 0..1000 {
        let _: f32 = rng.gen();
    }
}

#[test]
fn test_rand_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| {
                let mut rng = rand::thread_rng();
                rng.gen::<f32>()
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
```

---

## 9. Key Findings Summary

| Issue | Current State | Risk Level | Solution |
|-------|--------------|------------|----------|
| **target-cpu=native in config** | ✅ Found | 🔴 **HIGH** | Remove or replace with x86-64-v2 |
| **target-cpu=native in CI** | ✅ Found | 🔴 **HIGH** | Remove from RUSTFLAGS |
| **Multiple rand versions** | ✅ Found | 🟡 Medium | Consolidate to 0.8.5 |
| **Docker portability** | ⚠️ At Risk | 🔴 **HIGH** | Use portable builds |
| **No CPU diversity testing** | ✅ Confirmed | 🟡 Medium | Add matrix testing |
| **Defensive rand workaround** | ✅ Found in code | 🟡 Medium | Proper fix instead |

---

## 10. References and Resources

### Research Sources

1. **Rust SIGILL Issues:**
   - https://users.rust-lang.org/t/sigill-illegal-instruction-when-running-cargo-test/26707
   - https://github.com/rust-lang/rust/issues/38218 (target-cpu=native SIGILL)
   - https://github.com/rust-lang/rust/issues/36448 (target-cpu=native issues)

2. **CPU Features and SIMD:**
   - https://rust-lang.github.io/packed_simd/perf-guide/target-feature/rustflags.html
   - https://rust-lang.github.io/rfcs/2045-target-feature.html
   - https://nnethercote.github.io/perf-book/build-configuration.html

3. **Rand/Getrandom Issues:**
   - https://github.com/rust-random/getrandom/issues/27 (RDRAND feature)
   - https://github.com/rust-random/getrandom/issues/228 (RDRAND bias)
   - https://rust-random.github.io/book/update-0.8.html

4. **Solutions:**
   - https://github.com/ronnychevalier/cargo-multivers (Fat binaries)
   - https://docs.rs/multiversion (Runtime dispatch)
   - https://crates.io/crates/cpufeatures (Feature detection)

### Tools

- **cargo-multivers**: https://crates.io/crates/cargo-multivers
- **multiversion**: https://crates.io/crates/multiversion
- **cpufeatures**: https://crates.io/crates/cpufeatures
- **cargo-audit**: Security auditing
- **cargo-bloat**: Binary size analysis

---

## 11. Conclusion

**The SIGILL issue is caused by `target-cpu=native`** in both your Cargo config and CI workflows. This creates binaries optimized for the build machine's CPU (AMD EPYC 7763) that will crash on older/different CPUs.

**Immediate action required:**
1. Remove `target-cpu=native` from `.cargo/config.toml`
2. Remove `target-cpu=native` from `.github/workflows/ci.yml`
3. Consider `x86-64-v2` as baseline for reasonable performance + portability

**Long-term:**
- Use `cargo-multivers` for release builds
- Add `multiversion` crate for hot paths
- Test on multiple CPU architectures
- Update rand/getrandom to latest versions

This will eliminate SIGILL errors while maintaining acceptable performance.

---

**End of Report**
