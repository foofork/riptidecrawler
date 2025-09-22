# RipTide Crawler - Component Analysis

## Component Overview

RipTide Crawler consists of 5 main components organized in a workspace architecture. Each component has specific responsibilities and well-defined interfaces.

## Core Components

### 1. riptide-core (Shared Library)

**Purpose**: Foundation library providing shared functionality across all services.

**Module Structure**:
```
riptide-core/
├── src/
│   ├── lib.rs          # Public API exports
│   ├── cache.rs        # Redis caching layer
│   ├── component.rs    # WASM component integration
│   ├── extract.rs      # Content extraction logic
│   ├── fetch.rs        # HTTP client utilities
│   ├── gate.rs         # Content routing decisions
│   └── types.rs        # Shared data structures
└── Cargo.toml
```

**Key Dependencies**:
- `wasmtime` + `wasmtime-wasi` - WASM runtime
- `redis` - Caching backend
- `reqwest` - HTTP client
- `lol_html` - HTML parsing
- `wit-bindgen` - WASM component bindings

**Public API**:
```rust
pub use types::*;
// Exports: ExtractedDoc, CrawlOptions
```

**Core Types**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedDoc {
    pub url: String,
    pub title: Option<String>,
    pub byline: Option<String>,
    pub published_iso: Option<String>,
    pub markdown: String,
    pub text: String,
    pub links: Vec<String>,
    pub media: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlOptions {
    pub concurrency: usize,
    pub cache_mode: String,
    pub dynamic_wait_for: Option<String>,
    pub scroll_steps: u32,
    pub token_chunk_max: usize,
    pub token_overlap: usize,
}
```

**HTTP Client Configuration**:
```rust
// Optimized for performance and reliability
Client::builder()
    .user_agent("RipTide/1.0")
    .http2_prior_knowledge()
    .gzip(true)
    .brotli(true)
    .connect_timeout(Duration::from_secs(3))
    .timeout(Duration::from_secs(15))
```

---

### 2. riptide-api (REST API Gateway)

**Purpose**: Main API service exposing HTTP endpoints for crawling operations.

**Architecture**: Axum-based async web server

**Module Structure**:
```
riptide-api/
├── src/
│   ├── main.rs         # Server bootstrap and routing
│   ├── handlers.rs     # HTTP request handlers
│   └── models.rs       # Request/response schemas
└── Cargo.toml
```

**Dependencies**:
- `axum` - Web framework
- `tower-http` - Middleware (CORS, compression, tracing)
- `riptide-core` - Shared functionality

**Endpoints**:
- `GET /healthz` → `handlers::health`
- `POST /crawl` → `handlers::crawl`
- `POST /deepsearch` → `handlers::deepsearch`
- Fallback → `handlers::not_found`

**Middleware Stack**:
1. `CorsLayer::permissive()` - Cross-origin support
2. `CompressionLayer::new()` - Response compression
3. `TraceLayer::new_for_http()` - Request tracing

**Configuration**:
```rust
#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "configs/riptide.yml")]
    config: String,

    #[arg(long, default_value = "0.0.0.0:8080")]
    bind: String,
}
```

**Request Models**:
```rust
#[derive(Deserialize)]
pub struct CrawlBody {
    pub urls: Vec<String>,
}

#[derive(Deserialize)]
pub struct DeepSearchBody {
    pub query: String,
    pub limit: Option<u32>,
    pub country: Option<String>,
    pub locale: Option<String>,
}
```

**Response Models**:
```rust
#[derive(Serialize)]
pub struct CrawlResult {
    pub url: String,
    pub status: u16,
    pub from_cache: bool,
    pub markdown_path: Option<String>,
    pub json_path: Option<String>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}
```

---

### 3. riptide-headless (Browser Service)

**Purpose**: Chrome DevTools Protocol (CDP) service for dynamic content rendering.

**Technology**: Chromiumoxide for browser automation

**Module Structure**:
```
riptide-headless/
├── src/
│   ├── main.rs         # Server and routing
│   ├── cdp.rs          # Chrome DevTools Protocol
│   └── models.rs       # CDP request/response models
└── Cargo.toml
```

**Dependencies**:
- `chromiumoxide` - CDP client
- `axum` - Web framework
- `futures` - Async utilities

**Endpoints**:
- `POST /render` → `cdp::render`

**Service Configuration**:
- **Port**: 9123
- **Protocol**: HTTP/JSON API
- **Browser**: Headless Chrome/Chromium

**Use Cases**:
- JavaScript-heavy websites
- Single Page Applications (SPAs)
- Dynamic content requiring user interaction
- Sites with lazy loading

**CDP Integration**:
- Browser instance management
- Page lifecycle control
- Content extraction after rendering
- Screenshot capture (optional)
- PDF generation (optional)

---

### 4. riptide-extractor-wasm (WASM Component)

**Purpose**: High-performance content extraction using WebAssembly Component Model.

**Architecture**: WASM Component with WIT (WebAssembly Interface Types)

**Module Structure**:
```
riptide-extractor-wasm/
├── src/
│   └── lib.rs          # WASM component implementation
├── Cargo.toml
└── wit/                # Component interface definitions
```

**Build Configuration**:
```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.30"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**Component Interface** (WIT):
```wit
// Content extraction interface
interface extractor {
    record extracted-content {
        title: option<string>,
        byline: option<string>,
        content: string,
        links: list<string>,
        media: list<string>,
    }

    extract: func(html: string) -> extracted-content
}
```

**Capabilities**:
- **Article Extraction** - Main content identification
- **Metadata Parsing** - Title, author, date extraction
- **Link Discovery** - URL extraction and classification
- **Media Detection** - Image and video identification
- **Format Conversion** - HTML → Markdown/Text
- **Tokenization** - Content chunking for AI processing

