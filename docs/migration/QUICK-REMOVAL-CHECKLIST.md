# Quick Crate Removal Checklist

**Target:** `riptide-engine`, `riptide-headless-hybrid`
**Status:** ✅ READY FOR REMOVAL
**Date:** 2025-10-21

---

## ✅ Pre-Removal Verification (ALL COMPLETE)

- [x] Code migrated to `riptide-browser/src/hybrid/`
- [x] `cargo check --workspace` passes ✅
- [x] `cargo test --no-run` passes ✅
- [x] No imports of old crates in active code ✅
- [x] No dependencies on old crates in Cargo.toml ✅
- [x] Backup plan documented ✅

---

## 🚀 Removal Steps (Execute in Order)

### Step 1: Create Backup

```bash
mkdir -p /tmp/riptide-backup-$(date +%Y%m%d)
cp -r crates/riptide-engine /tmp/riptide-backup-$(date +%Y%m%d)/
cp -r crates/riptide-headless-hybrid /tmp/riptide-backup-$(date +%Y%m%d)/
cp Cargo.toml /tmp/riptide-backup-$(date +%Y%m%d)/Cargo.toml.bak
```

**Checkpoint:** ✅ Backup created

---

### Step 2: Remove Crate Directories

```bash
rm -rf crates/riptide-engine
rm -rf crates/riptide-headless-hybrid
```

**Checkpoint:** ✅ Old crates removed

---

### Step 3: Update Workspace Cargo.toml

**Option A: Manual Edit**

Edit `/workspaces/eventmesh/Cargo.toml` and remove these lines:
```toml
"crates/riptide-engine",
"crates/riptide-headless-hybrid",  # P1-C1: Spider-chrome integration
```

**Option B: Automated (Recommended)**

```bash
sed -i '/crates\/riptide-engine/d' Cargo.toml
sed -i '/crates\/riptide-headless-hybrid/d' Cargo.toml
```

**Checkpoint:** ✅ Workspace config updated

---

### Step 4: Verify Compilation

```bash
cargo clean
cargo check --workspace
```

**Expected:** Success (warnings are okay)

**Checkpoint:** ✅ Workspace compiles

---

### Step 5: Verify Tests Compile

```bash
cargo test --workspace --no-run
```

**Expected:** All tests compile successfully

**Checkpoint:** ✅ Tests compile

---

### Step 6: Check for References

```bash
rg "riptide-engine|riptide-headless-hybrid" --type rust --type toml
```

**Expected:** Only matches in docs/archive (no active code)

**Checkpoint:** ✅ No active references

---

## 🔄 Rollback (If Needed)

```bash
# Quick restore
cp -r /tmp/riptide-backup-$(date +%Y%m%d)/riptide-* crates/
cp /tmp/riptide-backup-$(date +%Y%m%d)/Cargo.toml.bak Cargo.toml
cargo check --workspace
```

---

## 📝 Post-Removal

### Git Commit

```bash
git add -A
git commit -m "refactor(phase3): Remove riptide-engine and riptide-headless-hybrid

- Consolidated into riptide-browser
- All functionality migrated
- Zero breaking changes

Phase 3 Task 4.4 COMPLETE
"
```

### Update Documentation

- [ ] Update COMPREHENSIVE-ROADMAP.md (mark Phase 3 complete)
- [ ] Archive old crate documentation
- [ ] Update architecture diagrams

---

## ✅ Success Confirmation

All steps complete when:

- ✅ `cargo check --workspace` succeeds
- ✅ `cargo test --no-run` succeeds
- ✅ No old crate directories exist
- ✅ Workspace Cargo.toml updated
- ✅ Git commit created

---

**Status:** 🟢 READY TO EXECUTE

**Estimated Time:** 10-15 minutes

**Risk Level:** 🟢 LOW (fully verified, rollback available)
