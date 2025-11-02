# RipTide CLI Refactoring Plan
## From Fat CLI to Thin HTTP Client

**Version**: 1.0
**Date**: 2025-01-15
**Status**: Planning

---

## Executive Summary

This document provides a comprehensive refactoring plan for the RipTide CLI, transitioning from:
- **Current State**: A "fat" Rust CLI with embedded business logic (extraction, browser control, PDF processing, etc.)
- **Target State**: A "thin" HTTP client that delegates ALL work to the RipTide API server

### Key Objectives

1. **Remove All Business Logic**: Strip out dependencies on `riptide-extraction`, `riptide-browser`, `riptide-pdf`, `riptide-workers`, etc.
2. **API-Only Architecture**: CLI becomes a pure HTTP client using `reqwest`
3. **Spec-Driven Design**: Single source of truth (YAML spec) drives all CLI behavior
4. **Feature Parity**: Essential workflow coverage (100% of core user needs)
5. **Maintainability**: Reduce CLI from 27 dependencies to ~15 core dependencies

### Final Recommendation: 7 Commands for v1.0

After thorough analysis of:
- Current Rust CLI (18 commands, 914-line extract.rs)
- Node.js CLI (16 commands)
- API endpoints (120+ routes, 59 primary endpoints)
- User workflows and extraction strategies

**v1.0 will have 7 commands**: `extract`, `spider`, `search`, `render`, `doctor`, `config`, `session`

**Coverage**: 100% of essential workflows | **Timeline**: 6 weeks | **Code Reduction**: 80%

---

## Table of Contents

