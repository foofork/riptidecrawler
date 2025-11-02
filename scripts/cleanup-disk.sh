#!/bin/bash
# Disk Space Cleanup Script for EventMesh
# This script aggressively cleans build artifacts while preserving important cached dependencies
# Usage: ./scripts/cleanup-disk.sh [--dry-run] [--aggressive]

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DRY_RUN=false
AGGRESSIVE=false
TOTAL_FREED=0

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --aggressive)
      AGGRESSIVE=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--dry-run] [--aggressive]"
      exit 1
      ;;
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

print_error() {
  echo -e "${RED}✗${NC} $1"
}

get_dir_size() {
  if [ -d "$1" ]; then
    du -sb "$1" 2>/dev/null | cut -f1 || echo "0"
  else
    echo "0"
  fi
}

format_bytes() {
  local bytes=$1
  if [ "$bytes" -ge 1073741824 ]; then
    echo "$(awk "BEGIN {printf \"%.2f\", $bytes/1073741824}")GB"
  elif [ "$bytes" -ge 1048576 ]; then
    echo "$(awk "BEGIN {printf \"%.2f\", $bytes/1048576}")MB"
  elif [ "$bytes" -ge 1024 ]; then
    echo "$(awk "BEGIN {printf \"%.2f\", $bytes/1024}")KB"
  else
    echo "${bytes}B"
  fi
}

clean_directory() {
  local dir=$1
  local desc=$2

  if [ ! -d "$dir" ]; then
    return
  fi

  local size_before=$(get_dir_size "$dir")

  if [ "$DRY_RUN" = true ]; then
    print_info "[DRY RUN] Would remove $desc: $dir ($(format_bytes $size_before))"
  else
    print_info "Removing $desc: $dir ($(format_bytes $size_before))"
    rm -rf "$dir"
    TOTAL_FREED=$((TOTAL_FREED + size_before))
  fi
}

clean_files_by_pattern() {
  local pattern=$1
  local desc=$2
  local base_dir=${3:-.}

  local files=$(find "$base_dir" -type f -name "$pattern" 2>/dev/null)
  if [ -z "$files" ]; then
    return
  fi

  local total_size=0
  while IFS= read -r file; do
    local size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null || echo "0")
    total_size=$((total_size + size))

    if [ "$DRY_RUN" = true ]; then
      print_info "[DRY RUN] Would remove $desc: $file"
    else
      rm -f "$file"
    fi
  done <<< "$files"

  if [ "$total_size" -gt 0 ]; then
    print_info "Cleaned $desc files ($(format_bytes $total_size))"
    if [ "$DRY_RUN" = false ]; then
      TOTAL_FREED=$((TOTAL_FREED + total_size))
    fi
  fi
}

# Get initial disk usage
print_header "Disk Space Cleanup - EventMesh"
echo "Mode: $([ "$DRY_RUN" = true ] && echo "DRY RUN" || echo "LIVE")"
echo "Aggressiveness: $([ "$AGGRESSIVE" = true ] && echo "AGGRESSIVE" || echo "STANDARD")"
echo ""

DISK_BEFORE=$(df -h . | tail -1 | awk '{print $5}')
print_info "Current disk usage: $DISK_BEFORE"
echo ""

# 1. RUST BUILD ARTIFACTS (HIGHEST PRIORITY - 14GB target)
print_header "1. Rust Build Artifacts"

# Clean incremental compilation cache (4GB) - safe to remove
clean_directory "target/debug/incremental" "Rust incremental compilation cache"

# Clean debug dependencies (9.5GB) - can be rebuilt
clean_directory "target/debug/deps" "Rust debug dependencies"

# Clean debug build artifacts
clean_directory "target/debug/.fingerprint" "Rust fingerprint cache"

# If aggressive mode, clean entire target directory
if [ "$AGGRESSIVE" = true ]; then
  clean_directory "target/debug" "Rust debug build (AGGRESSIVE)"
  clean_directory "target/release" "Rust release build (AGGRESSIVE)"
else
  # Clean specific build artifacts but keep executables
  clean_files_by_pattern "*.d" "Rust dependency files" "target/debug"
  clean_files_by_pattern "*.rlib" "Rust library files" "target/debug"
  clean_files_by_pattern "*.rmeta" "Rust metadata files" "target/debug"
