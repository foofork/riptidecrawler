# Pdfium Library - Quick Reference

## 🚀 Quick Start

```bash
# Install library
./scripts/install-pdfium.sh

# Configure environment
export LD_LIBRARY_PATH=/usr/local/lib

# Test
cargo test --package riptide-pdf
```

## 📚 Documentation

| Document | Purpose |
|----------|---------|
| [PDFIUM_QUICK_START.md](docs/PDFIUM_QUICK_START.md) | Get started in 3 steps |
| [pdfium-setup-guide.md](docs/pdfium-setup-guide.md) | Comprehensive setup guide |
| [pdfium-solution-summary.md](docs/pdfium-solution-summary.md) | Technical details |
| [PDFIUM_RESOLUTION.md](PDFIUM_RESOLUTION.md) | Complete resolution report |

## 🔧 Files

| File | Purpose |
|------|---------|
| `scripts/install-pdfium.sh` | Automated installation script |
| `.env.pdfium` | Environment configuration |

## ✅ Status

- ✅ Library Installed: `/usr/local/lib/libpdfium.so`
- ✅ System Registered: `ldconfig` cache updated
- ✅ Ready to Use: Set `LD_LIBRARY_PATH=/usr/local/lib`

## 📖 Quick Commands

```bash
# Verify installation
ldconfig -p | grep pdfium

# Run tests
export LD_LIBRARY_PATH=/usr/local/lib
cargo test --package riptide-pdf

# Start API server
export LD_LIBRARY_PATH=/usr/local/lib
cargo run --package riptide-api
```

## 🐳 Docker

```dockerfile
RUN curl -L "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7469/pdfium-linux-x64.tgz" | tar -xz && \
    cp lib/libpdfium.so /usr/local/lib/ && ldconfig
ENV LD_LIBRARY_PATH=/usr/local/lib
```

---

For complete details, see [PDFIUM_RESOLUTION.md](PDFIUM_RESOLUTION.md)
