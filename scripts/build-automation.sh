#!/bin/bash
# RipTide Build Automation Script with Performance Optimization
# This script provides comprehensive build pipeline automation

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
GIT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
VERSION=${VERSION:-"0.2.0"}

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

# Performance tracking
start_timer() {
    echo $(date +%s)
}

end_timer() {
    local start_time=$1
    local end_time=$(date +%s)
    echo $((end_time - start_time))
}

# Build targets
BUILD_TARGETS=(
    "native"
    "wasm32-wasip2"
)

# Optimization profiles
OPTIMIZATION_PROFILES=(
    "dev"
    "release"
    "release-lto"
)

# Main build function with performance optimization
build_optimized() {
    local target=${1:-"native"}
    local profile=${2:-"release"}
    local features=${3:-""}

    log_info "Building with target: $target, profile: $profile, features: $features"

    local start_time=$(start_timer)
    local cargo_flags=""

    # Configure build flags based on profile
    case "$profile" in
        "dev")
            cargo_flags="--profile dev"
            ;;
        "release")
            cargo_flags="--release"
            ;;
        "release-lto")
            cargo_flags="--release"
            # LTO configuration is in Cargo.toml
            ;;
    esac

    # Add target if not native
    if [ "$target" != "native" ]; then
        cargo_flags="$cargo_flags --target $target"
    fi

    # Add features if specified
    if [ -n "$features" ]; then
        cargo_flags="$cargo_flags --features $features"
    fi

    # Set build environment variables for optimization
    export CARGO_PROFILE_RELEASE_LTO=true
    export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
    export CARGO_PROFILE_RELEASE_PANIC="abort"
    export RUSTFLAGS="-C target-cpu=native -C opt-level=3"

    # Build command
    if ! cargo build $cargo_flags; then
        log_error "Build failed for target: $target, profile: $profile"
        return 1
    fi

    local build_time=$(end_timer $start_time)
    log_success "Build completed in ${build_time}s for target: $target, profile: $profile"

    # Store build metrics
    echo "{\"target\":\"$target\",\"profile\":\"$profile\",\"build_time\":$build_time,\"timestamp\":\"$BUILD_DATE\",\"commit\":\"$GIT_COMMIT\"}" >> "$PROJECT_ROOT/build-metrics.json"
}

# Component-specific WASM build with size optimization
build_wasm_component() {
    log_info "Building WASM component with size optimization"

    local start_time=$(start_timer)

    cd "$PROJECT_ROOT/wasm/riptide-extractor-wasm"

    # Build with maximum optimization
    if ! cargo component build \
        --release \
        --target wasm32-wasip2 \
        --workspace; then
        log_error "WASM component build failed"
        return 1
    fi

    # Optimize the WASM binary
    local wasm_file="$PROJECT_ROOT/target/wasm32-wasip2/release/riptide_extractor_wasm.wasm"
    if [ -f "$wasm_file" ]; then
        log_info "Optimizing WASM binary"

        # Use wasm-opt if available
        if command -v wasm-opt >/dev/null 2>&1; then
            wasm-opt -Oz --enable-bulk-memory --enable-simd "$wasm_file" -o "${wasm_file}.optimized"
            mv "${wasm_file}.optimized" "$wasm_file"
            log_success "WASM binary optimized"
        else
            log_warning "wasm-opt not found, skipping WASM optimization"
        fi

        # Report size
        local size=$(du -h "$wasm_file" | cut -f1)
        log_info "Final WASM component size: $size"
    fi

    local build_time=$(end_timer $start_time)
    log_success "WASM component built in ${build_time}s"

    cd "$PROJECT_ROOT"
}

# Comprehensive test runner with performance benchmarks
run_tests() {
    log_info "Running comprehensive test suite"

    local start_time=$(start_timer)

    # Unit tests
    log_info "Running unit tests"
    if ! cargo test --workspace --lib; then
        log_error "Unit tests failed"
        return 1
    fi

    # Integration tests
    log_info "Running integration tests"
    if ! cargo test --workspace --test '*'; then
        log_error "Integration tests failed"
        return 1
    fi

    # Performance benchmarks
    log_info "Running performance benchmarks"
    if ! cargo bench --workspace; then
        log_warning "Benchmarks failed or not available"
    fi

    # WASM component tests
    if [ -f "$PROJECT_ROOT/target/wasm32-wasip2/release/riptide_extractor_wasm.wasm" ]; then
        log_info "Testing WASM component"
        if ! cargo test --package riptide-core --test wasm_component_tests; then
            log_warning "WASM component tests failed"
        fi
    fi

    local test_time=$(end_timer $start_time)
    log_success "All tests completed in ${test_time}s"
}

# Performance monitoring and metrics collection
collect_performance_metrics() {
    log_info "Collecting performance metrics"

    local metrics_file="$PROJECT_ROOT/performance-metrics.json"
    local start_time=$(start_timer)

    # System information
    local cpu_info=$(lscpu | grep "Model name" | sed 's/^[^:]*: *//' || echo "unknown")
    local memory_info=$(free -h | grep "Mem:" | awk '{print $2}' || echo "unknown")
    local disk_info=$(df -h . | tail -1 | awk '{print $4}' || echo "unknown")

    # Build sizes
    local target_dir="$PROJECT_ROOT/target"
    local release_size=0
    local debug_size=0

    if [ -d "$target_dir/release" ]; then
        release_size=$(du -s "$target_dir/release" | cut -f1)
    fi

    if [ -d "$target_dir/debug" ]; then
        debug_size=$(du -s "$target_dir/debug" | cut -f1)
    fi

    # WASM component size
    local wasm_size=0
    local wasm_file="$target_dir/wasm32-wasip2/release/riptide_extractor_wasm.wasm"
    if [ -f "$wasm_file" ]; then
        wasm_size=$(stat -c%s "$wasm_file" 2>/dev/null || stat -f%z "$wasm_file" 2>/dev/null || echo 0)
    fi

    # Generate metrics JSON
    cat > "$metrics_file" << EOF
{
  "timestamp": "$BUILD_DATE",
  "git_commit": "$GIT_COMMIT",
  "version": "$VERSION",
  "system": {
    "cpu": "$cpu_info",
    "memory": "$memory_info",
    "disk_available": "$disk_info"
  },
  "build_sizes": {
    "release_kb": $release_size,
    "debug_kb": $debug_size,
    "wasm_bytes": $wasm_size
  },
  "collection_time": $(end_timer $start_time)
}
EOF

    log_success "Performance metrics collected: $metrics_file"
}

