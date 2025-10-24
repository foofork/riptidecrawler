# RipTide CLI

Command-line interface for the RipTide web crawler API.

**Version:** 2.1.0
**Status:** Production Ready (Phases 7.5, 9 complete)
**Platform:** Cross-platform (Linux, macOS, Windows)

## Installation

### Option 1: Cargo (Recommended for Rust users)
```bash
# Install from crates.io
cargo install riptide-cli

# Or install from source
cd /workspaces/eventmesh/crates/riptide-cli
cargo install --path .
```

### Option 2: Pre-built Binaries
```bash
# Download from GitHub Releases
# https://github.com/your-org/riptide/releases

# Linux/macOS
chmod +x riptide
sudo mv riptide /usr/local/bin/
```

### Option 3: Docker
```bash
# Run via Docker
docker run --rm riptide/cli:latest health

# Or use npx (Node.js wrapper - if available)
npx @riptide/cli crawl https://example.com
```

## Quick Start

```bash
# Check health
riptide health

# Crawl a URL
riptide crawl https://example.com

# Search the web
riptide search "web scraping tutorials"

# Interactive mode
riptide interactive
```

## Commands

### Commands Overview

- `riptide crawl` - Crawl URLs and extract content
- `riptide search` - Deep search the web
- `riptide health` - Check API health
- `riptide stream` - Real-time streaming
- `riptide session` - Manage sessions
- `riptide worker` - Worker queue management
- `riptide monitor` - Real-time monitoring
- `riptide profiling` - System profiling and diagnostics
- `riptide resources` - Monitor and manage resources
- `riptide llm` - LLM provider management
- `riptide spider` - Deep crawl websites
- `riptide batch` - Process URLs from file
- `riptide config` - Manage configuration
- `riptide interactive` - Interactive shell
- `riptide examples` - Show examples

### Core Commands

#### `crawl` - Crawl URLs

```bash
# Single URL
riptide crawl https://example.com

# Multiple URLs
riptide crawl https://example.com https://example.org

# With options
riptide crawl https://example.com \
  --concurrency 5 \
  --cache read_write \
  --format json \
  --output results.json
```

**Options:**
- `-c, --concurrency <number>` - Concurrency level (default: 3)
- `--cache <mode>` - Cache mode: auto|read_write|read_only|write_only|disabled
- `-o, --output <file>` - Save output to file
- `-f, --format <type>` - Output format: text|json|markdown
- `--extract <mode>` - Extraction mode: article|full
- `--timeout <seconds>` - Request timeout

#### `search` - Deep search

```bash
# Basic search
riptide search "python tutorials"

# With options
riptide search "web scraping" \
  --limit 20 \
  --include-content \
  --output results.json
```

**Options:**
- `-l, --limit <number>` - Maximum results (default: 10)
- `--include-content` - Include full content in results
- `-o, --output <file>` - Save output to file
- `-f, --format <type>` - Output format: text|json|markdown

#### `health` - Check API health

```bash
# Single check
riptide health

# Watch mode (continuous monitoring)
riptide health --watch --interval 5
```

**Options:**
- `-w, --watch` - Watch health status continuously
- `-i, --interval <seconds>` - Watch interval (default: 5)

### Streaming Commands

#### `stream` - Real-time streaming

```bash
# Stream results
riptide stream https://example.com https://example.org

# Save to file (NDJSON)
riptide stream https://example.com \
  --output results.ndjson \
  --concurrency 5
```

**Options:**
- `-c, --concurrency <number>` - Concurrency level
- `-f, --format <type>` - Output format: text|ndjson
- `-o, --output <file>` - Save to file

### Session Management

#### `session` - Manage sessions

```bash
# List sessions
riptide session list

# Create session
riptide session create my-session \
  --user-agent "MyBot/1.0" \
  --cookie "token=abc123"

# Delete session
riptide session delete <session-id>
```

### Worker Management

#### `worker` - Worker queue

```bash
# Get worker status
riptide worker status

# List active jobs
riptide worker jobs
```

### Monitoring

