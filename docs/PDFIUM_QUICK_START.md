# Pdfium Quick Start Guide

## TL;DR - Get PDF Processing Working Now

### One-Command Installation

```bash
./scripts/install-pdfium.sh
```

### One-Command Test

```bash
export LD_LIBRARY_PATH=/usr/local/lib && cargo test --package riptide-pdf
```

## Quick Setup (3 Steps)

### 1. Install Library (Choose One)

**Option A: Automated Script (Recommended)**
```bash
./scripts/install-pdfium.sh
```

**Option B: Manual Install**
```bash
curl -L "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7469/pdfium-linux-x64.tgz" | tar -xz
sudo cp lib/libpdfium.so /usr/local/lib/
sudo chmod 755 /usr/local/lib/libpdfium.so
sudo ldconfig
```

**Option C: Docker**
```dockerfile
RUN curl -L "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7469/pdfium-linux-x64.tgz" | tar -xz && \
    cp lib/libpdfium.so /usr/local/lib/ && ldconfig
```

### 2. Configure Environment

**For Current Session:**
```bash
export LD_LIBRARY_PATH=/usr/local/lib
```

**For Permanent Use (Choose One):**

```bash
# Option A: Add to shell profile (~/.bashrc or ~/.zshrc)
echo 'export LD_LIBRARY_PATH=/usr/local/lib:${LD_LIBRARY_PATH}' >> ~/.bashrc
source ~/.bashrc

# Option B: System-wide configuration
echo "/usr/local/lib" | sudo tee /etc/ld.so.conf.d/pdfium.conf
sudo ldconfig
```

**For Convenience:**
```bash
# Source the environment file
source .env.pdfium
```

### 3. Verify Installation

```bash
# Check library exists
ldconfig -p | grep pdfium

# Should output:
# libpdfium.so (libc6,x86-64) => /usr/local/lib/libpdfium.so
```

## Usage

### Run Tests
```bash
export LD_LIBRARY_PATH=/usr/local/lib
cargo test --package riptide-pdf
```

### Run API Server
```bash
export LD_LIBRARY_PATH=/usr/local/lib
cargo run --package riptide-api
```

### Process a PDF
```bash
curl -X POST http://localhost:3000/pdf/process \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com/document.pdf"}'
```

## Troubleshooting

### Error: libpdfium.so not found

**Solution 1: Set Library Path**
```bash
export LD_LIBRARY_PATH=/usr/local/lib
```

**Solution 2: Update Cache**
```bash
sudo ldconfig
```

**Solution 3: Verify Installation**
```bash
ls -lh /usr/local/lib/libpdfium.so
```

### Error: Permission denied

```bash
sudo chmod 755 /usr/local/lib/libpdfium.so
```

### Tests Still Fail

```bash
# Check if library is loaded
ldd target/debug/riptide-pdf | grep pdfium

# Reinstall if needed
./scripts/install-pdfium.sh
```

## Alternative: Static Linking

If you can't install system libraries, use static linking:

```toml
# Add to Cargo.toml
[dependencies]
pdfium-render = { version = "0.8", features = ["sync", "thread_safe", "static"] }
```

**Trade-offs:**
- ✅ No runtime dependencies
- ✅ Portable binary
- ❌ Larger binary size (+5-10 MB)
- ❌ Longer compilation time

## CI/CD

### GitHub Actions

```yaml
- name: Install Pdfium
  run: ./scripts/install-pdfium.sh

- name: Test
  run: cargo test --package riptide-pdf
  env:
    LD_LIBRARY_PATH: /usr/local/lib
```

### GitLab CI

```yaml
before_script:
  - ./scripts/install-pdfium.sh

variables:
  LD_LIBRARY_PATH: "/usr/local/lib"
```

### Docker

```dockerfile
# Add to Dockerfile
COPY scripts/install-pdfium.sh /tmp/
RUN /tmp/install-pdfium.sh
ENV LD_LIBRARY_PATH=/usr/local/lib
```

## Platform Support

| Platform | Status | Library Name |
|----------|--------|--------------|
| Linux x64 | ✅ Supported | libpdfium.so |
| Linux ARM64 | ✅ Supported | libpdfium.so |
| macOS x64 | ✅ Supported | libpdfium.dylib |
| macOS ARM64 | ✅ Supported | libpdfium.dylib |
| Windows x64 | ✅ Supported | pdfium.dll |

## Getting Help

1. **Full Documentation**: [docs/pdfium-setup-guide.md](pdfium-setup-guide.md)
2. **Solution Summary**: [docs/pdfium-solution-summary.md](pdfium-solution-summary.md)
3. **Installation Script**: [scripts/install-pdfium.sh](../scripts/install-pdfium.sh)

## Status Check

Run this to check your installation:

```bash
# Quick status check
echo "Library: $([ -f /usr/local/lib/libpdfium.so ] && echo '✓ Installed' || echo '✗ Missing')"
echo "Cache: $(ldconfig -p 2>/dev/null | grep -q pdfium && echo '✓ Registered' || echo '✗ Not found')"
echo "Env: $([ -n "$LD_LIBRARY_PATH" ] && echo "✓ Set ($LD_LIBRARY_PATH)" || echo '✗ Not set')"
```

## Current Installation Status

✅ **Library Installed**: `/usr/local/lib/libpdfium.so` (5.7 MB)
✅ **System Cache**: Registered via ldconfig
✅ **Ready to Use**: Set `LD_LIBRARY_PATH=/usr/local/lib`

---

**Next Steps:**
1. Set environment: `export LD_LIBRARY_PATH=/usr/local/lib`
2. Run tests: `cargo test --package riptide-pdf`
3. Start coding! 🚀
