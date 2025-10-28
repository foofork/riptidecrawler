#!/bin/bash
# ============================================================================
# Docker Pre-Flight Check for Riptide API
# ============================================================================
# Quick validation before running full test suite
# ============================================================================

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

ISSUES_FOUND=0

echo "============================================================================"
echo "Docker Pre-Flight Check"
echo "============================================================================"
echo ""

# Check 1: Required files exist
echo "Checking required files..."
REQUIRED_FILES=(
    "docker-compose.yml"
    ".env"
    "infra/docker/Dockerfile.api"
    "configs/riptide.yml"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo -e "${GREEN}✓${NC} Found: $file"
    else
        echo -e "${RED}✗${NC} Missing: $file"
        ((ISSUES_FOUND++))
    fi
done

echo ""

# Check 2: Docker environment
echo "Checking Docker environment..."
if docker ps >/dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Docker daemon is running"
else
    echo -e "${RED}✗${NC} Docker daemon is not accessible"
    ((ISSUES_FOUND++))
fi

if docker-compose --version >/dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Docker Compose is available"
else
    echo -e "${RED}✗${NC} Docker Compose is not available"
    ((ISSUES_FOUND++))
fi

echo ""

# Check 3: Port availability
echo "Checking port availability..."
REQUIRED_PORTS=(8080 6379 8081)

for port in "${REQUIRED_PORTS[@]}"; do
    if ! lsof -i ":$port" >/dev/null 2>&1 && ! netstat -tuln 2>/dev/null | grep -q ":$port "; then
        echo -e "${GREEN}✓${NC} Port $port is available"
    else
        echo -e "${YELLOW}⚠${NC} Port $port may be in use"
    fi
done

echo ""

# Check 4: Disk space
echo "Checking disk space..."
AVAILABLE_SPACE=$(df -BG . | tail -1 | awk '{print $4}' | sed 's/G//')
if [ "$AVAILABLE_SPACE" -gt 10 ]; then
    echo -e "${GREEN}✓${NC} Sufficient disk space: ${AVAILABLE_SPACE}GB available"
else
    echo -e "${YELLOW}⚠${NC} Low disk space: ${AVAILABLE_SPACE}GB available (10GB+ recommended)"
fi

echo ""

# Check 5: Environment variables
echo "Checking critical environment variables..."
if grep -q "REDIS_URL=" .env 2>/dev/null; then
    echo -e "${GREEN}✓${NC} REDIS_URL is configured"
else
    echo -e "${YELLOW}⚠${NC} REDIS_URL not configured in .env"
fi

echo ""

# Summary
echo "============================================================================"
if [ $ISSUES_FOUND -eq 0 ]; then
    echo -e "${GREEN}Pre-flight check passed! Ready to run full test suite.${NC}"
    exit 0
else
    echo -e "${YELLOW}Pre-flight check completed with $ISSUES_FOUND issue(s).${NC}"
    echo "Some tests may fail. Review issues above."
    exit 1
fi
