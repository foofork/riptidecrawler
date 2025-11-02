#!/bin/bash
# Smart Cleanup - Intelligent disk space management
# This script uses heuristics to decide what to clean based on:
# - Last access time
# - File age
# - Build frequency
# - Available disk space

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

WORKSPACE_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
DRY_RUN=false

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --dry-run) DRY_RUN=true; shift ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

print_header() {
  echo -e "${BLUE}========================================${NC}"
  echo -e "${BLUE}$1${NC}"
  echo -e "${BLUE}========================================${NC}"
}

print_info() {
  echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
  echo -e "${YELLOW}⚠${NC} $1"
}

# Get available disk space in GB
get_free_space_gb() {
  df -BG "$WORKSPACE_ROOT" | tail -1 | awk '{print $4}' | sed 's/G//'
}

# Determine cleanup strategy based on available space
determine_strategy() {
  local free_gb=$1

  if [ "$free_gb" -lt 5 ]; then
    echo "critical"
  elif [ "$free_gb" -lt 10 ]; then
    echo "aggressive"
  elif [ "$free_gb" -lt 20 ]; then
    echo "moderate"
  else
    echo "light"
  fi
}

print_header "Smart Disk Cleanup"

FREE_GB=$(get_free_space_gb)
STRATEGY=$(determine_strategy "$FREE_GB")

echo "Available space: ${FREE_GB}GB"
echo "Cleanup strategy: $STRATEGY"
echo "Mode: $([ "$DRY_RUN" = true ] && echo "DRY RUN" || echo "LIVE")"
echo ""

cd "$WORKSPACE_ROOT"

case $STRATEGY in
  critical)
    print_warning "CRITICAL: Very low disk space!"
    print_info "Running aggressive cleanup with all safety measures..."

    if [ "$DRY_RUN" = true ]; then
      ./scripts/cleanup-disk.sh --dry-run --aggressive
    else
      ./scripts/cleanup-disk.sh --aggressive

      # If still critical, suggest manual intervention
      FREE_GB=$(get_free_space_gb)
      if [ "$FREE_GB" -lt 5 ]; then
        echo ""
        print_warning "Still critically low on space!"
        echo "Manual intervention required:"
        echo "  1. Remove unused Docker images: docker system prune -a"
        echo "  2. Clean system caches: npm cache clean --force"
        echo "  3. Remove old logs: find . -name '*.log' -mtime +3 -delete"
        echo "  4. Check for large files: du -h . | sort -rh | head -20"
      fi
    fi
    ;;

  aggressive)
    print_warning "Low disk space - aggressive cleanup needed"

    if [ "$DRY_RUN" = true ]; then
      ./scripts/cleanup-disk.sh --dry-run --aggressive
    else
      ./scripts/cleanup-disk.sh --aggressive
    fi
    ;;

  moderate)
    print_info "Moderate disk space - standard cleanup"

    if [ "$DRY_RUN" = true ]; then
      ./scripts/cleanup-disk.sh --dry-run
    else
      ./scripts/cleanup-disk.sh
    fi
    ;;

  light)
    print_info "Good disk space - light cleanup only"

    # Only clean clearly unnecessary files
    if [ "$DRY_RUN" = false ]; then
      # Clean old logs
      find . -name "*.log" -mtime +14 -delete 2>/dev/null || true

      # Clean old test results
      find test-results -type f -mtime +14 -delete 2>/dev/null || true

      # Clean incremental compilation cache
      rm -rf target/debug/incremental 2>/dev/null || true

      print_info "Light cleanup complete"
    else
      print_info "[DRY RUN] Would clean old logs and incremental cache"
    fi
    ;;
esac

echo ""
print_header "Cleanup Complete"

FREE_GB=$(get_free_space_gb)
USAGE=$(df -h "$WORKSPACE_ROOT" | tail -1 | awk '{print $5}')

echo "Free space: ${FREE_GB}GB"
echo "Disk usage: $USAGE"

# Provide recommendations
echo ""
echo "💡 Recommendations:"
echo "  • Run cleanup before major builds"
echo "  • Keep target directory under 5GB"
echo "  • Clear old test results regularly"
echo "  • Use 'cargo clean' when switching branches"
echo ""

exit 0
