# Migration Guide - RipTide Architecture Refactor

## Overview

This guide helps you migrate from the old RipTide version to the new architecture with improved separation of concerns, configurable output directories, and dual CLI modes.

## What's Changed

### 1. CLI Architecture (Breaking Change)

**Old Behavior:**
- CLI had duplicate implementations of core logic
- Direct execution only, no API integration
- Inconsistent behavior between CLI and API
- Fixed output locations

**New Behavior:**
- **API-First Mode (Default)**: Routes through REST API
- **Direct Mode**: Available via `--direct` flag
- Consistent behavior across all clients
- Configurable output directories
- Centralized caching and monitoring

### 2. Output Directory Structure

**Old Structure:**
```
./  (mixed output in current directory)
```

**New Structure:**
```
./riptide-output/
  ├── extractions/
  ├── crawls/
  ├── searches/
  ├── cache/
  └── logs/
```

### 3. Environment Variables

**New Required Variables:**
```bash
# API Configuration (for API-First mode)
RIPTIDE_API_URL=http://localhost:8080  # Default API endpoint

# Output Configuration (optional)
RIPTIDE_OUTPUT_DIR=/path/to/output     # Base output directory
RIPTIDE_EXTRACT_DIR=/custom/extractions # Override extraction dir
RIPTIDE_CRAWL_DIR=/custom/crawls       # Override crawl dir
RIPTIDE_SEARCH_DIR=/custom/searches    # Override search dir

# Cache Configuration (optional)
RIPTIDE_CACHE_DIR=/path/to/cache       # Local cache location
RIPTIDE_CACHE_TTL=3600                 # Cache TTL in seconds
```

## Migration Steps

### Step 1: Update Environment Configuration

Create or update your `.env` file:

```bash
# Copy the example
cp .env.example .env

# Add new required variables
cat >> .env << 'EOF'
# CLI Configuration
RIPTIDE_API_URL=http://localhost:8080
RIPTIDE_OUTPUT_DIR=./riptide-output

# Optional: Custom directories
RIPTIDE_EXTRACT_DIR=./extractions
RIPTIDE_CRAWL_DIR=./crawls
RIPTIDE_SEARCH_DIR=./searches
EOF
```

### Step 2: Update CLI Usage

**Migration Pattern:**

```bash
# OLD: Direct execution (implicit)
riptide extract --url "https://example.com" -f output.md

# NEW: API-First mode (default, recommended)
riptide extract --url "https://example.com" -f output.md

# NEW: Direct mode (for local dev)
riptide extract --url "https://example.com" --direct -f output.md
```

**Output File Handling:**

```bash
# OLD: Saves to current directory
riptide extract --url "https://example.com" -f article.md
# Result: ./article.md

# NEW: Saves to configured output directory
riptide extract --url "https://example.com" -f article.md
# Result: ./riptide-output/extractions/article.md

# NEW: Override with explicit path
riptide extract --url "https://example.com" --output-dir ./custom -f article.md
# Result: ./custom/article.md
```

### Step 3: Start API Server (for API-First Mode)

```bash
# Build and start API server
cargo build --release -p riptide-api
./target/release/riptide-api --config configs/riptide.yml

# Or use Docker
docker-compose up -d

# Verify server is running
curl http://localhost:8080/healthz
```

### Step 4: Update Scripts and Automation

**Before:**
```bash
#!/bin/bash
riptide extract --url "$URL" -f output.md
cat output.md
```

**After:**
```bash
#!/bin/bash
# Option 1: Use API-First mode (recommended)
export RIPTIDE_API_URL=http://localhost:8080
export RIPTIDE_OUTPUT_DIR=./results
riptide extract --url "$URL" -f output.md
cat ./results/extractions/output.md

# Option 2: Use Direct mode (no server required)
riptide extract --url "$URL" --direct --output-dir ./ -f output.md
cat output.md
```

### Step 5: Update CI/CD Pipelines

**GitHub Actions Example:**

```yaml
# Before
- name: Extract content
  run: |
    riptide extract --url "${{ inputs.url }}" -f result.md
    cat result.md

# After (API-First)
- name: Start API server
  run: |
    docker-compose up -d
    sleep 5  # Wait for server to be ready

- name: Extract content
  env:
    RIPTIDE_API_URL: http://localhost:8080
    RIPTIDE_OUTPUT_DIR: ./output
  run: |
    riptide extract --url "${{ inputs.url }}" -f result.md
    cat ./output/extractions/result.md

- name: Cleanup
  run: docker-compose down

# Alternative (Direct mode for simple cases)
- name: Extract content
  run: |
    riptide extract --url "${{ inputs.url }}" --direct --output-dir ./ -f result.md
    cat result.md
```

