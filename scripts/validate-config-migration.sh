#!/bin/bash
# Configuration Migration Validation Script
# Usage: ./scripts/validate-config-migration.sh [pre|post]

set -euo pipefail

COLOR_RED='\033[0;31m'
COLOR_GREEN='\033[0;32m'
COLOR_YELLOW='\033[1;33m'
COLOR_BLUE='\033[0;34m'
COLOR_RESET='\033[0m'

log_info() {
    echo -e "${COLOR_BLUE}ℹ${COLOR_RESET}  $1"
}

log_success() {
    echo -e "${COLOR_GREEN}✅${COLOR_RESET} $1"
}

log_warning() {
    echo -e "${COLOR_YELLOW}⚠️${COLOR_RESET}  $1"
}

log_error() {
    echo -e "${COLOR_RED}❌${COLOR_RESET} $1"
}

validate_pre_migration() {
    echo ""
    log_info "=== PRE-MIGRATION VALIDATION ==="
    echo ""

    local errors=0

    # Check for running processes
    log_info "Checking for processes using config files..."
    if lsof +D /workspaces/eventmesh/config /workspaces/eventmesh/configs 2>/dev/null | grep -q .; then
        log_error "Processes are using config files. Close them before migrating."
        lsof +D /workspaces/eventmesh/config /workspaces/eventmesh/configs 2>/dev/null
        ((errors++))
    else
        log_success "No processes using config files"
    fi

    # Verify directories exist
    log_info "Verifying source directories exist..."
    if [ ! -d "/workspaces/eventmesh/config" ]; then
        log_error "/config directory not found"
        ((errors++))
    else
        log_success "/config directory exists"
    fi

    if [ ! -d "/workspaces/eventmesh/configs" ]; then
        log_error "/configs directory not found"
        ((errors++))
    else
        log_success "/configs directory exists"
    fi

    # Count files
    log_info "Counting configuration files..."
    local config_count=$(find /workspaces/eventmesh/config -type f 2>/dev/null | wc -l)
    local configs_count=$(find /workspaces/eventmesh/configs -type f 2>/dev/null | wc -l)
    local total=$((config_count + configs_count))

    echo "  /config:  $config_count files"
    echo "  /configs: $configs_count files"
    echo "  Total:    $total files"

    if [ "$total" -lt 10 ]; then
        log_warning "Expected at least 10 config files, found $total"
    else
        log_success "File count looks correct ($total files)"
    fi

    # Check git status
    log_info "Checking git status..."
    if ! git diff-index --quiet HEAD -- 2>/dev/null; then
        log_warning "Uncommitted changes detected. Consider committing before migration."
    else
        log_success "Working directory is clean"
    fi

    # Create backup
    log_info "Creating backup..."
    local backup_file="/tmp/config-backup-$(date +%Y%m%d-%H%M%S).tar.gz"
    if tar -czf "$backup_file" \
        /workspaces/eventmesh/config \
        /workspaces/eventmesh/configs 2>/dev/null; then
        log_success "Backup created: $backup_file"
        local backup_size=$(du -h "$backup_file" | cut -f1)
        echo "  Size: $backup_size"
    else
        log_error "Failed to create backup"
        ((errors++))
    fi

    # Create checksums
    log_info "Creating file checksums..."
    if find /workspaces/eventmesh/config /workspaces/eventmesh/configs -type f \
        -exec md5sum {} \; > /tmp/config-checksums-before.txt 2>/dev/null; then
        local checksum_count=$(wc -l < /tmp/config-checksums-before.txt)
        log_success "Checksums saved: /tmp/config-checksums-before.txt ($checksum_count files)"
    else
        log_error "Failed to create checksums"
        ((errors++))
    fi

    # Create git checkpoint
    log_info "Creating git checkpoint..."
    if git rev-parse --verify pre-config-migration >/dev/null 2>&1; then
        log_warning "Tag 'pre-config-migration' already exists"
        echo "  Use: git tag -d pre-config-migration  # to remove"
    else
        if git tag pre-config-migration 2>/dev/null; then
            log_success "Git tag created: pre-config-migration"
        else
            log_warning "Could not create git tag (may need to commit first)"
        fi
    fi

    echo ""
    if [ $errors -eq 0 ]; then
        log_success "=== PRE-MIGRATION VALIDATION PASSED ==="
        echo ""
        log_info "Ready to proceed with migration!"
        log_info "Backup file: $backup_file"
        echo ""
        return 0
    else
        log_error "=== PRE-MIGRATION VALIDATION FAILED ($errors errors) ==="
        echo ""
        log_error "Fix errors before proceeding"
        echo ""
        return 1
    fi
}

