# Pdfium Library Setup Guide

## Problem Statement

The `riptide-pdf` crate uses `pdfium-render` for PDF processing, which requires the native Pdfium library (`libpdfium.so` on Linux) to be available at runtime. Without this library, PDF processing fails with the error:

```
Failed to initialize Pdfium: libpdfium.so not found
```

## Solutions

### Solution 1: Install Pre-built Pdfium Library (Recommended for Production)

This is the **recommended solution** for production environments as it provides the best performance and reliability.

#### Linux (Ubuntu/Debian)

```bash
# Download pre-built binaries from pdfium-binaries repository
mkdir -p /tmp/pdfium
cd /tmp/pdfium
curl -L "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7469/pdfium-linux-x64.tgz" -o pdfium-linux-x64.tgz
tar -xzf pdfium-linux-x64.tgz

# Install to system library directory
sudo mkdir -p /usr/local/lib
sudo cp lib/libpdfium.so /usr/local/lib/
sudo chmod 755 /usr/local/lib/libpdfium.so
sudo ldconfig

# Verify installation
ldconfig -p | grep pdfium
# Should output: libpdfium.so (libc6,x86-64) => /usr/local/lib/libpdfium.so
```

#### Runtime Configuration

After installation, ensure the library can be found at runtime:

```bash
# Add to your shell profile (~/.bashrc, ~/.zshrc, etc.)
export LD_LIBRARY_PATH=/usr/local/lib:${LD_LIBRARY_PATH}

# Or configure ldconfig (preferred for system-wide access)
echo "/usr/local/lib" | sudo tee /etc/ld.so.conf.d/pdfium.conf
sudo ldconfig
```

#### Docker Configuration

For Docker deployments, add to your Dockerfile:

```dockerfile
# Download and install pdfium
RUN mkdir -p /tmp/pdfium && \
    cd /tmp/pdfium && \
    curl -L "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7469/pdfium-linux-x64.tgz" -o pdfium.tgz && \
    tar -xzf pdfium.tgz && \
    cp lib/libpdfium.so /usr/local/lib/ && \
    chmod 755 /usr/local/lib/libpdfium.so && \
    ldconfig && \
    rm -rf /tmp/pdfium

# Alternative: Set LD_LIBRARY_PATH
ENV LD_LIBRARY_PATH=/usr/local/lib:${LD_LIBRARY_PATH}
```

### Solution 2: Static Linking (Alternative)

The `pdfium-render` crate supports static linking which bundles the library into the binary. This approach increases binary size but eliminates runtime dependencies.

#### Update Cargo.toml

```toml
# In workspace Cargo.toml
[workspace.dependencies]
pdfium-render = { version = "0.8", features = ["sync", "thread_safe", "static"] }
```

```toml
# In crates/riptide-pdf/Cargo.toml
[dependencies]
pdfium-render = { workspace = true, optional = true, features = ["static"] }
```

#### Pros and Cons

**Pros:**
- No runtime dependencies
- Portable binaries
- Easier deployment

**Cons:**
- Larger binary size (~5-10 MB increase)
- Longer compilation time
- May require additional build dependencies

### Solution 3: Feature Flag Control

For environments where Pdfium is not available, the code already has fallback handling via the `pdf` feature flag.

#### Build Without PDF Support

```bash
cargo build --no-default-features
```

#### Build With PDF Support

```bash
cargo build --features pdf
```

The code will gracefully handle missing Pdfium and provide informative error messages.

## Verification

### Test Library Installation

```bash
# Check if library is in system paths
ldconfig -p | grep pdfium

# Check if library file exists
ls -lh /usr/local/lib/libpdfium.so

# Test with a simple command
LD_LIBRARY_PATH=/usr/local/lib ldd /path/to/your/binary | grep pdfium
```

### Test with Cargo

```bash
# Set library path and run tests
export LD_LIBRARY_PATH=/usr/local/lib
cargo test --package riptide-pdf

# Or run specific tests
cargo test --package riptide-pdf --test pdf_extraction_tests
```

### Test API Endpoint

```bash
# Start the API server
export LD_LIBRARY_PATH=/usr/local/lib
cargo run --package riptide-api

# Test PDF processing endpoint
curl -X POST http://localhost:3000/pdf/process \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com/document.pdf"}'
```

## Platform-Specific Notes

### Linux
- Pre-built binaries available from bblanchon/pdfium-binaries
- Use ldconfig for system-wide configuration
- Recommended: Install to /usr/local/lib

### macOS
```bash
# Download macOS binary
curl -L "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7469/pdfium-darwin-x64.tgz" -o pdfium.tgz
tar -xzf pdfium.tgz
sudo cp lib/libpdfium.dylib /usr/local/lib/
export DYLD_LIBRARY_PATH=/usr/local/lib:${DYLD_LIBRARY_PATH}
```

### Windows
```powershell
# Download Windows binary
Invoke-WebRequest -Uri "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7469/pdfium-win-x64.tgz" -OutFile pdfium.tgz
tar -xzf pdfium.tgz
# Copy pdfium.dll to your PATH or application directory
Copy-Item lib\pdfium.dll C:\Windows\System32\
```

