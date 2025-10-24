# RipTide CLI

Command-line interface for the RipTide web crawler and content extraction framework.

## Overview

`riptide-cli` provides a powerful command-line tool for interacting with RipTide's web crawling, content extraction, and search capabilities. It supports both API-first mode (routing through the REST API) and direct mode (local execution) for maximum flexibility.

## Features

- **Dual Operation Modes**: API-first (default) and direct execution modes
- **Content Extraction**: Extract content from URLs with multiple strategies
- **Web Crawling**: Deep crawl websites with configurable depth and concurrency
- **Search Integration**: Web search with content extraction
- **Cache Management**: View stats, clear cache, validate integrity
- **WASM Management**: Runtime info, benchmarks, health checks
- **System Monitoring**: Health checks, metrics, and diagnostics
- **Flexible Output**: JSON, table, and plain text formats
- **Configuration**: Environment variables and CLI flags

## Installation

### From Source

```bash
# Build from workspace root
cargo build --release -p riptide-cli

# Add to PATH (optional)
sudo cp target/release/riptide /usr/local/bin/

# Or run directly
./target/release/riptide --help
```

### Using Cargo

```bash
# Install from local workspace
cargo install --path crates/riptide-cli

# Run
riptide --help
```

## Operation Modes

### API-First Mode (Default)

Routes all commands through the REST API server:

```bash
# Requires running API server
./target/release/riptide-api &

# CLI commands use API
riptide extract --url "https://example.com"
```

**Benefits:**
- Centralized caching and monitoring
- Load balancing support
- Consistent with web/SDK clients
- Production-ready architecture

### Direct Mode

Executes commands locally without API server:

```bash
# Use --direct flag
riptide extract --url "https://example.com" --direct
```

**Benefits:**
- No API server required
- Lower latency for single operations
- Offline capability
- Development and testing

### API-Only Mode

Enforces API usage, fails if unavailable:

```bash
# Use --api-only flag
riptide extract --url "https://example.com" --api-only
```

## Commands

### Content Extraction

```bash
# Basic extraction
riptide extract --url "https://example.com"

# With confidence scoring
riptide extract --url "https://example.com" --show-confidence

# Strategy composition - chain multiple methods
riptide extract --url "https://example.com" --strategy "chain:css,regex"

# Parallel strategy execution
riptide extract --url "https://example.com" --strategy "parallel:all"

# Specific extraction method (wasm, css, llm, regex, auto, article)
riptide extract --url "https://example.com" --method css

# CSS selector extraction
riptide extract --url "https://example.com" --method css --selector "article.content"

# Save to file
riptide extract --url "https://example.com" -f output.md

# JSON output with metadata
riptide extract --url "https://example.com" --metadata -o json
```

### Web Crawling

```bash
# Basic crawl
riptide crawl --url "https://example.com" --depth 3 --max-pages 100

# Follow external links
riptide crawl --url "https://example.com" --follow-external

# Save results to directory
riptide crawl --url "https://example.com" -d ./crawl-results

# Streaming mode
riptide crawl --url "https://example.com" --stream
```

### Search

```bash
# Basic search
riptide search --query "rust web scraping" --limit 10

# Domain-specific search
riptide search --query "crawler" --domain "github.com"

# Table output
riptide search --query "content extraction" -o table
```

### Cache Management

```bash
# Check cache status
riptide cache status

# View cache statistics
riptide cache stats

# Clear all cache
riptide cache clear

# Validate cache integrity
riptide cache validate
```

### WASM Management

```bash
# Show WASM runtime information
riptide wasm info

# Run performance benchmarks
riptide wasm benchmark --iterations 100

# Check WASM health
riptide wasm health
```

### System Operations

```bash
# Health check
riptide health

# View system metrics
riptide metrics

# Validate configuration
riptide validate

# Comprehensive system check
riptide system-check
```

## Output Configuration

### Output Directory

Configure where CLI saves files:

```bash
# Environment Variables
export RIPTIDE_OUTPUT_DIR="/path/to/output"          # Base output directory
export RIPTIDE_EXTRACT_DIR="/path/to/extractions"    # Extraction-specific
export RIPTIDE_CRAWL_DIR="/path/to/crawl-results"    # Crawl-specific
export RIPTIDE_SEARCH_DIR="/path/to/search-results"  # Search-specific

# Command-line flags (override env vars)
riptide extract --url "https://example.com" --output-dir ./custom-output
riptide crawl --url "https://example.com" --output-dir ./crawl-data
```

### Default Directory Structure

```
./riptide-output/
  ├── extractions/       # Content extraction results
  ├── crawls/           # Crawl results
  ├── searches/         # Search results
  ├── cache/            # Local cache data
  └── logs/             # Operation logs
```

### Output Formats

```bash
# JSON output
riptide -o json extract --url "https://example.com"

# Table format
riptide -o table cache stats

# Plain text (default)
riptide extract --url "https://example.com"
```

## Global Options

```bash
# Specify API server URL
riptide --api-url "http://localhost:8080" health

# Use API key authentication
riptide --api-key "your-api-key" extract --url "https://example.com"

# Verbose logging
riptide -v extract --url "https://example.com"

# Debug logging
riptide -vv crawl --url "https://example.com"
```

## Configuration

### Environment Variables

```bash
# API connection
export RIPTIDE_API_URL="http://localhost:8080"
export RIPTIDE_API_KEY="your-api-key"

# Output configuration
export RIPTIDE_OUTPUT_DIR="/path/to/output"
export RIPTIDE_OUTPUT_FORMAT="json"  # json, table, text

# Operational mode
export RIPTIDE_MODE="api-first"      # api-first, direct, api-only

# Logging
export RUST_LOG="info"
export RIPTIDE_LOG_FILE="/path/to/riptide.log"
```

### Configuration File

Create `~/.riptide/config.toml`:

```toml
[api]
url = "http://localhost:8080"
key = "your-api-key"
timeout = 30

[output]
directory = "/path/to/output"
format = "json"

[crawl]
default_depth = 3
max_pages = 100
concurrency = 10

[extraction]
default_strategy = "chain:css"
show_confidence = true
```

## Examples

### Extract with Full Options

```bash
riptide extract \
  --url "https://blog.example.com/article" \
  --show-confidence \
  --strategy "chain:css" \
  --metadata \
  --output-dir ./articles \
  -f article.md \
  -o json
```

### Comprehensive Crawl

```bash
riptide crawl \
  --url "https://docs.example.com" \
  --depth 5 \
  --max-pages 500 \
  --follow-external \
  --output-dir ./docs-crawl \
  --stream \
  -o json
```

### System Validation

```bash
# Validate before production
riptide validate && \
riptide system-check && \
echo "✓ System ready for production"
```

### Monitoring Workflow

```bash
# Collect system metrics
riptide health -o json > health.json
riptide metrics -o json > metrics.json
riptide cache stats -o json > cache.json
```

## Integration with RipTide API

The CLI integrates seamlessly with the REST API:

```bash
# Start API server
./target/release/riptide-api --config configs/riptide.yml &

# CLI automatically uses API
riptide extract --url "https://example.com"

# Force direct mode if needed
riptide extract --url "https://example.com" --direct
```

## Testing

```bash
# Run CLI tests
cargo test -p riptide-cli

# Test with API server
./target/release/riptide-api &
cargo test -p riptide-cli --features api-integration

# Test direct mode
cargo test -p riptide-cli --features direct-mode
```

## License

Apache-2.0

## Related Crates

- **riptide-api**: REST API server
- **riptide-core**: Core extraction engine
- **riptide-extraction**: Content extraction
- **riptide-search**: Search integration
