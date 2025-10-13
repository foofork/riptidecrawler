#!/bin/bash
set -e

echo "🔍 RipTide Week 2 Phase 2A - Monitoring Stack Validation"
echo "========================================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASSED=0
FAILED=0

run_test() {
    local test_name="$1"
    local test_cmd="$2"

    echo -n "Testing: $test_name... "
    if eval "$test_cmd" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((PASSED++))
        return 0
    else
        echo -e "${RED}✗ FAIL${NC}"
        ((FAILED++))
        return 1
    fi
}

# Test 1: API Metrics Endpoint
echo -n "1. API metrics endpoint (localhost:8080/metrics)... "
METRIC_COUNT=$(curl -s http://localhost:8080/metrics 2>/dev/null | grep "^riptide_" | wc -l)
if [ "$METRIC_COUNT" -ge 30 ]; then
    echo -e "${GREEN}✓ PASS${NC} (${METRIC_COUNT} metrics)"
    ((PASSED++))
else
    echo -e "${RED}✗ FAIL${NC} (only ${METRIC_COUNT} metrics)"
    ((FAILED++))
fi

# Test 2-8: Individual component tests
run_test "2. Prometheus health (localhost:9090)" "curl -sf http://localhost:9090/-/healthy"

echo -n "3. Prometheus scraping RipTide API... "
TARGET_HEALTH=$(curl -s http://localhost:9090/api/v1/targets 2>/dev/null | jq -r '.data.activeTargets[] | select(.labels.job == "riptide-api") | .health')
if [ "$TARGET_HEALTH" = "up" ]; then
    echo -e "${GREEN}✓ PASS${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ FAIL${NC} (status: $TARGET_HEALTH)"
    ((FAILED++))
fi

echo -n "4. Query RipTide metrics from Prometheus... "
PROM_COUNT=$(curl -s 'http://localhost:9090/api/v1/label/__name__/values' 2>/dev/null | jq -r '.data[] | select(startswith("riptide_"))' | wc -l)
if [ "$PROM_COUNT" -ge 20 ]; then
    echo -e "${GREEN}✓ PASS${NC} (${PROM_COUNT} metric series)"
    ((PASSED++))
else
    echo -e "${RED}✗ FAIL${NC} (only ${PROM_COUNT} series)"
    ((FAILED++))
fi

run_test "5. Grafana health (localhost:3000)" "curl -sf http://localhost:3000/api/health"
run_test "6. AlertManager health (localhost:9093)" "curl -sf http://localhost:9093/-/healthy"
run_test "7. Node Exporter metrics (localhost:9100)" "curl -s http://localhost:9100/metrics | grep -q '^node_'"

# Test 8: Docker containers
echo -n "8. All monitoring containers running... "
EXPECTED_CONTAINERS=("riptide-prometheus" "riptide-grafana" "riptide-alertmanager" "riptide-node-exporter")
ALL_RUNNING=true
for container in "${EXPECTED_CONTAINERS[@]}"; do
    if ! docker ps --format '{{.Names}}' | grep -q "^${container}$"; then
        ALL_RUNNING=false
        break
    fi
done

if $ALL_RUNNING; then
    echo -e "${GREEN}✓ PASS${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ FAIL${NC}"
    ((FAILED++))
fi

echo ""
echo "========================================================="
echo -e "Results: ${GREEN}${PASSED} passed${NC}, ${RED}${FAILED} failed${NC}"
echo "========================================================="
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ Week 2 Phase 2A: COMPLETE${NC}"
    echo ""
    echo "📊 Monitoring Stack Endpoints:"
    echo "  • Prometheus:  http://localhost:9090"
    echo "  • Grafana:     http://localhost:3000 (admin/riptide_admin_change_me)"
    echo "  • AlertMgr:    http://localhost:9093"
    echo "  • Node Exp:    http://localhost:9100/metrics"
    echo "  • RipTide API: http://localhost:8080/metrics"
    echo ""
    echo "📈 Next Steps (Week 2 Phase 2B):"
    echo "  1. Configure Grafana dashboards (Overview, Gate Analysis, Performance, Quality)"
    echo "  2. Setup 8 alert rules in AlertManager"
    echo "  3. Test alert notifications"
    echo "  4. Document runbook for monitoring operations"
    echo ""
    exit 0
else
    echo -e "${RED}❌ Validation failed - please fix errors above${NC}"
    exit 1
fi
