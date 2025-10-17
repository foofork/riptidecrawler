#!/bin/bash

# ============================================================================
# RipTide Environment Setup Script
# ============================================================================
#
# This script:
# - Creates all required output directories
# - Validates environment configuration
# - Sets up proper permissions
# - Provides helpful feedback
#
# Usage:
#   ./scripts/setup-env.sh [--check-only] [--verbose]
#
# Options:
#   --check-only    Only check configuration, don't create directories
#   --verbose       Show detailed output
#   --help          Show this help message
# ============================================================================

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default options
CHECK_ONLY=false
VERBOSE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --check-only)
            CHECK_ONLY=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            head -n 20 "$0" | tail -n 16
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

print_info() {
    print_status "$BLUE" "ℹ $1"
}

print_success() {
    print_status "$GREEN" "✓ $1"
}

print_warning() {
    print_status "$YELLOW" "⚠ $1"
}

print_error() {
    print_status "$RED" "✗ $1"
}

# Function to check if .env file exists
check_env_file() {
    if [ ! -f ".env" ]; then
        print_warning ".env file not found"
        print_info "Creating .env from .env.example..."
        cp .env.example .env
        print_success ".env file created"
        print_warning "Please update .env with your configuration values"
        return 1
    fi
    return 0
}

# Function to load environment variables
load_env() {
    if [ -f ".env" ]; then
        # Load .env file, ignoring comments and empty lines
        export $(grep -v '^#' .env | grep -v '^$' | xargs)
        [ "$VERBOSE" = true ] && print_info "Loaded environment variables from .env"
    fi
}

# Function to create directory if it doesn't exist
create_directory() {
    local dir=$1
    local description=$2

    # Expand environment variables in path
    dir=$(eval echo "$dir")

    if [ -z "$dir" ]; then
        print_warning "Directory path not set: $description"
        return 1
    fi

    if [ "$CHECK_ONLY" = true ]; then
        if [ -d "$dir" ]; then
            [ "$VERBOSE" = true ] && print_success "$description exists: $dir"
        else
            print_warning "$description does not exist: $dir"
        fi
        return 0
    fi

    if [ ! -d "$dir" ]; then
        mkdir -p "$dir"
        print_success "Created $description: $dir"
    else
        [ "$VERBOSE" = true ] && print_info "$description already exists: $dir"
    fi

    # Check write permissions
    if [ ! -w "$dir" ]; then
        print_error "No write permission for $description: $dir"
        return 1
    fi
}

# Function to validate directory configuration
validate_directories() {
    print_info "Validating directory configuration..."

    local dirs_ok=true

    # Output directories
    create_directory "${RIPTIDE_OUTPUT_DIR:-./riptide-output}" "Base output directory" || dirs_ok=false
    create_directory "${RIPTIDE_SCREENSHOTS_DIR:-${RIPTIDE_OUTPUT_DIR}/screenshots}" "Screenshots directory" || dirs_ok=false
    create_directory "${RIPTIDE_HTML_DIR:-${RIPTIDE_OUTPUT_DIR}/html}" "HTML directory" || dirs_ok=false
    create_directory "${RIPTIDE_PDF_DIR:-${RIPTIDE_OUTPUT_DIR}/pdf}" "PDF directory" || dirs_ok=false
    create_directory "${RIPTIDE_REPORTS_DIR:-${RIPTIDE_OUTPUT_DIR}/reports}" "Reports directory" || dirs_ok=false
    create_directory "${RIPTIDE_ARTIFACTS_DIR:-${RIPTIDE_OUTPUT_DIR}/artifacts}" "Artifacts directory" || dirs_ok=false
    create_directory "${RIPTIDE_TEMP_DIR:-${RIPTIDE_OUTPUT_DIR}/temp}" "Temp directory" || dirs_ok=false
    create_directory "${RIPTIDE_LOGS_DIR:-${RIPTIDE_OUTPUT_DIR}/logs}" "Logs directory" || dirs_ok=false
    create_directory "${RIPTIDE_CACHE_DIR:-${RIPTIDE_OUTPUT_DIR}/cache}" "Cache directory" || dirs_ok=false

    if [ "$dirs_ok" = true ]; then
        print_success "All directories validated successfully"
    else
        print_warning "Some directories have issues (see above)"
    fi
}

# Function to validate numeric range
validate_range() {
    local value=$1
    local min=$2
    local max=$3
    local name=$4

    if [ -z "$value" ]; then
        [ "$VERBOSE" = true ] && print_info "$name not set (using default)"
        return 0
    fi

    if ! [[ "$value" =~ ^[0-9]+\.?[0-9]*$ ]]; then
        print_warning "$name must be numeric: $value"
        return 1
    fi

    if (( $(echo "$value < $min" | bc -l) )); then
        print_warning "$name ($value) is below minimum ($min)"
        return 1
    fi

    if (( $(echo "$value > $max" | bc -l) )); then
        print_warning "$name ($value) exceeds maximum ($max)"
        return 1
    fi

    [ "$VERBOSE" = true ] && print_success "$name is valid: $value"
    return 0
}

