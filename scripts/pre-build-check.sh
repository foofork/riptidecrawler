#!/bin/bash
# Pre-build Disk Space Check
# This script checks available disk space before starting a build
# and automatically cleans up if necessary

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Configuration
MIN_FREE_GB=10  # Minimum free space required in GB
AUTO_CLEAN=true # Automatically run cleanup if needed
WORKSPACE_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

print_info() {
  echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
  echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
  echo -e "${RED}✗${NC} $1"
}

# Get available disk space in GB
get_free_space_gb() {
  df -BG "$WORKSPACE_ROOT" | tail -1 | awk '{print $4}' | sed 's/G//'
}

# Get disk usage percentage
get_disk_usage() {
  df -h "$WORKSPACE_ROOT" | tail -1 | awk '{print $5}'
}

echo "🔍 Pre-build Disk Space Check"
echo "========================================"

FREE_GB=$(get_free_space_gb)
USAGE=$(get_disk_usage)

echo "Workspace: $WORKSPACE_ROOT"
echo "Free space: ${FREE_GB}GB"
echo "Disk usage: $USAGE"
echo ""

if [ "$FREE_GB" -lt "$MIN_FREE_GB" ]; then
  print_error "Insufficient disk space!"
  print_warning "Available: ${FREE_GB}GB, Required: ${MIN_FREE_GB}GB"

  if [ "$AUTO_CLEAN" = true ]; then
    echo ""
    print_info "Auto-cleanup enabled. Running cleanup script..."

    # Run cleanup script
    if [ -f "$WORKSPACE_ROOT/scripts/cleanup-disk.sh" ]; then
      bash "$WORKSPACE_ROOT/scripts/cleanup-disk.sh"

      # Check space again
      FREE_GB=$(get_free_space_gb)
      echo ""
      echo "After cleanup:"
      echo "Free space: ${FREE_GB}GB"
      echo "Disk usage: $(get_disk_usage)"

      if [ "$FREE_GB" -lt "$MIN_FREE_GB" ]; then
        print_error "Still insufficient space after cleanup!"
        print_warning "Try aggressive cleanup:"
        echo "  ./scripts/cleanup-disk.sh --aggressive"
        exit 1
      else
        print_info "Sufficient space available after cleanup"
      fi
    else
      print_error "Cleanup script not found at $WORKSPACE_ROOT/scripts/cleanup-disk.sh"
      exit 1
    fi
  else
    print_warning "Auto-cleanup disabled. Please free up disk space manually."
    print_info "Run: ./scripts/cleanup-disk.sh"
    exit 1
  fi
else
  print_info "Sufficient disk space available (${FREE_GB}GB free)"
fi

# Check target directory size
if [ -d "$WORKSPACE_ROOT/target" ]; then
  TARGET_SIZE=$(du -sh "$WORKSPACE_ROOT/target" 2>/dev/null | cut -f1)
  echo ""
  echo "📊 Build Artifact Status:"
  echo "Target directory size: $TARGET_SIZE"

  # Warn if target is > 10GB
  TARGET_GB=$(du -sb "$WORKSPACE_ROOT/target" 2>/dev/null | awk '{print int($1/1073741824)}')
  if [ "$TARGET_GB" -gt 10 ]; then
    print_warning "Target directory is large (${TARGET_GB}GB). Consider cleaning:"
    echo "  cargo clean"
    echo "  ./scripts/cleanup-disk.sh"
  fi
fi

echo ""
print_info "Pre-build check passed! Ready to build."
exit 0
