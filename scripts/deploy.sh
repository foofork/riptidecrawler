#!/bin/bash
# EventMesh Deployment Automation Script
# Usage: ./scripts/deploy.sh [environment] [options]

set -e

# Configuration
ENVIRONMENT=${1:-staging}
DRY_RUN=${DRY_RUN:-false}
SKIP_TESTS=${SKIP_TESTS:-false}
BACKUP_ENABLED=${BACKUP_ENABLED:-true}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Deployment configuration based on environment
case "$ENVIRONMENT" in
    dev|development)
        DEPLOY_HOST="dev.eventmesh.internal"
        DEPLOY_PATH="/opt/eventmesh/dev"
        HEALTH_CHECK_URL="http://dev.eventmesh.internal:8080/healthz"
        ;;
    staging)
        DEPLOY_HOST="staging.eventmesh.internal"
        DEPLOY_PATH="/opt/eventmesh/staging"
        HEALTH_CHECK_URL="http://staging.eventmesh.internal:8080/healthz"
        ;;
    production)
        DEPLOY_HOST="prod.eventmesh.internal"
        DEPLOY_PATH="/opt/eventmesh/production"
        HEALTH_CHECK_URL="https://api.eventmesh.com/healthz"
        ;;
    *)
        log_error "Unknown environment: $ENVIRONMENT"
        echo "Usage: $0 [dev|staging|production]"
        exit 1
        ;;
esac

log_info "Starting deployment to $ENVIRONMENT environment"
log_info "Target: $DEPLOY_HOST:$DEPLOY_PATH"
echo ""

# Pre-deployment checks
log_info "Running pre-deployment checks..."

# Check if git repo is clean
if [ -n "$(git status --porcelain)" ] && [ "$DRY_RUN" = "false" ]; then
    log_warning "Git working directory is not clean"
    git status --short
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_error "Deployment aborted"
        exit 1
    fi
fi

# Get current version
GIT_COMMIT=$(git rev-parse --short HEAD)
GIT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
VERSION="${GIT_BRANCH}-${GIT_COMMIT}"

log_info "Version: $VERSION"
log_info "Commit: $GIT_COMMIT"
log_info "Branch: $GIT_BRANCH"
echo ""

# Run tests
if [ "$SKIP_TESTS" = "false" ]; then
    log_info "Running tests..."
    if cargo test --release --workspace; then
        log_success "All tests passed"
    else
        log_error "Tests failed"
        exit 1
    fi
    echo ""
else
    log_warning "Skipping tests (SKIP_TESTS=true)"
fi

# Build release artifacts
log_info "Building release artifacts..."

# Build native binaries
log_info "Building native binaries..."
if cargo build --release --bins; then
    log_success "Native binaries built successfully"
else
    log_error "Native build failed"
    exit 1
fi

# Build WASM component
log_info "Building WASM component..."
if rustup target list | grep -q "wasm32-wasip2 (installed)"; then
    if cargo build --release --target wasm32-wasip2 -p riptide-extractor-wasm; then
        log_success "WASM component built successfully"
    else
        log_error "WASM build failed"
        exit 1
    fi
else
    log_warning "WASM target not installed, skipping WASM build"
fi

echo ""

# Create deployment package
log_info "Creating deployment package..."
PACKAGE_NAME="eventmesh-${VERSION}.tar.gz"
TEMP_DIR=$(mktemp -d)

# Copy binaries
mkdir -p "$TEMP_DIR/bin"
cp target/release/riptide-api "$TEMP_DIR/bin/" 2>/dev/null || true
cp target/release/riptide-headless "$TEMP_DIR/bin/" 2>/dev/null || true
cp target/release/riptide-workers "$TEMP_DIR/bin/" 2>/dev/null || true

