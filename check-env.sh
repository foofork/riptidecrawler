#!/bin/bash
# Quick environment verification script
# Run: ./check-env.sh

echo "=== User Check ==="
whoami  # Should be: codespace

echo -e "\n=== Environment Variables ==="
env | grep -E "(RUSTUP|CARGO|TMPDIR)" | sort
echo "Expected:"
echo "  CARGO_HOME=/home/codespace/.cargo"
echo "  RUSTUP_HOME=/home/codespace/.rustup"
echo "  RUSTUP_PERMIT_COPY_RENAME=1"
echo "  TMPDIR=/home/codespace/.rustup/tmp"

echo -e "\n=== PATH Check ==="
echo "First 3 PATH entries:"
echo $PATH | tr ':' '\n' | head -3
echo "Expected first: /home/codespace/.cargo/bin"

echo -e "\n=== Which Cargo ==="
which cargo
echo "Expected: /home/codespace/.cargo/bin/cargo"

echo -e "\n=== Cargo Symlink ==="
ls -la $(which cargo) 2>/dev/null || echo "Not a symlink (unexpected)"
echo "Expected: symlink to rustup"

echo -e "\n=== Rustup Which Cargo ==="
rustup which cargo
echo "Expected: /home/codespace/.rustup/toolchains/.../bin/cargo"

echo -e "\n=== Filesystem Check ==="
echo "TMPDIR filesystem:"
df -h ~/.rustup/tmp | tail -1
echo "RUSTUP_HOME filesystem:"
df -h ~/.rustup | tail -1
echo "Expected: Both on 'overlay' filesystem (same mount)"

echo -e "\n=== Versions ==="
cargo --version
rustc --version
rustup --version

echo -e "\n=== Bad PATH Check ==="
if echo $PATH | grep -q "rustup/toolchains"; then
    echo "⚠️  WARNING: Found hard-coded toolchain path in PATH!"
    echo "   This bypasses rustup management. Remove it from ~/.bashrc"
else
    echo "✅ No hard-coded toolchain paths found"
fi

echo -e "\n=== Summary ==="
if [ "$(whoami)" = "codespace" ] && \
   [ -n "$RUSTUP_HOME" ] && \
   [ -n "$CARGO_HOME" ] && \
   [ -n "$TMPDIR" ] && \
   [ "$(which cargo)" = "/home/codespace/.cargo/bin/cargo" ]; then
    echo "✅ Environment is correctly configured!"
else
    echo "⚠️  Some checks failed. Review output above."
    echo "   Try: source ~/.bashrc"
fi