fi

# 2. NODE MODULES (can be reinstalled from package-lock.json)
print_header "2. Node.js Dependencies"

if [ "$AGGRESSIVE" = true ]; then
  # Remove all node_modules
  find . -type d -name "node_modules" -prune 2>/dev/null | while read dir; do
    clean_directory "$dir" "Node.js modules"
  done
else
  # Only clean test/dev node_modules, keep production ones
  clean_directory "tests/playground/node_modules" "Test Node.js modules"
fi

# 3. BUILD OUTPUT DIRECTORIES
print_header "3. Build Output Directories"

clean_directory "playground/dist" "Playground build output"
clean_directory "cli/dist" "CLI build output"

# Find and clean other dist directories
find . -type d -name "dist" -not -path "*/node_modules/*" 2>/dev/null | while read dir; do
  clean_directory "$dir" "Build output"
done

# 4. TEST ARTIFACTS AND LOGS
print_header "4. Test Artifacts and Logs"

# Clean old test results but keep latest
if [ "$AGGRESSIVE" = true ]; then
  clean_directory "test-results" "Test results"
  clean_directory "coverage" "Test coverage"
  clean_directory "eval/results" "Evaluation results"
  clean_directory ".reports" "Test reports"
else
  # Keep recent results, clean old ones
  find test-results -type f -mtime +7 -name "*.log" -delete 2>/dev/null || true
  find eval/results -type f -mtime +7 -name "*.log" -delete 2>/dev/null || true
fi

# Clean log files older than 7 days
find . -type f -name "*.log" -mtime +7 -not -path "*/node_modules/*" 2>/dev/null | while read file; do
  if [ "$DRY_RUN" = true ]; then
    print_info "[DRY RUN] Would remove old log: $file"
  else
    rm -f "$file"
    print_info "Removed old log: $file"
  fi
done

# 5. CACHE DIRECTORIES
print_header "5. Cache Directories"

clean_directory ".cargo/registry/cache" "Cargo cache"
clean_directory ".cargo/git/checkouts" "Cargo git checkouts"

# Clean npm cache if aggressive
if [ "$AGGRESSIVE" = true ]; then
  if command -v npm &> /dev/null; then
    if [ "$DRY_RUN" = true ]; then
      print_info "[DRY RUN] Would clean npm cache"
    else
      print_info "Cleaning npm cache..."
      npm cache clean --force 2>/dev/null || true
    fi
  fi
fi

# 6. TEMPORARY FILES
print_header "6. Temporary Files"

clean_files_by_pattern "*.tmp" "Temporary files"
clean_files_by_pattern "*.temp" "Temporary files"
clean_files_by_pattern ".DS_Store" "macOS system files"
clean_directory "target/tmp" "Target temporary directory"

# 7. DOCUMENTATION BUILD ARTIFACTS
print_header "7. Documentation Build Artifacts"

if [ "$AGGRESSIVE" = true ]; then
  clean_directory "target/doc" "Rust documentation"
fi

# Summary
print_header "Cleanup Summary"

if [ "$DRY_RUN" = false ]; then
  echo -e "${GREEN}Total space freed: $(format_bytes $TOTAL_FREED)${NC}"
else
  echo -e "${YELLOW}This was a dry run. No files were actually deleted.${NC}"
  echo -e "${YELLOW}Potential space savings: $(format_bytes $TOTAL_FREED)${NC}"
fi

DISK_AFTER=$(df -h . | tail -1 | awk '{print $5}')
echo -e "Disk usage before: ${RED}$DISK_BEFORE${NC}"
echo -e "Disk usage after:  ${GREEN}$DISK_AFTER${NC}"

echo ""
print_info "Cleanup complete!"

if [ "$DRY_RUN" = true ]; then
  echo ""
  print_warning "Run without --dry-run to perform actual cleanup:"
  echo "  ./scripts/cleanup-disk.sh"
  echo ""
  print_warning "For aggressive cleanup (removes all build artifacts):"
  echo "  ./scripts/cleanup-disk.sh --aggressive"
fi

exit 0