1. [Current Architecture Analysis](#1-current-architecture-analysis)
2. [Target Architecture](#2-target-architecture)
3. [Dependency Cleanup](#3-dependency-cleanup)
4. [CLI Specification (YAML)](#4-cli-specification-yaml)
5. [Command Mapping](#5-command-mapping)
6. [Implementation Plan](#6-implementation-plan)
7. [Testing Strategy](#7-testing-strategy)
8. [Migration Path](#8-migration-path)
9. [Success Criteria](#9-success-criteria)

---

## 1. Current Architecture Analysis

### Current State Overview

**Rust CLI Analysis** (18 commands):
- `extract` - **914 lines** (PRIMARY use case, 30+ flags)
- `crawl` - 179 lines (simple multi-page)
- `render` - 35KB file (headless browser)
- `spider`, `search`, `health` - Core commands
- `cache`, `wasm`, `stealth`, `domain` - Internal/admin (12 commands)
- `session`, `job`, `pdf`, `tables`, `schema` - Management (6 commands)

**Node.js CLI** (16 commands):
- All thin HTTP clients (~300 lines each)
- Has `config` command (Rust doesn't!)
- Clean separation, no business logic

**Key Finding**: `extract` command is **5x larger** than `crawl` (914 vs 179 lines), proving it's the primary use case.

### Current Rust CLI Structure

**Location**: `/crates/riptide-cli/`

**Dependencies (27 total)**:
```toml
[dependencies]
# Core (KEEP)
anyhow, clap, tokio, serde, serde_json, reqwest, url

# Business Logic (REMOVE - These belong in the API server)
riptide-extraction      # Content extraction logic
riptide-browser        # Browser automation
riptide-pdf           # PDF processing
riptide-workers       # Job processing
riptide-cache         # Cache management
riptide-reliability   # Retry/circuit breaker
riptide-stealth       # Anti-detection
riptide-types         # Shared types
riptide-monitoring    # Metrics collection

# CLI-specific (KEEP)
colored, indicatif, comfy-table, dialoguer, csv
dirs, serde_yaml, ctrlc
```

**Current Features** (that should be API calls):
- Direct extraction execution
- Local browser control
- PDF processing
- WASM module loading
- Cache management
- Worker management
- Session handling

**Problem**: The CLI has 20+ dependencies on business logic crates, making it:
- Hard to maintain (changes require CLI rebuild)
- Deployment-heavy (large binary size)
- Architecturally wrong (business logic in client)

---

### Node.js CLI Structure

**Location**: `/cli/`

**Commands** (16 total):
```javascript
Commands:
  crawl       - Crawl one or more URLs
  search      - Deep search with content extraction
  spider      - Spider crawl with depth limit
  stream      - Stream crawl results
  render      - Render URL with headless browser
  health      - Check API health
  session     - Manage sessions
  worker      - Worker and job management
  monitor     - Monitor system metrics
  config      - Configuration management
  batch       - Batch operations
  interactive - Interactive mode
  profiling   - Domain profiling
  resources   - Resource management
  llm         - LLM provider management
```

**Architecture**:
- Pure HTTP client (axios)
- No business logic
- Clean separation of concerns
- ~300 lines per command (average)

**This is the model we should follow!**

---

## 2. Target Architecture

### Thin Client Design

```
┌─────────────────────────────────────────┐
│         RipTide CLI (Rust)              │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │   CLI Spec (cli.yaml)           │   │
│  │   - Commands & subcommands      │   │
│  │   - Flags & options             │   │
│  │   - Environment variables       │   │
│  │   - Help text                   │   │
│  │   - Exit codes                  │   │
│  └─────────────────────────────────┘   │
│              ↓                          │
│  ┌─────────────────────────────────┐   │
│  │   clap Parser (from spec)       │   │
│  └─────────────────────────────────┘   │
│              ↓                          │
│  ┌─────────────────────────────────┐   │
│  │   HTTP Client (reqwest)         │   │
│  │   - POST /crawl                 │   │
│  │   - POST /spider/crawl          │   │
│  │   - POST /deepsearch            │   │
│  │   - GET  /healthz               │   │
│  └─────────────────────────────────┘   │
│              ↓                          │
│  ┌─────────────────────────────────┐   │
│  │   Output Formatter              │   │
│  │   - JSON                        │   │
│  │   - Table                       │   │
│  │   - Text                        │   │
│  │   - NDJSON streaming            │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
              ↓
    HTTP (JSON/NDJSON)
              ↓
┌─────────────────────────────────────────┐
│     RipTide API Server (Port 8080)      │
│  All business logic lives here          │
└─────────────────────────────────────────┘
```

### Core Principles

1. **No Business Logic**: CLI only formats requests and responses
2. **Stateless**: All state managed by API server
3. **Spec-Driven**: YAML spec is single source of truth
4. **Streaming-First**: Support NDJSON, SSE, WebSocket
5. **Error Mapping**: HTTP status → CLI exit codes

---

## 3. Dependency Cleanup

### Minimal Dependencies (Target: 15 total)

```toml
[dependencies]
# Core dependencies (6)
anyhow = "1.0"           # Error handling
clap = { version = "4.5", features = ["derive", "env"] }
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"       # JSON serialization
serde_yaml = "0.9"       # For cli.yaml spec

# HTTP client (2)
reqwest = { version = "0.12", features = ["json", "stream"] }
url = "2.5"              # URL parsing/validation

# CLI utilities (5)
colored = "2.1"          # Terminal colors
indicatif = "0.17"       # Progress bars
comfy-table = "7.1"      # Table formatting
dirs = "5.0"             # Config directory paths
ctrlc = "3.4"            # Ctrl+C handling

# Config management (2)
env_logger = "0.11"      # Logging
chrono = "0.4"           # Timestamps
```

**Removed** (21 dependencies):
```toml
# REMOVE - Business logic belongs in API server
riptide-extraction, riptide-browser, riptide-pdf
riptide-workers, riptide-cache, riptide-reliability
riptide-stealth, riptide-types, riptide-monitoring

# REMOVE - Not needed for thin client
spider_chrome, scraper, humantime, urlencoding
opentelemetry, once_cell, num_cpus, sha2, uuid
dialoguer, csv, futures, async-trait, tracing
```

---

## 4. CLI Specification (YAML)

### File: `/cli-spec/cli.yaml`

This is the **single source of truth** for all CLI behavior.

```yaml
# RipTide CLI Specification
# This file drives all CLI behavior - commands, flags, help text, exit codes

version: "1.0.0"
name: "riptide"
about: "High-performance web crawler and content extraction CLI"
author: "RipTide Team <team@riptide.dev>"

# Global configuration
config:
  precedence: ["flags", "env", "config_file"]  # flags > env > ~/.config/riptide/config.yaml
  config_path: "~/.config/riptide/config.yaml"
  base_url:
    default: "http://localhost:8080"
    env: "RIPTIDE_BASE_URL"
    flag: "--url"

# Global flags (available to all commands)
global_flags:
  - name: url
    long: --url
    short: -u
    env: RIPTIDE_BASE_URL
    default: "http://localhost:8080"
    help: "RipTide API server URL"

  - name: api-key
    long: --api-key
    short: -k
    env: RIPTIDE_API_KEY
    help: "API authentication key"

  - name: output
    long: --output
    short: -o
    values: [json, table, text, ndjson]
    default: text
    help: "Output format"

  - name: quiet
    long: --quiet
    short: -q
    type: bool
    default: false
    help: "Suppress progress output (stderr)"

  - name: verbose
    long: --verbose
    short: -v
    type: bool
    default: false
    help: "Verbose output"

# Exit codes
exit_codes:
  success: 0
  user_error: 1      # Invalid args, config, network issues
  server_error: 2    # 5xx responses, protocol errors
  invalid_args: 3    # clap validation failure

# Commands
commands:
  # ===== CRAWL =====
  - name: crawl
    about: "Crawl one or more URLs"
    api:
      method: POST
      path: "/crawl"
      streaming_variant: "/crawl/stream"

    args:
      - name: urls
        type: positional
        required: true
        multiple: true
        help: "URLs to crawl"

    flags:
      - name: stream
        long: --stream
        type: bool
        default: false
        help: "Stream results as NDJSON"

      - name: concurrency
        long: --concurrency
        short: -c
        type: int
        default: 5
        env: RIPTIDE_CONCURRENCY
        help: "Concurrent requests"

      - name: timeout
        long: --timeout
        short: -t
        type: int
        default: 30
        env: RIPTIDE_TIMEOUT
        help: "Request timeout (seconds)"

      - name: cache
        long: --cache
        values: [auto, read_write, read_only, write_only, disabled]
        default: auto
        help: "Cache mode"

      - name: output-file
        long: --output-file
        short: -f
        type: path
        help: "Save results to file"

      - name: proxy
        long: --proxy
        env: RIPTIDE_PROXY
        help: "HTTP/HTTPS proxy URL"

      - name: rate-limit
        long: --rate-limit
        type: int
        help: "Requests per second limit"

      - name: robots
        long: --robots
        values: [respect, ignore]
        default: respect
        help: "robots.txt handling"

    examples:
      - command: "riptide crawl https://example.com"
        description: "Crawl a single URL"

      - command: "riptide crawl https://example.com https://example.org --stream"
        description: "Stream results for multiple URLs"

      - command: "riptide crawl https://example.com -c 10 -o out.jsonl --stream"
        description: "High concurrency with file output"

  # ===== SPIDER =====
  - name: spider
    about: "Deep crawl with spider engine"
    api:
      method: POST
      path: "/spider/crawl"

    args:
      - name: url
        type: positional
        required: true
        help: "Starting URL"

    flags:
      - name: depth
        long: --depth
        short: -d
        type: int
        default: 3
        help: "Maximum crawl depth"

      - name: pages
        long: --pages
        short: -p
        type: int
        default: 100
        help: "Maximum pages to crawl"

      - name: strategy
        long: --strategy
        values: [breadth_first, depth_first, best_first]
        default: breadth_first
        help: "Crawl strategy"

    examples:
      - command: "riptide spider https://docs.example.com --depth 5 --pages 500"
        description: "Deep crawl documentation site"

  # ===== SEARCH =====
  - name: search
    about: "Deep search with content extraction"
    api:
      method: POST
      path: "/deepsearch"
      streaming_variant: "/deepsearch/stream"

    args:
      - name: query
        type: positional
        required: true
        help: "Search query"

    flags:
      - name: limit
        long: --limit
        short: -l
        type: int
        default: 10
        help: "Maximum results"

      - name: stream
        long: --stream
        type: bool
        default: false
        help: "Stream results as NDJSON"

      - name: include-content
        long: --include-content
        type: bool
        default: false
        help: "Extract full content"

    examples:
      - command: 'riptide search "web scraping" --limit 20'
        description: "Search with custom limit"

  # ===== DOCTOR =====
  - name: doctor
    about: "System health diagnostics"
    api:
      method: GET
      path: "/healthz"

    flags:
      - name: full
        long: --full
        type: bool
        default: false
        help: "Full diagnostic report including Redis, headless pool, DNS"

    logic:
      # Special handling: check multiple endpoints and provide remediation
      checks:
        - name: api_server
          endpoint: /healthz
          critical: true
          remediation: "Start API server with: ./target/release/riptide-api"

        - name: redis
          endpoint: /healthz
          field: dependencies.redis.status
          critical: true
          remediation: "Start Redis with: docker run -d -p 6379:6379 redis:7-alpine"

        - name: headless_pool
          endpoint: /healthz
          field: dependencies.headless_service.status
          critical: false
          remediation: "JavaScript pages will fail. Start Chrome pool with docker-compose."

        - name: outbound_dns
          test: "resolve example.com"
          critical: false

    examples:
      - command: "riptide doctor"
        description: "Quick health check"

      - command: "riptide doctor --full"
        description: "Comprehensive system diagnostics"

# HTTP Error to Exit Code Mapping
error_mapping:
  # 4xx errors -> exit 1 (user/config error)
  400: 1  # Bad Request
  401: 1  # Unauthorized
  403: 1  # Forbidden
  404: 1  # Not Found
  429: 1  # Too Many Requests

  # 5xx errors -> exit 2 (server/protocol error)
  500: 2  # Internal Server Error
  502: 2  # Bad Gateway
  503: 2  # Service Unavailable
  504: 2  # Gateway Timeout

  # Network errors -> exit 1
  connection_refused: 1
  timeout: 1
  dns_failed: 1

# Output format templates
output_formats:
  crawl_success:
    json: "{results: [...], summary: {total, successful, failed, cached}}"
    text: |
      ✓ Crawled {total} URLs

      Successful: {successful}
      Failed: {failed}
      Cached: {cached}

    table: |
      | URL | Status | Size | Time |
      |-----|--------|------|------|
```

### Why YAML Spec?

1. **Single Source of Truth**: All commands, flags, help text in one place
2. **Easy to Update**: Non-programmers can update help text
3. **Code Generation**: Generate clap definitions from spec
4. **Documentation**: Auto-generate markdown docs from spec
5. **Validation**: Validate CLI behavior against spec in tests

---

## 5. Command Mapping

### v1.0 Commands (7 Total)

Based on analysis of API capabilities and user workflows:

**Core Workflow (5 commands):**
1. `extract` - Advanced extraction with strategy control (PRIMARY use case)
2. `spider` - Deep crawling with frontier management
3. `search` - Web search with content extraction
4. `render` - Headless browser for JavaScript-heavy sites
5. `doctor` - System diagnostics and health checks

**Operations (2 commands):**
6. `config` - Configuration management (critical gap in Rust CLI)
7. `session` - Session management (12 API endpoints exist)

### Complete Command → API Endpoint Mapping

| CLI Command | API Endpoint | Method | Extraction Control | Streaming | Primary Use |
|------------|--------------|--------|-------------------|-----------|-------------|
| `extract <url>` | `/extract` | POST | ✅ Yes (strategy, quality, selector) | No | Advanced extraction |
| `spider <url>` | `/spider/crawl` | POST | ❌ Automatic only | No | Deep site crawls |
| `search <query>` | `/deepsearch` | POST | ❌ Automatic only | ✅ Yes | Web discovery |
| `render <url>` | `/render` | POST | N/A | No | JS-heavy pages |
| `doctor` | `/healthz` + diagnostics | GET | N/A | No | Troubleshooting |
| `config <cmd>` | Local file ops | N/A | N/A | No | Configuration |
| `session <cmd>` | `/sessions/*` | Various | N/A | No | Auth crawling |

**Important**: Only `/extract` endpoint supports extraction strategy control. Others use automatic extraction.

### Why These 7 Commands?

**Comparison Matrix:**

| Command | Current Rust | Node CLI | API Endpoint | v1.0? | Reason |
|---------|-------------|----------|--------------|-------|--------|
| extract | ✅ (914 lines!) | ❌ | `/extract` | ✅ | PRIMARY - proves importance |
| spider | ❌ | ✅ | `/spider/crawl` | ✅ | Advanced crawling |
| search | ✅ | ✅ | `/deepsearch` | ✅ | Core workflow |
| render | ✅ | ✅ | `/render` | ✅ | Modern web essential |
| doctor/health | ✅ | ✅ | `/healthz` | ✅ | Ops requirement |
| config | ❌ | ✅ | Local | ✅ | Critical gap |
| session | ✅ | ✅ | `/sessions/*` | ✅ | 12 endpoints exist |
| crawl | ✅ | ✅ | `/crawl` | ⏸️ | Covered by spider |
| monitor | ❌ | ✅ | Various | ⏸️ v1.1 | Nice-to-have |
| worker | ❌ | ✅ | `/workers/*` | ⏸️ v1.1 | Nice-to-have |

**Deferred to v1.1+**: monitor, worker, batch, interactive, llm, cache, wasm, stealth, metrics, etc.

See [CLI-EXTRACTION-STRATEGY-ANALYSIS.md](CLI-EXTRACTION-STRATEGY-ANALYSIS.md) for extraction strategy details.

### Streaming Protocol: NDJSON

All streaming endpoints return newline-delimited JSON:

```bash
# Client reads line-by-line
POST /crawl/stream
{"url": "https://example.com", ...}

# Server streams responses
{"type": "progress", "url": "https://example.com", "status": "fetching"}
{"type": "progress", "url": "https://example.com", "status": "extracting"}
{"type": "result", "url": "https://example.com", "title": "...", "content": "..."}
{"type": "complete", "total": 1, "successful": 1, "failed": 0}
```

**CLI Implementation**:
```rust
// Pseudo-code
async fn stream_crawl(urls: Vec<String>) -> Result<()> {
    let response = client
        .post(&format!("{}/crawl/stream", base_url))
        .json(&json!({ "urls": urls }))
        .send()
        .await?;

    let mut stream = response.bytes_stream();
    let mut buffer = Vec::new();

    while let Some(chunk) = stream.next().await {
        buffer.extend_from_slice(&chunk?);

        // Process complete lines
        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let line = buffer.drain(..=pos).collect::<Vec<_>>();
            let json: serde_json::Value = serde_json::from_slice(&line)?;

            // Output to stdout (data) or stderr (progress)
            match json["type"].as_str() {
                Some("progress") => {
                    if !quiet {
                        eprintln!("⏳ {}", json["status"]);
                    }
                }
                Some("result") => {
                    println!("{}", serde_json::to_string(&json)?);
                }
                _ => {}
            }
        }
    }

    Ok(())
}
```

---

## 6. Implementation Plan

### Phase 1: Foundation (Week 1)

**Goal**: Set up spec-driven architecture

**Tasks**:
1. Create `/cli-spec/cli.yaml` with full specification
2. Remove all business logic dependencies from `Cargo.toml`
3. Create spec parser: `cli-spec/src/parser.rs`
4. Generate clap structs from YAML spec
5. Write tests for spec validation

**Deliverables**:
- [ ] `/cli-spec/cli.yaml` complete
- [ ] Spec parser working
- [ ] `Cargo.toml` cleaned (15 deps max)
- [ ] Generated clap code compiles

**Test**:
```bash
cargo test -p riptide-cli --test spec_validation
```

---

### Phase 2: Core Commands (Week 2-3)

**Goal**: Implement all 7 v1.0 commands

**New CLI Structure**:
```
crates/riptide-cli/
├── Cargo.toml           # Minimal dependencies
├── src/
│   ├── main.rs          # <150 lines - just parse & dispatch
│   ├── lib.rs
│   ├── spec.rs          # Load and parse cli.yaml
│   ├── client.rs        # HTTP client wrapper
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── extract.rs   # POST /extract (PRIMARY - advanced options)
│   │   ├── spider.rs    # POST /spider/crawl
│   │   ├── search.rs    # POST /deepsearch
│   │   ├── render.rs    # POST /render
│   │   ├── doctor.rs    # GET /healthz + diagnostics
│   │   ├── config.rs    # Local config file operations
│   │   └── session.rs   # POST /sessions/* (12 endpoints)
│   ├── output/
│   │   ├── mod.rs
│   │   ├── json.rs      # JSON formatting
│   │   ├── table.rs     # Table formatting
│   │   ├── text.rs      # Human-readable text
│   │   └── stream.rs    # NDJSON streaming
│   ├── config.rs        # Config file handling
│   └── error.rs         # Error types and exit code mapping
└── tests/
    ├── integration/     # Integration tests with mock API
    └── snapshots/       # Snapshot tests for output formats
```

**Key Files**:

#### `src/main.rs` (~150 lines max)
```rust
use anyhow::Result;
use clap::Parser;

mod spec;
mod client;
mod commands;
mod output;
mod config;
mod error;

use error::ExitCode;

#[derive(Parser)]
#[command(name = "riptide")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "High-performance web crawler and content extraction CLI")]
struct Cli {
    /// RipTide API server URL
    #[arg(long, env = "RIPTIDE_BASE_URL", default_value = "http://localhost:8080")]
    url: String,

    /// API key for authentication
    #[arg(long, env = "RIPTIDE_API_KEY")]
    api_key: Option<String>,

    /// Output format
    #[arg(long, short = 'o', default_value = "text")]
    output: String,

    /// Quiet mode (no progress to stderr)
    #[arg(long, short = 'q')]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    /// Extract content with advanced options (PRIMARY command)
    Extract(commands::extract::ExtractArgs),
    /// Deep crawl with frontier management
    Spider(commands::spider::SpiderArgs),
    /// Search web with content extraction
    Search(commands::search::SearchArgs),
    /// Render JavaScript-heavy pages
    Render(commands::render::RenderArgs),
    /// System health diagnostics
    Doctor(commands::doctor::DoctorArgs),
    /// Configuration management
    Config(commands::config::ConfigArgs),
    /// Session management for authenticated crawling
    Session(commands::session::SessionArgs),
}

#[tokio::main]
async fn main() {
    std::process::exit(match run().await {
        Ok(()) => ExitCode::Success as i32,
        Err(e) => {
            eprintln!("Error: {}", e);
            error::map_error_to_exit_code(&e) as i32
        }
    });
}

async fn run() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    // Create HTTP client
    let client = client::ApiClient::new(cli.url, cli.api_key)?;

    // Dispatch to command
    match cli.command {
        Commands::Extract(args) => commands::extract::execute(client, args, cli.output).await,
        Commands::Spider(args) => commands::spider::execute(client, args, cli.output).await,
        Commands::Search(args) => commands::search::execute(client, args, cli.output, cli.quiet).await,
        Commands::Render(args) => commands::render::execute(client, args, cli.output).await,
        Commands::Doctor(args) => commands::doctor::execute(client, args, cli.output).await,
        Commands::Config(args) => commands::config::execute(args).await, // No API needed
        Commands::Session(args) => commands::session::execute(client, args, cli.output).await,
    }
}
```

#### `src/client.rs` (~100 lines)
```rust
use anyhow::Result;
use reqwest::{Client, Response};
use serde_json::Value;

pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()?;

        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    pub async fn post_json(&self, path: &str, body: Value) -> Result<Response> {
        let mut req = self.client
            .post(format!("{}{}", self.base_url, path))
            .json(&body);

        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }

        Ok(req.send().await?)
    }

    pub async fn get(&self, path: &str) -> Result<Response> {
        let mut req = self.client
            .get(format!("{}{}", self.base_url, path));

        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }

        Ok(req.send().await?)
    }

    // Add streaming support
    pub async fn post_stream(&self, path: &str, body: Value) -> Result<Response> {
        // Similar to post_json but return streaming response
        self.post_json(path, body).await
    }
}
```

#### `src/commands/extract.rs` (~180 lines)
```rust
use anyhow::Result;
use clap::Args;
use serde_json::json;

use crate::client::ApiClient;
use crate::output;

#[derive(Args)]
pub struct ExtractArgs {
    /// URL to extract content from
    url: String,

    /// Extraction strategy: auto, css, wasm, llm, multi
    #[arg(long, default_value = "multi")]
    strategy: String,

    /// CSS selector (for css strategy)
    #[arg(long)]
    selector: Option<String>,

    /// Regex pattern (for regex strategy)
    #[arg(long)]
    pattern: Option<String>,

    /// Minimum quality threshold (0.0-1.0)
    #[arg(long, default_value = "0.7")]
    quality_threshold: f64,

    /// Timeout in milliseconds
    #[arg(long, default_value = "30000")]
    timeout: u64,

    /// Save to file
    #[arg(long, short = 'f')]
    output_file: Option<String>,

    /// Include metadata in output
    #[arg(long)]
    metadata: bool,
}

pub async fn execute(
    client: ApiClient,
    args: ExtractArgs,
    output_format: String,
) -> Result<()> {
    let mut options = json!({
        "strategy": args.strategy,
        "quality_threshold": args.quality_threshold,
        "timeout_ms": args.timeout,
    });

    // Add CSS selector if provided
    if let Some(selector) = args.selector {
        options["selector"] = json!(selector);
    }

    // Add regex pattern if provided
    if let Some(pattern) = args.pattern {
        options["pattern"] = json!(pattern);
    }

    let body = json!({
        "url": args.url,
        "mode": "standard",
        "options": options,
    });

    let response = client.post_json("/extract", body).await?;
    let result: serde_json::Value = response.json().await?;

    // Format and output
    let formatted = output::format(&result, &output_format)?;

    if let Some(file) = args.output_file {
        std::fs::write(file, formatted)?;
    } else {
        println!("{}", formatted);
    }

    Ok(())
}
```

#### `src/commands/spider.rs` (~100 lines)
```rust
use anyhow::Result;
use clap::Args;
use serde_json::json;
use futures_util::StreamExt;

use crate::client::ApiClient;
use crate::output;

#[derive(Args)]
pub struct CrawlArgs {
    /// URLs to crawl
    urls: Vec<String>,

    /// Stream results as NDJSON
    #[arg(long)]
    stream: bool,

    /// Concurrent requests
    #[arg(long, short = 'c', default_value = "5")]
    concurrency: u32,

    /// Request timeout (seconds)
    #[arg(long, short = 't', default_value = "30")]
    timeout: u64,

    /// Cache mode
    #[arg(long, default_value = "auto")]
    cache: String,

    /// Save to file
    #[arg(long, short = 'f')]
    output_file: Option<String>,
}

pub async fn execute(
    client: ApiClient,
    args: CrawlArgs,
    output_format: String,
    quiet: bool,
) -> Result<()> {
    let body = json!({
        "urls": args.urls,
        "options": {
            "concurrency": args.concurrency,
            "timeout": args.timeout,
            "cache_mode": args.cache,
        }
    });

    if args.stream {
        // Stream results
        let response = client.post_stream("/crawl/stream", body).await?;
        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.extend_from_slice(&chunk);

            // Process complete lines
            while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                let line = buffer.drain(..=pos).collect::<Vec<_>>();
                let json: serde_json::Value = serde_json::from_slice(&line)?;

                // Output based on type
                match json["type"].as_str() {
                    Some("progress") if !quiet => {
                        eprintln!("⏳ {}", json["status"].as_str().unwrap_or(""));
                    }
                    Some("result") => {
                        println!("{}", serde_json::to_string(&json)?);
                    }
                    Some("complete") if !quiet => {
                        let total = json["total"].as_u64().unwrap_or(0);
                        let successful = json["successful"].as_u64().unwrap_or(0);
                        eprintln!("\n✓ Completed: {}/{} successful", successful, total);
                    }
                    _ => {}
                }
            }
        }
    } else {
        // Batch request
        let response = client.post_json("/crawl", body).await?;
        let result: serde_json::Value = response.json().await?;

        // Format and output
        let formatted = output::format(&result, &output_format)?;

        if let Some(file) = args.output_file {
            std::fs::write(file, formatted)?;
        } else {
            println!("{}", formatted);
        }
    }

    Ok(())
}
```

#### `src/commands/doctor.rs` (~200 lines)
```rust
use anyhow::Result;
use clap::Args;
use colored::Colorize;

use crate::client::ApiClient;

#[derive(Args)]
pub struct DoctorArgs {
    /// Full diagnostic report
    #[arg(long)]
    full: bool,
}

pub async fn execute(
    client: ApiClient,
    args: DoctorArgs,
    _output_format: String,
) -> Result<()> {
    println!("{}", "RipTide System Diagnostics".bold());
    println!("{}", "=".repeat(50));

    // Check API server
    print!("Checking API server... ");
    match client.get("/healthz").await {
        Ok(response) if response.status().is_success() => {
            let health: serde_json::Value = response.json().await?;
            println!("{}", "✓ OK".green());

            if let Some(version) = health["version"].as_str() {
                println!("  Version: {}", version);
            }

            // Check dependencies
            if let Some(deps) = health["dependencies"].as_object() {
                println!("\n{}", "Dependencies:".bold());

                for (name, status) in deps {
                    let health_status = status["status"].as_str().unwrap_or("unknown");
                    let symbol = if health_status == "healthy" { "✓" } else { "✗" };
                    let color_fn = if health_status == "healthy" {
                        str::green
                    } else {
                        str::red
                    };

                    println!("  {} {} {}", symbol, name, color_fn(health_status));

                    // Show remediation if unhealthy
                    if health_status != "healthy" {
                        match name.as_str() {
                            "redis" => println!("    💡 Start Redis: docker run -d -p 6379:6379 redis:7-alpine"),
                            "headless_service" => println!("    💡 Start Chrome: docker-compose up chrome-service"),
                            _ => {}
                        }
                    }
                }
            }

            if args.full {
                // Additional checks
                println!("\n{}", "Network Diagnostics:".bold());

                // DNS check
                print!("  DNS resolution... ");
                match tokio::net::lookup_host("example.com:80").await {
                    Ok(_) => println!("{}", "✓ OK".green()),
                    Err(e) => println!("{} {}", "✗ FAILED".red(), e),
                }
            }
        }
        Ok(response) => {
            println!("{}", "✗ UNHEALTHY".red());
            println!("  Status: {}", response.status());
            anyhow::bail!("API server is unhealthy");
        }
        Err(e) => {
            println!("{}", "✗ UNREACHABLE".red());
            println!("  Error: {}", e);
            println!("\n{}", "💡 Remediation:".yellow());
            println!("  1. Check if API server is running:");
            println!("     ps aux | grep riptide-api");
            println!("  2. Start API server:");
            println!("     ./target/release/riptide-api");
            println!("  3. Or set custom URL:");
            println!("     export RIPTIDE_BASE_URL=http://production-server:8080");
            anyhow::bail!("Cannot connect to API server");
        }
    }

    Ok(())
}
```

**Deliverables**:
- [ ] All 7 commands working
- [ ] `extract` with full strategy control (PRIMARY)
- [ ] `spider` for deep crawling
- [ ] `search` with streaming support
- [ ] `render` for JS-heavy sites
- [ ] `doctor` with diagnostics
- [ ] `config` for self-service configuration
- [ ] `session` for authenticated crawling
- [ ] Streaming support (NDJSON)
- [ ] File output (`-o file.jsonl`)
- [ ] Progress on stderr, data on stdout
- [ ] Exit code mapping working

---

### Phase 3: Output Formatting (Week 3)

**Goal**: Polish output formats

**Tasks**:
1. Implement JSON formatter
2. Implement table formatter (using `comfy-table`)
3. Implement text formatter (human-readable)
4. Add color support (respect `NO_COLOR` env var)
5. Add progress indicators (using `indicatif`)

**Example**:
```bash
# JSON output
$ riptide crawl https://example.com -o json
{"results": [...], "summary": {...}}

# Table output
$ riptide search "web scraping" -o table
┌────┬──────────────────┬─────────┐
│ #  │ Title            │ URL     │
├────┼──────────────────┼─────────┤
│ 1  │ Beautiful Soup   │ ...     │
└────┴──────────────────┴─────────┘

# Text output (default)
$ riptide crawl https://example.com
✓ Crawled 1 URL

Title: Example Domain
URL: https://example.com
Cached: false

Summary:
  ✓ Successful: 1
  ✗ Failed: 0
  📦 Cached: 0
```

---

### Phase 4: Configuration & Tests (Week 4)

**Goal**: Complete configuration system and comprehensive tests

**Configuration Precedence**:
```
Flags > Environment Variables > Config File > Defaults
```

**Config File**: `~/.config/riptide/config.yaml`
```yaml
api:
  url: "http://localhost:8080"
  key: "your-api-key"
  timeout: 30

output:
  format: "text"  # json, table, text

crawl:
  concurrency: 5
  cache_mode: "auto"
  timeout: 30
```

**Testing Strategy**:

1. **Unit Tests**: Each module has unit tests
2. **Integration Tests**: Test with mock API server
3. **Snapshot Tests**: Golden file comparison for output formats
4. **Parity Tests**: Compare Node CLI vs Rust CLI output

**Mock API Server** for tests:
```rust
// tests/mock_server.rs
use axum::{Router, routing::post, Json};
use serde_json::json;

async fn mock_crawl() -> Json<serde_json::Value> {
    Json(json!({
        "results": [{
            "url": "https://example.com",
            "title": "Example Domain",
            "content": "This domain is for use in examples..."
        }],
        "summary": {
            "total": 1,
            "successful": 1,
            "failed": 0,
            "cached": 0
        }
    }))
}

pub async fn start_mock_server() -> String {
    let app = Router::new()
        .route("/crawl", post(mock_crawl))
        .route("/healthz", axum::routing::get(|| async { "OK" }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("http://{}", addr)
}
```

**Integration Test Example**:
```rust
// tests/integration/crawl_test.rs
#[tokio::test]
async fn test_crawl_command() {
    let mock_url = mock_server::start_mock_server().await;

    // Run CLI command
    let output = Command::new("riptide")
        .arg("--url").arg(&mock_url)
        .arg("crawl")
        .arg("https://example.com")
        .arg("-o").arg("json")
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());

    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["summary"]["successful"], 1);
}
```

**Snapshot Test Example**:
```rust
// tests/snapshots/crawl_output.rs
#[test]
fn test_crawl_text_output() {
    let result = json!({
        "results": [{ "url": "https://example.com", "title": "Example" }],
        "summary": { "total": 1, "successful": 1, "failed": 0 }
    });

    let output = output::text::format_crawl(&result).unwrap();

    insta::assert_snapshot!(output);
}
```

---

### Phase 5: CI/CD & Release (Week 5)

**Goal**: Automated builds and releases

**GitHub Actions Workflow**:
```yaml
# .github/workflows/cli-release.yml
name: CLI Release

on:
  push:
    tags: ['cli-v*']

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        arch: [x86_64, aarch64]

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.arch }}-unknown-linux-gnu

      - name: Build CLI
        run: cargo build --release -p riptide-cli

      - name: Run tests
        run: cargo test -p riptide-cli

      - name: Package binary
        run: |
          cd target/release
          tar czf riptide-${{ matrix.os }}-${{ matrix.arch }}.tar.gz riptide

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: riptide-${{ matrix.os }}-${{ matrix.arch }}
          path: target/release/riptide-*.tar.gz

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            target/release/riptide-*.tar.gz
```

**Cargo Publish**:
```bash
# Prepare Cargo.toml for publishing
cd crates/riptide-cli
cargo publish --dry-run
cargo publish
```

**Installation Methods**:
```bash
# Method 1: Pre-built binaries
curl -L https://github.com/org/riptide/releases/download/cli-v1.0.0/riptide-linux-x86_64.tar.gz | tar xz
sudo mv riptide /usr/local/bin/

# Method 2: Cargo install
cargo install riptide-cli

# Method 3: Build from source
git clone https://github.com/org/riptide.git
cd riptide
cargo build --release -p riptide-cli
```

---

## 7. Testing Strategy

### Test Levels

1. **Unit Tests** (per module)
   - `client.rs`: HTTP client
   - `config.rs`: Configuration loading
   - `output/*.rs`: Formatters
   - `error.rs`: Exit code mapping

2. **Integration Tests** (with mock API)
   - All commands with various flags
   - Streaming behavior
   - Error handling
   - File output

3. **Snapshot Tests** (golden files)
   - JSON output format
   - Table output format
   - Text output format
   - Error messages

4. **Parity Tests** (Node vs Rust CLI)
   - Same input → same output
   - Normalized JSON comparison

### Test Coverage Requirements

- Minimum 90% code coverage
- All error paths tested
- All exit codes validated
- All output formats tested

### Example Test Suite

```bash
# Run all CLI tests
cargo test -p riptide-cli

# Run with coverage
cargo tarpaulin -p riptide-cli --out Html

# Run snapshot tests
cargo test -p riptide-cli --test snapshots

# Run integration tests with mock server
cargo test -p riptide-cli --test integration

# Parity tests (requires Node CLI)
cargo test -p riptide-cli --test parity --features parity-tests
```

---

## 8. Migration Path

### For Users

**Before** (current fat CLI):
```bash
# CLI has direct execution mode
riptide extract --url "https://example.com" --direct
```

**After** (thin CLI):
```bash
# CLI always uses API (API must be running)
riptide crawl https://example.com

# If API not running:
Error: Cannot connect to API server at http://localhost:8080

💡 Remediation:
  1. Start API server: ./target/release/riptide-api
  2. Or set custom URL: export RIPTIDE_BASE_URL=http://production:8080
```

### Migration Guide

**Document**: `/docs/CLI-MIGRATION-GUIDE.md`

```markdown
# Migrating to RipTide CLI v1.0

## Breaking Changes

1. **Removed `--direct` flag**: All operations now go through API
2. **Removed local execution**: API server must be running
3. **Simplified dependencies**: Smaller binary, faster installs

## What Changed

| Old (v0.9.x) | New (v1.0) |
|-------------|-----------|
| `riptide extract --url X --direct` | `riptide crawl X` (requires API) |
| `riptide --api-url X crawl` | `riptide --url X crawl` |
| Local WASM execution | API handles extraction |

## Migration Steps

1. **Start API Server**:
   ```bash
   # Option 1: Docker (recommended)
   docker-compose up -d

   # Option 2: Binary
   ./target/release/riptide-api
   ```

2. **Update CLI**:
   ```bash
   cargo install riptide-cli --version 1.0.0
   ```

3. **Update Scripts**: Remove `--direct` flags

4. **Set Environment**:
   ```bash
   export RIPTIDE_BASE_URL="http://localhost:8080"
   ```

## Why This Change?

- **Maintainability**: Single codebase for business logic
- **Performance**: Centralized caching and resource pooling
- **Reliability**: API handles retries, rate limiting, circuit breaking
- **Simplicity**: Smaller CLI binary, easier to install
```

---

## 9. Success Criteria

### Functional Requirements

- [ ] All Node CLI commands have Rust equivalents
- [ ] Streaming works (NDJSON, SSE)
- [ ] File output works (`-o file.json`)
- [ ] Progress on stderr, data on stdout
- [ ] Exit codes match specification
- [ ] Configuration precedence correct
- [ ] `doctor` command provides helpful diagnostics

### Non-Functional Requirements

- [ ] CLI binary < 15MB (release mode)
- [ ] Dependency count ≤ 15
- [ ] No business logic crates in dependencies
- [ ] Tests pass on Linux, macOS, Windows
- [ ] 90%+ code coverage
- [ ] Documentation complete
- [ ] CI/CD automated

### Quality Gates

1. **Compilation**: `cargo build --release -p riptide-cli` succeeds
2. **Tests**: `cargo test -p riptide-cli` passes
3. **Lints**: `cargo clippy -p riptide-cli` clean
4. **Format**: `cargo fmt -p riptide-cli` clean
5. **Parity**: Node vs Rust output matches
6. **Snapshot**: All golden files match

---

## Appendix A: Complete CLI Spec

See `/cli-spec/cli.yaml` for full specification (150+ lines).

Key sections:
- Commands (crawl, spider, search, doctor)
- Global flags (--url, --api-key, --output, --quiet)
- Environment variables (RIPTIDE_*)
- Exit codes (0, 1, 2, 3)
- Output formats (json, table, text, ndjson)
- Examples for each command

---

## Appendix B: File Output Specification

### Atomic Writes

All file output must be atomic (temp file + rename):

```rust
async fn write_output_file(path: &str, content: &str) -> Result<()> {
    let temp_path = format!("{}.tmp", path);

    // Write to temp file
    tokio::fs::write(&temp_path, content).await?;

    // Atomic rename
    tokio::fs::rename(&temp_path, path).await?;

    Ok(())
}
```

### Formats

- `.json`: Pretty-printed JSON
- `.jsonl` / `.ndjson`: Newline-delimited JSON (one object per line)
- `.txt`: Human-readable text
- `.csv`: CSV format (for search results, etc.)

---

## Appendix C: Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("API connection failed: {0}")]
    ApiConnection(String),

    #[error("API returned error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),
}

pub fn map_error_to_exit_code(error: &anyhow::Error) -> ExitCode {
    if let Some(cli_error) = error.downcast_ref::<CliError>() {
        match cli_error {
            CliError::ApiConnection(_) => ExitCode::UserError,
            CliError::Network(_) => ExitCode::UserError,
            CliError::ApiError { status, .. } => {
                if *status >= 500 {
                    ExitCode::ServerError
                } else {
                    ExitCode::UserError
                }
            }
            CliError::Config(_) => ExitCode::UserError,
            CliError::Parse(_) => ExitCode::InvalidArgs,
        }
    } else {
        ExitCode::UserError
    }
}
```

---

## Appendix D: Project Timeline

| Week | Phase | Deliverables |
|------|-------|-------------|
| 1 | Foundation | CLI spec, parser, clean Cargo.toml |
| 2-3 | Core Commands | extract, spider, search, render, doctor working |
| 3-4 | Operations | config, session commands |
| 4-5 | Output & Tests | All formats (json/table/text), 90%+ coverage |
| 5-6 | CI/CD | Automated builds, releases |

**Total**: 6 weeks to production-ready thin CLI

**Implementation Complexity**:

| Command | Lines (est.) | API Calls | Complexity | Priority |
|---------|-------------|-----------|------------|----------|
| extract | ~180 | 1 | Medium (many flags) | HIGH (PRIMARY) |
| spider | ~100 | 1 | Low | HIGH |
| search | ~80 | 1-2 | Low | HIGH |
| render | ~80 | 1 | Low | HIGH |
| doctor | ~200 | 2-3 | Medium (diagnostics) | HIGH |
| config | ~120 | 0 | Low (file ops) | MEDIUM |
| session | ~100 | 1-2 | Low | MEDIUM |
| **Total** | **~860** | **7-11** | **Manageable** | **All v1.0** |

**vs Current Rust CLI**: ~4,000+ lines across 18 commands
**Code Reduction**: ~80% less code to maintain

---

## Conclusion

This refactoring transforms the RipTide CLI from a "fat" client with embedded business logic to a "thin" HTTP client that delegates all work to the API server.

### Final Recommendation Summary

**v1.0 Commands (7 total)**:
1. `extract` - Advanced extraction (PRIMARY - 914 lines in current CLI proves importance)
2. `spider` - Deep crawling with frontier
3. `search` - Web search + extraction
4. `render` - JavaScript-heavy pages
5. `doctor` - System diagnostics
6. `config` - Configuration management (critical gap)
7. `session` - Authenticated crawling (12 API endpoints)

**Benefits**:
- **Maintainability**: Business logic in one place (API server)
- **Simplicity**: CLI is simple HTTP client (~860 lines vs ~4,000)
- **Coverage**: 100% of essential user workflows
- **Reduction**: 80% less code to maintain
- **Deployment**: Smaller binary, fewer dependencies (15 vs 27)

**Timeline**: 6 weeks | **Coverage**: 100% essential workflows | **Deferred**: 11 advanced commands to v1.1+

### Validation Checklist

- ✅ Can extract content with control? YES (`extract` command with strategy/quality/selector)
- ✅ Can crawl sites deeply? YES (`spider` command)
- ✅ Can search web? YES (`search` command)
- ✅ Can handle JS sites? YES (`render` command)
- ✅ Can troubleshoot? YES (`doctor` command)
- ✅ Can configure CLI? YES (`config` command - fixes critical gap)
- ✅ Can use sessions? YES (`session` command - 12 API endpoints)
- ✅ All workflows covered? YES (100% essential)
- ✅ Achievable in timeline? YES (6 weeks)
- ✅ Maintainable? YES (80% code reduction)

### Related Documentation

- **[CLI-EXTRACTION-STRATEGY-ANALYSIS.md](CLI-EXTRACTION-STRATEGY-ANALYSIS.md)** - Detailed extraction strategy analysis
- **[/cli-spec/cli.yaml](../cli-spec/cli.yaml)** - Complete CLI specification (to be created)
- **[/docs/02-api-reference/ENDPOINT_CATALOG.md](02-api-reference/ENDPOINT_CATALOG.md)** - API endpoint reference

### Next Steps

1. ✅ Review and approve this plan
2. ⏸️ Create `/cli-spec/cli.yaml` with all 7 commands
3. ⏸️ Begin Phase 1 implementation (foundation)
4. ⏸️ Weekly progress reviews
5. ⏸️ Release candidate in 6 weeks

**Questions? Issues?**
Open an issue at: https://github.com/your-org/riptide/issues
