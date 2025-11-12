# 🚪 RipTide CLI - Command-Line Interface

> **Layer**: API (Entry Points & Composition Root)
> **Role**: Command-line interface for web crawling and content extraction
> **Architecture**: Hexagonal (Ports & Adapters)

A powerful command-line tool for the RipTide web crawler. The CLI provides an ergonomic interface to all RipTide capabilities with rich output formatting and sensible defaults.

---

## 📋 Quick Overview

The RipTide CLI is an alternative entry point to the HTTP API, designed for developers, automation scripts, and power users. It provides the same capabilities as the REST API but through a familiar command-line interface.

**Key Features:**
- ✅ **6 Primary Commands**: Extract, Spider, Search, Render, Doctor, Session
- ✅ **Multiple Output Formats**: JSON, text, table
- ✅ **Strategy Control**: Auto, CSS, WASM, LLM, Multi-strategy extraction
- ✅ **Session Management**: Persistent browser sessions with cookies
- ✅ **Streaming Support**: Real-time crawl progress
- ✅ **Environment Configuration**: Flexible config via env vars or flags

---

## 🎯 ApplicationContext Integration

Like the HTTP API, the CLI uses the **ApplicationContext** composition root pattern when running in direct mode (local execution without API server).

```rust
// CLI can operate in two modes:

// 1. API Mode (default) - Calls HTTP API
let client = ApiClient::new(api_url, api_key)?;
client.extract(url).await?;

// 2. Direct Mode - Uses ApplicationContext directly
let ctx = ApplicationContext::new(config, health_checker).await?;
ctx.extraction_facade.extract_content(url).await?;
```

This design allows the CLI to:
- **Share code** with the HTTP API (same business logic)
- **Test without network** (direct mode for unit tests)
- **Run offline** (no API server required)
- **Debug locally** (direct access to components)

---

## 🚀 Installation

### From Source (Recommended)

```bash
# Build from workspace root
cargo build --release -p riptide-cli

# Add to PATH (optional)
sudo cp target/release/riptide /usr/local/bin/

# Or run directly
./target/release/riptide --help
```

### Using Cargo Install

```bash
# Install from local workspace
cd /path/to/riptidecrawler
cargo install --path crates/riptide-cli

# Verify installation
riptide --version
```

### Binary Location

After building, the binary is located at:
```
target/release/riptide
```

---

## 📖 Commands

### 1. Extract - Content Extraction (PRIMARY)

Extract content from URLs using intelligent strategies.

**Basic Usage:**
```bash
# Extract with auto strategy
riptide extract https://example.com

# Multiple URLs
riptide extract https://example.com https://news.ycombinator.com

# Specific strategy
riptide extract https://example.com --strategy css --selector "article"

# Multi-strategy extraction (tries all, merges results)
riptide extract https://example.com --strategy multi

# Save to file
riptide extract https://example.com -f output.md

# JSON output with metadata
riptide extract https://example.com -o json --metadata
```

**All Options:**
```bash
riptide extract [OPTIONS] <URLS>...

Arguments:
  <URLS>...  URLs to extract content from

Options:
  -s, --strategy <STRATEGY>
          Extraction strategy [default: multi]
          Values: auto, css, wasm, llm, multi

  --selector <SELECTOR>
          CSS selector (required for css strategy)

  --pattern <PATTERN>
          Regex pattern for extraction

  --quality-threshold <THRESHOLD>
          Minimum quality score (0.0-1.0) [default: 0.7]

  -t, --timeout <MS>
          Extraction timeout in milliseconds [default: 30000]

  -c, --concurrency <NUM>
          Number of concurrent requests [default: 5]

  --cache <MODE>
          Cache mode [default: auto]
          Values: auto, read_write, read_only, write_only, disabled

  -f, --output-file <FILE>
          Save results to file

  -o, --output <FORMAT>
          Output format [default: text]
          Values: json, text, table
```

**Examples:**
```bash
# Extract article content
riptide extract https://blog.example.com/post --strategy css --selector "article"

# Batch extraction with concurrency
riptide extract \
  https://example.com \
  https://another.com \
  https://third.com \
  --concurrency 10 \
  -f results.json \
  -o json

# High-quality extraction
riptide extract https://example.com --quality-threshold 0.9 --strategy multi

# Fast extraction (skip cache, low threshold)
riptide extract https://example.com --cache disabled --quality-threshold 0.5
```