**Performance Benefits**:
- Near-native speed
- Memory safety
- Sandboxed execution
- Language agnostic interface
- Hot-swappable modules

---

### 5. riptide-workers (Background Processing)

**Purpose**: Dedicated worker service for background crawling tasks.

**Status**: Defined in workspace but not yet implemented

**Planned Architecture**:
```
riptide-workers/
├── src/
│   ├── main.rs         # Worker service
│   ├── queue.rs        # Job queue management
│   ├── worker.rs       # Task processing
│   └── scheduler.rs    # Job scheduling
└── Cargo.toml
```

**Planned Features**:
- Redis-based job queue
- Distributed worker scaling
- Task prioritization
- Retry mechanisms
- Progress tracking

---

## Component Interactions

### Data Flow Diagram

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client    │───▶│ riptide-api │───▶│ riptide-    │
│   Request   │    │ (Gateway)   │    │ core        │
└─────────────┘    └─────────────┘    │ (Logic)     │
                           │           └─────────────┘
                           │                   │
                           ▼                   ▼
                   ┌─────────────┐    ┌─────────────┐
                   │ riptide-    │    │   Redis     │
                   │ headless    │    │  (Cache)    │
                   │ (Dynamic)   │    └─────────────┘
                   └─────────────┘           │
                           │                 │
                           ▼                 ▼
                   ┌─────────────────────────────────┐
                   │    riptide-extractor-wasm      │
                   │      (Content Processing)      │
                   └─────────────────────────────────┘
```

### Component Dependencies

```
riptide-api ─────────┐
                     ├──▶ riptide-core
riptide-headless ────┘

riptide-core ────────┐
                     ├──▶ Redis
                     ├──▶ WASM Runtime
                     └──▶ HTTP Client

riptide-extractor-wasm ──▶ WASM Component Model
```

### Communication Patterns

**Synchronous Communication**:
- API → Core (function calls)
- Core → WASM (component calls)
- API → Headless (HTTP requests)

**Asynchronous Communication**:
- Core → Redis (caching)
- Future: Workers → Redis (job queue)

**Configuration**:
- All components read from `configs/riptide.yml`
- Environment variables for secrets
- Docker environment for service discovery

---

## Interface Analysis

### riptide-core Interfaces

**Public API**:
```rust
// HTTP utilities
pub fn http_client() -> Client
pub async fn get(client: &Client, url: &str) -> Result<Response>

// Data types
pub use types::{ExtractedDoc, CrawlOptions};
```

**Internal Modules**:
- `cache::*` - Redis operations
- `component::*` - WASM integration
- `extract::*` - Content processing
- `gate::*` - Routing logic

### riptide-api Interfaces

**HTTP API**:
```
GET  /healthz                 → HealthResponse
POST /crawl                   → CrawlResult[]
POST /deepsearch              → SearchStatus
*    (fallback)               → 404 Not Found
```

**Internal Dependencies**:
```rust
use riptide_core::{ExtractedDoc, CrawlOptions};
```

### riptide-headless Interfaces

**HTTP API**:
```
POST /render                  → RenderedContent
```

**CDP Integration**:
- Browser instance management
- Page navigation and rendering
- Content extraction post-render

### WASM Component Interfaces

**Component Model**:
```wit
interface extractor {
    extract: func(html: string) -> extracted-content
}
```

**Host Integration**:
```rust
// Via wasmtime component model
let component = Component::from_file(&engine, wasm_path)?;
let instance = linker.instantiate(&mut store, &component)?;
```

---

## Configuration Interface

### Configuration Schema (`configs/riptide.yml`)

```yaml
# Search API configuration
search:
  provider: "serper"
  api_key_env: "SERPER_API_KEY"
  country: "us"
  locale: "en"
  per_query_limit: 25

# HTTP crawling configuration
crawl:
  concurrency: 16
  max_redirects: 5
  timeout_ms: 20000
  user_agent_mode: "rotate"
  robots_policy: "obey"
  cache: "read_through"
  max_response_mb: 20

# Content extraction configuration
extraction:
  wasm_module_path: "/opt/riptide/extractor/extractor.wasm"
  version_tag: "trek:0.1"
  mode: "article"
  produce_markdown: true
  produce_json: true
  token_chunk_max: 1200
  token_overlap: 120

# Dynamic content configuration
dynamic:
  enable_headless_fallback: true
  wait_for: null
  scroll:
    enabled: true
    steps: 8
    step_px: 2000
    delay_ms: 300

# Stealth mode configuration
stealth:
  enabled: true
  random_ua: true
  viewport: [1280, 800]
  timezone: "Europe/Amsterdam"
  locale: "en-US"

# Infrastructure configuration
redis:
  url: "redis://redis:6379/0"

artifacts:
  base_dir: "/data/artifacts"
```

### Environment Variables Interface

**Required**:
- `SERPER_API_KEY` - Search API authentication

**Optional**:
- `RUST_LOG` - Logging level (default: "info")
- `REDIS_URL` - Override Redis connection
- `HEADLESS_URL` - Override headless service URL

---

## Extension Points

### Adding New Extractors
1. Create new WASM component implementing extractor interface
2. Update `extraction.wasm_module_path` configuration
3. Deploy new WASM file to artifacts directory

### Adding New Search Providers
1. Implement search provider in `riptide-core`
2. Update configuration schema
3. Add provider-specific API integration

### Adding New Output Formats
1. Extend `ExtractedDoc` structure
2. Update WASM extractor component
3. Modify API response models

### Horizontal Scaling
1. Deploy multiple API instances behind load balancer
2. Implement worker service for background processing
3. Use Redis for distributed coordination

This component analysis provides a comprehensive understanding of how each part of the RipTide Crawler system works individually and in concert with others.