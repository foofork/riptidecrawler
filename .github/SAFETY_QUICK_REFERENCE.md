# Safety Audit Quick Reference

## 🚦 CI Status Check: "Safety Audit"

Your PR must pass all safety checks before merge. Here's how to fix common issues:

---

## ❌ "Undocumented unsafe blocks found"

**Fix**: Add `// SAFETY:` comment above the unsafe block

```rust
// BAD - No documentation
let value = unsafe { ptr.read() };

// GOOD - Documented
// SAFETY: ptr is valid and properly aligned because we just allocated
// it using Box::new, and the allocation succeeded.
let value = unsafe { ptr.read() };
```

**Test locally**:
```bash
.github/workflows/scripts/check-unsafe.sh
```

---

## ❌ "error: used unwrap() on a Result value"

**Fix**: Use proper error handling instead of `.unwrap()` or `.expect()`

```rust
// BAD - Can panic
let value = map.get(&key).unwrap();

// GOOD - Return error
let value = map.get(&key).ok_or(Error::KeyNotFound)?;

// GOOD - Provide default
let value = map.get(&key).unwrap_or(&default);

// GOOD - Pattern match
if let Some(value) = map.get(&key) {
    // handle value
}
```

**Test locally**:
```bash
cargo clippy --lib --bins -- -D clippy::unwrap_used -D clippy::expect_used
```

---

## ⚠️ "Miri detected undefined behavior"

**Common causes**:
- Invalid pointer dereference
- Use-after-free
- Uninitialized memory access
- Data race

**Fix**: Review the unsafe code in question and ensure memory safety invariants

**Test locally**:
```bash
rustup toolchain install nightly
cargo +nightly miri setup
cargo +nightly miri test -p riptide-core memory_manager
```

---

## ❌ "Missing WASM safety documentation"

**Fix**: Add required comment to `bindings.rs`

```rust
// SAFETY: Required for WASM component model FFI
```

**Test locally**:
```bash
.github/workflows/scripts/check-wasm-safety.sh
```

---

## 🚀 Quick Pre-Commit Checklist

Before pushing, run:
```bash
# 1. Check unsafe code (30 seconds)
.github/workflows/scripts/check-unsafe.sh

# 2. Check WASM safety (10 seconds)
.github/workflows/scripts/check-wasm-safety.sh

# 3. Run Clippy (2-3 minutes)
cargo clippy --workspace --all-features --lib --bins -- \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D warnings
```

---

## 📚 Full Documentation

See [docs/development/safety-audit.md](../docs/development/safety-audit.md) for complete guide.

---

## ⏱️ CI Times

- Unsafe audit: ~30s
- Clippy checks: ~2-3 min
- Miri tests: ~3-5 min
- WASM docs: ~10s
- **Total**: ~6-9 minutes

---

## 💡 Pro Tips

1. **Unwrap in tests is OK**: Test files can use `.unwrap()` and `.expect()`
2. **Document, don't avoid**: Unsafe code is fine when properly documented
3. **Run scripts first**: Local scripts are 10x faster than waiting for CI
4. **Cache helps**: Second+ CI runs are ~50% faster due to caching

---

## 🆘 Still Stuck?

1. Check detailed error in CI logs
2. Read [safety-audit.md](../docs/development/safety-audit.md)
3. Ask in PR comments
4. Tag maintainers for help
