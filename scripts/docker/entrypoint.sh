#!/bin/bash
# ============================================================================
# RipTide Docker Entrypoint Script
# ============================================================================
# Performs initialization and setup before starting the application
# - Creates required directories
# - Validates configuration
# - Runs migrations (if needed)
# - Sets up permissions
# - Handles signals properly
# ============================================================================

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Log function
log() {
    echo -e "[$(date +'%Y-%m-%d %H:%M:%S')] ${BLUE}[ENTRYPOINT]${NC} $1"
}

log_success() {
    echo -e "[$(date +'%Y-%m-%d %H:%M:%S')] ${GREEN}[ENTRYPOINT]${NC} ✓ $1"
}

log_error() {
    echo -e "[$(date +'%Y-%m-%d %H:%M:%S')] ${RED}[ENTRYPOINT]${NC} ✗ $1"
}

log_warning() {
    echo -e "[$(date +'%Y-%m-%d %H:%M:%S')] ${YELLOW}[ENTRYPOINT]${NC} ⚠ $1"
}

# ============================================================================
# Signal Handling
# ============================================================================
_term() {
    log "Caught SIGTERM signal!"
    kill -TERM "$child" 2>/dev/null
}

_int() {
    log "Caught SIGINT signal!"
    kill -INT "$child" 2>/dev/null
}

trap _term SIGTERM
trap _int SIGINT

# ============================================================================
# Pre-flight Checks
# ============================================================================
log "Starting RipTide initialization..."

# Check if running as root (should not be)
if [ "$(id -u)" -eq 0 ]; then
    log_warning "Running as root is not recommended"
fi

# Check required environment variables
REQUIRED_VARS=(
    "RIPTIDE_API_HOST"
    "RIPTIDE_API_PORT"
    "REDIS_URL"
)

for var in "${REQUIRED_VARS[@]}"; do
    if [ -z "${!var}" ]; then
        log_error "Required environment variable ${var} is not set"
        exit 1
    fi
done

log_success "Environment variables validated"

# ============================================================================
# Directory Setup
# ============================================================================
log "Setting up directories..."

DIRECTORIES=(
    "${RIPTIDE_OUTPUT_DIR:-/opt/riptide/data}"
    "${RIPTIDE_CACHE_DIR:-/opt/riptide/cache}"
    "${RIPTIDE_LOGS_DIR:-/opt/riptide/logs}"
    "${RIPTIDE_SCREENSHOTS_DIR:-${RIPTIDE_OUTPUT_DIR}/screenshots}"
    "${RIPTIDE_HTML_DIR:-${RIPTIDE_OUTPUT_DIR}/html}"
    "${RIPTIDE_PDF_DIR:-${RIPTIDE_OUTPUT_DIR}/pdf}"
)

for dir in "${DIRECTORIES[@]}"; do
    if [ ! -d "$dir" ]; then
        mkdir -p "$dir"
        log "Created directory: $dir"
    fi
done

log_success "Directories configured"

# ============================================================================
# Configuration Validation
# ============================================================================
log "Validating configuration..."

# Check if WASM module exists
WASM_PATH="${RIPTIDE_WASM_PATH:-/opt/riptide/extractor/extractor.wasm}"
if [ ! -f "$WASM_PATH" ]; then
    log_error "WASM module not found at: $WASM_PATH"
    exit 1
fi

log_success "WASM module found at: $WASM_PATH"

# Check if config file exists
CONFIG_PATH="${RIPTIDE_CONFIG_PATH:-/opt/riptide/configs/riptide.yml}"
if [ ! -f "$CONFIG_PATH" ]; then
    log_warning "Config file not found at: $CONFIG_PATH (using defaults)"
else
    log_success "Config file found at: $CONFIG_PATH"
fi

# ============================================================================
# Redis Connection Check
# ============================================================================
log "Checking Redis connection..."

# Extract Redis host and port from URL
REDIS_HOST=$(echo "$REDIS_URL" | sed -E 's#redis://([^:]+):.*#\1#')
REDIS_PORT=$(echo "$REDIS_URL" | sed -E 's#redis://[^:]+:([0-9]+).*#\1#')

# Wait for Redis to be ready (max 30 seconds)
MAX_RETRIES=30
RETRY_COUNT=0

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if timeout 1 bash -c "cat < /dev/null > /dev/tcp/${REDIS_HOST}/${REDIS_PORT}" 2>/dev/null; then
        log_success "Redis is ready at ${REDIS_HOST}:${REDIS_PORT}"
        break
    fi

    RETRY_COUNT=$((RETRY_COUNT + 1))
    if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
        log_error "Redis is not available after ${MAX_RETRIES} attempts"
        exit 1
    fi

    log "Waiting for Redis... (attempt $RETRY_COUNT/$MAX_RETRIES)"
    sleep 1
done

# ============================================================================
# Application Startup
# ============================================================================
log "Starting RipTide API..."
log "  Host: ${RIPTIDE_API_HOST}"
log "  Port: ${RIPTIDE_API_PORT}"
log "  Log Level: ${RUST_LOG:-info}"

# Execute the main application
exec "$@" &
child=$!

# Wait for the child process
wait "$child"
EXIT_CODE=$?

log "RipTide API exited with code: $EXIT_CODE"
exit $EXIT_CODE