# Memory leak detection
run_memory_analysis() {
    log_info "Running memory analysis"

    if command -v valgrind >/dev/null 2>&1; then
        log_info "Running Valgrind memory check"
        cargo build --release

        # Run memory check on the API server
        if [ -f "$PROJECT_ROOT/target/release/riptide-api" ]; then
            valgrind --tool=memcheck \
                --leak-check=full \
                --show-leak-kinds=all \
                --track-origins=yes \
                --log-file="$PROJECT_ROOT/valgrind-report.txt" \
                timeout 30s "$PROJECT_ROOT/target/release/riptide-api" --help || true

            log_success "Valgrind report generated: valgrind-report.txt"
        fi
    else
        log_warning "Valgrind not available, skipping memory analysis"
    fi
}

# Docker build with multi-stage optimization
build_docker() {
    log_info "Building optimized Docker image"

    local start_time=$(start_timer)

    # Create optimized Dockerfile if it doesn't exist
    cat > "$PROJECT_ROOT/Dockerfile.optimized" << 'EOF'
# Multi-stage build for maximum optimization
FROM rust:1.75-slim as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install WASM tools
RUN rustup target add wasm32-wasip2
RUN cargo install cargo-component

WORKDIR /app
COPY . .

# Build with maximum optimization
ENV CARGO_PROFILE_RELEASE_LTO=true
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
ENV RUSTFLAGS="-C target-cpu=native -C opt-level=3"

RUN cargo build --release --workspace
RUN cargo component build --release --target wasm32-wasip2

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy optimized binaries
COPY --from=builder /app/target/release/riptide-api ./
COPY --from=builder /app/target/release/riptide-headless ./
COPY --from=builder /app/target/wasm32-wasip2/release/*.wasm ./

# Set up runtime user
RUN useradd -m -u 1001 riptide
USER riptide

EXPOSE 8080
CMD ["./riptide-api"]
EOF

    # Build Docker image
    if ! docker build -f "$PROJECT_ROOT/Dockerfile.optimized" -t "riptide:optimized-$VERSION" "$PROJECT_ROOT"; then
        log_error "Docker build failed"
        return 1
    fi

    local build_time=$(end_timer $start_time)
    log_success "Docker image built in ${build_time}s"
}

# CI/CD pipeline automation
run_ci_pipeline() {
    log_info "Running CI/CD pipeline"

    local overall_start=$(start_timer)

    # Pre-build checks
    log_info "Running pre-build checks"
    cargo fmt --check || { log_error "Code formatting check failed"; return 1; }
    cargo clippy --workspace --all-targets -- -D warnings || { log_error "Clippy check failed"; return 1; }

    # Security audit
    if command -v cargo-audit >/dev/null 2>&1; then
        cargo audit || log_warning "Security audit found issues"
    fi

    # Build all targets and profiles
    for target in "${BUILD_TARGETS[@]}"; do
        for profile in "${OPTIMIZATION_PROFILES[@]}"; do
            if ! build_optimized "$target" "$profile"; then
                log_error "Build failed for $target/$profile"
                return 1
            fi
        done
    done

    # Build WASM component
    if ! build_wasm_component; then
        log_error "WASM component build failed"
        return 1
    fi

    # Run tests
    if ! run_tests; then
        log_error "Tests failed"
        return 1
    fi

    # Collect metrics
    collect_performance_metrics

    # Memory analysis
    run_memory_analysis

    local pipeline_time=$(end_timer $overall_start)
    log_success "CI/CD pipeline completed in ${pipeline_time}s"
}

# Help function
show_help() {
    cat << EOF
RipTide Build Automation Script

Usage: $0 [COMMAND] [OPTIONS]

Commands:
    build TARGET PROFILE FEATURES  Build with specified options
    wasm                           Build WASM component
    test                           Run test suite
    ci                             Run full CI pipeline
    docker                         Build Docker image
    metrics                        Collect performance metrics
    memory                         Run memory analysis
    help                           Show this help

Examples:
    $0 build native release
    $0 build wasm32-wasip2 release-lto
    $0 wasm
    $0 ci
    $0 docker

Environment Variables:
    VERSION                        Override version (default: 0.2.0)
    CARGO_PROFILE_RELEASE_LTO      Enable LTO (default: true)
    RUSTFLAGS                      Additional Rust flags
EOF
}

# Main execution
main() {
    local command=${1:-"help"}

    case "$command" in
        "build")
            build_optimized "${2:-native}" "${3:-release}" "${4:-}"
            ;;
        "wasm")
            build_wasm_component
            ;;
        "test")
            run_tests
            ;;
        "ci")
            run_ci_pipeline
            ;;
        "docker")
            build_docker
            ;;
        "metrics")
            collect_performance_metrics
            ;;
        "memory")
            run_memory_analysis
            ;;
        "help"|*)
            show_help
            ;;
    esac
}

# Execute main function with all arguments
main "$@"