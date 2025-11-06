# Rustup Environment Fix - Cross-Device Link Error

**Issue:** `Invalid cross-device link (os error 18)` when using cargo/rustup
**Root Cause:** Rustup cannot atomically rename files between toolchain and tmp directories
**Impact:** Prevents cargo commands from running via rustup proxy

---

## ✅ SOLUTION: Use Cargo Directly

The Rust toolchain is installed and functional, but rustup's proxy mechanism is broken. Use cargo directly:

### Recommended Fix: Add to PATH

```bash
# Add to ~/.bashrc or current session
export PATH="/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"

# Verify it works
cargo --version

# Test RipTide compilation
cargo check -p riptide-api --features full
```

### Alternative Solutions

<details>
<summary>Option 1: Direct Binary Path (Quick Test)</summary>

```bash
/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo check
```
</details>

<details>
<summary>Option 2: Create Alias</summary>

```bash
alias cargo='/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo'
alias rustc='/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc'
source ~/.bashrc
```
</details>

<details>
<summary>Option 3: Reinstall Rustup (Advanced)</summary>

```bash
export RUSTUP_HOME=/tmp/rustup-home
export CARGO_HOME=/tmp/cargo-home
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
rustup toolchain install stable
```
</details>

---

## 🎯 Verification

Once fixed, verify with:

```bash
# Basic verification
cargo --version
cargo check -p riptide-api --features full

# Comprehensive testing
cargo clippy -p riptide-api --all-features -- -D warnings
cargo test -p riptide-spider

# Feature matrix (run separately)
cargo check -p riptide-api --no-default-features
cargo check -p riptide-api --features browser
cargo check -p riptide-api --features llm
```

---

## 📊 Status

**Current State:**
- ✅ Rust toolchain installed (cargo 1.90.0, rustc 1.90.0)
- ✅ All binaries present and functional
- ✅ Direct binary access works
- ❌ Rustup proxy broken (cross-device link error)

**Next Steps:**
1. Run full feature combination matrix
2. Execute clippy with `-D warnings`
3. Run test suite
4. Update completion report

---

## 🔍 Technical Details

### Why It Fails

```
error: could not rename component file from
'/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/share/zsh/site-functions'
to '/root/.rustup/tmp/...'

Caused by: Invalid cross-device link (os error 18)
```

**Explanation:**
- Rustup tries to atomically rename files during toolchain operations
- Linux `rename()` syscall fails across filesystem boundaries
- Overlayfs or container mounts cause the issue despite same filesystem in `df`

**Why Direct Cargo Works:**
- Bypasses rustup's toolchain management layer
- Executes cargo binary directly without proxy
- No file operations across filesystem boundaries

---

**Created:** 2025-11-05
**Issue:** rustup cross-device link error
**Status:** Workaround documented and tested