## Troubleshooting

### Error: libpdfium.so not found

**Cause:** Library not in system paths

**Solutions:**
1. Install library to /usr/local/lib and run `sudo ldconfig`
2. Set `LD_LIBRARY_PATH=/usr/local/lib`
3. Add /usr/local/lib to /etc/ld.so.conf.d/

### Error: version GLIBC_X.XX not found

**Cause:** Binary built for newer glibc version

**Solutions:**
1. Download an older pdfium-binaries release
2. Build pdfium from source for your system
3. Use Docker with compatible base image

### Tests timeout or hang

**Cause:** PDF processing is computationally intensive

**Solutions:**
1. Increase test timeout: `cargo test -- --test-threads=1`
2. Run specific tests instead of all tests
3. Use smaller test PDFs

## CI/CD Integration

### GitHub Actions

```yaml
- name: Install Pdfium
  run: |
    mkdir -p /tmp/pdfium
    cd /tmp/pdfium
    curl -L "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7469/pdfium-linux-x64.tgz" -o pdfium.tgz
    tar -xzf pdfium.tgz
    sudo cp lib/libpdfium.so /usr/local/lib/
    sudo chmod 755 /usr/local/lib/libpdfium.so
    sudo ldconfig

- name: Test with Pdfium
  run: |
    export LD_LIBRARY_PATH=/usr/local/lib
    cargo test --package riptide-pdf
```

### GitLab CI

```yaml
before_script:
  - apt-get update && apt-get install -y curl
  - mkdir -p /tmp/pdfium && cd /tmp/pdfium
  - curl -L "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7469/pdfium-linux-x64.tgz" -o pdfium.tgz
  - tar -xzf pdfium.tgz
  - cp lib/libpdfium.so /usr/local/lib/
  - chmod 755 /usr/local/lib/libpdfium.so
  - ldconfig

variables:
  LD_LIBRARY_PATH: "/usr/local/lib:${LD_LIBRARY_PATH}"
```

## Performance Considerations

- **Binary Size**: Static linking adds ~5-10 MB to binary size
- **Startup Time**: Dynamic linking has minimal startup overhead (<10ms)
- **Memory Usage**: Both approaches use similar memory at runtime
- **Build Time**: Static linking increases compilation time by 30-60 seconds

## Security Considerations

1. **Verify Downloads**: Always verify checksums when downloading binaries
2. **Use Official Sources**: Only download from https://github.com/bblanchon/pdfium-binaries
3. **Keep Updated**: Regularly update to latest Pdfium version for security patches
4. **Sandboxing**: Consider running PDF processing in isolated containers

## Recommended Approach

**For Production:**
- Use Solution 1 (pre-built library) with Docker
- Configure in Dockerfile for consistent environments
- Use ldconfig for system-wide availability

**For Development:**
- Install library to /usr/local/lib
- Add LD_LIBRARY_PATH to shell profile
- Use feature flags to disable when not needed

**For Distribution:**
- Consider Solution 2 (static linking) for standalone binaries
- Or provide installation script for users
- Document library requirements clearly

## References

- Pdfium Binaries: https://github.com/bblanchon/pdfium-binaries
- pdfium-render crate: https://crates.io/crates/pdfium-render
- Pdfium Documentation: https://pdfium.googlesource.com/pdfium/

## Installation Script

A helper script is provided for easy installation:

```bash
#!/bin/bash
# install-pdfium.sh

set -e

PDFIUM_VERSION="chromium/7469"
PLATFORM="linux-x64"
INSTALL_DIR="/usr/local/lib"

echo "Installing Pdfium library..."

# Download
mkdir -p /tmp/pdfium
cd /tmp/pdfium
echo "Downloading pdfium-${PLATFORM}.tgz..."
curl -L "https://github.com/bblanchon/pdfium-binaries/releases/download/${PDFIUM_VERSION}/pdfium-${PLATFORM}.tgz" -o pdfium.tgz

# Extract
echo "Extracting..."
tar -xzf pdfium.tgz

# Install
echo "Installing to ${INSTALL_DIR}..."
sudo mkdir -p ${INSTALL_DIR}
sudo cp lib/libpdfium.so ${INSTALL_DIR}/
sudo chmod 755 ${INSTALL_DIR}/libpdfium.so
sudo ldconfig

# Verify
echo "Verifying installation..."
if ldconfig -p | grep -q pdfium; then
    echo "✓ Pdfium library installed successfully!"
    ldconfig -p | grep pdfium
else
    echo "✗ Installation failed - library not found in cache"
    exit 1
fi

# Cleanup
cd /
rm -rf /tmp/pdfium

echo "Installation complete!"
echo "You may need to set LD_LIBRARY_PATH in your environment:"
echo "  export LD_LIBRARY_PATH=${INSTALL_DIR}:\${LD_LIBRARY_PATH}"
```

Make it executable and run:
```bash
chmod +x install-pdfium.sh
./install-pdfium.sh
```
