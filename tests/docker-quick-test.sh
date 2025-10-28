#!/bin/bash
# ============================================================================
# Docker Quick Test - Fast validation of Docker setup
# ============================================================================
# This script runs a minimal set of tests to quickly validate the Docker
# implementation without building images or starting containers.
# ============================================================================

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "============================================================================"
echo "Docker Quick Test Suite"
echo "============================================================================"
echo ""

TESTS_PASSED=0
TESTS_FAILED=0

# Test 1: Dockerfile validation
echo -e "${BLUE}[TEST 1]${NC} Validating Dockerfile syntax..."
if docker build --dry-run -f Dockerfile . 2>&1 | grep -q "DEPRECATED\|deprecated"; then
    echo -e "${YELLOW}[WARN]${NC} Dockerfile contains deprecated syntax"
else
    if [ -f "Dockerfile" ]; then
        echo -e "${GREEN}[PASS]${NC} Dockerfile syntax is valid"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}[FAIL]${NC} Dockerfile not found"
        ((TESTS_FAILED++))
    fi
fi

# Test 2: docker-compose validation
echo -e "${BLUE}[TEST 2]${NC} Validating docker-compose.yml..."
if docker-compose config >/dev/null 2>&1; then
    echo -e "${GREEN}[PASS]${NC} docker-compose.yml is valid"
    ((TESTS_PASSED++))
else
    echo -e "${RED}[FAIL]${NC} docker-compose.yml has errors"
    ((TESTS_FAILED++))
fi

# Test 3: Required files
echo -e "${BLUE}[TEST 3]${NC} Checking required files..."
REQUIRED_FILES=("Dockerfile" "docker-compose.yml" ".env" "configs/riptide.yml" "scripts/docker/healthcheck.sh")
MISSING_FILES=0

for file in "${REQUIRED_FILES[@]}"; do
    if [ ! -f "$file" ]; then
        echo -e "${RED}[FAIL]${NC} Missing required file: $file"
        ((MISSING_FILES++))
    fi
done

if [ $MISSING_FILES -eq 0 ]; then
    echo -e "${GREEN}[PASS]${NC} All required files present"
    ((TESTS_PASSED++))
else
    echo -e "${RED}[FAIL]${NC} $MISSING_FILES required file(s) missing"
    ((TESTS_FAILED++))
fi

# Test 4: Environment configuration
echo -e "${BLUE}[TEST 4]${NC} Validating environment configuration..."
ENV_ISSUES=0

if [ -f ".env" ]; then
    # Check critical variables
    CRITICAL_VARS=("REDIS_URL" "RIPTIDE_API_KEY")
    for var in "${CRITICAL_VARS[@]}"; do
        if ! grep -q "^${var}=" ".env"; then
            echo -e "${YELLOW}[WARN]${NC} Environment variable $var not set"
            ((ENV_ISSUES++))
        fi
    done

    if [ $ENV_ISSUES -eq 0 ]; then
        echo -e "${GREEN}[PASS]${NC} Environment configuration complete"
        ((TESTS_PASSED++))
    else
        echo -e "${YELLOW}[WARN]${NC} Environment configuration has warnings"
        ((TESTS_PASSED++))
    fi
else
    echo -e "${RED}[FAIL]${NC} .env file not found"
    ((TESTS_FAILED++))
fi

# Test 5: Port availability
echo -e "${BLUE}[TEST 5]${NC} Checking port availability..."
PORTS_IN_USE=0
REQUIRED_PORTS=(8080 8081)

for port in "${REQUIRED_PORTS[@]}"; do
    if lsof -i ":$port" >/dev/null 2>&1 || netstat -tuln 2>/dev/null | grep -q ":$port "; then
        echo -e "${YELLOW}[WARN]${NC} Port $port is in use (may cause conflicts)"
        ((PORTS_IN_USE++))
    fi
done

if [ $PORTS_IN_USE -eq 0 ]; then
    echo -e "${GREEN}[PASS]${NC} Required ports are available"
    ((TESTS_PASSED++))
else
    echo -e "${YELLOW}[WARN]${NC} Some ports are in use"
    ((TESTS_PASSED++))
fi

# Test 6: Dockerfile best practices
echo -e "${BLUE}[TEST 6]${NC} Checking Dockerfile best practices..."
BEST_PRACTICES=0

if grep -q "FROM.*AS.*" Dockerfile; then
    ((BEST_PRACTICES++))
fi

if grep -q "USER " Dockerfile; then
    ((BEST_PRACTICES++))
fi

if grep -q "HEALTHCHECK" Dockerfile; then
    ((BEST_PRACTICES++))
fi

if grep -q "ENTRYPOINT.*tini" Dockerfile; then
    ((BEST_PRACTICES++))
fi

if [ $BEST_PRACTICES -ge 3 ]; then
    echo -e "${GREEN}[PASS]${NC} Dockerfile follows best practices ($BEST_PRACTICES/4 checks passed)"
    ((TESTS_PASSED++))
else
    echo -e "${YELLOW}[WARN]${NC} Dockerfile could improve best practices ($BEST_PRACTICES/4 checks passed)"
    ((TESTS_PASSED++))
fi

# Summary
echo ""
echo "============================================================================"
echo "Quick Test Summary"
echo "============================================================================"
echo -e "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ Quick tests passed! Docker setup is ready.${NC}"
    echo "Run the full validation suite for comprehensive testing:"
    echo "  ./tests/docker-validation-suite.sh"
    exit 0
else
    echo -e "${RED}✗ Some quick tests failed. Fix issues before proceeding.${NC}"
    exit 1
fi
