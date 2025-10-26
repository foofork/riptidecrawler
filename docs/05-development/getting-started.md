# Getting Started with RipTide Development

## Quick Setup

RipTide is a high-performance web content extraction system built in Rust with WebAssembly acceleration and Chrome DevTools Protocol fallback. This guide will get you up and running for development.

## Prerequisites

- **Rust**: 1.70+ stable toolchain
- **Docker**: For containerized services (Redis, etc.)
- **Git**: For version control
- **Chrome/Chromium**: For headless service development

## Environment Setup

### 1. Install Rust and Tools

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Add WebAssembly target
rustup target add wasm32-wasip2

# Install development tools
cargo install just
cargo install cargo-deny
cargo install cargo-audit
```

### 2. Clone and Build

```bash
git clone <repository-url>
cd RipTide

# Build everything
cargo build --workspace --release

# Build WASM module
cd wasm/riptide-extractor-wasm
cargo build --release --target wasm32-wasip2
```

### 3. Environment Configuration

```bash
# Set required environment variables
export SERPER_API_KEY=your_serper_api_key_here
export REDIS_URL=redis://localhost:6379/0
export RUST_LOG=info

# Optional: set headless service URL
export HEADLESS_URL=http://localhost:9123
```

## Development Workflow

### Local Development

```bash
# Start dependencies (Redis)
docker run -d -p 6379:6379 redis:7-alpine

# Run API server in development mode
cargo run --bin riptide-api

# Run headless service (separate terminal)
cargo run --bin riptide-headless

# Run tests
cargo test --workspace

# Check code formatting
cargo fmt --all
cargo clippy --workspace --all-targets
```

### Docker Development

```bash
# Build and start all services
docker-compose up -d --build

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

## Project Structure

```
riptide/
├── crates/
│   ├── riptide-core/      # Core crawling logic
│   ├── riptide-api/       # REST API service
│   ├── riptide-headless/  # Chrome CDP service
│   └── riptide-workers/   # Background workers
├── wasm/
│   └── riptide-extractor-wasm/  # WASM content extractor
├── docs/                  # Documentation
├── tests/                 # Integration tests
├── infra/                 # Docker and deployment
├── configs/               # Configuration files
└── scripts/               # Build and utility scripts
```

## Core Components

### RipTide Core (`crates/riptide-core`)

The heart of the crawler containing:
- **Fetch**: HTTP client with compression and retry logic
- **Gate**: Decision engine for fast vs. headless crawling
- **Extract**: WebAssembly content extraction host
- **Cache**: Redis-backed caching layer

### API Service (`crates/riptide-api`)

REST API providing:
- `/crawl` - Crawl URLs and extract content
- `/deepsearch` - Search-driven crawling via SERP APIs
- `/healthz` - Health checks and service status

### Headless Service (`crates/riptide-headless`)

Chrome DevTools Protocol service for:
- Dynamic content rendering
- JavaScript execution
- Screenshot capture
- Complex page interactions

### WASM Extractor (`wasm/riptide-extractor-wasm`)

WebAssembly module for fast content extraction:
- Article text extraction
- Metadata parsing
- Link and media discovery
- Markdown generation

## Development Commands

### Building

```bash
# Build all crates
cargo build --workspace

# Build release
cargo build --workspace --release

# Build WASM module
cd wasm/riptide-extractor-wasm
cargo build --target wasm32-wasip2 --release
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run specific test suite
cargo test --package riptide-core

# Run integration tests
cargo test --test e2e

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --workspace --all-targets -- -D warnings

# Check dependencies
cargo deny check

# Security audit
cargo audit
```

## First Steps

1. **Set environment variables**: Export required environment variables
2. **Start Redis**: `docker run -d -p 6379:6379 redis:7-alpine`
3. **Run tests**: Ensure everything works with `cargo test --workspace`
4. **Start API service**: `cargo run --bin riptide-api`
5. **Test health check**: `curl http://localhost:8080/healthz`
6. **Start headless service**: `cargo run --bin riptide-headless` (optional)
7. **Make a test request**: Try the examples in the API documentation

## Configuration

RipTide primarily uses environment variables for configuration:

- `REDIS_URL` - Redis connection string
- `SERPER_API_KEY` - Search API key
- `HEADLESS_URL` - Headless service URL
- `RUST_LOG` - Logging level

Optional YAML configuration files are in the `configs/` directory for advanced settings.

See the [Configuration Guide](../user/configuration.md) for detailed options.

## IDE Setup

### VS Code

Recommended extensions:
- rust-analyzer
- Even Better TOML
- Docker
- GitLens

### Vim/Neovim

```vim
" Add to your config
Plug 'rust-lang/rust.vim'
Plug 'neoclide/coc.nvim', {'branch': 'release'}
```

## Debugging

### API Issues

```bash
# Check API logs
docker-compose logs api

# Test endpoints directly
curl -X POST localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{"urls": ["https://example.com"]}'
```

### WASM Issues

```bash
# Test WASM module directly
echo "<html><body>Test</body></html>" | \
  wasmtime run --env RIPTIDE_URL=test.com \
  target/wasm32-wasip2/release/riptide-extractor-wasm.wasm
```

### Chrome CDP Issues

```bash
# Check headless service
curl -X POST localhost:9123/render \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'
```

## Next Steps

- Read the [Contributing Guidelines](contributing.md)
- Learn about [Coding Standards](coding-standards.md)
- Explore [Testing Strategies](testing.md)
- Check out [API Usage Examples](../user/api-usage.md)

## Getting Help

- **Issues**: Check GitHub issues for known problems
- **Discussions**: Join GitHub discussions for questions
- **Documentation**: Browse the full documentation in `docs/`
- **Code**: Look at examples in `tests/` and `examples/`