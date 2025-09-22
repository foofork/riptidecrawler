# Fixing "syncing channel updates for 'stable-x86_64-unknown-linux-gnu'" Error

## Quick Diagnosis

The "syncing channel updates" line is just rustup's **info message** - the real error appears right after it. Common causes: network issues, corrupted cache, proxy/TLS problems, or mixed user/root installs.

## Quick Fixes (Try in Order)

### 1. Update rustup itself & retry
```bash
rustup self update
rustup update stable
```

### 2. Clear corrupted cache/downloads
```bash
rm -rf ~/.rustup/tmp/* ~/.rustup/downloads/*
rustup update stable
```

### 3. Force reinstall stable toolchain
```bash
rustup toolchain remove stable-x86_64-unknown-linux-gnu
rustup toolchain install stable
```

### 4. Check user vs root permissions
Ensure you're not mixing user and root installs:
```bash
# Check ownership
ls -la ~/.rustup ~/.cargo
# Should all be owned by your user, not root
```

### 5. Corporate proxy/firewall setup
```bash
# Set proxy if needed
export HTTPS_PROXY=http://user:pass@proxy.company:3128
export HTTP_PROXY=http://user:pass@proxy.company:3128

# Corporate TLS certificate
export CURL_CA_BUNDLE=/path/to/corporate-ca-bundle.pem
rustup update stable
```

### 6. Try different CDN/mirror
```bash
export RUSTUP_DIST_SERVER=https://static.rust-lang.org
rustup update stable
```

### 7. Force IPv4 (DNS/IPv6 issues)
```bash
RUSTUP_USE_CURL=1 rustup update stable
```

### 8. WSL clock sync (WSL users only)
```powershell
# Windows (run as admin)
w32tm /resync
```
```bash
# Back in WSL
sudo hwclock -s || true
```

## Get Detailed Error Information

```bash
# Verbose logging to see the actual error
RUST_LOG=rustup=debug rustup -v update stable

# Or alternative verbose mode
RUSTUP_LOG=debug rustup update stable
```

## Verify Your Setup

```bash
# Check current configuration
rustup show
rustup show home

# Verify target platform
rustup target list --installed
```

## RipTide Project Specific

For this project, ensure you have the required targets:
```bash
rustup target add wasm32-wasip2
rustup target add wasm32-wasi  # fallback compatibility
```

## Still Having Issues?

1. **Check the line AFTER "syncing channel updates"** - that's the real error
2. **Common error patterns:**
   - `curl: (60) SSL certificate problem` → TLS/certificate issue
   - `curl: (7) Failed to connect` → Network/firewall issue
   - `curl: (28) Operation timed out` → Network timeout
   - `curl: (56) Recv failure` → Connection interrupted

3. **Last resort - complete reinstall:**
```bash
# Backup your config first
cp -r ~/.cargo/config* /tmp/cargo-backup/ 2>/dev/null || true

# Remove everything
rustup self uninstall

# Fresh install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Restore targets
rustup target add wasm32-wasip2
```

## Prevention

- Don't mix `sudo` and user rustup commands
- Keep rustup updated: `rustup self update` monthly
- In corporate environments, ensure proxy/TLS settings are permanent in your shell profile

---

**For RipTide Development**: Once rustup is working, run our build automation:
```bash
just build    # Builds entire workspace including WASM
just test     # Runs test suite
just ci       # Full CI simulation
```