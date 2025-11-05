# 🔧 Environment Setup Instructions

**Status:** ✅ Correctly Configured (2025-11-05)
**User:** codespace (NOT root)
**Shell:** bash

---

## ✅ The CORRECT Setup (Already Applied)

Your environment is **already configured correctly** in `~/.bashrc`:

```bash
# Rustup/Cargo Configuration (EXDEV fix)
export RUSTUP_HOME="$HOME/.rustup"
export CARGO_HOME="$HOME/.cargo"
mkdir -p "$RUSTUP_HOME/tmp"
export TMPDIR="$RUSTUP_HOME/tmp"
export RUSTUP_PERMIT_COPY_RENAME=1
export PATH="$HOME/.cargo/bin:$PATH"
```

---

## 🚀 How to Use This Setup

### In Every New Terminal Session

The environment is **automatically configured** when you open a new terminal because it's in `~/.bashrc`.

To verify it's active:

```bash
# Check environment variables
env | grep -E "(RUSTUP|CARGO|TMPDIR)"

# You should see:
# RUSTUP_HOME=/home/codespace/.rustup
# CARGO_HOME=/home/codespace/.cargo
# TMPDIR=/home/codespace/.rustup/tmp
# RUSTUP_PERMIT_COPY_RENAME=1
```

### In Current Terminal Session

If you modified `.bashrc` and need to reload:

```bash
source ~/.bashrc
```

### For Background Processes / Scripts

If you're running scripts that start new shells, they will automatically inherit the correct environment from `.bashrc`.

For explicit control in a script:

```bash
#!/bin/bash
# Your script will automatically source .bashrc for interactive shells
# For non-interactive, add this at the top:
source "$HOME/.bashrc"

# Then run cargo commands
cargo build
cargo test
```

---

## ⚠️ What NOT to Do

### ❌ WRONG: Hard-code Toolchain Path

**Never do this:**
```bash
# BAD - bypasses rustup management
export PATH="/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"
```