validate_post_migration() {
    echo ""
    log_info "=== POST-MIGRATION VALIDATION ==="
    echo ""

    local errors=0
    local warnings=0

    # Check target directory structure
    log_info "Validating new directory structure..."

    local required_dirs=(
        "/workspaces/eventmesh/config/application"
        "/workspaces/eventmesh/config/feature-flags"
        "/workspaces/eventmesh/config/monitoring/dashboards"
        "/workspaces/eventmesh/config/monitoring/alerts"
    )

    for dir in "${required_dirs[@]}"; do
        if [ -d "$dir" ]; then
            log_success "$(basename "$dir")/ exists"
        else
            log_error "$dir missing"
            ((errors++))
        fi
    done

    # Verify old directory removed
    log_info "Verifying old directory removed..."
    if [ -d "/workspaces/eventmesh/configs" ]; then
        log_error "/configs directory still exists"
        ((errors++))
    else
        log_success "/configs directory removed"
    fi

    # Count files in new structure
    log_info "Counting files in new structure..."

    local app_count=$(find /workspaces/eventmesh/config/application -type f 2>/dev/null | wc -l)
    local flags_count=$(find /workspaces/eventmesh/config/feature-flags -type f 2>/dev/null | wc -l)
    local dash_count=$(find /workspaces/eventmesh/config/monitoring/dashboards -type f 2>/dev/null | wc -l)
    local alert_count=$(find /workspaces/eventmesh/config/monitoring/alerts -type f 2>/dev/null | wc -l)
    local total_count=$((app_count + flags_count + dash_count + alert_count + 1))  # +1 for gate_thresholds

    echo "  Application configs: $app_count (expected: 6)"
    echo "  Feature flags:       $flags_count (expected: 2)"
    echo "  Dashboards:          $dash_count (expected: 3)"
    echo "  Alerts:              $alert_count (expected: 1)"
    echo "  Total:               $total_count (expected: ~13)"

    if [ "$app_count" -ne 6 ]; then
        log_warning "Expected 6 application configs, found $app_count"
        ((warnings++))
    else
        log_success "Application config count correct"
    fi

    if [ "$dash_count" -ne 3 ]; then
        log_warning "Expected 3 dashboards, found $dash_count"
        ((warnings++))
    else
        log_success "Dashboard count correct"
    fi

    # Verify specific critical files
    log_info "Verifying critical files..."

    local critical_files=(
        "/workspaces/eventmesh/config/application/riptide.yml"
        "/workspaces/eventmesh/config/application/resource_management.toml"
        "/workspaces/eventmesh/config/monitoring/alerts/streaming-alerts.yaml"
    )

    for file in "${critical_files[@]}"; do
        if [ -f "$file" ]; then
            log_success "$(basename "$file") exists"
        else
            log_error "$file missing"
            ((errors++))
        fi
    done

    # Check for old path references
    log_info "Checking for outdated path references..."

    local doc_refs=$(grep -r "configs/" /workspaces/eventmesh/docs --include="*.md" 2>/dev/null | wc -l)
    if [ "$doc_refs" -gt 0 ]; then
        log_warning "Found $doc_refs references to 'configs/' in documentation"
        echo "  Sample references:"
        grep -r "configs/" /workspaces/eventmesh/docs --include="*.md" 2>/dev/null | head -3 | sed 's/^/  /'
        ((warnings++))
    else
        log_success "No 'configs/' references in documentation"
    fi

    local script_refs=$(grep -r "configs/" /workspaces/eventmesh/scripts --include="*.sh" 2>/dev/null | wc -l)
    if [ "$script_refs" -gt 0 ]; then
        log_warning "Found $script_refs references to 'configs/' in scripts"
        echo "  Sample references:"
        grep -r "configs/" /workspaces/eventmesh/scripts --include="*.sh" 2>/dev/null | head -3 | sed 's/^/  /'
        ((warnings++))
    else
        log_success "No 'configs/' references in scripts"
    fi

    # Compare file checksums (if available)
    if [ -f "/tmp/config-checksums-before.txt" ]; then
        log_info "Comparing file checksums..."

        find /workspaces/eventmesh/config -type f \
            -exec md5sum {} \; > /tmp/config-checksums-after.txt 2>/dev/null

        local before_count=$(wc -l < /tmp/config-checksums-before.txt)
        local after_count=$(wc -l < /tmp/config-checksums-after.txt)

        echo "  Before: $before_count files"
        echo "  After:  $after_count files"

        if [ "$before_count" -eq "$after_count" ]; then
            log_success "File count matches (no files lost)"
        else
            log_warning "File count mismatch (before: $before_count, after: $after_count)"
            ((warnings++))
        fi
    else
        log_warning "Pre-migration checksums not found, skipping comparison"
        ((warnings++))
    fi

    # Check file permissions
    log_info "Checking file permissions..."
    local bad_perms=$(find /workspaces/eventmesh/config -type f ! -perm -644 2>/dev/null | wc -l)
    if [ "$bad_perms" -gt 0 ]; then
        log_warning "Found $bad_perms files with incorrect permissions"
        ((warnings++))
    else
        log_success "All files have correct permissions"
    fi

    # Git status
    log_info "Checking git status..."
    if ! git diff-index --quiet HEAD -- 2>/dev/null; then
        log_success "Changes detected (expected after migration)"
        echo "  Modified/Added/Deleted files:"
        git status --short | head -5 | sed 's/^/  /'
        if [ "$(git status --short | wc -l)" -gt 5 ]; then
            echo "  ... and $(($(git status --short | wc -l) - 5)) more"
        fi
    else
        log_warning "No changes detected (unexpected)"
        ((warnings++))
    fi

    echo ""
    if [ $errors -eq 0 ]; then
        if [ $warnings -eq 0 ]; then
            log_success "=== POST-MIGRATION VALIDATION PASSED ==="
            echo ""
            log_info "Migration completed successfully!"
            log_info "Next steps:"
            echo "  1. Review changes: git status"
            echo "  2. Test build: cargo build --release"
            echo "  3. Test Docker: docker build -t riptide-test ."
            echo "  4. Commit changes: git add -A && git commit -m 'feat: consolidate config directories'"
            echo "  5. Tag completion: git tag post-config-migration"
            echo ""
        else
            log_success "=== POST-MIGRATION VALIDATION PASSED (with $warnings warnings) ==="
            echo ""
            log_warning "Review warnings above before proceeding"
            echo ""
        fi
        return 0
    else
        log_error "=== POST-MIGRATION VALIDATION FAILED ($errors errors, $warnings warnings) ==="
        echo ""
        log_error "Fix errors before proceeding"
        log_info "Consider rollback: git reset --hard pre-config-migration"
        echo ""
        return 1
    fi
}

show_usage() {
    cat << EOF
Configuration Migration Validation Script

Usage:
  $0 [pre|post]

Commands:
  pre   - Validate before migration (creates backup, checksums, git tag)
  post  - Validate after migration (verifies structure, counts files)

Examples:
  $0 pre    # Run before starting migration
  $0 post   # Run after completing migration

See also: docs/config-migration-plan.md
EOF
}

# Main script
case "${1:-}" in
    pre)
        validate_pre_migration
        ;;
    post)
        validate_post_migration
        ;;
    -h|--help|help)
        show_usage
        ;;
    *)
        log_error "Invalid command: ${1:-}"
        echo ""
        show_usage
        exit 1
        ;;
esac
