#!/bin/bash

# ============================================================================
# RipTide Environment Validation Script
# ============================================================================
#
# This script validates all environment variables for:
# - Correct types and formats
# - Valid ranges and constraints
# - Required dependencies
# - Conflicting settings
# - Security concerns
#
# Usage:
#   ./scripts/validate-env.sh [--strict] [--json] [--fix]
#
# Options:
#   --strict    Exit with error on any warnings
#   --json      Output results in JSON format
#   --fix       Attempt to auto-fix common issues
#   --help      Show this help message
#
# Exit codes:
#   0 - All validations passed
#   1 - Validation errors found
#   2 - Configuration file missing
# ============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Options
STRICT_MODE=false
JSON_OUTPUT=false
AUTO_FIX=false

# Counters
ERRORS=0
WARNINGS=0
INFO=0

# Results array for JSON output
declare -a RESULTS

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --strict) STRICT_MODE=true; shift ;;
        --json) JSON_OUTPUT=true; shift ;;
        --fix) AUTO_FIX=true; shift ;;
        --help)
            head -n 23 "$0" | tail -n 19
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Logging functions
log_error() {
    ((ERRORS++))
    if [ "$JSON_OUTPUT" = false ]; then
        echo -e "${RED}✗ ERROR: $1${NC}"
    fi
    RESULTS+=("{\"level\":\"error\",\"message\":\"$1\"}")
}

log_warning() {
    ((WARNINGS++))
    if [ "$JSON_OUTPUT" = false ]; then
        echo -e "${YELLOW}⚠ WARNING: $1${NC}"
    fi
    RESULTS+=("{\"level\":\"warning\",\"message\":\"$1\"}")
}

log_info() {
    ((INFO++))
    if [ "$JSON_OUTPUT" = false ]; then
        echo -e "${BLUE}ℹ INFO: $1${NC}"
    fi
    RESULTS+=("{\"level\":\"info\",\"message\":\"$1\"}")
}

log_success() {
    if [ "$JSON_OUTPUT" = false ]; then
        echo -e "${GREEN}✓ $1${NC}"
    fi
}

# Load .env file
load_env() {
    if [ ! -f ".env" ]; then
        log_error ".env file not found"
        if [ "$JSON_OUTPUT" = true ]; then
            echo "{\"status\":\"error\",\"message\":\".env file not found\",\"results\":[]}"
        fi
        exit 2
    fi

    export $(grep -v '^#' .env | grep -v '^$' | xargs 2>/dev/null || true)
}

# Validation functions
validate_required() {
    local var_name=$1
    local var_value=${!var_name}

    if [ -z "$var_value" ]; then
        log_error "$var_name is required but not set"
        return 1
    fi
    return 0
}

