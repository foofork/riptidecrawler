.PHONY: help ci ci-quick install-tools fmt lint test test-unit test-integration build build-wasm audit clean coverage coverage-html coverage-lcov coverage-json coverage-open coverage-report
.PHONY: docker-build-api docker-build-headless docker-build-playground docker-build-all docker-up docker-down docker-logs docker-up-dev docker-down-dev docker-logs-dev
.PHONY: build-release build-ci build-fast-dev test-api test-spider test-extraction test-streaming test-browser
.PHONY: test-cli-commands test-engine-selection test-profiles-api bench bench-pool bench-wasm bench-report
.PHONY: build-wasm-optimized test-wasm dev-server quick-start smoke-test pre-commit monitor-health load-test

help:
	@echo "🔧 RipTide Local Development Commands"
	@echo ""
	@echo "Quick checks (run before commit):"
	@echo "  make ci-quick    - Fast checks (~1min): fmt, clippy, unit tests"
	@echo ""
	@echo "Full validation (run before push):"
	@echo "  make ci          - Full CI mirror: all checks + integration tests"
	@echo ""
	@echo "Individual checks:"
	@echo "  make fmt         - Format code"
	@echo "  make lint        - Run clippy lints"
	@echo "  make test        - Run all tests"
	@echo "  make test-unit   - Run unit tests only"
	@echo "  make test-int    - Run integration tests only"
	@echo "  make build       - Build workspace"
	@echo "  make audit       - Security & license audit"
	@echo ""
	@echo "Coverage commands:"
	@echo "  make coverage         - Generate coverage (lcov format)"
	@echo "  make coverage-html    - Generate HTML coverage report"
	@echo "  make coverage-lcov    - Generate lcov.info for Codecov"
	@echo "  make coverage-json    - Generate JSON coverage report"
	@echo "  make coverage-open    - Generate and open HTML report"
	@echo "  make coverage-report  - Full coverage report (all formats)"
	@echo ""
	@echo "Docker commands:"
	@echo "  make docker-build-all - Build all Docker images"
	@echo "  make docker-up        - Start all services (FULL - API + Chrome + Redis) ✅"
	@echo "  make docker-down      - Stop all services"
	@echo "  make docker-logs      - View container logs"
	@echo "  make docker-up-lite   - Start lite services (WASM-only, no Chrome)"
	@echo "  make docker-down-lite - Stop lite services"
	@echo "  make docker-logs-lite - View lite container logs"
	@echo ""
	@echo "Profile-specific builds:"
	@echo "  make build-release    - Build with release profile"
	@echo "  make build-ci         - Build with CI profile"
	@echo "  make build-fast-dev   - Build with fast-dev profile"
	@echo ""
	@echo "Crate-specific tests:"
	@echo "  make test-api         - Test riptide-api"
	@echo "  make test-spider      - Test riptide-spider"
	@echo "  make test-extraction  - Test riptide-extraction"
	@echo "  make test-streaming   - Test riptide-streaming"
	@echo "  make test-browser     - Test riptide-browser"
	@echo ""
	@echo "Feature tests (Phase 9/10/10.4):"
	@echo "  make test-cli-commands     - Test CLI commands"
	@echo "  make test-engine-selection - Test engine selection"
	@echo "  make test-profiles-api     - Test profiles API"
	@echo ""
	@echo "Benchmarking:"
	@echo "  make bench            - Run all benchmarks"
	@echo "  make bench-pool       - Benchmark connection pool"
	@echo "  make bench-wasm       - Benchmark WASM features"
	@echo "  make bench-report     - Generate benchmark baseline"
	@echo ""
	@echo "WASM builds:"
	@echo "  make build-wasm           - Build WASM modules"
	@echo "  make build-wasm-optimized - Build optimized WASM"
	@echo "  make test-wasm            - Test WASM extractors"
	@echo ""
	@echo "Developer workflows:"
	@echo "  make dev-server       - Run development API server"
	@echo "  make quick-start      - Build and start all services"
	@echo "  make smoke-test       - Run smoke tests"
	@echo "  make pre-commit       - Run pre-commit checks"
	@echo ""
	@echo "Monitoring & load testing:"
	@echo "  make monitor-health   - Monitor service health"
	@echo "  make load-test        - Run load tests"
	@echo ""
	@echo "Setup:"
	@echo "  make install-tools - Install cargo-deny, cargo-audit, cargo-llvm-cov"
	@echo ""
	@echo "See docs/LOCAL_CI.md for full workflow guide"

ci-quick:
	@./scripts/ci-quick.sh

ci:
	@./scripts/ci-local.sh

install-tools:
	@echo "📦 Installing CI tools..."
	@cargo install cargo-deny cargo-audit cargo-llvm-cov --locked
	@rustup component add llvm-tools-preview

fmt:
	@cargo fmt --all

lint:
	@cargo clippy --workspace --all-targets -- -D warnings

test:
	@cargo test --workspace -- --nocapture

test-unit:
	@cargo test --workspace --lib --bins -- --nocapture

test-int:
	@cargo test --workspace --tests -- --nocapture

build:
	@cargo build --workspace

audit:
	@echo "🔒 Security audit..."
	@cargo audit
	@echo "📜 License check..."
	@cargo deny check

clean:
	@cargo clean

# Coverage targets using cargo-llvm-cov
coverage:
	@echo "📊 Running coverage analysis (lcov format)..."
	@cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

coverage-html:
	@echo "📊 Generating HTML coverage report..."
	@cargo llvm-cov --all-features --workspace --html
	@echo "✅ HTML report generated in target/llvm-cov/html/"

