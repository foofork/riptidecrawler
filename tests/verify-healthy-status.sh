#!/bin/bash
# Quick verification that all services are healthy

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo "Checking service health..."
HEALTH=$(curl -s http://localhost:8080/api/health/detailed | jq -r '.status')

if [ "$HEALTH" = "healthy" ]; then
    echo -e "${GREEN}✓ Status: HEALTHY${NC}"

    # Show all component statuses
    echo ""
    echo "Component Status:"
    curl -s http://localhost:8080/api/health/detailed | jq -r '.dependencies | to_entries[] | "\(.key): \(.value.status)"'

    exit 0
else
    echo -e "${RED}✗ Status: $HEALTH${NC}"
    echo ""
    echo "Component Status:"
    curl -s http://localhost:8080/api/health/detailed | jq '.dependencies'
    exit 1
fi
