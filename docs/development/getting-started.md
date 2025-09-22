# Getting Started with RipTide Crawler Development

## Quick Setup

RipTide is a high-performance web crawler built in Rust with WebAssembly acceleration and Chrome DevTools Protocol fallback. This guide will get you up and running for development.

## Prerequisites

- **Rust**: Latest stable toolchain
- **Docker**: For containerized services
- **Git**: For version control
- **Just**: Task runner (optional but recommended)

## Environment Setup

### 1. Install Rust and Tools

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Add WebAssembly target
rustup target add wasm32-wasi
rustup target add wasm32-wasip2

# Install development tools
cargo install just
cargo install cargo-deny
cargo install cargo-audit
```

### 2. Clone and Build

```bash
git clone <repository-url>
cd eventmesh

# Build everything (uses Justfile)
just build

# Alternative: use scripts directly
./scripts/build_all.sh
```

### 3. Environment Configuration

```bash
# Copy environment template
cp .env.example .env

# Edit .env and add your API keys:
# SERPER_API_KEY=your_serper_api_key_here
# REDIS_URL=redis://localhost:6379
```

## Development Workflow

### Local Development

```bash
# Start dependencies (Redis)
docker-compose -f docker-compose.simple.yml up -d

# Run API server in development mode
cargo run --bin riptide-api

# Run headless service (separate terminal)
cargo run --bin riptide-headless

# Run tests
just test

# Check code formatting
just fmt
just lint
```

### Docker Development

```bash
# Build and start all services
just up

# View logs
docker-compose logs -f

# Stop services
just down
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
- Health checks and metrics endpoints

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

1. **Explore the codebase**: Start with `crates/riptide-core/src/lib.rs`
2. **Run tests**: Ensure everything works with `cargo test --workspace`
3. **Start services**: Use `just up` to run the full stack
4. **Make a test request**: Try the examples in the API documentation
5. **Check logs**: Monitor `docker-compose logs -f` for any issues

## Configuration

RipTide uses YAML configuration files in the `configs/` directory:

- `riptide.yml` - Main configuration
- `policies.yml` - Crawling policies and restrictions
- `fingerprints.yml` - User agents and stealth settings

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