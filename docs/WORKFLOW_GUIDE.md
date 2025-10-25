# GitHub Actions Workflow Guide

## Overview

This repository uses a streamlined CI/CD setup with **minimal noise** and **maximum clarity**.

## Active Workflows

### 1. **ci.yml** - Core CI (Blocking) ✅

**Runs:** Every PR and push to `main`
**Blocks:** Yes - required for merge
**What it does:**
- Format check (`cargo fmt`)
- Clippy with `-D warnings`
- Unit + integration tests (nextest)
- Build native binaries
- Build WASM components
- Binary size checks
- Benchmark runs (main only)

**Path filters:**
- Ignores: `docs/**`, `**.md`, Docker/workflow/metrics changes, API changes
- API changes trigger `api-validation.yml` instead

**Status check required:** `CI / Final Validation`

---

### 2. **api-validation.yml** - API Contract Tests (Blocking when API changes) ✅

**Runs:** Only when API files change
**Blocks:** Yes - required for API changes
**What it does:**
- OpenAPI validation (spectral + swagger-cli)
- Contract testing (Dredd)
- API fuzzing (Schemathesis)

**Path filters:**
```yaml
paths:
  - 'docs/api/openapi.yaml'
  - 'crates/riptide-api/**'
  - 'tests/api/**'
```

**Status check required:** `API Validation / validation-complete`

---

### 3. **docker-build.yml** - Docker Images (Non-blocking for PRs) 🐳

**Runs:**
- PRs: Build only (no push)
- `main` pushes: Build + push to registry
- Tags (`v*`): Build + push with version tags

**Blocks:** No - informational for PRs
**What it does:**
- Builds Docker images for `api` and `headless` services
- Pushes to `ghcr.io` on main/tags
- Smoke tests images on PRs

**Path filters:**
- Ignores: `docs/**`, `**.md`, CI workflows

---

### 4. **quality-gates.yml** - Advisory Quality Checks ⚠️

**Runs:** Every PR and push to `main` (Rust file changes only)
**Blocks:** No - advisory (will become blocking once baseline is clean)
**What it does:**
- Security audit (`cargo-audit`, `cargo-deny`)
- Unsafe code audit with SAFETY comments
- Clippy (no `unwrap`/`expect` in production)
- Refactoring metrics (file length checks)
- Miri memory safety tests
- Code coverage (uploaded to Codecov)

**Path filters:**
```yaml
paths:
  - 'crates/**/*.rs'
  - 'wasm/**/*.rs'
  - 'Cargo.toml'
  - 'Cargo.lock'
```

**Continuation:** `continue-on-error: true` on all jobs
**Goal:** Clean up violations, then make blocking

---

### 5. **metrics.yml** - Performance Tracking (Scheduled) 📊

**Runs:** Nightly at 2 AM UTC + manual
**Blocks:** No
**What it does:**
- Tracks build times
- Tracks test times
- Monitors cache hit rates
- Collects resource usage
- Commits metrics to repo (main only)

**Schedule:** `cron: "0 2 * * *"`

---

### 6. **rust-autofix.yml** - Auto-formatting (Helper) 🤖

**Runs:** When Rust files change in PRs
**Blocks:** No
**What it does:**
- Runs `cargo fmt --all`
- Runs `cargo clippy --fix` (safe rules)
- Creates autofix PR if changes detected

**Permissions:** Requires PR from same repo (not forks)

---

## Required Status Checks

### GitHub Repository Settings

Navigate to: **Settings → Branches → Branch protection rules for `main`**

**Required status checks before merging:**

```text
✅ CI / Final Validation
✅ API Validation / validation-complete (when API paths change)
```

**Optional but recommended:**
- `Docker Build Complete` (informational)
- `Quality Summary` (advisory - once baseline is clean, make required)

---

## Workflow Triggers Summary

| Workflow | PR | Push (main) | Schedule | Manual | Path-filtered |
|----------|----|--------------:|----------|--------|---------------|
| ci.yml | ✅ | ✅ | - | ✅ | Yes (ignores docs, API) |
| api-validation.yml | ✅ | ✅ | - | ✅ | Yes (API files only) |
| docker-build.yml | Build only | Build + Push | - | ✅ | Yes (ignores docs) |
| quality-gates.yml | ✅ | ✅ | - | ✅ | Yes (Rust only) |
| metrics.yml | - | - | ✅ (nightly) | ✅ | No |
| rust-autofix.yml | ✅ | - | - | - | Yes (Rust only) |

---

## Reducing CI Noise

### Before this consolidation:
- ❌ 8-10 workflows running on every PR
- ❌ Duplicate builds and tests
- ❌ Confusing overlapping checks
- ❌ Metrics running per-push

### After this consolidation:
- ✅ 2-3 workflows per PR (depending on what changed)
- ✅ 1 main CI workflow (blocking)
- ✅ 1 API validation (only when needed)
- ✅ Quality gates advisory (non-blocking until clean)
- ✅ Metrics nightly (not per-push)

---

## Development Workflow

### Typical PR Flow:

1. **PR opened** → Triggers:
   - `ci.yml` (if non-API code changed)
   - `api-validation.yml` (if API changed)
   - `docker-build.yml` (build only, no push)
   - `quality-gates.yml` (advisory)
   - `rust-autofix.yml` (creates autofix PR if needed)

2. **Required for merge:**
   - ✅ CI / Final Validation: success
   - ✅ API Validation / validation-complete: success (if API changed)

3. **Merged to main** → Triggers:
   - All above workflows
   - Docker images pushed to registry
   - Benchmarks run
   - Metrics collected (if scheduled)

4. **Nightly** → Triggers:
   - `metrics.yml` runs and commits performance data

---

## Troubleshooting

### "Too many checks running"
- Check path filters - make sure changes aren't triggering unnecessary workflows
- Docs-only changes should skip CI entirely

### "Build failed but locally it works"
- Check Rust toolchain version (CI uses `stable`)
- Check `RUSTFLAGS` - CI sets `-Dwarnings`
- Check WASM target: `rustup target add wasm32-wasip2`

### "Quality gates failing"
- These are **advisory** - they won't block your PR
- Fix violations gradually to move toward blocking status

### "Metrics workflow not running"
- It's scheduled for 2 AM UTC daily
- Run manually via Actions tab → "CI/CD Metrics Collection" → "Run workflow"

---

## Making Quality Gates Blocking

Once violations are cleaned up:

1. Remove `continue-on-error: true` from jobs in `quality-gates.yml`
2. Add `Quality Summary` to required status checks
3. Update this doc

---

## Architecture Notes

### Why separate workflows?

- **ci.yml**: Fast feedback for core functionality
- **api-validation.yml**: Heavy validation only when API changes
- **docker-build.yml**: Separate build+push logic from CI
- **quality-gates.yml**: Ratchet approach - advisory first, blocking later
- **metrics.yml**: Historical tracking without slowing PRs

### Why path filters?

- Avoid running full CI on docs-only changes
- Avoid running API validation when API didn't change
- Reduce CI time by 30-50% on average

### Why scheduled metrics?

- Metrics are observability, not gating
- Running nightly avoids CI noise
- Still available for troubleshooting via manual trigger

---

## Future Improvements

- [ ] Add cross-platform testing to quality-gates once advisory issues are clean
- [ ] Add WASM-specific benchmarks
- [ ] Add release automation workflow
- [ ] Add deployment workflows (staging, production)
- [ ] Add performance regression detection

---

## Contact

For questions or improvements, see:
- GitHub Issues: https://github.com/YOUR_ORG/eventmesh/issues
- CLAUDE.md for development setup
