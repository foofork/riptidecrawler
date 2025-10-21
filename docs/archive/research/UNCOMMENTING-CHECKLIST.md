# Spider-Chrome Migration: Uncommenting Checklist

**Quick Reference for Re-enabling Commented Code**

---

## 🔴 HIGH PRIORITY (Block Phase 2)

### 1. Browser Pool Manager ✋ READY TO UNCOMMENT
**File**: `/workspaces/eventmesh/crates/riptide-cli/src/commands/mod.rs`
**Line**: 29
**Action**:
```rust
// UNCOMMENT THIS:
pub mod browser_pool_manager;
```

**Prerequisites**:
- ✅ File exists and complete (200+ lines)
- ✅ Uses spider_chrome natively
- ⚠️ Verify spider_chrome API stability
- ⚠️ Test health checks work

**Testing**:
```bash
cargo build --package riptide-cli --features browser-pool
cargo test --package riptide-cli browser_pool_manager
```

**Estimated Effort**: 2-4 hours (testing + validation)

---

### 2. Optimized Executor ✋ READY TO UNCOMMENT
**File**: `/workspaces/eventmesh/crates/riptide-cli/src/commands/mod.rs`
**Line**: 34
**Action**:
```rust
// UNCOMMENT THIS:
pub mod optimized_executor;
```

**Prerequisites**:
- ⚠️ Depends on browser_pool_manager being enabled FIRST
- ✅ File exists and complete (200+ lines)
- ✅ Integrates all optimization modules

**Testing**:
```bash
cargo build --package riptide-cli --features optimizations
cargo test --package riptide-cli optimized_executor
```

**Estimated Effort**: 1-2 hours (after browser_pool_manager works)

---

### 3. Spider Health Check 📝 NEEDS IMPLEMENTATION
**File**: `/workspaces/eventmesh/crates/riptide-api/src/health.rs`
**Lines**: 179-189
**Action**: Replace `None` with actual implementation

**Implementation Plan** (already documented in code):
```rust
// Current:
spider_engine: None,  // Line 178

// Replace with:
spider_engine: Some(self.check_spider_health().await),
```

**Implementation Steps**:
1. Check spider engine initialization status
2. Test crawl queue connectivity
3. Verify spider worker pool health
4. Return status with response time metrics

**Blocker**: Spider engine must be initialized in AppState

**Estimated Effort**: 4-6 hours

---

## 🟡 MEDIUM PRIORITY (Phase 2 Re-enable)

### 4. CDP Module Re-enable 📦 PHASE 2
**Files**:
- `/workspaces/eventmesh/crates/riptide-headless/src/lib.rs:65`
- `/workspaces/eventmesh/crates/riptide-engine/src/lib.rs:64`

**Action**:
```rust
// UNCOMMENT THIS:
pub mod cdp;
```

**Prerequisites**:
- ⚠️ Resolve chromiumoxide version conflicts
- ⚠️ Test CDP HTTP API endpoints
- ⚠️ Verify stealth integration

**Testing**:
```bash
cargo build --package riptide-headless --features cdp
cargo test --package riptide-headless cdp
```

**Estimated Effort**: 1 day

---

### 5. Headless Service Main 🚀 PHASE 2
**File**: `/workspaces/eventmesh/crates/riptide-headless/src/main.rs.disabled`
**Action**: Rename to `main.rs` (remove `.disabled`)

**Prerequisites**:
- ✅ CDP module must be enabled FIRST
- ✅ File complete and ready (100+ lines)
- ⚠️ Test standalone server startup

**Testing**:
```bash
cargo run --package riptide-headless
curl http://localhost:9123/healthz
curl -X POST http://localhost:9123/render -d '{"url":"https://example.com"}'
```

**Estimated Effort**: 2-4 hours

---

### 6. Screenshot Functionality 📸 TYPE FIXES
**File**: `/workspaces/eventmesh/crates/riptide-cli/src/commands/render.rs`
**Line**: 687-689

**Action**: Remove warning and implement screenshot

**Current**:
```rust
// TODO: Re-implement with proper chromiumoxide type access
output::print_warning("Screenshot functionality temporarily disabled - type visibility issues");
```

**Solution**: Use spider-chrome native types (already done in browser_abstraction)

**Reference**: See `/workspaces/eventmesh/crates/riptide-browser-abstraction/src/spider_impl.rs:161-195`

**Estimated Effort**: 2-3 hours

---

### 7. PDF Functionality 📄 TYPE FIXES
**File**: `/workspaces/eventmesh/crates/riptide-cli/src/commands/render.rs`
**Line**: 775-776

**Action**: Remove warning and implement PDF

**Solution**: Use spider-chrome native types (already done in browser_abstraction)

**Reference**: See `/workspaces/eventmesh/crates/riptide-browser-abstraction/src/spider_impl.rs:200-235`

**Estimated Effort**: 2-3 hours

---

### 8. Headless Tests 🧪 IMPORT FIXES
**File**: `/workspaces/eventmesh/crates/riptide-headless/tests/headless_tests.rs.disabled`
**Action**:
1. Rename to `headless_tests.rs` (remove `.disabled`)
2. Fix spider_chrome import on line 7

