# 🔧 Codespaces Rustup EXDEV Fix

**Issue:** Cross-device link errors when running `rustup update` or installing components
**Root Cause:** `/tmp` and `~/.rustup` are on different filesystems in Codespaces
**Status:** ✅ FIXED (2025-11-05)

---

## The Problem

In GitHub Codespaces (and similar container environments), the filesystem layout causes issues:

```
/home/codespace  → overlay filesystem (/)
/tmp             → /dev/sdb1 (separate mount)
```

When `rustup` tries to install or update components, it:
1. Downloads files to `/tmp`
2. Attempts atomic rename to `~/.rustup/...`
3. **Fails** with EXDEV error (can't rename across filesystems)

## ❌ WRONG Fix (Don't Do This)

```bash
# This hard-codes a toolchain path and breaks rustup management
export PATH="/root/.rustup/toolchains/1.90.0-x86_64-unknown-linux-gnu/bin:$PATH"
```

**Why this is bad:**
- Bypasses rustup's toolchain management
- Breaks `rustup update`, `rustfmt`, per-directory overrides
- Hard-codes arch/version (not portable)
- Assumes root user (Codespaces uses 'codespace' user)

## ✅ CORRECT Fix (Already Applied)

### What We Did

Added to `~/.bashrc`:
```bash
# Rustup/Cargo Configuration (EXDEV fix)
export RUSTUP_HOME="$HOME/.rustup"
export CARGO_HOME="$HOME/.cargo"
mkdir -p "$RUSTUP_HOME/tmp"
export TMPDIR="$RUSTUP_HOME/tmp"
export RUSTUP_PERMIT_COPY_RENAME=1
export PATH="$HOME/.cargo/bin:$PATH"
```

### Why This Works

1. **Same filesystem**: `TMPDIR` now points to `~/.rustup/tmp` (same mount as `~/.rustup`)
2. **Atomic renames work**: Files can be renamed within the overlay filesystem
3. **Fallback enabled**: `RUSTUP_PERMIT_COPY_RENAME=1` allows copy+rename if needed
4. **Uses rustup shims**: PATH points to `~/.cargo/bin`, not direct toolchain bins
5. **Portable**: Works for any user, any architecture

### Verification

```bash
# Check environment
bash -c 'source ~/.bashrc && env | grep -E "(RUSTUP|CARGO|TMPDIR)"'

# Verify same filesystem
df -h ~/.rustup/tmp
# Should show: overlay (same as /)

# Test rustup commands
rustup update
rustup component add rustfmt
cargo --version
```

## For Future Codespaces

If you rebuild the container or use a new Codespace, add this to `.devcontainer/devcontainer.json`:

```json
{
  "containerEnv": {
    "RUSTUP_HOME": "/home/codespace/.rustup",
    "CARGO_HOME": "/home/codespace/.cargo",
    "TMPDIR": "/home/codespace/.rustup/tmp",
    "RUSTUP_PERMIT_COPY_RENAME": "1",
    "PATH": "/home/codespace/.cargo/bin:${containerEnv:PATH}"
  },
  "postCreateCommand": "mkdir -p $RUSTUP_HOME/tmp && rustup default stable"
}
```

See: `.devcontainer/devcontainer.json.example` for full configuration.

## Troubleshooting

### Issue: "rustup: command not found"
```bash
# Fix PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Issue: Still getting EXDEV errors
```bash
# Verify tmpdir is on same filesystem
df -h ~/.rustup/tmp
df -h ~/.rustup

# Should both show 'overlay' filesystem

# Clean up and retry
rm -rf ~/.rustup/tmp/*
rustup self update
```

### Issue: Cargo/rustc version mismatch
```bash
# Use rustup to manage versions (don't hard-code PATH)
rustup default stable
rustup update

# Verify shims are used
which cargo  # should be ~/.cargo/bin/cargo
rustup which cargo  # shows actual toolchain binary
```

## References

- [rustup Issue #988](https://github.com/rust-lang/rustup/issues/988) - EXDEV errors
- [rustup Issue #1229](https://github.com/rust-lang/rustup/issues/1229) - Cross-device links
- [Codespaces docs](https://docs.github.com/en/codespaces) - Container configuration

---

**Last Updated:** 2025-11-05
**Applied By:** Claude Code
**Status:** Active in current Codespace
