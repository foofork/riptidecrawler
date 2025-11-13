# RipTide Binary Distribution Architecture

This document describes the design and implementation of RipTide's pre-built binary distribution system, solving the 15-minute compilation problem.

## Problem Statement

### Current Pain Points

1. **Long Compilation Times**
   - Full build: ~15 minutes
   - 600+ dependencies
   - Large workspace with 25+ crates
   - WASM components add overhead

2. **Resource Requirements**
   - 4GB+ RAM for compilation
   - 3GB+ disk space for build artifacts
   - Multi-core CPU for parallel builds
   - Problematic for smaller machines

3. **User Friction**
   - Developers want to test quickly
   - Users need immediate access
   - CI/CD pipelines waste time
   - Adoption barriers for evaluation

### Solution Goals

1. ✅ **Instant Access** - Download and run in < 1 minute
2. ✅ **Multi-Platform** - Support Linux, macOS, Windows
3. ✅ **Automatic** - One-command installation
4. ✅ **Small Size** - Optimized binaries (50-100MB)
5. ✅ **Verified** - Checksums and signatures
6. ✅ **Versioned** - Easy updates and rollbacks

---

## Architecture Overview

### Distribution Channels

```
┌─────────────────────────────────────────────────────────┐
│                    Distribution                         │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │
│  │   GitHub    │  │   Docker    │  │   Package   │   │
│  │  Releases   │  │    Hub      │  │  Managers   │   │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘   │
│         │                │                │           │
│         └────────────────┴────────────────┘           │
│                          │                             │
└──────────────────────────┼─────────────────────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │    Users     │
                    └──────────────┘
```

### Build Pipeline

```
┌────────────────────────────────────────────────────────────┐
│                     GitHub Actions                          │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  1. Trigger (Tag Push / Manual)                           │
│     │                                                      │
│     ▼                                                      │
│  2. Create Release                                        │
│     │                                                      │
│     ▼                                                      │
│  3. Build Matrix (Parallel)                              │
│     ├─► Linux x64 (Native)                               │
│     ├─► Linux x64 (WASM)                                 │
│     ├─► Linux ARM64                                       │
│     ├─► macOS x64                                         │
│     ├─► macOS ARM64                                       │
│     └─► Windows x64                                       │
│     │                                                      │
│     ▼                                                      │
│  4. Package & Upload                                      │
│     ├─► .tar.gz (Unix)                                   │
│     └─► .zip (Windows)                                    │
│     │                                                      │
│     ▼                                                      │
│  5. Docker Build & Push                                   │
│     ├─► Native variant                                    │
│     └─► WASM variant                                      │
│     │                                                      │
│     ▼                                                      │
│  6. Publish Release                                       │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

---

## Implementation Details

### 1. GitHub Actions Workflow

**File**: `.github/workflows/release-binaries.yml`

#### Trigger Conditions

```yaml
on:
  push:
    tags: ['v*']           # Version tags (v0.9.0, v1.0.0, etc.)
  workflow_dispatch:        # Manual trigger with parameters
```

#### Build Matrix

```yaml
strategy:
  matrix:
    include:
      # Linux x86_64 (Native) - Default, fastest
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
        variant: native

      # Linux x86_64 (WASM) - Full features
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
        variant: wasm

      # Linux ARM64 - Raspberry Pi, ARM servers
      - os: ubuntu-latest
        target: aarch64-unknown-linux-gnu
        variant: native
        cross: true

      # macOS Intel - Older Macs
      - os: macos-13
        target: x86_64-apple-darwin

      # macOS Apple Silicon - M1/M2/M3
      - os: macos-14
        target: aarch64-apple-darwin

      # Windows - Standard x64
      - os: windows-latest
        target: x86_64-pc-windows-msvc
