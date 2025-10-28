#!/bin/bash
# ============================================================================
# Docker Post-Build Quick Test
# ============================================================================
# Fast validation after rebuilding Docker images
# ============================================================================

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}============================================================================${NC}"
echo -e "${BLUE}Docker Post-Build Quick Test${NC}"
echo -e "${BLUE}============================================================================${NC}"
echo ""

# Test 1: Verify healthcheck script exists in container
echo -e "${BLUE}[TEST 1]${NC} Checking healthcheck script in container..."
if docker exec riptide-api test -f /usr/local/bin/healthcheck; then
    echo -e "${GREEN}[PASS]${NC} Healthcheck script exists"
else
    echo -e "${RED}[FAIL]${NC} Healthcheck script not found"
    exit 1
fi

# Test 2: Verify healthcheck script is executable
echo -e "${BLUE}[TEST 2]${NC} Checking healthcheck script permissions..."
if docker exec riptide-api test -x /usr/local/bin/healthcheck; then
    echo -e "${GREEN}[PASS]${NC} Healthcheck script is executable"
else
    echo -e "${RED}[FAIL]${NC} Healthcheck script not executable"
    exit 1
fi

# Test 3: Verify curl is installed
echo -e "${BLUE}[TEST 3]${NC} Checking curl installation..."
if docker exec riptide-api which curl >/dev/null 2>&1; then
    CURL_VERSION=$(docker exec riptide-api curl --version | head -1)
    echo -e "${GREEN}[PASS]${NC} curl is installed: $CURL_VERSION"
else
    echo -e "${RED}[FAIL]${NC} curl is not installed"
    exit 1
fi

# Test 4: Test healthcheck script execution
echo -e "${BLUE}[TEST 4]${NC} Testing healthcheck script execution..."
if docker exec riptide-api /usr/local/bin/healthcheck; then
    echo -e "${GREEN}[PASS]${NC} Healthcheck script executes successfully"
else
    echo -e "${YELLOW}[WARN]${NC} Healthcheck script returned non-zero (API may still be starting)"
fi

# Test 5: Test /healthz endpoint directly
echo -e "${BLUE}[TEST 5]${NC} Testing /healthz endpoint..."
if docker exec riptide-api curl -f -s http://localhost:8080/healthz >/dev/null 2>&1; then
    HEALTH_RESPONSE=$(docker exec riptide-api curl -s http://localhost:8080/healthz)
    echo -e "${GREEN}[PASS]${NC} /healthz endpoint responding"
    echo -e "        Response: $(echo $HEALTH_RESPONSE | head -c 100)..."
else
    echo -e "${YELLOW}[WARN]${NC} /healthz endpoint not yet responding (may still be starting)"
fi

# Test 6: Check Docker health status
echo -e "${BLUE}[TEST 6]${NC} Checking Docker health status..."
HEALTH_STATUS=$(docker inspect riptide-api --format='{{.State.Health.Status}}' 2>/dev/null || echo "unknown")
if [ "$HEALTH_STATUS" = "healthy" ]; then
    echo -e "${GREEN}[PASS]${NC} Container is healthy"
elif [ "$HEALTH_STATUS" = "starting" ]; then
    echo -e "${YELLOW}[WARN]${NC} Container is still starting (health check in progress)"
else
    echo -e "${RED}[INFO]${NC} Container health status: $HEALTH_STATUS"
fi

# Test 7: Check for health check errors in logs
echo -e "${BLUE}[TEST 7]${NC} Checking recent container logs..."
if docker logs riptide-api --tail 20 2>&1 | grep -qi "error\|panic\|fatal"; then
    echo -e "${YELLOW}[WARN]${NC} Found error messages in logs:"
    docker logs riptide-api --tail 20 2>&1 | grep -i "error\|panic\|fatal" | head -5
else
    echo -e "${GREEN}[PASS]${NC} No critical errors in recent logs"
fi

echo ""
echo -e "${BLUE}============================================================================${NC}"
echo -e "${GREEN}Post-Build Quick Test Complete${NC}"
echo -e "${BLUE}============================================================================${NC}"
echo ""
echo -e "Container Status:"
docker ps | grep riptide-api || echo "Container not running"
echo ""
echo -e "Next Steps:"
echo -e "  1. Wait 30 seconds for full startup if health is 'starting'"
echo -e "  2. Run: curl http://localhost:8080/healthz | jq"
echo -e "  3. Run full validation: ./tests/docker-validation-suite.sh"
echo ""