**Why it's wrong:**
- Hard-codes architecture (`x86_64-unknown-linux-gnu`)
- Hard-codes toolchain version (breaks when rustup updates)
- Assumes `/root` (but you're user `codespace`)
- Bypasses rustup's toolchain management
- Breaks `rustup override`, `rustfmt`, component installs

### ❌ WRONG: Modify /root/.bashrc

**You are user `codespace`, not `root`.**

```bash
whoami
# Output: codespace (NOT root)

# Your config is in: ~/.bashrc
# Which is: /home/codespace/.bashrc
# NOT: /root/.bashrc
```

---

## 🔍 Verify Correct Setup

Run this complete verification:

```bash
#!/bin/bash
echo "=== User Check ==="
whoami  # Should be: codespace

echo -e "\n=== Environment Variables ==="
env | grep -E "(RUSTUP|CARGO|TMPDIR)"
# Should show:
#   RUSTUP_HOME=/home/codespace/.rustup
#   CARGO_HOME=/home/codespace/.cargo
#   TMPDIR=/home/codespace/.rustup/tmp
#   RUSTUP_PERMIT_COPY_RENAME=1

echo -e "\n=== PATH Check ==="
echo $PATH | tr ':' '\n' | head -5
# First entry should be: /home/codespace/.cargo/bin

echo -e "\n=== Which Cargo ==="
which cargo
# Should be: /home/codespace/.cargo/bin/cargo

ls -la $(which cargo)
# Should show: symlink to rustup

echo -e "\n=== Rustup Which Cargo ==="
rustup which cargo
# Should show: /home/codespace/.rustup/toolchains/.../bin/cargo

echo -e "\n=== Filesystem Check ==="
df -h ~/.rustup/tmp | tail -1 | awk '{print "Filesystem: " $1 ", Mounted on: " $6}'
# Should show: overlay filesystem, mounted on /

echo -e "\n=== Versions ==="
cargo --version
rustc --version
rustup --version

echo -e "\n✅ If all checks pass, your environment is correctly configured!"
```

Save this as `check-env.sh` and run:

```bash
chmod +x check-env.sh
./check-env.sh
```

---

## 📝 What to Tell AI Assistants (Claude Web, etc.)

When working with AI assistants on this project, tell them:

```
IMPORTANT: Environment Setup Rules

1. User is 'codespace' (NOT root)
2. Rust environment is ALREADY configured in ~/.bashrc
3. DO NOT modify PATH to point to toolchain binaries directly
4. DO NOT hard-code /root/.rustup/... paths
5. DO NOT try to "fix" rustup - it's already working correctly

The correct environment is:
- RUSTUP_HOME=$HOME/.rustup
- CARGO_HOME=$HOME/.cargo
- TMPDIR=$HOME/.rustup/tmp
- PATH includes ~/.cargo/bin (rustup shims)

To reload environment: source ~/.bashrc
To verify setup: run ./check-env.sh

If cargo/rustup commands fail, check the actual ERROR MESSAGE first.
Most issues are compilation errors, not environment problems.
```

---

## 🐛 Troubleshooting

### Issue: "Command not found: cargo"

```bash
# Check PATH
echo $PATH | grep cargo

# If not there, reload bashrc
source ~/.bashrc

# Verify
which cargo
```

### Issue: "EXDEV error" from rustup

```bash
# Check TMPDIR is set correctly
echo $TMPDIR
# Should be: /home/codespace/.rustup/tmp

# Check it's on same filesystem
df -h ~/.rustup/tmp
df -h ~/.rustup
# Should both show same filesystem (overlay)

# If not, reload bashrc
source ~/.bashrc
```

### Issue: AI tried to use wrong PATH

```bash
# Check for bad PATH in bashrc
grep "rustup/toolchains" ~/.bashrc

# If found, remove those lines
sed -i '/rustup\/toolchains/d' ~/.bashrc

# Reload
source ~/.bashrc

# Verify clean PATH
which cargo
# Should be: /home/codespace/.cargo/bin/cargo (NOT .../toolchains/...)
```

### Issue: Cargo check fails with "unresolved import"

**This is NOT an environment issue!**

This is a compilation error. Common causes:
- Missing feature flags (e.g., `#[cfg(feature = "jemalloc")]`)
- Missing dependencies
- Conditional compilation issues

Check the actual error message and fix the code, not the environment.

---

## 🔄 For New Codespace / Container Rebuild

If you rebuild the container or create a new Codespace, the `.bashrc` changes will be lost.

To make it permanent, add to `.devcontainer/devcontainer.json`:

```json
{
  "containerEnv": {
    "RUSTUP_HOME": "/home/codespace/.rustup",
    "CARGO_HOME": "/home/codespace/.cargo",
    "TMPDIR": "/home/codespace/.rustup/tmp",
    "RUSTUP_PERMIT_COPY_RENAME": "1"
  },
  "postCreateCommand": "mkdir -p $RUSTUP_HOME/tmp && rustup default stable"
}
```

See: `.devcontainer/devcontainer.json.example` for full template.

---

## 📊 Quick Reference

| Command | Expected Output |
|---------|-----------------|
| `whoami` | `codespace` |
| `which cargo` | `/home/codespace/.cargo/bin/cargo` |
| `rustup which cargo` | `/home/codespace/.rustup/toolchains/.../bin/cargo` |
| `echo $TMPDIR` | `/home/codespace/.rustup/tmp` |
| `echo $RUSTUP_HOME` | `/home/codespace/.rustup` |
| `df -h $TMPDIR` | `overlay` filesystem |

---

## ✅ Summary

**DO:**
- ✅ Use `source ~/.bashrc` in new sessions
- ✅ Verify environment with the check script
- ✅ Use rustup commands normally (update, component add, etc.)
- ✅ Run cargo commands normally (build, test, check, clippy)
- ✅ Trust the existing setup - it's correct!

**DON'T:**
- ❌ Hard-code toolchain paths in PATH
- ❌ Modify /root/.bashrc (you're not root)
- ❌ Try to "fix" rustup by bypassing it
- ❌ Assume environment issues for compilation errors
- ❌ Listen to AI suggestions that modify toolchain paths

---

**Last Updated:** 2025-11-05
**Verified Working:** ✅ Yes
**Current User:** codespace
**Current Shell:** bash