#### `monitor` - Real-time monitoring

```bash
# Monitor health and performance
riptide monitor --interval 30

# Monitor with specific metrics
riptide monitor --score
riptide monitor --metrics
```

**Options:**
- `-i, --interval <seconds>` - Update interval (default: 30)
- `--score` - Show health score
- `--metrics` - Show performance metrics

### Profiling

System profiling and diagnostics:

```bash
# Memory profile
riptide profiling memory
riptide profile memory --format json

# CPU profile
riptide profiling cpu

# Identify bottlenecks
riptide profiling bottlenecks

# Memory allocations
riptide profiling allocations

# Detect memory leaks
riptide profiling leaks

# Create heap snapshot
riptide profiling snapshot
```

**Options:**
- `-f, --format <type>` - Output format (json|text), default: text
- `-o, --output <file>` - Save output to file

### Resources

Monitor and manage system resources:

```bash
# Overall resource status
riptide resources status

# Browser pool metrics
riptide resources browser-pool

# Rate limiter status
riptide resources rate-limiter

# Memory usage by component
riptide resources memory

# Performance metrics
riptide resources performance

# PDF processing status
riptide resources pdf

# Watch mode (continuous monitoring)
riptide resources status --watch
riptide resources browser-pool --watch --interval 5
```

**Options:**
- `-f, --format <type>` - Output format (json|text|table), default: table
- `-o, --output <file>` - Save output to file
- `-w, --watch` - Continuous monitoring mode
- `-i, --interval <seconds>` - Watch interval, default: 10

### LLM Provider Management

Manage LLM provider configuration:

```bash
# List available providers
riptide llm providers

# Switch active provider
riptide llm switch openai
riptide llm switch anthropic

# Get configuration
riptide llm config get
riptide llm config get temperature

# Set configuration
riptide llm config set temperature 0.8
riptide llm config set max_tokens 2000
```

**Options:**
- `-f, --format <type>` - Output format (json|text|table), default: table

### Spider (Deep Crawling)

#### `spider` - Deep crawl

```bash
# Start spider
riptide spider https://example.com \
  --max-depth 3 \
  --max-pages 50
```

**Options:**
- `-d, --max-depth <number>` - Maximum crawl depth (default: 2)
- `-p, --max-pages <number>` - Maximum pages (default: 10)
- `-o, --output <file>` - Save job info to file

### Batch Processing

#### `batch` - Process URLs from file

```bash
# Create urls.txt with one URL per line
echo "https://example.com" > urls.txt
echo "https://example.org" >> urls.txt

# Process batch
riptide batch urls.txt \
  --concurrency 10 \
  --output results.json
```

**Options:**
- `-c, --concurrency <number>` - Concurrency level (default: 5)
- `-o, --output <file>` - Save output to file
- `-f, --format <type>` - Output format: json|ndjson|csv
- `--continue-on-error` - Continue processing on errors

### Configuration

#### `config` - Manage configuration

```bash
# List all configuration
riptide config list

# Get specific value
riptide config get api-url

# Set value
riptide config set api-url http://localhost:8080
riptide config set api-key YOUR_API_KEY

# Reset to defaults
riptide config reset
```