validate_url() {
    local var_name=$1
    local var_value=${!var_name}
    local required=${2:-false}

    if [ -z "$var_value" ]; then
        if [ "$required" = true ]; then
            log_error "$var_name is required but not set"
            return 1
        fi
        return 0
    fi

    if [[ ! "$var_value" =~ ^https?:// ]]; then
        log_error "$var_name must be a valid HTTP/HTTPS URL: $var_value"
        return 1
    fi

    return 0
}

validate_integer() {
    local var_name=$1
    local var_value=${!var_name}
    local min=$2
    local max=$3
    local required=${4:-false}

    if [ -z "$var_value" ]; then
        if [ "$required" = true ]; then
            log_error "$var_name is required but not set"
            return 1
        fi
        return 0
    fi

    if ! [[ "$var_value" =~ ^[0-9]+$ ]]; then
        log_error "$var_name must be an integer: $var_value"
        return 1
    fi

    if [ "$var_value" -lt "$min" ]; then
        log_error "$var_name ($var_value) is below minimum ($min)"
        if [ "$AUTO_FIX" = true ]; then
            log_info "Auto-fixing $var_name to $min"
            export $var_name=$min
        fi
        return 1
    fi

    if [ "$var_value" -gt "$max" ]; then
        log_error "$var_name ($var_value) exceeds maximum ($max)"
        if [ "$AUTO_FIX" = true ]; then
            log_info "Auto-fixing $var_name to $max"
            export $var_name=$max
        fi
        return 1
    fi

    return 0
}

validate_float() {
    local var_name=$1
    local var_value=${!var_name}
    local min=$2
    local max=$3
    local required=${4:-false}

    if [ -z "$var_value" ]; then
        if [ "$required" = true ]; then
            log_error "$var_name is required but not set"
            return 1
        fi
        return 0
    fi

    if ! [[ "$var_value" =~ ^[0-9]+\.?[0-9]*$ ]]; then
        log_error "$var_name must be a number: $var_value"
        return 1
    fi

    if (( $(echo "$var_value < $min" | bc -l) )); then
        log_error "$var_name ($var_value) is below minimum ($min)"
        return 1
    fi

    if (( $(echo "$var_value > $max" | bc -l) )); then
        log_error "$var_name ($var_value) exceeds maximum ($max)"
        return 1
    fi

    return 0
}

validate_boolean() {
    local var_name=$1
    local var_value=${!var_name}

    if [ -z "$var_value" ]; then
        return 0
    fi

    local lower_value=$(echo "$var_value" | tr '[:upper:]' '[:lower:]')
    if [[ ! "$lower_value" =~ ^(true|false|1|0|yes|no)$ ]]; then
        log_error "$var_name must be a boolean (true/false): $var_value"
        return 1
    fi

    return 0
}

validate_enum() {
    local var_name=$1
    local var_value=${!var_name}
    shift
    local valid_values=("$@")

    if [ -z "$var_value" ]; then
        return 0
    fi

    for valid in "${valid_values[@]}"; do
        if [ "$var_value" = "$valid" ]; then
            return 0
        fi
    done

    log_error "$var_name has invalid value '$var_value'. Valid: ${valid_values[*]}"
    return 1
}

validate_path() {
    local var_name=$1
    local var_value=${!var_name}
    local must_exist=${2:-false}
    local must_be_writable=${3:-false}

    if [ -z "$var_value" ]; then
        return 0
    fi

    # Expand variables
    var_value=$(eval echo "$var_value")

    if [ "$must_exist" = true ] && [ ! -e "$var_value" ]; then
        log_error "$var_name path does not exist: $var_value"
        return 1
    fi

    if [ "$must_be_writable" = true ] && [ -e "$var_value" ] && [ ! -w "$var_value" ]; then
        log_error "$var_name path is not writable: $var_value"
        return 1
    fi

    return 0
}

# Security validations
validate_security() {
    log_info "Checking security configuration..."

    # Check if API key is set when auth is required
    if [ "$REQUIRE_AUTH" = "true" ]; then
        if [ -z "$RIPTIDE_API_KEY" ]; then
            log_error "REQUIRE_AUTH is true but RIPTIDE_API_KEY is not set"
        elif [ "$RIPTIDE_API_KEY" = "your_api_key_here" ]; then
            log_error "RIPTIDE_API_KEY is set to example value"
        elif [ ${#RIPTIDE_API_KEY} -lt 16 ]; then
            log_warning "RIPTIDE_API_KEY is too short (< 16 characters)"
        fi
    fi

    # Check for example API keys
    if [ "$SERPER_API_KEY" = "your_serper_api_key_here" ]; then
        log_warning "SERPER_API_KEY is set to example value"
    fi

    # Check TLS configuration
    if [ "$RIPTIDE_ENABLE_TLS" = "true" ]; then
        validate_path "RIPTIDE_TLS_CERT_PATH" true false
        validate_path "RIPTIDE_TLS_KEY_PATH" true false
    fi
}

# Performance validations
validate_performance() {
    log_info "Checking performance configuration..."

    # Check render timeout (3s recommendation)
    if [ -n "$RIPTIDE_RENDER_TIMEOUT" ] && [ "$RIPTIDE_RENDER_TIMEOUT" -gt 5 ]; then
        log_warning "RIPTIDE_RENDER_TIMEOUT ($RIPTIDE_RENDER_TIMEOUT) is higher than recommended (3s)"
    fi

    # Check pool size (3 cap recommendation)
    if [ -n "$RIPTIDE_HEADLESS_POOL_SIZE" ] && [ "$RIPTIDE_HEADLESS_POOL_SIZE" -gt 3 ]; then
        log_warning "RIPTIDE_HEADLESS_POOL_SIZE ($RIPTIDE_HEADLESS_POOL_SIZE) exceeds recommended cap (3)"
    fi

    # Check PDF semaphore (2 requirement)
    if [ -n "$RIPTIDE_MAX_CONCURRENT_PDF" ] && [ "$RIPTIDE_MAX_CONCURRENT_PDF" -ne 2 ]; then
        log_warning "RIPTIDE_MAX_CONCURRENT_PDF ($RIPTIDE_MAX_CONCURRENT_PDF) differs from requirement (2)"
    fi

    # Check rate limiting (1.5 RPS requirement)
    if [ -n "$RIPTIDE_RATE_LIMIT_RPS" ]; then
        if (( $(echo "$RIPTIDE_RATE_LIMIT_RPS < 1.0" | bc -l) )); then
            log_warning "RIPTIDE_RATE_LIMIT_RPS ($RIPTIDE_RATE_LIMIT_RPS) is quite restrictive"
        fi
    fi

    # Check memory configuration
    if [ -n "$RIPTIDE_MEMORY_LIMIT_MB" ] && [ -n "$RIPTIDE_MEMORY_MAX_PER_REQUEST_MB" ]; then
        if [ "$RIPTIDE_MEMORY_MAX_PER_REQUEST_MB" -gt "$RIPTIDE_MEMORY_LIMIT_MB" ]; then
            log_error "RIPTIDE_MEMORY_MAX_PER_REQUEST_MB > RIPTIDE_MEMORY_LIMIT_MB"
        fi
    fi
}

# Dependency validations
validate_dependencies() {
    log_info "Checking dependencies..."

    # Check search backend dependencies
    case "$SEARCH_BACKEND" in
        serper)
            validate_required "SERPER_API_KEY"
            ;;
        searxng)
            validate_required "SEARXNG_BASE_URL"
            validate_url "SEARXNG_BASE_URL" true
            ;;
        none)
            # No dependencies
            ;;
        *)
            if [ -n "$SEARCH_BACKEND" ]; then
                log_error "Invalid SEARCH_BACKEND: $SEARCH_BACKEND (valid: serper, none, searxng)"
            fi
            ;;
    esac

    # Check spider dependencies
    if [ "$SPIDER_ENABLE" = "true" ]; then
        validate_required "SPIDER_BASE_URL"
        validate_url "SPIDER_BASE_URL" true
    fi

    # Check LLM provider dependencies
    if [ -n "$AZURE_OPENAI_KEY" ]; then
        validate_required "AZURE_OPENAI_ENDPOINT"
    fi
}

