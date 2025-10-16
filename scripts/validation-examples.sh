#!/bin/bash
# RipTide Validation and System Check - Usage Examples
# This script demonstrates various ways to use the validation commands

set -e

echo "========================================="
echo "RipTide Validation Examples"
echo "========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to run and display command
run_example() {
    local description=$1
    local command=$2

    echo -e "${GREEN}Example: ${description}${NC}"
    echo -e "${YELLOW}Command: ${command}${NC}"
    echo "---"

    if eval "$command"; then
        echo -e "${GREEN}✓ Success${NC}"
    else
        echo -e "${RED}✗ Failed (exit code: $?)${NC}"
    fi

    echo ""
    echo "========================================="
    echo ""
}

# 1. Basic validation
run_example "Basic validation check" \
    "cargo run --bin riptide -- validate"

# 2. Comprehensive validation
run_example "Comprehensive validation with all checks" \
    "cargo run --bin riptide -- validate --comprehensive"

# 3. WASM-only check
run_example "Check WASM setup only" \
    "cargo run --bin riptide -- validate --wasm"

# 4. JSON output
run_example "Validation with JSON output (for CI/CD)" \
    "cargo run --bin riptide -- validate --comprehensive --format json"

# 5. Custom WASM path
run_example "Validation with custom WASM path" \
    "cargo run --bin riptide -- validate --wasm-path ./wasm/riptide-extractor-wasm/pkg/riptide_extractor_wasm_bg.wasm"

# 6. System health check
run_example "Standard system health check" \
    "cargo run --bin riptide -- system check"

# 7. Production readiness
run_example "Production readiness check (strict)" \
    "cargo run --bin riptide -- system check --production"

# 8. Performance profiling
run_example "Performance baseline profiling" \
    "cargo run --bin riptide -- system profile"

# 9. System check with JSON output
run_example "System check with JSON output" \
    "cargo run --bin riptide -- system check --format json"

echo ""
echo "========================================="
echo "CI/CD Integration Examples"
echo "========================================="
echo ""

# CI/CD Example 1: Parse JSON output
echo -e "${GREEN}Example: Parse JSON validation results${NC}"
echo -e "${YELLOW}Command:${NC}"
cat << 'EOF'
cargo run --bin riptide -- validate --comprehensive --format json > validation.json
failed_count=$(jq -r '.summary.failed' validation.json)
if [ "$failed_count" -gt 0 ]; then
    echo "Validation failed: $failed_count checks failed"
    exit 1
fi
echo "All validation checks passed"
EOF
echo ""

# CI/CD Example 2: Production gate
echo -e "${GREEN}Example: Production deployment gate${NC}"
echo -e "${YELLOW}Command:${NC}"
cat << 'EOF'
if cargo run --bin riptide -- system check --production; then
    echo "System is production ready"
    # Proceed with deployment
else
    echo "System is NOT production ready - deployment blocked"
    exit 1
fi
EOF
echo ""

# CI/CD Example 3: Save reports as artifacts
echo -e "${GREEN}Example: Generate validation reports${NC}"
echo -e "${YELLOW}Command:${NC}"
cat << 'EOF'
# Generate validation report
cargo run --bin riptide -- validate --comprehensive --format json > reports/validation.json

# Generate system check report
cargo run --bin riptide -- system check --format json > reports/system-check.json

# Generate performance baseline
cargo run --bin riptide -- system profile --format json > reports/performance-baseline.json
EOF
echo ""

echo "========================================="
echo "Troubleshooting Examples"
echo "========================================="
echo ""

# Troubleshooting 1: Build WASM module
echo -e "${GREEN}Example: Fix missing WASM module${NC}"
echo -e "${YELLOW}Command:${NC}"
cat << 'EOF'
cd wasm/riptide-extractor-wasm
wasm-pack build --target web
cd ../..
cargo run --bin riptide -- validate --wasm
EOF
echo ""

# Troubleshooting 2: Start Redis
echo -e "${GREEN}Example: Fix Redis connection${NC}"
echo -e "${YELLOW}Command:${NC}"
cat << 'EOF'
# Using Docker
docker run -d -p 6379:6379 redis:latest

# Or using local Redis
sudo systemctl start redis-server

# Verify
redis-cli ping

# Run validation
cargo run --bin riptide -- validate
EOF
echo ""

# Troubleshooting 3: Check API
echo -e "${GREEN}Example: Fix API connectivity${NC}"
echo -e "${YELLOW}Command:${NC}"
cat << 'EOF'
# Start API server in background
cargo run --bin riptide-api &

# Wait for startup
sleep 5

# Check health
curl http://localhost:8080/healthz

# Run validation
cargo run --bin riptide -- validate
EOF
echo ""

echo "========================================="
echo "Advanced Usage Examples"
echo "========================================="
echo ""

# Advanced 1: Complete pre-flight check
echo -e "${GREEN}Example: Complete pre-flight check script${NC}"
echo -e "${YELLOW}Script:${NC}"
cat << 'EOF'
#!/bin/bash
set -e

echo "Starting RipTide pre-flight checks..."

# 1. Validate WASM
echo "Checking WASM module..."
cargo run --bin riptide -- validate --wasm

# 2. Check dependencies
echo "Checking system dependencies..."
command -v redis-cli >/dev/null 2>&1 || echo "Warning: redis-cli not found"
command -v chromium >/dev/null 2>&1 || command -v google-chrome >/dev/null 2>&1 || echo "Warning: Chrome/Chromium not found"

# 3. Comprehensive validation
echo "Running comprehensive validation..."
cargo run --bin riptide -- validate --comprehensive --format json > validation.json

# 4. Check results
failed=$(jq -r '.summary.failed' validation.json)
warnings=$(jq -r '.summary.warnings' validation.json)

echo "Validation complete: $failed failures, $warnings warnings"

if [ "$failed" -gt 0 ]; then
    echo "Pre-flight checks FAILED"
    jq -r '.checks[] | select(.status == "Fail") | "\(.name): \(.message)"' validation.json
    exit 1
fi

echo "Pre-flight checks PASSED"
EOF
echo ""

# Advanced 2: Monitoring script
echo -e "${GREEN}Example: System monitoring script${NC}"
echo -e "${YELLOW}Script:${NC}"
cat << 'EOF'
#!/bin/bash

while true; do
    # Run system check
    cargo run --bin riptide -- system check --format json > /tmp/health.json

    # Extract metrics
    timestamp=$(jq -r '.timestamp' /tmp/health.json)
    passed=$(jq -r '.summary.passed' /tmp/health.json)
    failed=$(jq -r '.summary.failed' /tmp/health.json)

    echo "[$timestamp] System Health: $passed passed, $failed failed"

    if [ "$failed" -gt 0 ]; then
        echo "WARNING: System health degraded"
        # Send alert
    fi

    # Check every 5 minutes
    sleep 300
done
EOF
echo ""

echo "========================================="
echo "Environment Variable Examples"
echo "========================================="
echo ""

echo -e "${GREEN}Example: Using environment variables${NC}"
echo -e "${YELLOW}Commands:${NC}"
cat << 'EOF'
# Set WASM path
export RIPTIDE_WASM_PATH=/path/to/module.wasm

# Set API URL
export RIPTIDE_API_URL=http://localhost:8080

# Set cache directory
export RIPTIDE_CACHE_DIR=/tmp/riptide-cache

# Set log level
export RUST_LOG=debug

# Run validation
cargo run --bin riptide -- validate --comprehensive
EOF
echo ""

echo "========================================="
echo "All examples completed!"
echo "========================================="