---

### 2. Spider - Deep Web Crawling

Perform recursive crawling with frontier management.

**Basic Usage:**
```bash
# Basic crawl
riptide spider https://example.com --depth 3 --max-pages 100

# Follow external links
riptide spider https://example.com --follow-external

# Streaming mode (real-time progress)
riptide spider https://example.com --stream

# Save results
riptide spider https://example.com -d ./crawl-results -o json
```

**All Options:**
```bash
riptide spider [OPTIONS] <URL>

Arguments:
  <URL>  Starting URL for crawl

Options:
  --depth <DEPTH>
          Maximum crawl depth [default: 3]

  --max-pages <PAGES>
          Maximum pages to crawl [default: 100]

  --follow-external
          Follow external links (different domain)

  --stream
          Enable streaming mode (real-time progress)

  -d, --output-dir <DIR>
          Directory to save results

  -o, --output <FORMAT>
          Output format [default: text]
```

**Examples:**
```bash
# Deep documentation crawl
riptide spider https://docs.example.com \
  --depth 5 \
  --max-pages 500 \
  --stream \
  -d ./docs-crawl

# Sitemap discovery
riptide spider https://example.com \
  --depth 2 \
  --max-pages 50 \
  -o json > sitemap.json

# Full site mirror
riptide spider https://example.com \
  --depth 10 \
  --max-pages 10000 \
  --follow-external \
  -d ./full-mirror
```

---

### 3. Search - Web Search Integration

Search the web and extract content from results.

**Basic Usage:**
```bash
# Basic search
riptide search --query "rust web scraping" --limit 10

# Domain-specific search
riptide search --query "crawler" --domain "github.com"

# Table output
riptide search --query "content extraction" -o table

# Save results
riptide search --query "rust async" -f results.json -o json
```

**All Options:**
```bash
riptide search [OPTIONS] --query <QUERY>

Options:
  --query <QUERY>
          Search query

  --limit <NUM>
          Maximum results [default: 10]

  --domain <DOMAIN>
          Restrict search to specific domain

  -f, --output-file <FILE>
          Save results to file

  -o, --output <FORMAT>
          Output format [default: text]
```

**Examples:**
```bash
# Research query
riptide search --query "machine learning papers 2024" --limit 20 -o json

# Site-specific search
riptide search --query "api documentation" --domain "docs.rs" --limit 5

# Export to CSV-like table
riptide search --query "rust crates" -o table > results.txt
```

---

### 4. Render - Headless Browser Rendering

Render JavaScript-heavy pages using a headless browser.

**Basic Usage:**
```bash
# Basic render
riptide render https://spa.example.com

# Wait for selector
riptide render https://example.com --wait-for ".content-loaded"

# Full page screenshot
riptide render https://example.com --screenshot screenshot.png

# Extract after render
riptide render https://spa.example.com --extract --selector "#app"
```

**All Options:**
```bash
riptide render [OPTIONS] <URL>

Arguments:
  <URL>  URL to render

Options:
  --wait-for <SELECTOR>
          CSS selector to wait for before extraction

  --screenshot <FILE>
          Save screenshot to file

  --extract
          Extract content after rendering

  --selector <SELECTOR>
          CSS selector for extraction (with --extract)

  --timeout <MS>
          Render timeout in milliseconds [default: 30000]

  -o, --output <FORMAT>
          Output format [default: text]
```

**Examples:**
```bash
# Render SPA and extract
riptide render https://app.example.com \
  --wait-for ".app-loaded" \
  --extract \
  --selector "#main-content"

# Full page screenshot with long timeout
riptide render https://complex-app.com \
  --screenshot page.png \
  --timeout 60000

# Debug rendering
riptide render https://example.com \
  --screenshot before.png \
  --extract \
  -o json
```

---

### 5. Doctor - System Diagnostics

Check system health and diagnose issues.

**Basic Usage:**
```bash
# Full system check
riptide doctor

# Verbose diagnostics
riptide doctor -v

# JSON output for monitoring
riptide doctor -o json
```

**All Options:**
```bash
riptide doctor [OPTIONS]

Options:
  -v, --verbose
          Show detailed diagnostics

  -o, --output <FORMAT>
          Output format [default: text]
```