## Breaking Changes Summary

### 1. Default CLI Mode Changed
- **Impact**: CLI now requires running API server by default
- **Migration**: Start API server OR use `--direct` flag
- **Benefit**: Consistent behavior, centralized caching, better monitoring

### 2. Output Directory Locations
- **Impact**: Files no longer saved to current directory by default
- **Migration**: Use `RIPTIDE_OUTPUT_DIR` or `--output-dir` flag
- **Benefit**: Better organization, no directory pollution

### 3. API Communication
- **Impact**: CLI makes HTTP calls to API server (API-First mode)
- **Migration**: Ensure API server is accessible
- **Benefit**: Load balancing, centralized processing, better error handling

### 4. Configuration Loading
- **Impact**: New environment variables for customization
- **Migration**: Update `.env` file with new variables
- **Benefit**: More flexibility, better defaults

## Compatibility Matrix

| Feature | Old Version | New Version (API-First) | New Version (Direct) |
|---------|-------------|-------------------------|----------------------|
| Requires API Server | ❌ | ✅ | ❌ |
| Output Directory Config | ❌ | ✅ | ✅ |
| Centralized Caching | ❌ | ✅ | ❌ |
| Load Balancing | ❌ | ✅ | ❌ |
| Monitoring/Metrics | ❌ | ✅ | ❌ |
| Simple Local Dev | ✅ | ❌ | ✅ |
| Production Ready | ❌ | ✅ | ⚠️ |

## Rollback Instructions

If you need to rollback to the old version:

```bash
# 1. Check out previous version
git checkout <previous-tag>

# 2. Rebuild CLI
cargo build --release -p riptide-cli

# 3. Restore old environment
rm .env
cp .env.old .env  # If you backed it up

# 4. Use old commands
riptide extract --url "https://example.com" -f output.md
```

## Feature Parity

All old CLI features are available in both modes:

| Feature | API-First Mode | Direct Mode |
|---------|---------------|-------------|
| Content Extraction | ✅ | ✅ |
| Web Crawling | ✅ | ✅ |
| Search | ✅ | ✅ |
| Cache Management | ✅ (centralized) | ✅ (local) |
| WASM Operations | ✅ | ✅ |
| Health Checks | ✅ | ✅ |
| Metrics | ✅ (enhanced) | ✅ (basic) |

## Troubleshooting

### Issue: "Connection refused" error

**Cause**: API server not running (API-First mode)

**Solution**:
```bash
# Option 1: Start API server
./target/release/riptide-api

# Option 2: Use Direct mode
riptide <command> --direct
```

### Issue: Output files not found

**Cause**: Looking in wrong directory (new output structure)

**Solution**:
```bash
# Check configured output directory
echo $RIPTIDE_OUTPUT_DIR

# Files are in subdirectories
ls -la ./riptide-output/extractions/
ls -la ./riptide-output/crawls/

# Or specify explicit path
riptide extract --url "..." --output-dir ./ -f output.md
```

### Issue: Environment variables not loaded

**Cause**: `.env` file not in correct location or format

**Solution**:
```bash
# Verify .env location
ls -la .env

# Check format (no spaces around =)
cat .env

# Explicitly load environment
source .env
riptide <command>

# Or use command-line flags
riptide --api-url http://localhost:8080 extract --url "..."
```

### Issue: Slow performance in API-First mode

**Cause**: Network latency or API server load

**Solution**:
```bash
# Check API server health
curl http://localhost:8080/healthz

# Check API server metrics
curl http://localhost:8080/metrics

# Use Direct mode for local dev
riptide <command> --direct

# Or optimize API server
# - Increase worker threads
# - Scale horizontally
# - Use local Redis cache
```

## Getting Help

- **Documentation**: See [docs/](../README.md) for detailed guides
- **Issues**: Report bugs at [GitHub Issues](https://github.com/your-org/riptide/issues)
- **Discussions**: Ask questions at [GitHub Discussions](https://github.com/your-org/riptide/discussions)
- **Migration Support**: Contact team@riptide.dev

## Next Steps

1. Review [System Design Documentation](../architecture/SYSTEM_DESIGN.md)
2. Configure [Output Directories](../configuration/OUTPUT_DIRECTORIES.md)
3. Read [Rollout Plan](../ROLLOUT_PLAN.md) for timeline
4. Check [Architecture Refactor Summary](../ARCHITECTURE_REFACTOR_SUMMARY.md)
