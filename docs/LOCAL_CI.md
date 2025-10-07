# Local CI/CD Workflow

Run the same checks locally that GitHub Actions runs - catch issues before pushing!

## 🚀 Quick Start

```bash
# One-time setup (installs cargo-deny, cargo-audit)
make install-tools

# Before every commit (30s-1min)
make ci-quick

# Before pushing (5-10min)
make ci
```

## ⚡ Performance Tiers

### Tier 1: Instant Feedback (0-10s)
Perfect for rapid iteration on any machine:
```bash
make fmt        # Auto-format code
make lint       # Clippy warnings
```

### Tier 2: Quick Pre-Commit (30s-1min)
Run before every commit - lightweight:
```bash
make ci-quick   # Runs: fmt + clippy + unit tests
```
**What it catches:** 90% of CI failures
**Hardware:** Works on weak machines

### Tier 3: Full Validation (5-10min)
Run before pushing - comprehensive:
```bash
make ci         # Runs everything CI does (except Docker)
```
**What it catches:** 99% of CI failures
**Hardware:** Any dev machine

### Tier 4: CI Only (10-30min)
Let GitHub Actions handle these:
- Multi-platform Docker builds
- Cross-platform matrix builds
- Artifact publishing
- Performance benchmarks over time

## 📋 Individual Checks

```bash
# Code quality
make fmt                # Format code
make lint               # Run clippy

# Testing
make test-unit          # Unit tests only (fast)
make test-int           # Integration tests (slower)
make test               # All tests

# Security
make audit              # cargo audit + cargo deny

# Building
make build              # Native build
make build-wasm         # WASM build
```

## 🔧 What Gets Checked

| Check | Local (quick) | Local (full) | CI Only |
|-------|---------------|--------------|---------|
| Formatting | ✅ | ✅ | ✅ |
| Clippy lints | ✅ | ✅ | ✅ |
| Unit tests | ✅ | ✅ | ✅ |
| Integration tests | ❌ | ✅ | ✅ |
| Security audit | ❌ | ✅ | ✅ |
| License check | ❌ | ✅ | ✅ |
| Native build | ❌ | ✅ | ✅ |
| WASM build | ❌ | Optional | ✅ |
| Docker build | ❌ | ❌ | ✅ |
| Multi-platform | ❌ | ❌ | ✅ |

## 🎯 Recommended Workflow

**On any machine (including weak laptops):**

```bash
# 1. Make changes
vim src/...

# 2. Quick check (30s)
make ci-quick

# 3. Commit if green
git add . && git commit

# 4. Full validation before push (5min)
make ci

# 5. Push if green
git push
```

**Benefits:**
- ✅ Catch 90%+ of issues in 30s
- ✅ No wasted CI minutes
- ✅ Faster feedback loop
- ✅ Works on slow machines

## 💡 Performance Tips

**Speed up incremental builds:**
```bash
# Use sccache for cached compilation
cargo install sccache
export RUSTC_WRAPPER=sccache
```

**Parallel tests:**
```bash
# Use all CPU cores
cargo test -- --test-threads=8
```

**Skip slow tests during iteration:**
```bash
cargo test --lib          # Skip integration tests
cargo test test_name      # Run specific test
```

## 🐛 Troubleshooting

**"cargo-deny not found"**
```bash
make install-tools
```

**"Tests too slow"**
```bash
# Just run unit tests
make test-unit

# Or skip WASM in full CI
SKIP_WASM=1 make ci
```

**"Want to auto-run on file changes"**
```bash
cargo install cargo-watch
make watch
```

## 📊 Time Comparisons

| Workflow | Local | GitHub CI |
|----------|-------|-----------|
| Quick check | 30s | - |
| Full validation | 5-10min | 8-12min |
| Docker builds | N/A | 15-20min |
| **Total feedback** | **30s** | **25min** |

**Running locally first saves ~20min per push!**