**What It Checks:**
- ✅ API server connectivity
- ✅ Redis connection
- ✅ Browser pool health
- ✅ Resource availability (CPU, memory, disk)
- ✅ Configuration validity
- ✅ Dependency versions

**Example Output:**
```
System Health Check
==================

✓ API Server: Connected (http://localhost:8080)
✓ Redis: Healthy (redis://localhost:6379)
✓ Browser Pool: 2/3 browsers available
✓ Memory: 512MB / 2GB (25%)
✓ Disk: 50GB / 100GB (50%)

Configuration:
  REDIS_URL: redis://localhost:6379
  RIPTIDE_BASE_URL: http://localhost:8080
  Output Directory: ./riptide-output

All systems operational ✓
```

---

### 6. Session - Session Management

Manage persistent browser sessions with cookies.

**Basic Usage:**
```bash
# List sessions
riptide session list

# Create new session
riptide session create --ttl 3600

# Get session info
riptide session info <session-id>

# Delete session
riptide session delete <session-id>

# Set cookie
riptide session set-cookie <session-id> \
  --domain "example.com" \
  --name "auth_token" \
  --value "abc123"
```

**All Subcommands:**
```bash
riptide session <SUBCOMMAND>

Subcommands:
  list               List all sessions
  create             Create new session
  info               Get session information
  delete             Delete a session
  set-cookie         Set cookie in session
  get-cookies        Get cookies from session
  clear-cookies      Clear all cookies in session
```

**Examples:**
```bash
# Create authenticated session
SESSION_ID=$(riptide session create --ttl 7200 -o json | jq -r '.session_id')

# Set auth cookie
riptide session set-cookie $SESSION_ID \
  --domain "api.example.com" \
  --name "token" \
  --value "eyJ..."

# Use session for extraction
riptide extract https://api.example.com/protected \
  --session $SESSION_ID

# Clean up
riptide session delete $SESSION_ID
```

---

## 🔧 Configuration

### Environment Variables

The CLI respects the following environment variables:

#### API Connection
```bash
# API server URL (required for API mode)
export RIPTIDE_BASE_URL="http://localhost:8080"

# API authentication (optional)
export RIPTIDE_API_KEY="your-api-key-here"
```

#### Output Directories
```bash
# Base output directory
export RIPTIDE_OUTPUT_DIR="/path/to/output"

# Command-specific directories
export RIPTIDE_EXTRACT_DIR="/path/to/extractions"
export RIPTIDE_CRAWL_DIR="/path/to/crawl-results"
export RIPTIDE_SEARCH_DIR="/path/to/search-results"
```

#### Logging
```bash
# Log level (error, warn, info, debug, trace)
export RUST_LOG="info"
export RIPTIDE_LOG_LEVEL="debug"
```

### Command-Line Flags (Global)

These flags work with all commands:

```bash
riptide [GLOBAL OPTIONS] <COMMAND> [COMMAND OPTIONS]

Global Options:
  --url <URL>
          API server URL [env: RIPTIDE_BASE_URL] [default: http://localhost:8080]

  --api-key <KEY>
          API key for authentication [env: RIPTIDE_API_KEY]

  -o, --output <FORMAT>
          Output format: json, text, table [default: text]

  -q, --quiet
          Suppress progress output to stderr

  -v, --verbose
          Show detailed debug information
```

### Configuration Priority

Configuration values are resolved in this order (highest to lowest priority):

1. **Command-line flags** (e.g., `--url http://custom:8080`)
2. **Environment variables** (e.g., `RIPTIDE_BASE_URL`)
3. **Default values** (e.g., `http://localhost:8080`)

**Example:**
```bash
# Environment sets default
export RIPTIDE_BASE_URL="http://prod-api:8080"

# Flag overrides for this command only
riptide --url http://localhost:8080 extract https://example.com
```

### Default Directory Structure

When no custom output directory is specified:

```
./riptide-output/
  ├── extractions/       # riptide extract results
  ├── crawls/           # riptide spider results
  ├── searches/         # riptide search results
  ├── cache/            # Local cache data
  └── logs/             # Operation logs
```

---

## 📊 Output Formats

### Text (Default)

Human-readable output optimized for terminals:

```bash
$ riptide extract https://example.com

Extracting content from https://example.com...
✓ Strategy: WASM
✓ Quality: 0.95
✓ Size: 4.2 KB

=== Content ===
Example Domain
This domain is for use in illustrative examples...
```

