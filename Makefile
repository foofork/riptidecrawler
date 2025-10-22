.PHONY: help ci ci-quick install-tools fmt lint test test-unit test-integration build build-wasm audit clean coverage coverage-html coverage-lcov coverage-json coverage-open coverage-report

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