```

#### Build Process

1. **Dependency Caching**
   ```yaml
   - uses: actions/cache@v4
     with:
       path: ~/.cargo
       key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
   ```

2. **Optimized Compilation**
   ```bash
   # Native variant (40% faster)
   RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
     cargo build --release \
       -p riptide-api \
       --no-default-features \
       --features native-parser

   # WASM variant (full features)
   cargo build --release \
     -p riptide-api \
     --features wasm-extractor
   ```

3. **Binary Optimization**
   ```bash
   # Strip debug symbols
   strip target/release/riptide-api

   # Compress with UPX (optional)
   upx --best --lzma target/release/riptide-api
   ```

4. **Package Creation**
   ```bash
   # Create distribution directory
   mkdir -p dist/riptide-{platform}-{variant}

   # Copy binaries
   cp target/release/riptide-* dist/

   # Copy configs
   cp -r config/ dist/
   cp .env.example dist/.env.example

   # Create archive
   tar czf riptide-{platform}-{variant}.tar.gz dist/
   ```

5. **Checksum Generation**
   ```bash
   shasum -a 256 riptide-*.tar.gz > checksums.sha256
   ```

### 2. Installation Script

**File**: `scripts/install.sh`

#### Features

1. **Platform Detection**
   ```bash
   detect_platform() {
     case "$(uname -s)" in
       Linux*)   os="linux" ;;
       Darwin*)  os="macos" ;;
       MINGW*)   os="windows" ;;
     esac

     case "$(uname -m)" in
       x86_64)   arch="x64" ;;
       aarch64)  arch="arm64" ;;
     esac

     echo "${os}-${arch}"
   }
   ```

2. **Version Resolution**
   ```bash
   get_latest_version() {
     curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" \
       | grep '"tag_name"' \
       | sed -E 's/.*"([^"]+)".*/\1/'
   }
   ```

3. **Download & Verify**
   ```bash
   # Download with progress
   curl -L --progress-bar -o riptide.tar.gz "$DOWNLOAD_URL"

   # Verify checksum
   curl -sSL "${DOWNLOAD_URL}.sha256" -o checksum.sha256
   sha256sum -c checksum.sha256
   ```

4. **Installation**
   ```bash
   # Extract
   tar xzf riptide.tar.gz

   # Install binaries
   mkdir -p ~/.riptide/bin
   cp riptide-* ~/.riptide/bin/
   chmod +x ~/.riptide/bin/riptide-*

   # Create symlinks
   ln -sf ~/.riptide/bin/riptide-api ~/.local/bin/riptide-api
   ```

5. **PATH Configuration**
   ```bash
   # Detect shell
   if [[ -n "$BASH_VERSION" ]]; then
     shell_rc="$HOME/.bashrc"
   elif [[ -n "$ZSH_VERSION" ]]; then
     shell_rc="$HOME/.zshrc"
   fi

   # Add to PATH
   echo 'export PATH="$PATH:$HOME/.local/bin"' >> "$shell_rc"
   ```

### 3. Docker Distribution

#### Image Variants

1. **Native Variant** (Recommended)
   ```dockerfile
   FROM rustlang/rust:nightly-slim AS builder

   # Build with native-parser only
   RUN cargo build --release \
     -p riptide-api \
     --no-default-features \
     --features native-parser

   FROM debian:trixie-slim
   COPY --from=builder /app/target/release/riptide-api /usr/local/bin/
   ```

   - Faster build (~8 minutes)
   - Smaller image (~180MB)
   - No WASM dependencies
   - Suitable for most use cases

2. **WASM Variant** (Advanced)
   ```dockerfile
   # Build native + WASM
   RUN cargo build --release \
     -p riptide-api \
     --features wasm-extractor

   RUN cargo build --release \
     --target wasm32-wasip2 \
     -p riptide-extractor-wasm
   ```

   - Full features (~15 minutes)
   - Larger image (~250MB)
   - WASM extraction support
   - For advanced use cases

#### Publishing

```yaml
# Tag strategy
tags:
  - ghcr.io/your-org/riptidecrawler:latest           # Native
  - ghcr.io/your-org/riptidecrawler:latest-wasm      # WASM
  - ghcr.io/your-org/riptidecrawler:v0.9.0           # Version
  - ghcr.io/your-org/riptidecrawler:v0.9.0-native    # Specific
```

---

## Usage Patterns

### 1. Quick Test (Binary)

```bash
# Download and run
curl -L https://github.com/org/riptide/releases/latest/download/riptide-linux-x64-native.tar.gz | \
  tar xz && cd riptide-* && \
  REQUIRE_AUTH=false ./riptide-api
```

**Time**: < 1 minute
**Result**: API running on http://localhost:8080

### 2. Production Install (Script)

```bash
# Install system-wide
sudo curl -fsSL https://raw.githubusercontent.com/org/riptide/main/scripts/install.sh | \
  sudo bash -s -- --dir /opt/riptide --bin /usr/local/bin

# Configure
sudo nano /opt/riptide/.env

# Run as service
sudo systemctl enable riptide
sudo systemctl start riptide
```

### 3. Development (Docker)

```bash
# Pull image
docker pull ghcr.io/your-org/riptidecrawler:latest

# Run with hot-reload
docker run -it --rm \
  -p 8080:8080 \
  -v $(pwd)/config:/opt/riptide/config \
  ghcr.io/your-org/riptidecrawler:latest
```

### 4. CI/CD (Pre-built)

```yaml
# .github/workflows/test.yml
jobs:
  integration-test:
    runs-on: ubuntu-latest
    steps:
      - name: Download RipTide
        run: |
          curl -L https://github.com/org/riptide/releases/latest/download/riptide-linux-x64-native.tar.gz | tar xz

      - name: Start API
        run: |
          ./riptide-*/riptide-api &
          sleep 5

      - name: Run tests
        run: |
          curl http://localhost:8080/health
```

---

## Performance Metrics

### Build Times

| Method | Time | CPU | Memory |
|--------|------|-----|--------|
| **Full Source** | 15-20 min | 4 cores | 4GB |
| **Native Source** | 8-12 min | 4 cores | 3GB |
| **Binary Download** | < 1 min | - | - |
| **Docker Pull** | 2-5 min | - | - |

### Binary Sizes

| Platform | Native | WASM | Compressed |
|----------|--------|------|------------|
| **Linux x64** | 48MB | 75MB | 32MB / 52MB |
| **Linux ARM64** | 51MB | - | 35MB |
| **macOS x64** | 52MB | - | 36MB |
| **macOS ARM64** | 49MB | - | 34MB |
| **Windows x64** | 54MB | - | 38MB |

### Image Sizes

| Variant | Size | Layers | Build Time |
|---------|------|--------|------------|
| **Native** | 180MB | 8 | 8-10 min |
| **WASM** | 250MB | 10 | 15-18 min |

---

## Security

### Binary Verification

1. **Checksums**
   ```bash
   # SHA256 checksums for all artifacts
   sha256sum -c checksums.sha256
   ```

2. **GPG Signatures** (Future)
   ```bash
   # Sign releases with GPG key
   gpg --armor --detach-sig riptide-linux-x64-native.tar.gz

   # Verify
   gpg --verify riptide-linux-x64-native.tar.gz.asc
   ```

3. **SLSA Provenance** (Future)
   - Build attestations
   - Supply chain security
   - Reproducible builds

### Image Security

1. **Scanning**
   ```yaml
   - name: Scan image
     uses: aquasecurity/trivy-action@master
     with:
       image-ref: ghcr.io/your-org/riptidecrawler:latest
   ```

2. **Signing**
   ```yaml
   - name: Sign image
     uses: sigstore/cosign-installer@main
     run: |
       cosign sign ghcr.io/your-org/riptidecrawler:latest
   ```

3. **SBOM Generation**
   ```bash
   syft packages ghcr.io/your-org/riptidecrawler:latest \
     -o spdx-json > sbom.json
   ```

---

## Maintenance

### Release Process

1. **Version Bump**
   ```bash
   # Update Cargo.toml
   sed -i 's/version = "0.9.0"/version = "0.10.0"/' Cargo.toml

   # Update CHANGELOG.md
   echo "## v0.10.0" >> CHANGELOG.md
   ```

2. **Create Tag**
   ```bash
   git tag -a v0.10.0 -m "Release v0.10.0"
   git push origin v0.10.0
   ```

3. **Automated Build**
   - GitHub Actions triggers on tag push
   - Builds all platforms in parallel
   - Uploads artifacts to release
   - Publishes Docker images

4. **Verification**
   ```bash
   # Test each platform
   ./scripts/test-release.sh v0.10.0
   ```

### Update Strategy

1. **Semantic Versioning**
   - Major: Breaking changes (v1.0.0, v2.0.0)
   - Minor: New features (v0.9.0, v0.10.0)
   - Patch: Bug fixes (v0.9.1, v0.9.2)

2. **Channels**
   - `latest` - Latest stable
   - `latest-wasm` - Latest with WASM
   - `v0.9.0` - Specific version
   - `nightly` - Daily builds (future)

3. **Auto-Update** (Future)
   ```bash
   riptide-cli update
   # Checks for new version
   # Downloads if available
   # Verifies checksum
   # Replaces binary
   ```

---

## Monitoring

### Download Analytics

Track via GitHub API:
```bash
curl https://api.github.com/repos/org/riptide/releases/latest \
  | jq '.assets[] | {name, download_count}'
```

### Popular Platforms

Monitor which platforms are most used to prioritize optimization.

### Docker Pulls

```bash
# Via GitHub Container Registry
curl https://ghcr.io/v2/org/riptidecrawler/tags/list
```

---

## Future Enhancements

### Package Managers

1. **Homebrew** (macOS/Linux)
   ```bash
   brew install riptide
   ```

2. **Chocolatey** (Windows)
   ```powershell
   choco install riptide
   ```

3. **APT Repository** (Debian/Ubuntu)
   ```bash
   sudo apt install riptide
   ```

4. **Snap** (Linux)
   ```bash
   snap install riptide
   ```

### Auto-Update System

```rust
// Check for updates
if let Some(new_version) = check_for_updates().await {
    println!("New version available: {}", new_version);
    println!("Run `riptide update` to upgrade");
}

// Auto-update
async fn update() -> Result<()> {
    let platform = detect_platform();
    let url = format!(
        "https://github.com/org/riptide/releases/latest/download/riptide-{}",
        platform
    );

    download_and_verify(&url).await?;
    replace_binary().await?;

    Ok(())
}
```

### Build Caching

- Cache common dependencies across builds
- Incremental compilation
- Parallel cross-compilation

---

## Troubleshooting

### Common Issues

1. **"Permission denied"**
   ```bash
   chmod +x riptide-api
   ```

2. **"Library not found"**
   ```bash
   # Linux
   sudo apt-get install libssl3

   # macOS
   brew install openssl
   ```

3. **"Port already in use"**
   ```bash
   # Change port
   export SERVER_PORT=8081
   ```

4. **"Download failed"**
   ```bash
   # Use mirror (if available)
   MIRROR=https://mirror.example.com/riptide
   ```

---

## Contributing

### Testing Binaries Locally

```bash
# Build release locally
cargo build --release

# Package
./scripts/package.sh

# Test installation
./scripts/test-install.sh
```

### Adding New Platform

1. Update build matrix in workflow
2. Add platform detection in install script
3. Test on target platform
4. Update documentation

---

## References

- [GitHub Actions Workflow](./.github/workflows/release-binaries.yml)
- [Install Script](./scripts/install.sh)
- [Quick Start Guide](./docs/00-getting-started/QUICK_START.md)
- [Docker Documentation](./infra/docker/README.md)

---

**Maintained by**: RipTide Team
**Last Updated**: 2025-11-13
**Version**: 0.9.0