coverage-lcov:
	@echo "📊 Generating lcov coverage for Codecov..."
	@cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
	@echo "✅ Coverage report: lcov.info"

coverage-json:
	@echo "📊 Generating JSON coverage report..."
	@cargo llvm-cov --all-features --workspace --json --output-path coverage.json
	@echo "✅ JSON report: coverage.json"

coverage-open:
	@echo "📊 Generating and opening HTML coverage report..."
	@cargo llvm-cov --all-features --workspace --open

coverage-report:
	@echo "📊 Generating comprehensive coverage reports..."
	@cargo llvm-cov --all-features --workspace --html --lcov --output-path lcov.info
	@cargo llvm-cov --all-features --workspace --json --output-path coverage.json
	@echo "✅ Coverage reports generated:"
	@echo "  - HTML: target/llvm-cov/html/index.html"
	@echo "  - LCOV: lcov.info"
	@echo "  - JSON: coverage.json"

# ============================================================================
# TIER 1: CRITICAL TARGETS (Phase 10+)
# ============================================================================

# Docker Build Targets (Phase 10+)
docker-build-api:
	@echo "🐳 Building riptide-api Docker image..."
	docker build -f infra/docker/Dockerfile.api -t riptide-api:latest .

docker-build-headless:
	@echo "🐳 Building riptide-headless Docker image..."
	docker build -f infra/docker/Dockerfile.headless -t riptide-headless:latest .

docker-build-playground:
	@echo "🐳 Building riptide-playground Docker image..."
	docker build -f playground/Dockerfile -t riptide-playground:latest ./playground

docker-build-all: docker-build-api docker-build-headless docker-build-playground
	@echo "✅ All Docker images built successfully"

# Docker Compose Targets
docker-up:
	@echo "🚀 Starting FULL services (API + Chrome Browser + Redis + Swagger)..."
	@echo "   Memory: ~1.2GB | Features: ✅ Chrome rendering ✅ JavaScript ✅ SPA pages"
	docker-compose up -d

docker-down:
	@echo "🛑 Stopping all services..."
	docker-compose down

docker-logs:
	@echo "📋 Viewing container logs..."
	docker-compose logs -f

docker-up-lite:
	@echo "🚀 Starting LITE services (API + Redis + Swagger, no Chrome)..."
	@echo "   Memory: ~440MB | Features: ✅ WASM extraction ❌ No JavaScript"
	docker-compose -f docker-compose.lite.yml up -d

docker-down-lite:
	@echo "🛑 Stopping lite services..."
	docker-compose -f docker-compose.lite.yml down

docker-logs-lite:
	@echo "📋 Viewing lite container logs..."
	docker-compose -f docker-compose.lite.yml logs -f

# Profile-Aware Build Targets
build-release:
	@echo "🔨 Building with release profile..."
	cargo build --workspace --profile release

build-ci:
	@echo "🔨 Building with CI profile..."
	cargo build --workspace --profile ci

build-fast-dev:
	@echo "🔨 Building with fast-dev profile..."
	cargo build --workspace --profile fast-dev

# Core Crate Testing
test-api:
	@echo "🧪 Testing riptide-api..."
	cargo test --package riptide-api

test-spider:
	@echo "🧪 Testing riptide-spider..."
	cargo test --package riptide-spider

test-extraction:
	@echo "🧪 Testing riptide-extraction..."
	cargo test --package riptide-extraction

test-streaming:
	@echo "🧪 Testing riptide-streaming..."
	cargo test --package riptide-streaming

test-browser:
	@echo "🧪 Testing riptide-browser..."
	cargo test --package riptide-browser

# Phase 9/10/10.4 Feature Testing
test-cli-commands:
	@echo "🧪 Testing CLI commands..."
	./scripts/test-cli-commands.sh

test-engine-selection:
	@echo "🧪 Testing engine selection..."
	cargo test --package riptide-reliability engine_selection

test-profiles-api:
	@echo "🧪 Testing profiles API..."
	cargo test --package riptide-api profiles

# ============================================================================
# TIER 2: HIGH VALUE TARGETS
# ============================================================================

# Benchmarking
bench:
	@echo "⚡ Running all benchmarks..."
	cargo bench --workspace

bench-pool:
	@echo "⚡ Benchmarking connection pool..."
	cargo bench --package riptide-performance pool_benchmark

bench-wasm:
	@echo "⚡ Benchmarking WASM features..."
	cargo bench --workspace --features wasm-bench

bench-report:
	@echo "⚡ Generating benchmark baseline..."
	cargo bench --workspace -- --save-baseline main

# WASM Builds
build-wasm-optimized:
	@echo "🦀 Building optimized WASM..."
	./scripts/build-wasm-optimized.sh

test-wasm:
	@echo "🧪 Testing WASM extractors..."
	cargo test --package riptide-extractor-wasm

# Developer Workflows
dev-server:
	@echo "🚀 Starting development API server..."
	cargo run --bin riptide-api

quick-start: docker-build-all docker-up
	@echo "✅ Quick start complete - all services running"

smoke-test:
	@echo "🧪 Running smoke tests..."
	./scripts/smoke-test.sh

pre-commit: fmt lint test-unit
	@echo "✅ Pre-commit checks passed"

# Monitoring & Load Testing
monitor-health:
	@echo "📊 Monitoring service health..."
	./scripts/monitor-health.sh

load-test:
	@echo "⚡ Running load tests..."
	./scripts/load-test.sh
