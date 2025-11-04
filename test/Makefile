.PHONY: fixtures-up fixtures-down help

help:  ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

fixtures-up:  ## Start local test fixtures (optional)
	@echo "╔═══════════════════════════════════════════════════════════════╗"
	@echo "║  Starting test fixtures (Docker Compose - OPTIONAL)          ║"
	@echo "╚═══════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "ℹ️  Note: This is optional for development"
	@echo "   CI uses recorded fixtures instead of live Docker services"
	@echo ""
	@echo "⚠️  Docker Compose configuration not yet implemented"
	@echo "   For now, CI/tests use wiremock recorded responses"
	@echo ""
	@echo "See: /workspaces/eventmesh/docs/development/TEST-FIXTURES.md"

fixtures-down:  ## Stop local test fixtures
	@echo "╔═══════════════════════════════════════════════════════════════╗"
	@echo "║  Stopping test fixtures                                       ║"
	@echo "╚═══════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "⚠️  Docker Compose configuration not yet implemented"
	@echo "   No fixtures to stop (using recorded responses)"