**Current**:
```rust
// use spider_chrome::BrowserConfig;
```

**Fix**:
```rust
use spider_chrome::BrowserConfig;
```

**Testing**:
```bash
cargo test --package riptide-headless
```

**Estimated Effort**: 1 hour

---

## 🟢 LOW PRIORITY (Post-Migration)

### 9. Facade Integration Test
**File**: `/workspaces/eventmesh/crates/riptide-facade/tests/facade_composition_integration.rs:105`
**Action**: Implement when facades ready
**Estimated Effort**: 2-4 hours

---

### 10. Extraction Modules Refactor
**Files**:
- `/workspaces/eventmesh/crates/riptide-extraction/src/lib.rs` (lines 38, 41, 49)
- `/workspaces/eventmesh/crates/riptide-extraction/src/strategies/mod.rs` (lines 19, 25)

**Action**: Architectural refactoring (not migration-blocked)
**Estimated Effort**: 1-2 days

---

## ⚙️ Configuration Changes

### Enable SPIDER_ENABLE Flag
**File**: Environment variables
**Action**: Set runtime flag
```bash
export SPIDER_ENABLE=true
```

**Location in Code**: `/workspaces/eventmesh/crates/riptide-api/src/state.rs:321`

---

### Enable headless Feature Flag
**File**: Cargo.toml (crate-level)
**Action**: Add to default features or build command
```bash
cargo build --features headless
cargo test --features headless
```

---

## 📊 Progress Tracking

### Phase 2A (Week 6-7)
- [ ] Browser Pool Manager uncommented
- [ ] Optimized Executor uncommented
- [ ] Spider Health Check implemented
- [ ] Integration tests passing

### Phase 2B (Week 8-9)
- [ ] CDP modules re-enabled
- [ ] Screenshot functionality fixed
- [ ] PDF functionality fixed
- [ ] Headless tests re-enabled
- [ ] Headless service running

### Phase 3 (Post-Migration)
- [ ] Facade integration complete
- [ ] Extraction modules refactored
- [ ] Documentation updated

---

## 🧪 Testing Commands

### After Each Uncomment
```bash
# Full build with features
cargo build --workspace --all-features

# Run specific crate tests
cargo test --package riptide-cli
cargo test --package riptide-headless
cargo test --package riptide-api

# Integration tests
cargo test --test spider_chrome_tests --features headless
cargo test --test spider_chrome_benchmarks --features headless

# Clippy and format
cargo clippy --workspace --all-features
cargo fmt --all
```

---

## ⚠️ Known Issues

### 1. Type Visibility
**Issue**: CDP types not visible in some contexts
**Workaround**: Use spider-chrome native types from `chromiumoxide_cdp`
**Status**: Resolved in browser_abstraction layer

### 2. Ownership Issues
**Issue**: `close()` requires ownership, Arc prevents calling
**Workaround**: Rely on Arc drop for cleanup
**Status**: Documented limitation, acceptable

### 3. Version Conflicts
**Issue**: chromiumoxide version conflicts in Phase 1
**Resolution**: Phase 2 dependency resolution
**Status**: Planned work

---

## 📝 Quick Reference: Files to Modify

| Priority | File | Line(s) | Action | Effort |
|----------|------|---------|--------|--------|
| HIGH | `cli/commands/mod.rs` | 29 | Uncomment module | 2-4h |
| HIGH | `cli/commands/mod.rs` | 34 | Uncomment module | 1-2h |
| HIGH | `api/health.rs` | 178-189 | Implement check | 4-6h |
| MEDIUM | `headless/lib.rs` | 65 | Uncomment module | 1d |
| MEDIUM | `engine/lib.rs` | 64 | Uncomment module | 1d |
| MEDIUM | `cli/commands/render.rs` | 687-689 | Fix screenshot | 2-3h |
| MEDIUM | `cli/commands/render.rs` | 775-776 | Fix PDF | 2-3h |
| MEDIUM | `headless/tests/*.disabled` | - | Rename files | 1h |

**Total Estimated Effort**:
- Phase 2A: 7-12 hours
- Phase 2B: 2-3 days
- Phase 3: 2-3 days

---

## 🎯 Success Criteria

### Browser Pool Manager
- [x] File compiles without errors
- [ ] Health checks run every 30s
- [ ] Auto-restart on failure works
- [ ] Resource monitoring accurate
- [ ] No memory leaks

### Optimized Executor
- [x] File compiles without errors
- [ ] All optimization modules integrate
- [ ] Cache hits improve performance
- [ ] Timeouts adapt correctly
- [ ] Benchmarks show improvement

### Spider Health Check
- [ ] Returns status in <100ms
- [ ] Detects connection failures
- [ ] Reports worker pool health
- [ ] Integrates with health endpoint
- [ ] Alerts on degradation

---

**Last Updated**: 2025-10-20
**Status**: Phase 1 Complete (35%)
**Next Action**: Uncomment browser_pool_manager (HIGH PRIORITY)
