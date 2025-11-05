# Rustup Environment Fix - Cross-Device Link Error

**Issue:** `Invalid cross-device link (os error 18)` when using cargo/rustup
**Root Cause:** Rustup cannot atomically rename files between toolchain and tmp directories
**Impact:** Prevents cargo commands from running via rustup proxy

---

## ✅ SOLUTION: Use Cargo Directly

The Rust toolchain is installed and functional, but rustup's proxy mechanism is broken. Use cargo directly:

### Option 1: Direct Binary Path (Immediate)

```bash
# Use full path to cargo
/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo check
/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo clippy
/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo test
```

### Option 2: Add to PATH (Recommended)

```bash
# Add to ~/.bashrc or current session
export PATH="/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"

# Now cargo works normally
cargo check
cargo clippy
cargo test
```

### Option 3: Create Alias (Quick Fix)

```bash
# Add to ~/.bashrc
alias cargo='/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo'
alias rustc='/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc'
alias clippy-driver='/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/clippy-driver'

# Apply immediately
source ~/.bashrc
```

### Option 4: Fix Rustup (Advanced)

```bash
# 1. Set rustup home to avoid tmp issues
export RUSTUP_HOME=/tmp/rustup-home
export CARGO_HOME=/tmp/cargo-home

# 2. Reinstall rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path

# 3. Install toolchain
rustup toolchain install stable
```

---

## 🎯 Verification Commands

Once fixed, verify with:

```bash
# 1. Check cargo works
cargo --version

# 2. Test compilation
cargo check -p riptide-api --features full

# 3. Run clippy
cargo clippy -p riptide-api --all-features -- -D warnings

# 4. Run tests
cargo test -p riptide-spider

# 5. Test all feature combinations
cargo check -p riptide-api --no-default-features
cargo check -p riptide-api --features browser
cargo check -p riptide-api --features llm
cargo check -p riptide-api --features full
```

---

## 📊 Current Status

- ✅ Rust toolchain installed (cargo 1.90.0, rustc 1.90.0)
- ✅ All binaries present and functional
- ❌ Rustup proxy broken (cross-device link error)
- ✅ Direct binary access works

**Recommendation:** Use **Option 2** (PATH) for permanent fix.

---

## 🔍 Technical Details

### Error Analysis

```
error: could not rename component file from
'/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/share/zsh/site-functions'
to '/root/.rustup/tmp/...'

Caused by: Invalid cross-device link (os error 18)
```

**Why it happens:**
- Rustup tries to atomically rename files during toolchain updates
- Linux `rename()` syscall fails across filesystem boundaries
- Even though both paths show same filesystem in `df`, overlayfs or container mounts cause the issue

**Why direct cargo works:**
- Bypasses rustup's toolchain management
- Executes cargo binary directly without proxy layer
- No file operations across filesystem boundaries

---

## 🚀 Quick Start (Copy-Paste)

```bash
# Immediate fix - run this now:
export PATH="/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"

# Verify it works:
cargo --version

# Test RipTide compilation:
cd /home/user/riptidecrawler
cargo check -p riptide-api --features full
```

---

## 📝 Next Steps After Fix

1. ✅ Run full feature combination matrix
2. ✅ Execute clippy with `-D warnings`
3. ✅ Run test suite
4. ✅ Update completion report with verification results
5. ✅ Commit final verification status

---

**Created:** 2025-11-05
**Issue:** rustup cross-device link error
**Status:** Workaround documented and tested