### JSON

Machine-readable output for scripting:

```bash
$ riptide extract https://example.com -o json

{
  "url": "https://example.com",
  "status": "success",
  "content": "Example Domain\nThis domain is for use...",
  "strategy_used": "wasm",
  "quality_score": 0.95,
  "content_size": 4243,
  "metadata": {
    "timestamp": "2024-01-15T10:30:00Z",
    "duration_ms": 234
  }
}
```

### Table

Tabular output for multiple results:

```bash
$ riptide search --query "rust" -o table

URL                          | Title           | Quality | Size
-----------------------------------------------------------------
https://rust-lang.org        | Rust Language   | 0.98    | 12KB
https://doc.rust-lang.org    | Documentation   | 0.95    | 45KB
https://crates.io            | Crates Registry | 0.92    | 23KB
```

---

## 🎯 Common Workflows

### Workflow 1: Extract Articles from Blog

```bash
#!/bin/bash
# Extract all blog posts and save as markdown files

# Create output directory
mkdir -p blog-archive

# Get list of post URLs
URLS=$(riptide spider https://blog.example.com \
  --depth 2 \
  --max-pages 100 \
  -o json | jq -r '.pages[].url')

# Extract each post
for URL in $URLS; do
  SLUG=$(echo $URL | sed 's/.*\///' | sed 's/\.html//')
  riptide extract "$URL" \
    --strategy css \
    --selector "article" \
    -f "blog-archive/${SLUG}.md"
  echo "Saved: $SLUG"
done

echo "Archive complete: $(ls -1 blog-archive | wc -l) posts"
```

### Workflow 2: Monitor Website Changes

```bash
#!/bin/bash
# Monitor website for changes

URL="https://example.com/status"
BASELINE="baseline.txt"
CURRENT="current.txt"

# First run: Create baseline
if [ ! -f "$BASELINE" ]; then
  riptide extract "$URL" --strategy auto > "$BASELINE"
  echo "Baseline created"
  exit 0
fi

# Subsequent runs: Compare
riptide extract "$URL" --strategy auto > "$CURRENT"

if diff -q "$BASELINE" "$CURRENT" > /dev/null; then
  echo "No changes detected"
else
  echo "Changes detected!"
  diff "$BASELINE" "$CURRENT"

  # Update baseline
  cp "$CURRENT" "$BASELINE"
fi
```

### Workflow 3: Build Documentation Index

```bash
#!/bin/bash
# Crawl documentation and build searchable index

DOCS_URL="https://docs.example.com"
INDEX_FILE="docs-index.json"

echo "Crawling documentation..."

# Deep crawl with streaming
riptide spider "$DOCS_URL" \
  --depth 5 \
  --max-pages 1000 \
  --stream \
  -o json > crawl-results.json

# Extract content from each page
jq -r '.pages[].url' crawl-results.json | while read URL; do
  echo "Processing: $URL"
  riptide extract "$URL" \
    --strategy multi \
    --quality-threshold 0.8 \
    -o json >> "$INDEX_FILE"
done

echo "Index built: $INDEX_FILE"
echo "Total pages: $(jq -s length $INDEX_FILE)"
```

### Workflow 4: Authenticated Crawling

```bash
#!/bin/bash
# Crawl protected content with session cookies

# Create session
SESSION=$(riptide session create --ttl 3600 -o json | jq -r '.session_id')
echo "Session created: $SESSION"

# Set authentication cookie
riptide session set-cookie "$SESSION" \
  --domain "app.example.com" \
  --name "auth_token" \
  --value "$AUTH_TOKEN" \
  --secure \
  --http-only

# Crawl protected area
riptide spider https://app.example.com/dashboard \
  --session "$SESSION" \
  --depth 3 \
  -d protected-content

# Cleanup
riptide session delete "$SESSION"
echo "Done"
```

---

## 🧪 Testing

### Running Tests

```bash
# All CLI tests
cargo test -p riptide-cli

# With output
cargo test -p riptide-cli -- --nocapture

# Integration tests only
cargo test -p riptide-cli --test '*'

# Specific command tests
cargo test -p riptide-cli extract_command
```

### Test Without API Server

The CLI can run in direct mode for testing:

