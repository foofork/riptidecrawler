# Local CI/CD Workflow

Run the same checks locally that GitHub Actions runs - catch issues before pushing!

## 🚀 Quick Start

```bash
# One-time setup (installs cargo-deny, cargo-audit, git hooks)
make install-tools
./scripts/setup-git-hooks.sh

# Your new workflow:
# 1. Make changes
vim src/...

# 2. Commit (auto-formats instantly)
git commit -am "feat: my changes"

# 3. Quick validation before push (30s)
make ci-quick

# 4. Full validation before push (5min)
make ci

# 5. Push with confidence
git push
```

## 🪝 Git Hooks (Hybrid Mode)

**Auto-format on commit (instant):**
- Every `git commit` auto-runs `cargo fmt`
- Formats your code instantly (no delay)
- Auto-restages formatted files
- Never blocks your workflow

**Manual validation (when you want):**
- `make ci-quick` - Quick checks before push (~30s)
- `make ci` - Full validation before push (~5min)
- You control when to run expensive checks

**Skip auto-format if needed:**
```bash
git commit --no-verify -m "WIP: testing"
```

## ⚡ Performance Tiers

### Tier 0: Auto-format (instant)
Happens automatically on every commit:
```bash
git commit -am "..."  # Auto-formats, no waiting
```

### Tier 1: Quick Check (30s-1min)
Run manually before push:
```bash
make ci-quick   # fmt + clippy + unit tests
```
**What it catches:** 90% of CI failures
**Hardware:** Works on weak machines

### Tier 2: Full Validation (5-10min)
Run manually before important pushes:
```bash
make ci         # Everything CI does (except Docker)
```
**What it catches:** 99% of CI failures
**Hardware:** Any dev machine

### Tier 3: CI Only (10-30min)
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
```

## 🔧 What Gets Checked

| Check | Auto (commit) | Manual (quick) | Manual (full) | CI Only |
|-------|---------------|----------------|---------------|---------|
| Formatting | ✅ | ✅ | ✅ | ✅ |
| Clippy lints | ❌ | ✅ | ✅ | ✅ |
| Unit tests | ❌ | ✅ | ✅ | ✅ |
| Integration tests | ❌ | ❌ | ✅ | ✅ |
| Security audit | ❌ | ❌ | ✅ | ✅ |
| License check | ❌ | ❌ | ✅ | ✅ |
| Native build | ❌ | ❌ | ✅ | ✅ |
| WASM build | ❌ | ❌ | Optional | ✅ |
| Docker build | ❌ | ❌ | ❌ | ✅ |

## 🎯 Recommended Workflow

**On any machine (including weak laptops):**

```bash
# 1. Make changes
vim src/my-feature.rs

# 2. Commit (auto-formats instantly)
git commit -am "feat: add my feature"
# ✅ Code formatted

# 3. Continue iterating
vim src/...
git commit -am "fix: typo"
# ✅ Code formatted

# 4. Before pushing, quick check (30s)
make ci-quick
# ✅ Clippy passed
# ✅ Unit tests passed

# 5. Full validation (5min)
make ci
# ✅ All checks passed

# 6. Push with confidence
git push
```

**Benefits:**
- ✅ Zero-friction commits (instant auto-format)
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

**"Want to disable auto-format for one commit"**
```bash
git commit --no-verify -m "WIP: testing"
```

**"Want to remove git hooks"**
```bash
rm .git/hooks/pre-commit
```

## 📊 Time Comparisons

| Workflow | Time | When |
|----------|------|------|
| Auto-format | Instant | Every commit |
| Quick check | 30s | Before push |
| Full validation | 5-10min | Before push |
| GitHub CI | 25min | After push |

**Result: 30s of local validation saves 25min of CI waiting!**

## 🔄 Hook Management

**Check if hooks are installed:**
```bash
ls -la .git/hooks/pre-commit
```

**Reinstall hooks:**
```bash
./scripts/setup-git-hooks.sh
```

**Temporarily disable auto-format:**
```bash
# Per-commit
git commit --no-verify -m "message"

# Or remove hook
rm .git/hooks/pre-commit
```

**Re-enable:**
```bash
./scripts/setup-git-hooks.sh
```