**Configuration Keys:**
- `api-url` - RipTide API URL (default: http://localhost:8080)
- `api-key` - API key for authentication
- `default-concurrency` - Default concurrency level
- `default-cache-mode` - Default cache mode
- `default-format` - Default output format
- `color-output` - Enable/disable colored output

### Interactive Mode

#### `interactive` - Interactive shell

```bash
riptide interactive
```

Features:
- Menu-driven interface
- No command memorization needed
- Step-by-step prompts
- Visual feedback

### Utilities

#### `examples` - Show examples

```bash
riptide examples
```

Displays usage examples for all commands.

#### `--version` - Show version

```bash
riptide --version
```

#### `--help` - Show help

```bash
riptide --help
riptide crawl --help
```

## Global Options

Available for all commands:

- `-u, --url <url>` - RipTide API URL (overrides config)
- `-k, --api-key <key>` - API key (overrides config)
- `--no-color` - Disable colored output
- `--json` - Output raw JSON (no formatting)
- `--debug` - Enable debug mode with stack traces

## Environment Variables

```bash
# Set API URL
export RIPTIDE_API_URL=http://localhost:8080

# Set API key
export RIPTIDE_API_KEY=your-api-key

# Use in commands
riptide crawl https://example.com
```

## Output Formats

### Text (Default)

Human-readable colored output with summaries.

### JSON

```bash
riptide crawl https://example.com --format json
```

Machine-readable JSON for scripting.

### Markdown

```bash
riptide crawl https://example.com --format markdown
```

Formatted markdown for documentation.

### NDJSON (Streaming)

```bash
riptide stream https://example.com --format ndjson
```

Newline-delimited JSON for streaming.

### CSV (Batch)

```bash
riptide batch urls.txt --format csv
```

Comma-separated values for spreadsheets.

## Examples

### Basic Crawling

```bash
# Crawl and save
riptide crawl https://example.com --output result.json

# Crawl with caching
riptide crawl https://example.com --cache read_write

# Crawl as markdown
riptide crawl https://example.com --format markdown > article.md
```

### Batch Processing

```bash
# Create URL list
cat > urls.txt << EOF
https://example.com
https://example.org
https://example.net
EOF

# Process batch
riptide batch urls.txt --concurrency 5 --format csv > results.csv
```

### Monitoring Pipeline

```bash
# Continuous monitoring
riptide monitor --interval 10 > monitor.log &

# Health alerts
riptide health --watch | grep -v healthy && echo "ALERT!"
```

### Scripting

```bash
#!/bin/bash
# Crawl and extract titles

for url in $(cat urls.txt); do
  riptide crawl "$url" --json | jq -r '.results[0].document.title'
done
```

### CI/CD Integration

```bash
# GitHub Actions
- name: Check RipTide Health
  run: |
    npm install -g @riptide/cli
    riptide health || exit 1

# Docker
docker run --rm \
  -e RIPTIDE_API_URL=http://api:8080 \
  node:18 \
  sh -c "npm install -g @riptide/cli && riptide crawl https://example.com"
```

## Configuration File

Config stored at: `~/.config/riptide-cli/config.json`

```json
{
  "api-url": "http://localhost:8080",
  "api-key": "",
  "default-concurrency": 3,
  "default-cache-mode": "auto",
  "default-format": "text",
  "color-output": true
}
```

## Troubleshooting

### Connection Errors

```bash
# Check API is running
curl http://localhost:8080/healthz

# Set correct URL
riptide config set api-url http://localhost:8080

# Or use environment variable
export RIPTIDE_API_URL=http://localhost:8080
```

### Authentication Errors

```bash
# Set API key
riptide config set api-key YOUR_KEY

# Or pass directly
riptide crawl https://example.com --api-key YOUR_KEY
```

### Debug Mode

```bash
# Enable debug output
riptide crawl https://example.com --debug
```

## Development

```bash
# Clone repository
git clone https://github.com/your-org/riptide-api.git
cd riptide-api/cli

# Install dependencies
npm install

# Link for local development
npm link

# Run locally
riptide crawl https://example.com

# Run tests
npm test

# Lint
npm run lint
```

## Publishing

```bash
# Publish to npm
npm publish --access public

# Or use GitHub Actions (automatic on tag)
git tag cli-v1.0.0
git push origin cli-v1.0.0
```

## Links

- [GitHub Repository](https://github.com/your-org/riptide-api)
- [API Documentation](https://github.com/your-org/riptide-api/tree/main/docs)
- [Issues](https://github.com/your-org/riptide-api/issues)
- [Web Playground](http://localhost:3000)
- [Python SDK](https://pypi.org/project/riptide-client/)

## License

MIT License - see [LICENSE](../LICENSE) file

## Support

- GitHub Issues: [Report bugs](https://github.com/your-org/riptide-api/issues)
- Email: support@riptide.dev
- Discord: [Join community](https://discord.gg/riptide)
