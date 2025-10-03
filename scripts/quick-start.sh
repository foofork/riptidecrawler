#!/bin/bash
set -e

# RipTide Quick Start Script
# This script automates the setup and deployment of RipTide API

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🌊 RipTide Quick Start${NC}"
echo "================================"
echo ""

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker is not installed. Please install Docker first.${NC}"
    echo "Visit: https://docs.docker.com/get-docker/"
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo -e "${RED}❌ Docker Compose is not installed. Please install Docker Compose first.${NC}"
    echo "Visit: https://docs.docker.com/compose/install/"
    exit 1
fi

# Determine docker compose command
if docker compose version &> /dev/null 2>&1; then
    DOCKER_COMPOSE="docker compose"
else
    DOCKER_COMPOSE="docker-compose"
fi

echo -e "${GREEN}✅ Docker and Docker Compose are installed${NC}"
echo ""

# Check if .env file exists
if [ ! -f .env ]; then
    echo -e "${YELLOW}⚠️  No .env file found. Creating from .env.example...${NC}"
    if [ -f .env.example ]; then
        cp .env.example .env
        echo -e "${GREEN}✅ Created .env file${NC}"
        echo -e "${YELLOW}📝 Please edit .env and add your API keys if needed${NC}"
    else
        echo -e "${YELLOW}⚠️  No .env.example found. Creating minimal .env...${NC}"
        cat > .env << EOF
# RipTide Configuration
REDIS_URL=redis://redis:6379
WASM_PATH=/opt/riptide/extractor/extractor.wasm
RUST_LOG=info

# Optional: Add your API keys here
# SERPER_API_KEY=your_key_here
# OPENAI_API_KEY=your_key_here
EOF
        echo -e "${GREEN}✅ Created minimal .env file${NC}"
    fi
    echo ""
fi

# Start services
echo -e "${BLUE}🚀 Starting RipTide services...${NC}"
$DOCKER_COMPOSE up -d

echo ""
echo -e "${YELLOW}⏳ Waiting for RipTide to be ready...${NC}"

# Wait for health check
MAX_ATTEMPTS=60
ATTEMPT=0
until curl -sf http://localhost:8080/healthz > /dev/null 2>&1; do
    ATTEMPT=$((ATTEMPT + 1))
    if [ $ATTEMPT -ge $MAX_ATTEMPTS ]; then
        echo -e "${RED}❌ Timeout waiting for RipTide to start${NC}"
        echo "Check logs with: $DOCKER_COMPOSE logs"
        exit 1
    fi
    echo -n "."
    sleep 2
done

echo ""
echo ""
echo -e "${GREEN}✅ RipTide is ready!${NC}"
echo ""
echo "================================"
echo -e "${BLUE}📍 Access Points:${NC}"
echo ""
echo -e "  🌊 API:          ${GREEN}http://localhost:8080${NC}"
echo -e "  📚 Swagger UI:   ${GREEN}http://localhost:8081${NC}"
echo -e "  ❤️  Health Check: ${GREEN}http://localhost:8080/healthz${NC}"
echo ""
echo "================================"
echo -e "${BLUE}🧪 Quick Test:${NC}"
echo ""
echo -e "  ${YELLOW}curl http://localhost:8080/healthz${NC}"
echo ""
echo -e "  ${YELLOW}curl -X POST http://localhost:8080/crawl \\${NC}"
echo -e "    ${YELLOW}-H 'Content-Type: application/json' \\${NC}"
echo -e "    ${YELLOW}-d '{\"urls\":[\"https://example.com\"]}'${NC}"
echo ""
echo "================================"
echo -e "${BLUE}📖 Next Steps:${NC}"
echo ""
echo "  • View API docs:    Open http://localhost:8081 in your browser"
echo "  • Run test script:  ./scripts/test-riptide.sh"
echo "  • View logs:        $DOCKER_COMPOSE logs -f"
echo "  • Stop services:    $DOCKER_COMPOSE down"
echo ""
echo -e "${GREEN}Happy crawling! 🌊${NC}"
