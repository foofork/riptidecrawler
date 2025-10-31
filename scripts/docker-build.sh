#!/bin/bash
# Docker Build Script for RipTide API
# Supports building native-only or WASM-enabled images

set -e

MODE="${1:-native}"
DOCKERFILE="${2:-infra/docker/Dockerfile.api}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Banner
echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         RipTide Docker Image Builder                  ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Function to build native image
build_native() {
    echo -e "${GREEN}🏗️  Building native-only image (faster, smaller)...${NC}"
    echo -e "${YELLOW}   • No WASM dependency${NC}"
    echo -e "${YELLOW}   • ~200MB total size${NC}"
    echo -e "${YELLOW}   • ~5 minute build time${NC}"
    echo ""

    docker build \
      --build-arg ENABLE_WASM=false \
      -t riptide-api:native \
      -t riptide-api:latest \
      -f "$DOCKERFILE" \
      .

    echo ""
    echo -e "${GREEN}✅ Built: riptide-api:native${NC}"
    docker images riptide-api:native --format "   Size: {{.Size}} | Created: {{.CreatedSince}}"
}

# Function to build WASM image
build_wasm() {
    echo -e "${GREEN}🏗️  Building WASM-enabled image (slower, larger)...${NC}"
    echo -e "${YELLOW}   • Includes WASM runtime${NC}"
    echo -e "${YELLOW}   • ~350MB total size (+75%)${NC}"
    echo -e "${YELLOW}   • ~8 minute build time (+60%)${NC}"
    echo ""

    docker build \
      --build-arg ENABLE_WASM=true \
      -t riptide-api:wasm \
      -f "$DOCKERFILE" \
      .

    echo ""
    echo -e "${GREEN}✅ Built: riptide-api:wasm${NC}"
    docker images riptide-api:wasm --format "   Size: {{.Size}} | Created: {{.CreatedSince}}"
}

# Function to show comparison
show_comparison() {
    echo ""
    echo -e "${BLUE}📊 Image Size Comparison:${NC}"
    echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
    docker images riptide-api --format "table {{.Repository}}:{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}" | \
        grep -E "REPOSITORY|riptide-api"
    echo ""

    # Calculate size difference
    NATIVE_SIZE=$(docker images riptide-api:native --format "{{.Size}}" | head -1)
    WASM_SIZE=$(docker images riptide-api:wasm --format "{{.Size}}" | head -1)

    echo -e "${YELLOW}Native variant:${NC} Recommended for 99% of deployments"
    echo -e "${YELLOW}WASM variant:${NC}   For security-critical applications only"
    echo ""
}

# Main script logic
case "$MODE" in
  native)
    build_native
    echo ""
    echo -e "${GREEN}🚀 Quick Start:${NC}"
    echo -e "   ${BLUE}docker run -d -p 8080:8080 -e REDIS_URL=redis://redis:6379 riptide-api:native${NC}"
    echo ""
    echo -e "${YELLOW}💡 Tip:${NC} This is the recommended variant for most users"
    echo -e "   See docs/FEATURES.md for feature comparison"
    ;;

  wasm)
    build_wasm
    echo ""
    echo -e "${GREEN}🚀 Quick Start:${NC}"
    echo -e "   ${BLUE}docker run -d -p 8080:8080 -e REDIS_URL=redis://redis:6379 \\${NC}"
    echo -e "   ${BLUE}  -e WASM_EXTRACTOR_PATH=/opt/riptide/extractor/extractor.wasm \\${NC}"
    echo -e "   ${BLUE}  riptide-api:wasm${NC}"
    echo ""
    echo -e "${YELLOW}⚠️  Note:${NC} WASM variant is 4x slower than native"
    echo -e "   Use only for security-critical applications"
    ;;

  both)
    echo -e "${BLUE}Building both image variants...${NC}"
    echo ""
    build_native
    echo ""
    echo -e "${BLUE}────────────────────────────────────────────────────────${NC}"
    echo ""
    build_wasm
    show_comparison

    echo -e "${GREEN}🎯 Recommendation:${NC}"
    echo -e "   • ${YELLOW}Use native${NC} for production (faster, smaller)"
    echo -e "   • ${YELLOW}Use WASM${NC} only for untrusted HTML sources"
    echo ""
    echo -e "${BLUE}📚 Documentation:${NC}"
    echo -e "   • Features: ${YELLOW}docs/FEATURES.md${NC}"
    echo -e "   • Docker:   ${YELLOW}docs/DOCKER.md${NC}"
    ;;

  test)
    echo -e "${YELLOW}🧪 Building test image (native, no optimizations)...${NC}"
    docker build \
      --build-arg ENABLE_WASM=false \
      --target builder \
      -t riptide-api:test \
      -f "$DOCKERFILE" \
      .
    echo -e "${GREEN}✅ Test image built${NC}"
    ;;

  *)
    echo -e "${RED}❌ Invalid mode: $MODE${NC}"
    echo ""
    echo -e "${YELLOW}Usage:${NC} $0 {native|wasm|both|test} [dockerfile]"
    echo ""
    echo -e "${BLUE}Modes:${NC}"
    echo -e "  ${GREEN}native${NC}  - Build native-only image (default, recommended)"
    echo -e "            • Fast extraction (2-5ms)"
    echo -e "            • Small size (~200MB)"
    echo -e "            • Quick build (~5min)"
    echo ""
    echo -e "  ${GREEN}wasm${NC}    - Build WASM-enabled image (specialized)"
    echo -e "            • Sandboxed extraction (10-20ms)"
    echo -e "            • Larger size (~350MB)"
    echo -e "            • Slower build (~8min)"
    echo ""
    echo -e "  ${GREEN}both${NC}    - Build both variants and show comparison"
    echo -e "            • Useful for benchmarking"
    echo ""
    echo -e "  ${GREEN}test${NC}    - Build test/debug image"
    echo ""
    echo -e "${BLUE}Examples:${NC}"
    echo -e "  ${YELLOW}$0 native${NC}                    # Build native image"
    echo -e "  ${YELLOW}$0 wasm${NC}                      # Build WASM image"
    echo -e "  ${YELLOW}$0 both${NC}                      # Build both and compare"
    echo -e "  ${YELLOW}$0 native Dockerfile.custom${NC}  # Use custom Dockerfile"
    echo ""
    echo -e "${BLUE}See also:${NC}"
    echo -e "  • ${YELLOW}docs/FEATURES.md${NC} - Feature comparison"
    echo -e "  • ${YELLOW}docs/DOCKER.md${NC}   - Docker deployment guide"
    exit 1
    ;;
esac

echo ""
echo -e "${GREEN}════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Build complete! ✨${NC}"
echo -e "${GREEN}════════════════════════════════════════════════════════${NC}"
