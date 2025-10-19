# RipTide Validation and System Check Commands

## Overview

The RipTide CLI provides comprehensive validation and system check commands to ensure your environment is properly configured and ready for operation. These commands are designed to catch configuration issues early and provide actionable remediation steps.

## Commands

### `riptide validate`

Basic validation command that checks essential system components.

#### Usage

```bash
# Basic validation
riptide validate

# Comprehensive validation with all checks
riptide validate --comprehensive

# Check WASM setup only
riptide validate --wasm

# JSON output for CI/CD integration
riptide validate --format json

# Custom WASM path
riptide validate --wasm-path /path/to/module.wasm
```

#### Options

- `--comprehensive`: Run all validation checks (recommended for initial setup)
- `--wasm`: Check WASM module availability only
- `--format <FORMAT>`: Output format (`text` or `json`, default: `text`)
- `--wasm-path <PATH>`: Path to WASM module (overrides `RIPTIDE_WASM_PATH`)
- `--continue-on-failure`: Continue running checks even if some fail

#### Exit Codes

- `0`: All checks passed or warnings only
- `1`: One or more checks failed

### `riptide system check`

Comprehensive system health check with categorized output.

#### Usage

```bash
# Standard system check
riptide system check

# Production readiness check (stricter)
riptide system check --production

# Performance baseline profiling
riptide system profile

# JSON output
riptide system check --format json
```

#### Options

- `--production`: Run production-readiness checks (treats warnings as failures)
- `--profile`: Run performance baseline profiling
- `--format <FORMAT>`: Output format (`text` or `json`)
- `--skip <CHECKS>`: Skip specific checks (comma-separated)

#### Exit Codes

- `0`: System healthy (all checks passed)
- `1`: System has critical issues

## Validation Checks

### Core Services

#### API Connectivity
- **What it checks**: Connection to RipTide API server
- **Pass criteria**: API responds to health check
- **Remediation**: Ensure API server is running with `cargo run --bin riptide-api`

#### Redis
- **What it checks**: Redis connection status
- **Pass criteria**: Redis connected and responding
- **Warning**: Redis not configured (acceptable for local operation)
- **Remediation**:
  ```bash
  # Docker
  docker run -d -p 6379:6379 redis:latest

  # Local install
  sudo apt-get install redis-server
  ```

### Extraction Engine

#### WASM Module
- **What it checks**: WASM module availability and readability
- **Pass criteria**: WASM module found and accessible
- **Remediation**:
  ```bash
  cd wasm/riptide-extractor-wasm
  wasm-pack build --target web
  ```

#### Headless Browser
- **What it checks**: Chrome/Chromium availability
- **Pass criteria**: Browser found and version accessible
- **Remediation**:
  ```bash
  # Ubuntu/Debian
  sudo apt-get install chromium-browser

  # macOS
  brew install chromium

  # Or set CHROME_PATH environment variable
  ```

### Infrastructure

#### Network Connectivity
- **What it checks**: Internet connection availability
- **Pass criteria**: Can reach external test URLs
- **Remediation**: Check network connection and firewall settings

#### System Resources
- **What it checks**: CPU count and available memory
- **Pass criteria**: ≥2 CPU cores, ≥512MB available memory
- **Warning**: Low resources (may impact performance)

#### Filesystem Permissions
- **What it checks**: Write access to cache directory
- **Pass criteria**: Can create and write to cache directory
- **Remediation**: Check permissions or set `RIPTIDE_CACHE_DIR`

### Configuration

#### Configuration Validation
- **What it checks**: Environment variables and settings
- **Pass criteria**: All required configuration present
- **Warning**: Optional configuration missing

#### Dependencies
- **What it checks**: Optional development tools
- **Warning**: Tools like `wasm-pack` not found (only needed for development)

## JSON Output Format

When using `--format json`, the output follows this structure:

```json
{
  "timestamp": "2025-10-16T12:00:00Z",
  "checks": [
    {
      "name": "API Connectivity",
      "status": "Pass",
      "message": "API server is reachable",
      "remediation": null,
      "details": null
    },
    {
      "name": "WASM Module",
      "status": "Pass",
      "message": "WASM module available at /path/to/module.wasm (123456 bytes)",
      "remediation": null,
      "details": {
        "path": "/path/to/module.wasm",
        "size_bytes": 123456
      }
    }
  ],
  "summary": {
    "total_checks": 9,
    "passed": 7,
    "failed": 0,
    "warnings": 2,
    "skipped": 0,
    "overall_status": "Warning"
  }
}
```

### Status Values

- `Pass`: Check passed successfully
- `Fail`: Check failed (critical issue)
- `Warning`: Check completed with warnings (non-critical)
- `Skipped`: Check was skipped

## CI/CD Integration

### GitHub Actions Example

