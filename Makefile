.PHONY: help ci ci-quick install-tools fmt lint test test-unit test-integration build build-wasm audit clean

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
	@echo "Setup:"
	@echo "  make install-tools - Install cargo-deny, cargo-audit"
	@echo ""
	@echo "See docs/LOCAL_CI.md for full workflow guide"

ci-quick:
	@./scripts/ci-quick.sh

ci:
	@./scripts/ci-local.sh

install-tools:
	@echo "📦 Installing CI tools..."
	@cargo install cargo-deny cargo-audit --locked

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