# Main validation
main() {
    if [ "$JSON_OUTPUT" = false ]; then
        echo ""
        echo -e "${BLUE}RipTide Environment Validation${NC}"
        echo ""
    fi

    load_env

    # Core configuration
    validate_url "RIPTIDE_API_URL" false
    validate_url "REDIS_URL" false
    validate_url "HEADLESS_URL" false

    # CLI configuration
    validate_enum "RIPTIDE_CLI_MODE" "api_first" "api_only" "direct"
    validate_enum "RIPTIDE_CLI_OUTPUT_FORMAT" "json" "text" "table" "markdown"
    validate_boolean "RIPTIDE_CLI_VERBOSE"

    # Performance limits
    validate_integer "RIPTIDE_MAX_CONCURRENT_RENDERS" 1 100 false
    validate_integer "RIPTIDE_MAX_CONCURRENT_PDF" 1 10 false
    validate_integer "RIPTIDE_MAX_CONCURRENT_WASM" 1 20 false

    # Timeouts
    validate_integer "RIPTIDE_RENDER_TIMEOUT" 1 30 false
    validate_integer "RIPTIDE_PDF_TIMEOUT" 5 120 false
    validate_integer "RIPTIDE_WASM_TIMEOUT" 1 60 false
    validate_integer "RIPTIDE_HTTP_TIMEOUT" 1 120 false
    validate_integer "RIPTIDE_GLOBAL_TIMEOUT" 5 300 false

    # Rate limiting
    validate_boolean "RIPTIDE_RATE_LIMIT_ENABLED"
    validate_float "RIPTIDE_RATE_LIMIT_RPS" 0.1 100.0 false
    validate_float "RIPTIDE_RATE_LIMIT_JITTER" 0.0 1.0 false
    validate_integer "RIPTIDE_RATE_LIMIT_BURST_CAPACITY" 1 20 false

    # Browser pool
    validate_integer "RIPTIDE_HEADLESS_POOL_SIZE" 1 10 false
    validate_integer "RIPTIDE_HEADLESS_MIN_POOL_SIZE" 1 5 false
    validate_integer "RIPTIDE_HEADLESS_IDLE_TIMEOUT" 30 3600 false

    # Memory
    validate_integer "RIPTIDE_MEMORY_LIMIT_MB" 512 16384 false
    validate_integer "RIPTIDE_MEMORY_MAX_PER_REQUEST_MB" 64 2048 false
    validate_float "RIPTIDE_MEMORY_PRESSURE_THRESHOLD" 0.5 0.95 false
    validate_boolean "RIPTIDE_MEMORY_AUTO_GC"

    # Search
    validate_enum "SEARCH_BACKEND" "serper" "none" "searxng"
    validate_integer "SEARCH_TIMEOUT" 1 300 false
    validate_boolean "SEARCH_ENABLE_URL_PARSING"

    # Spider
    validate_boolean "SPIDER_ENABLE"
    validate_integer "SPIDER_MAX_DEPTH" 1 10 false
    validate_integer "SPIDER_MAX_PAGES" 1 10000 false
    validate_integer "SPIDER_CONCURRENCY" 1 20 false
    validate_boolean "SPIDER_RESPECT_ROBOTS"

    # Worker
    validate_integer "WORKER_POOL_SIZE" 1 32 false

    # Cache
    validate_integer "CACHE_TTL" 60 604800 false
    validate_boolean "ENABLE_COMPRESSION"
    validate_boolean "ENABLE_MULTI_TENANCY"

    # Auth
    validate_boolean "REQUIRE_AUTH"

    # Telemetry
    validate_boolean "TELEMETRY_ENABLED"
    validate_enum "TELEMETRY_EXPORTER_TYPE" "otlp" "stdout"
    validate_float "TELEMETRY_SAMPLING_RATIO" 0.0 1.0 false

    # Development
    validate_enum "RUST_LOG" "error" "warn" "info" "debug" "trace"
    validate_boolean "RIPTIDE_DEV_MODE"

    # Run comprehensive checks
    validate_security
    validate_performance
    validate_dependencies

    # Output results
    if [ "$JSON_OUTPUT" = true ]; then
        local status="success"
        if [ $ERRORS -gt 0 ]; then
            status="error"
        elif [ "$STRICT_MODE" = true ] && [ $WARNINGS -gt 0 ]; then
            status="warning"
        fi

        echo "{"
        echo "  \"status\": \"$status\","
        echo "  \"errors\": $ERRORS,"
        echo "  \"warnings\": $WARNINGS,"
        echo "  \"info\": $INFO,"
        echo "  \"results\": ["
        printf '%s\n' "${RESULTS[@]}" | paste -sd ',' -
        echo "  ]"
        echo "}"
    else
        echo ""
        echo "Validation Summary:"
        echo "  Errors:   $ERRORS"
        echo "  Warnings: $WARNINGS"
        echo "  Info:     $INFO"
        echo ""

        if [ $ERRORS -eq 0 ]; then
            if [ $WARNINGS -eq 0 ]; then
                log_success "All validations passed!"
            else
                log_success "Validation passed with $WARNINGS warning(s)"
            fi
        else
            log_error "Validation failed with $ERRORS error(s)"
        fi
        echo ""
    fi

    # Exit code
    if [ $ERRORS -gt 0 ]; then
        exit 1
    elif [ "$STRICT_MODE" = true ] && [ $WARNINGS -gt 0 ]; then
        exit 1
    else
        exit 0
    fi
}

main