```yaml
name: RipTide Validation

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest

    services:
      redis:
        image: redis:latest
        ports:
          - 6379:6379

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build project
        run: cargo build --release

      - name: Run RipTide API (background)
        run: |
          cargo run --bin riptide-api &
          sleep 5

      - name: Validate system
        run: |
          cargo run --bin riptide -- validate --comprehensive --format json > validation.json
          cat validation.json

      - name: Check validation results
        run: |
          # Parse JSON and fail if critical checks failed
          if jq -e '.summary.failed > 0' validation.json; then
            echo "Validation failed"
            exit 1
          fi

      - name: Upload validation report
        uses: actions/upload-artifact@v3
        with:
          name: validation-report
          path: validation.json
```

### GitLab CI Example

```yaml
validate:
  stage: test
  image: rust:latest

  services:
    - redis:latest

  variables:
    REDIS_URL: "redis://redis:6379"

  script:
    - cargo build --release
    - cargo run --bin riptide-api &
    - sleep 5
    - cargo run --bin riptide -- validate --comprehensive --format json | tee validation.json
    - |
      if [ $(jq -r '.summary.failed' validation.json) -gt 0 ]; then
        echo "Validation failed"
        exit 1
      fi

  artifacts:
    reports:
      junit: validation.json
    paths:
      - validation.json
```

## Production Deployment Checklist

Use `riptide system check --production` before deploying to production:

```bash
riptide system check --production --format json > production-check.json
```

### Production Mode Differences

1. **Stricter checks**: All warnings treated as failures
2. **Required components**: All core services must be operational
3. **Clear pass/fail**: No ambiguity in production readiness

### Example Output

```
Production Readiness Check
━━━━━━━━━━━━━━━━━━━━━━━━━━━

Running strict production validation...

Core Services
━━━━━━━━━━━━
  ✓ API Connectivity: API server is reachable
  ✓ Redis: Redis connection established

Extraction Engine
━━━━━━━━━━━━━━━━━
  ✓ WASM Module: WASM module available at /path/to/module.wasm (123456 bytes)
  ✓ Headless Browser: Browser available: Google Chrome 120.0.6099.109

Infrastructure
━━━━━━━━━━━━━━
  ✓ Network Connectivity: Internet connection available
  ✓ System Resources: System resources adequate (4 CPUs, 2048MB available)
  ✓ Filesystem Permissions: Cache directory writable: /home/user/.cache/riptide

Configuration
━━━━━━━━━━━━━
  ✓ Configuration: Configuration is valid
  ⚠ Dependencies: Optional dependencies missing: wasm-pack (optional, for WASM development)

System Check Summary
━━━━━━━━━━━━━━━━━━━━
Total Checks: 9 | Passed: 8 | Failed: 0 | Warnings: 1

✗ NOT RECOMMENDED for production deployment
```

## Performance Profiling

The `--profile` flag runs performance baseline tests:

```bash
riptide system profile
```

### Output Example

```
Performance Baseline Profile
━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Running performance baseline tests...

Performance Baseline Results
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
API Latency: 45ms
✓ Excellent API response time

Timestamp: 2025-10-16T12:00:00Z

ℹ Baseline profile complete
```

## Environment Variables

The validation system respects these environment variables:

- `RIPTIDE_API_URL`: API server URL (default: `http://localhost:8080`)
- `RIPTIDE_WASM_PATH`: Path to WASM module
- `RIPTIDE_CACHE_DIR`: Cache directory location
- `CHROME_PATH`: Custom Chrome/Chromium path
- `RUST_LOG`: Logging level

## Troubleshooting

### WASM Module Not Found

```bash
# Check environment variable
echo $RIPTIDE_WASM_PATH

# Build WASM module
cd wasm/riptide-extractor-wasm
wasm-pack build --target web

# Or set path explicitly
export RIPTIDE_WASM_PATH=/path/to/riptide_extractor_wasm_bg.wasm
```

### API Not Reachable

```bash
# Check if API is running
curl http://localhost:8080/healthz

# Start API server
cargo run --bin riptide-api

# Or check custom URL
export RIPTIDE_API_URL=http://custom-host:8080
```

### Redis Connection Failed

```bash
# Check Redis status
redis-cli ping

# Start Redis (Docker)
docker run -d -p 6379:6379 redis:latest

# Start Redis (local)
sudo systemctl start redis-server
```

### Permission Denied (Cache)

```bash
# Check cache directory
ls -la ~/.cache/riptide

# Create with proper permissions
mkdir -p ~/.cache/riptide
chmod 755 ~/.cache/riptide

# Or use custom location
export RIPTIDE_CACHE_DIR=/tmp/riptide-cache
```

## Best Practices

1. **Run comprehensive validation** on first setup:
   ```bash
   riptide validate --comprehensive
   ```

2. **Use production checks** before deployment:
   ```bash
   riptide system check --production
   ```

3. **Integrate in CI/CD** with JSON output:
   ```bash
   riptide validate --comprehensive --format json
   ```

4. **Profile performance** periodically:
   ```bash
   riptide system profile
   ```

5. **Keep WASM module updated**:
   ```bash
   cd wasm/riptide-extractor-wasm
   wasm-pack build --target web
   ```

## Further Reading

- [RipTide Architecture](./architecture.md)
- [Configuration Guide](./configuration.md)
- [CI/CD Integration](./cicd.md)
- [Production Deployment](./production.md)