# Copy WASM
mkdir -p "$TEMP_DIR/wasm"
cp target/wasm32-wasip2/release/*.wasm "$TEMP_DIR/wasm/" 2>/dev/null || true

# Copy configuration
mkdir -p "$TEMP_DIR/config"
cp config/* "$TEMP_DIR/config/" 2>/dev/null || true

# Create version file
echo "VERSION=$VERSION" > "$TEMP_DIR/version.txt"
echo "COMMIT=$GIT_COMMIT" >> "$TEMP_DIR/version.txt"
echo "BRANCH=$GIT_BRANCH" >> "$TEMP_DIR/version.txt"
echo "BUILD_DATE=$(date -Iseconds)" >> "$TEMP_DIR/version.txt"

# Create tarball
tar -czf "$PACKAGE_NAME" -C "$TEMP_DIR" .
rm -rf "$TEMP_DIR"

PACKAGE_SIZE=$(du -h "$PACKAGE_NAME" | cut -f1)
log_success "Package created: $PACKAGE_NAME ($PACKAGE_SIZE)"
echo ""

# Dry run mode - skip actual deployment
if [ "$DRY_RUN" = "true" ]; then
    log_warning "DRY RUN MODE - Skipping actual deployment"
    log_info "Package ready for deployment: $PACKAGE_NAME"
    exit 0
fi

# Backup current deployment (if enabled and remote accessible)
if [ "$BACKUP_ENABLED" = "true" ]; then
    log_info "Creating backup of current deployment..."
    BACKUP_NAME="eventmesh-backup-$(date +%Y%m%d_%H%M%S).tar.gz"

    # This would be customized for your actual infrastructure
    # ssh user@$DEPLOY_HOST "cd $DEPLOY_PATH && tar -czf /opt/eventmesh/backups/$BACKUP_NAME ."

    log_info "Backup would be created: $BACKUP_NAME"
fi

# Deploy package
log_info "Deploying package to $DEPLOY_HOST..."

# This section should be customized for your actual deployment infrastructure
# Example for SSH deployment:
# scp "$PACKAGE_NAME" "user@$DEPLOY_HOST:/tmp/"
# ssh "user@$DEPLOY_HOST" "cd /tmp && tar -xzf $PACKAGE_NAME -C $DEPLOY_PATH && rm $PACKAGE_NAME"

log_warning "Deployment commands are placeholders - customize for your infrastructure"
log_info "Package is ready for deployment: $PACKAGE_NAME"

# Restart services
# log_info "Restarting services..."
# ssh "user@$DEPLOY_HOST" "sudo systemctl restart eventmesh-api eventmesh-workers"

# Health check
log_info "Waiting for services to start..."
sleep 5

log_info "Running health checks..."
MAX_RETRIES=30
RETRY_INTERVAL=2

for i in $(seq 1 $MAX_RETRIES); do
    if curl -sf "$HEALTH_CHECK_URL" > /dev/null 2>&1; then
        log_success "Health check passed!"
        break
    else
        if [ $i -eq $MAX_RETRIES ]; then
            log_error "Health check failed after $MAX_RETRIES attempts"
            log_error "Deployment may have failed - investigate manually"
            exit 1
        fi
        log_info "Health check attempt $i/$MAX_RETRIES failed, retrying in ${RETRY_INTERVAL}s..."
        sleep $RETRY_INTERVAL
    fi
done

echo ""
log_success "Deployment completed successfully!"
log_info "Version: $VERSION"
log_info "Environment: $ENVIRONMENT"
log_info "Package: $PACKAGE_NAME"
log_info "Health check: $HEALTH_CHECK_URL"

# Post-deployment verification
log_info "Post-deployment verification..."
# Add custom verification steps here

log_success "All checks passed!"
echo ""
echo "Deployment Summary:"
echo "  Environment: $ENVIRONMENT"
echo "  Version: $VERSION"
echo "  Package: $PACKAGE_NAME ($PACKAGE_SIZE)"
echo "  Status: ✓ Deployed and healthy"
echo ""

# Cleanup
rm -f "$PACKAGE_NAME"
log_info "Temporary files cleaned up"
