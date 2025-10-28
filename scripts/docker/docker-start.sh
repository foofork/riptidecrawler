#!/bin/bash
# ============================================================================
# RipTide Docker Quick Start Script
# ============================================================================
# Simplified script to build and start RipTide with Docker Compose
#
# Usage:
#   ./scripts/docker/docker-start.sh [production|dev]
#   ./scripts/docker/docker-start.sh production --build
#   ./scripts/docker/docker-start.sh dev --logs
# ============================================================================

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
MODE="${1:-production}"
SHIFT_ARGS=1

# Parse environment
if [[ "$MODE" != "production" && "$MODE" != "dev" ]]; then
    echo -e "${RED}Invalid mode: ${MODE}${NC}"
    echo "Usage: $0 [production|dev] [--build|--logs|--down]"
    exit 1
fi

shift $SHIFT_ARGS

# Compose file selection
if [ "$MODE" = "production" ]; then
    COMPOSE_FILE="docker-compose.production.yml"
    PROJECT_NAME="riptide-prod"
else
    COMPOSE_FILE="docker-compose.dev.yml"
    PROJECT_NAME="riptide-dev"
fi

cd "$PROJECT_ROOT"

# Header
echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${NC}           ${GREEN}RipTide Docker Quick Start${NC}                    ${CYAN}║${NC}"
echo -e "${CYAN}╠════════════════════════════════════════════════════════════╣${NC}"
echo -e "${CYAN}║${NC}  Mode: ${YELLOW}${MODE}${NC}                                            ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}  Compose: ${COMPOSE_FILE}                ${CYAN}║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# ============================================================================
# Pre-flight Checks
# ============================================================================
echo -e "${BLUE}[1/5]${NC} Checking prerequisites..."

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo -e "${RED}✗ Docker is not installed${NC}"
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo -e "${RED}✗ Docker Compose is not installed${NC}"
    exit 1
fi

# Check if Docker daemon is running
if ! docker info &> /dev/null; then
    echo -e "${RED}✗ Docker daemon is not running${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Docker and Docker Compose are ready${NC}"

# ============================================================================
# Environment Configuration
# ============================================================================
echo -e "${BLUE}[2/5]${NC} Checking environment configuration..."

if [ ! -f ".env" ]; then
    echo -e "${YELLOW}⚠ .env file not found, copying from .env.example${NC}"
    cp .env.example .env
    echo -e "${YELLOW}⚠ Please edit .env and add your API keys${NC}"
    echo -e "${YELLOW}⚠ Press Enter to continue or Ctrl+C to abort...${NC}"
    read
fi

echo -e "${GREEN}✓ Environment configuration found${NC}"

# ============================================================================
# Build Images (if requested or needed)
# ============================================================================
if [[ "$*" == *"--build"* ]] || [ ! "$(docker images -q riptide-api:latest 2> /dev/null)" ]; then
    echo -e "${BLUE}[3/5]${NC} Building Docker images..."

    if [ "$MODE" = "production" ]; then
        echo -e "${CYAN}Building production images (this may take 10-15 minutes)...${NC}"
    else
        echo -e "${CYAN}Building development images...${NC}"
    fi

    docker-compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" build

    echo -e "${GREEN}✓ Images built successfully${NC}"
else
    echo -e "${BLUE}[3/5]${NC} Skipping build (use --build to force rebuild)"
fi

# ============================================================================
# Start Services
# ============================================================================
echo -e "${BLUE}[4/5]${NC} Starting services..."

if [[ "$*" == *"--down"* ]]; then
    echo -e "${YELLOW}Stopping services...${NC}"
    docker-compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" down
    echo -e "${GREEN}✓ Services stopped${NC}"
    exit 0
fi

# Start services
docker-compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" up -d

# Wait for services to be healthy
echo -e "${CYAN}Waiting for services to start...${NC}"
sleep 5

# ============================================================================
# Health Check
# ============================================================================
echo -e "${BLUE}[5/5]${NC} Checking service health..."

MAX_RETRIES=30
RETRY_COUNT=0

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -sf http://localhost:8080/healthz > /dev/null 2>&1; then
        echo -e "${GREEN}✓ RipTide API is healthy${NC}"
        break
    fi

    RETRY_COUNT=$((RETRY_COUNT + 1))
    if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
        echo -e "${RED}✗ API failed to start (check logs with: docker-compose -f $COMPOSE_FILE logs)${NC}"
        exit 1
    fi

    echo -e "${YELLOW}⏳ Waiting for API to be ready... ($RETRY_COUNT/$MAX_RETRIES)${NC}"
    sleep 2
done

# ============================================================================
# Success Summary
# ============================================================================
echo ""
echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${NC}              ${GREEN}✓ RipTide is Running!${NC}                        ${CYAN}║${NC}"
echo -e "${CYAN}╠════════════════════════════════════════════════════════════╣${NC}"
echo -e "${CYAN}║${NC}                                                            ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}  ${GREEN}API:${NC}          http://localhost:8080                    ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}  ${GREEN}Swagger UI:${NC}   http://localhost:8081                    ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}  ${GREEN}Health:${NC}       http://localhost:8080/healthz            ${CYAN}║${NC}"

if [ "$MODE" = "dev" ]; then
    echo -e "${CYAN}║${NC}  ${GREEN}Redis UI:${NC}     http://localhost:8082                    ${CYAN}║${NC}"
fi

echo -e "${CYAN}║${NC}                                                            ${CYAN}║${NC}"
echo -e "${CYAN}╠════════════════════════════════════════════════════════════╣${NC}"
echo -e "${CYAN}║${NC}  Useful Commands:                                          ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}                                                            ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}  View logs:                                                ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}    docker-compose -f $COMPOSE_FILE logs -f      ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}                                                            ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}  Stop services:                                            ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}    docker-compose -f $COMPOSE_FILE down          ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}                                                            ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}  Restart services:                                         ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}    docker-compose -f $COMPOSE_FILE restart       ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}                                                            ${CYAN}║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Show logs if requested
if [[ "$*" == *"--logs"* ]]; then
    echo -e "${CYAN}Following logs (Ctrl+C to exit)...${NC}"
    docker-compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" logs -f
fi
