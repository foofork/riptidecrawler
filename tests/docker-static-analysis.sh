#!/bin/bash
# ============================================================================
# Docker Static Analysis - No container execution
# ============================================================================
# Validates Docker configuration without building or running containers
# ============================================================================

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PASSED=0
FAILED=0

echo "Docker Static Analysis Results"
echo "==============================="
echo ""

# Test 1: File existence
echo "1. Required Files Check"
FILES=("Dockerfile" "docker-compose.yml" ".env" "configs/riptide.yml" "scripts/docker/healthcheck.sh")
for f in "${FILES[@]}"; do
    if [ -f "$f" ]; then
        echo "  ✓ $f"
        ((PASSED++))
    else
        echo "  ✗ $f MISSING"
        ((FAILED++))
    fi
done
echo ""

# Test 2: Dockerfile analysis
echo "2. Dockerfile Best Practices"
if grep -q "FROM.*AS" Dockerfile; then
    echo "  ✓ Multi-stage build"
    ((PASSED++))
else
    echo "  ✗ No multi-stage build"
    ((FAILED++))
fi

if grep -q "USER riptide" Dockerfile; then
    echo "  ✓ Non-root user configured"
    ((PASSED++))
else
    echo "  ✗ Running as root"
    ((FAILED++))
fi

if grep -q "HEALTHCHECK" Dockerfile; then
    echo "  ✓ Health check configured"
    ((PASSED++))
else
    echo "  ✗ No health check"
    ((FAILED++))
fi

if grep -q "tini" Dockerfile; then
    echo "  ✓ Init system (tini) configured"
    ((PASSED++))
else
    echo "  ✗ No init system"
    ((FAILED++))
fi
echo ""

# Test 3: docker-compose structure
echo "3. Docker Compose Configuration"
if grep -q "riptide-api:" docker-compose.yml; then
    echo "  ✓ riptide-api service defined"
    ((PASSED++))
else
    echo "  ✗ riptide-api service missing"
    ((FAILED++))
fi

if grep -q "redis:" docker-compose.yml; then
    echo "  ✓ redis service defined"
    ((PASSED++))
else
    echo "  ✗ redis service missing"
    ((FAILED++))
fi

if grep -q "networks:" docker-compose.yml; then
    echo "  ✓ Network configuration present"
    ((PASSED++))
else
    echo "  ✗ No network configuration"
    ((FAILED++))
fi
echo ""

# Test 4: Environment configuration
echo "4. Environment Variables"
if grep -q "REDIS_URL=" .env; then
    echo "  ✓ REDIS_URL configured"
    ((PASSED++))
else
    echo "  ✗ REDIS_URL not set"
    ((FAILED++))
fi

if grep -q "RUST_LOG=" .env; then
    echo "  ✓ Logging configured"
    ((PASSED++))
else
    echo "  ⚠ RUST_LOG not set (default will be used)"
fi
echo ""

# Test 5: Security checks
echo "5. Security Configurations"
if grep -q "REQUIRE_AUTH=" .env; then
    REQUIRE_AUTH=$(grep "REQUIRE_AUTH=" .env | cut -d= -f2)
    if [ "$REQUIRE_AUTH" = "true" ]; then
        echo "  ✓ Authentication required"
        ((PASSED++))
    else
        echo "  ⚠ Authentication disabled"
    fi
else
    echo "  ⚠ Authentication not configured"
fi

if grep -q "API_KEYS=" .env; then
    echo "  ✓ API keys configured"
    ((PASSED++))
else
    echo "  ⚠ No API keys set"
fi
echo ""

# Summary
echo "Summary"
echo "======="
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "✓ Static analysis passed!"
    exit 0
else
    echo "✗ Static analysis found $FAILED issue(s)"
    exit 1
fi