# Function to validate URL
validate_url() {
    local url=$1
    local name=$2

    if [ -z "$url" ]; then
        [ "$VERBOSE" = true ] && print_info "$name not set"
        return 0
    fi

    if [[ ! "$url" =~ ^https?:// ]]; then
        print_warning "$name must be a valid HTTP/HTTPS URL: $url"
        return 1
    fi

    [ "$VERBOSE" = true ] && print_success "$name is valid: $url"
    return 0
}

# Function to validate boolean
validate_boolean() {
    local value=$1
    local name=$2

    if [ -z "$value" ]; then
        [ "$VERBOSE" = true ] && print_info "$name not set (using default)"
        return 0
    fi

    local lower_value=$(echo "$value" | tr '[:upper:]' '[:lower:]')
    if [[ ! "$lower_value" =~ ^(true|false|1|0|yes|no)$ ]]; then
        print_warning "$name must be a boolean (true/false): $value"
        return 1
    fi

    [ "$VERBOSE" = true ] && print_success "$name is valid: $value"
    return 0
}

# Function to validate critical settings
validate_critical_settings() {
    print_info "Validating critical settings..."

    local settings_ok=true

    # URLs
    validate_url "$RIPTIDE_API_URL" "RIPTIDE_API_URL" || settings_ok=false
    validate_url "$REDIS_URL" "REDIS_URL" || settings_ok=false
    validate_url "$HEADLESS_URL" "HEADLESS_URL" || settings_ok=false

    # Timeouts
    validate_range "$RIPTIDE_RENDER_TIMEOUT" 1 30 "RIPTIDE_RENDER_TIMEOUT" || settings_ok=false
    validate_range "$RIPTIDE_PDF_TIMEOUT" 5 120 "RIPTIDE_PDF_TIMEOUT" || settings_ok=false
    validate_range "$RIPTIDE_WASM_TIMEOUT" 1 60 "RIPTIDE_WASM_TIMEOUT" || settings_ok=false

    # Limits
    validate_range "$RIPTIDE_MAX_CONCURRENT_RENDERS" 1 100 "RIPTIDE_MAX_CONCURRENT_RENDERS" || settings_ok=false
    validate_range "$RIPTIDE_MAX_CONCURRENT_PDF" 1 10 "RIPTIDE_MAX_CONCURRENT_PDF" || settings_ok=false
    validate_range "$RIPTIDE_HEADLESS_POOL_SIZE" 1 10 "RIPTIDE_HEADLESS_POOL_SIZE" || settings_ok=false

    # Memory
    validate_range "$RIPTIDE_MEMORY_LIMIT_MB" 512 16384 "RIPTIDE_MEMORY_LIMIT_MB" || settings_ok=false
    validate_range "$RIPTIDE_MEMORY_PRESSURE_THRESHOLD" 0.5 0.95 "RIPTIDE_MEMORY_PRESSURE_THRESHOLD" || settings_ok=false

    # Rate limiting
    validate_range "$RIPTIDE_RATE_LIMIT_RPS" 0.1 100.0 "RIPTIDE_RATE_LIMIT_RPS" || settings_ok=false
    validate_range "$RIPTIDE_RATE_LIMIT_JITTER" 0.0 1.0 "RIPTIDE_RATE_LIMIT_JITTER" || settings_ok=false

    # Booleans
    validate_boolean "$RIPTIDE_RATE_LIMIT_ENABLED" "RIPTIDE_RATE_LIMIT_ENABLED" || settings_ok=false
    validate_boolean "$RIPTIDE_MEMORY_AUTO_GC" "RIPTIDE_MEMORY_AUTO_GC" || settings_ok=false
    validate_boolean "$REQUIRE_AUTH" "REQUIRE_AUTH" || settings_ok=false

    if [ "$settings_ok" = true ]; then
        print_success "All critical settings validated successfully"
    else
        print_warning "Some settings have validation issues (see above)"
    fi
}

# Function to check for conflicting settings
check_conflicts() {
    print_info "Checking for conflicting configurations..."

    local conflicts=false

    # Check if min pool size > max pool size
    if [ -n "$RIPTIDE_HEADLESS_MIN_POOL_SIZE" ] && [ -n "$RIPTIDE_HEADLESS_POOL_SIZE" ]; then
        if [ "$RIPTIDE_HEADLESS_MIN_POOL_SIZE" -gt "$RIPTIDE_HEADLESS_POOL_SIZE" ]; then
            print_error "RIPTIDE_HEADLESS_MIN_POOL_SIZE ($RIPTIDE_HEADLESS_MIN_POOL_SIZE) > RIPTIDE_HEADLESS_POOL_SIZE ($RIPTIDE_HEADLESS_POOL_SIZE)"
            conflicts=true
        fi
    fi

    # Check if memory per request > global limit
    if [ -n "$RIPTIDE_MEMORY_MAX_PER_REQUEST_MB" ] && [ -n "$RIPTIDE_MEMORY_LIMIT_MB" ]; then
        if [ "$RIPTIDE_MEMORY_MAX_PER_REQUEST_MB" -gt "$RIPTIDE_MEMORY_LIMIT_MB" ]; then
            print_error "RIPTIDE_MEMORY_MAX_PER_REQUEST_MB ($RIPTIDE_MEMORY_MAX_PER_REQUEST_MB) > RIPTIDE_MEMORY_LIMIT_MB ($RIPTIDE_MEMORY_LIMIT_MB)"
            conflicts=true
        fi
    fi

    # Check if auth is required but no API key
    if [ "$REQUIRE_AUTH" = "true" ] && [ -z "$RIPTIDE_API_KEY" ]; then
        print_error "REQUIRE_AUTH is true but RIPTIDE_API_KEY is not set"
        conflicts=true
    fi

    # Check search backend configuration
    if [ "$SEARCH_BACKEND" = "serper" ] && [ -z "$SERPER_API_KEY" ]; then
        print_error "SEARCH_BACKEND is 'serper' but SERPER_API_KEY is not set"
        conflicts=true
    fi

    if [ "$SEARCH_BACKEND" = "searxng" ] && [ -z "$SEARXNG_BASE_URL" ]; then
        print_error "SEARCH_BACKEND is 'searxng' but SEARXNG_BASE_URL is not set"
        conflicts=true
    fi

    # Check spider configuration
    if [ "$SPIDER_ENABLE" = "true" ] && [ -z "$SPIDER_BASE_URL" ]; then
        print_error "SPIDER_ENABLE is true but SPIDER_BASE_URL is not set"
        conflicts=true
    fi

    if [ "$conflicts" = false ]; then
        print_success "No conflicting configurations found"
    else
        print_error "Configuration conflicts detected (see above)"
        return 1
    fi

    return 0
}

# Function to generate environment report
generate_report() {
    print_info "Environment Configuration Report"
    echo ""
    echo "Output Directories:"
    echo "  Base:        ${RIPTIDE_OUTPUT_DIR:-./riptide-output}"
    echo "  Screenshots: ${RIPTIDE_SCREENSHOTS_DIR:-${RIPTIDE_OUTPUT_DIR}/screenshots}"
    echo "  HTML:        ${RIPTIDE_HTML_DIR:-${RIPTIDE_OUTPUT_DIR}/html}"
    echo "  PDF:         ${RIPTIDE_PDF_DIR:-${RIPTIDE_OUTPUT_DIR}/pdf}"
    echo ""
    echo "API Configuration:"
    echo "  URL:         ${RIPTIDE_API_URL:-http://localhost:8080}"
    echo "  Auth:        ${REQUIRE_AUTH:-false}"
    echo "  CLI Mode:    ${RIPTIDE_CLI_MODE:-api_first}"
    echo ""
    echo "Performance Settings:"
    echo "  Render Timeout:   ${RIPTIDE_RENDER_TIMEOUT:-3}s"
    echo "  Max Renders:      ${RIPTIDE_MAX_CONCURRENT_RENDERS:-10}"
    echo "  Pool Size:        ${RIPTIDE_HEADLESS_POOL_SIZE:-3}"
    echo "  Memory Limit:     ${RIPTIDE_MEMORY_LIMIT_MB:-2048}MB"
    echo "  Rate Limit:       ${RIPTIDE_RATE_LIMIT_RPS:-1.5} RPS"
    echo ""
}

# Main execution
main() {
    echo ""
    print_info "RipTide Environment Setup"
    echo ""

    # Check if .env file exists
    check_env_file

    # Load environment variables
    load_env

    # Validate directories
    validate_directories
    echo ""

    # Validate critical settings
    validate_critical_settings
    echo ""

    # Check for conflicts
    check_conflicts
    echo ""

    # Generate report
    if [ "$VERBOSE" = true ]; then
        generate_report
    fi

    # Final summary
    if [ "$CHECK_ONLY" = true ]; then
        print_info "Configuration check complete"
    else
        print_success "Environment setup complete"
    fi

    echo ""
    print_info "Next steps:"
    echo "  1. Review and update .env with your configuration"
    echo "  2. Run './scripts/validate-env.sh' to validate settings"
    echo "  3. Start RipTide: 'cargo run --bin riptide-api'"
    echo ""
}

# Run main function
main
