#!/bin/bash
# Disk Space Monitor
# Monitors disk usage and sends alerts when thresholds are exceeded
# Can be run as a cron job or in CI/CD

set -e

# Configuration
WARNING_THRESHOLD=75  # Warn at 75% usage
CRITICAL_THRESHOLD=85 # Critical at 85% usage
WORKSPACE_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

# Color codes
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m'

# Get current disk usage percentage (without % sign)
get_usage_percent() {
  df -h "$WORKSPACE_ROOT" | tail -1 | awk '{print $5}' | sed 's/%//'
}

# Get free space in GB
get_free_gb() {
  df -BG "$WORKSPACE_ROOT" | tail -1 | awk '{print $4}' | sed 's/G//'
}

# Generate disk usage report
generate_report() {
  echo "📊 Disk Space Report - $(date)"
  echo "=========================================="
  echo ""

  # Overall usage
  df -h "$WORKSPACE_ROOT" | tail -1 | awk '{print "Filesystem: " $1 "\nSize: " $2 "\nUsed: " $3 "\nAvail: " $4 "\nUse%: " $5 "\nMounted: " $6}'
  echo ""

  # Top directories by size
  echo "📁 Largest Directories (Top 10):"
  du -sh "$WORKSPACE_ROOT"/* 2>/dev/null | sort -rh | head -10 | while read size dir; do
    echo "  $size  $(basename $dir)"
  done
  echo ""

  # Breakdown of known large directories
  echo "🔍 Known Build Artifacts:"
  for dir in target/debug target/release cli/node_modules playground/node_modules; do
    if [ -d "$WORKSPACE_ROOT/$dir" ]; then
      size=$(du -sh "$WORKSPACE_ROOT/$dir" 2>/dev/null | cut -f1)
      echo "  $size  $dir"
    fi
  done
  echo ""

  # File type breakdown
  echo "📄 File Type Breakdown (Target directory):"
  if [ -d "$WORKSPACE_ROOT/target" ]; then
    echo "  Rust libs (.rlib): $(find "$WORKSPACE_ROOT/target" -name "*.rlib" 2>/dev/null | wc -l) files"
    echo "  Dependencies (.d): $(find "$WORKSPACE_ROOT/target" -name "*.d" 2>/dev/null | wc -l) files"
    echo "  Metadata (.rmeta): $(find "$WORKSPACE_ROOT/target" -name "*.rmeta" 2>/dev/null | wc -l) files"
  fi
  echo ""
}

# Main monitoring logic
USAGE=$(get_usage_percent)
FREE_GB=$(get_free_gb)

echo "🔍 Disk Space Monitor"
echo "Current usage: ${USAGE}% (${FREE_GB}GB free)"

if [ "$USAGE" -ge "$CRITICAL_THRESHOLD" ]; then
  echo -e "${RED}❌ CRITICAL: Disk usage above ${CRITICAL_THRESHOLD}%!${NC}"
  generate_report

  echo ""
  echo "⚠️  IMMEDIATE ACTION REQUIRED:"
  echo "  1. Run aggressive cleanup: ./scripts/cleanup-disk.sh --aggressive"
  echo "  2. Or use smart cleanup: ./scripts/smart-cleanup.sh"
  echo ""

  exit 2

elif [ "$USAGE" -ge "$WARNING_THRESHOLD" ]; then
  echo -e "${YELLOW}⚠️  WARNING: Disk usage above ${WARNING_THRESHOLD}%${NC}"
  generate_report

  echo ""
  echo "💡 Recommended Actions:"
  echo "  • Run cleanup: ./scripts/cleanup-disk.sh"
  echo "  • Clean Rust artifacts: cargo clean"
  echo "  • Remove old logs: find . -name '*.log' -mtime +7 -delete"
  echo ""

  exit 1

else
  echo -e "${GREEN}✓ Disk usage is healthy${NC}"

  # Still show report if requested
  if [ "$1" = "--report" ]; then
    echo ""
    generate_report
  fi

  exit 0
fi
