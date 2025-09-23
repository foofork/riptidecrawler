.PHONY: help setup build test check clean run dev

help:
	@echo "Available commands:"
	@echo "  make setup  - Setup development environment"
	@echo "  make build  - Fast incremental build"
	@echo "  make test   - Run tests in parallel"
	@echo "  make check  - Run quality checks"
	@echo "  make clean  - Clean build artifacts and ports"
	@echo "  make run    - Run the application"
	@echo "  make dev    - Run with hot-reload"
	@echo "  make all    - Build, test, and check"

setup:
	@./scripts/dev-setup.sh

build:
	@./scripts/quick-build.sh

test:
	@./scripts/fast-test.sh

check:
	@./scripts/quality-check.sh

clean:
	@echo "🧹 Cleaning..."
	@cargo clean
	@./scripts/dev-setup.sh

run: build
	@cargo run --bin riptide-api

dev:
	@./scripts/dev-run.sh

all: build test check
	@echo "✅ All checks passed!"