```rust
#[tokio::test]
async fn test_extract_command() {
    // Set up test environment
    std::env::set_var("RIPTIDE_MODE", "direct");

    // Run command
    let result = run_extract_command(vec!["https://example.com"]).await;

    assert!(result.is_ok());
}
```

---

## 🔍 Troubleshooting

### Common Issues

**API Connection Failed**
```bash
# Check API server is running
curl http://localhost:8080/healthz

# Verify URL
echo $RIPTIDE_BASE_URL

# Try with explicit URL
riptide --url http://localhost:8080 doctor
```

**Permission Denied (Installation)**
```bash
# Don't have sudo? Install to local bin
cargo build --release -p riptide-cli
mkdir -p ~/.local/bin
cp target/release/riptide ~/.local/bin/
export PATH="$HOME/.local/bin:$PATH"
```

**Output Directory Errors**
```bash
# Check directory exists and is writable
ls -ld ./riptide-output

# Create if missing
mkdir -p ./riptide-output/{extractions,crawls,searches}

# Or specify custom directory
riptide extract https://example.com --output-dir /tmp/riptide
```

**Slow Performance**
```bash
# Increase concurrency
riptide extract https://example.com --concurrency 20

# Disable cache for fresh content
riptide extract https://example.com --cache disabled

# Lower quality threshold for speed
riptide extract https://example.com --quality-threshold 0.5
```

### Debug Mode

```bash
# Enable verbose logging
export RUST_LOG=riptide_cli=debug

# Or use -v flag
riptide -v extract https://example.com

# Full backtrace on errors
export RUST_BACKTRACE=full
riptide extract https://example.com
```

---

## 📚 Integration with API

The CLI seamlessly integrates with the RipTide API server:

```bash
# Start API server (in another terminal)
cargo run -p riptide-api -- --bind 0.0.0.0:8080

# CLI automatically uses API
riptide extract https://example.com

# Or specify API URL explicitly
riptide --url http://localhost:8080 extract https://example.com

# Use API key for authentication
riptide --api-key "secret-key-123" extract https://example.com
```

### API vs Direct Mode

| Mode | When to Use | Pros | Cons |
|------|-------------|------|------|
| **API** (default) | Production, shared resources | Centralized caching, monitoring, rate limiting | Requires API server running |
| **Direct** (`--direct` flag) | Development, offline, testing | No server needed, lower latency | No shared cache, no monitoring |

```bash
# Force direct mode (no API server)
riptide extract https://example.com --direct

# Force API mode (fail if server unavailable)
riptide extract https://example.com --api-only
```

---

## 🏛️ Architecture Notes

### CLI Architecture

```
┌─────────────────────────────────────┐
│      CLI Entry Point (main.rs)     │
│  - Parse arguments                  │
│  - Load configuration               │
│  - Initialize ApiClient             │
└──────────────┬──────────────────────┘
               │
        ┌──────▼──────┐
        │  API Client  │
        │  (wrapper)   │
        └──────┬───────┘
               │
    ┌──────────▼──────────────┐
    │   HTTP Calls to API     │
    │   (API mode - default)  │
    └─────────────────────────┘
               OR
    ┌──────────▼──────────────┐
    │  ApplicationContext     │
    │  (Direct mode)          │
    └─────────────────────────┘
```

### Command Structure

Each command is a separate module in `/src/commands/`:

```
src/commands/
  ├── mod.rs           # Command registration
  ├── extract.rs       # Extract command
  ├── spider.rs        # Spider command
  ├── search.rs        # Search command
  ├── render.rs        # Render command
  ├── doctor.rs        # Doctor command
  └── session.rs       # Session command
```

Each command module exports:
- `Args` struct (clap arguments)
- `execute()` function (command logic)
- Request/Response types

---

## 📚 Related Crates

| Crate | Purpose | Used By |
|-------|---------|---------|
| `riptide-api` | HTTP API server | CLI (API mode) |
| `riptide-facade` | Application services | CLI (direct mode) |
| `riptide-core` | Domain logic | Facades |
| `clap` | Argument parsing | CLI parser |
| `serde_json` | JSON serialization | Output formatting |

---

## 🤝 Contributing

See main project `CONTRIBUTING.md` for guidelines.

**Before adding commands:**
1. Understand existing command patterns in `/src/commands/`
2. Add argument structs with proper validation
3. Implement both API and direct modes
4. Add comprehensive tests
5. Update this README with examples

---

## 📄 License

Apache-2.0
