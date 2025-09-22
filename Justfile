set shell := ["bash", "-euco", "pipefail"]
default: build

# Build all binaries and WASM
build:
    ./scripts/build_all.sh

# Format code
fmt:
    cargo fmt --all

# Run linter
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Run tests
test:
    cargo test --workspace -- --nocapture

# Build WASM module
wasm:
    cd wasm/riptide-extractor-wasm && cargo build --release --target wasm32-wasip1

# Start Docker stack
up:
    cd infra/docker && docker compose up --build -d

# Stop Docker stack
down:
    cd infra/docker && docker compose down

# View logs
logs:
    cd infra/docker && docker compose logs -f

# Clean build artifacts
clean:
    cargo clean
    rm -rf target/

# Run security checks
security:
    cargo deny check

# Quick development cycle
dev: fmt lint test

# Full CI simulation
ci: fmt lint test security build

# Health check
health:
    @echo "Checking API health..."
    @curl -sf http://localhost:8080/healthz || echo "API not running"
    @echo "Checking Headless health..."
    @curl -sf http://localhost:9123/healthz || echo "Headless not running"

# Test crawl endpoint
test-crawl:
    curl -s -X POST localhost:8080/crawl \
        -H 'content-type: application/json' \
        -d '{"urls": ["https://example.com"]}' | jq .

# Test render endpoint
test-render:
    curl -s -X POST localhost:9123/render \
        -H 'content-type: application/json' \
        -d '{"url":"https://example.com", "scroll_steps":2}' | jq .