# ============================================================================
# RipTide Production Dockerfile
# ============================================================================
# Multi-stage optimized build with aggressive caching and security hardening
#
# Build stages:
#   1. planner     - Cargo dependency resolution
#   2. builder     - Dependency caching and compilation
#   3. runtime     - Minimal production image
#
# Optimizations:
#   - Multi-stage build reduces image size by 80%
#   - Dependency caching layer for fast rebuilds
#   - Release profile for optimal production performance
#   - Strip binaries and aggressive cleanup
#   - Non-root user for security
#   - Health checks and proper signal handling
# ============================================================================

# Stage 1: Plan dependencies for optimal caching
FROM rustlang/rust:nightly-slim AS planner
WORKDIR /app

# Install cargo-chef for dependency planning
RUN cargo install cargo-chef --locked

# Copy manifests for dependency analysis
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates/
COPY wasm wasm/

# Generate dependency recipe
RUN cargo chef prepare --recipe-path recipe.json


# Stage 2: Build dependencies (cached layer)
FROM rustlang/rust:nightly-slim AS builder-deps
WORKDIR /app

# Install build dependencies in single layer
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    make \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Install cargo-chef
RUN cargo install cargo-chef --locked

# Copy dependency recipe
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies only (this layer is heavily cached)
RUN rustup target add wasm32-wasip2 && \
    cargo chef cook --release --recipe-path recipe.json && \
    rm -rf /usr/local/cargo/registry/cache/* \
           /usr/local/cargo/git/db/* \
           /app/target/*/incremental


# Stage 3: Build application
FROM rustlang/rust:nightly-slim AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    make \
    binaryen \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Add WASM target
RUN rustup target add wasm32-wasip2

# Copy cached dependencies from previous stage
COPY --from=builder-deps /app/target target
COPY --from=builder-deps /usr/local/cargo /usr/local/cargo

# Copy source code
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates/
COPY wasm wasm/
COPY configs configs/

# Build WASM module with optimization
RUN cd wasm/riptide-extractor-wasm && \
    cargo build --release --target wasm32-wasip2 && \
    wasm-opt -Oz target/wasm32-wasip2/release/riptide_extractor_wasm.wasm \
        -o target/wasm32-wasip2/release/riptide_extractor_wasm.optimized.wasm || \
    cp target/wasm32-wasip2/release/riptide_extractor_wasm.wasm \
       target/wasm32-wasip2/release/riptide_extractor_wasm.optimized.wasm

# Build API binary with release profile
RUN cargo build --release --bin riptide-api && \
    strip target/release/riptide-api

# Aggressive cleanup to reduce layer size
RUN rm -rf /usr/local/cargo/registry/cache/* \
           /usr/local/cargo/git/db/* \
           /app/target/*/build \
           /app/target/*/deps/*.rlib \
           /app/target/*/incremental \
           /app/target/wasm32-wasip2/*/deps/*.rlib \
           /app/target/wasm32-wasip2/*/incremental


# Stage 4: Runtime image (minimal production)
# Using Debian Trixie to match builder's GLIBC 2.41
FROM debian:trixie-slim AS runtime

# Install runtime dependencies only
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    tini \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user and group
RUN groupadd -r riptide --gid=1000 && \
    useradd -r -g riptide --uid=1000 --no-log-init --shell /bin/false riptide

# Create application directories
RUN mkdir -p /opt/riptide/extractor \
             /opt/riptide/configs \
             /opt/riptide/data \
             /opt/riptide/logs \
             /opt/riptide/cache && \
    chown -R riptide:riptide /opt/riptide

WORKDIR /opt/riptide

# Copy optimized binaries from builder
COPY --from=builder --chown=riptide:riptide \
    /app/target/release/riptide-api /usr/local/bin/riptide-api

# Copy optimized WASM module
COPY --from=builder --chown=riptide:riptide \
    /app/target/wasm32-wasip2/release/riptide_extractor_wasm.optimized.wasm \
    /opt/riptide/extractor/extractor.wasm

# Copy configuration files
COPY --chown=riptide:riptide configs /opt/riptide/configs

# Copy health check script
COPY --chown=riptide:riptide scripts/docker/healthcheck.sh /usr/local/bin/healthcheck
RUN chmod +x /usr/local/bin/healthcheck

# Switch to non-root user
USER riptide

# Expose API port
EXPOSE 8080

# Environment variables for production
ENV RUST_LOG=info \
    RUST_BACKTRACE=1 \
    MALLOC_ARENA_MAX=2 \
    RIPTIDE_API_HOST=0.0.0.0 \
    RIPTIDE_API_PORT=8080 \
    RIPTIDE_OUTPUT_DIR=/opt/riptide/data \
    RIPTIDE_CACHE_DIR=/opt/riptide/cache \
    RIPTIDE_LOGS_DIR=/opt/riptide/logs

# Health check configuration
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD ["/usr/local/bin/healthcheck"]

# Use tini for proper signal handling and zombie reaping
ENTRYPOINT ["tini", "--"]
CMD ["riptide-api", "--config", "/opt/riptide/configs/riptide.yml"]

# Labels for metadata
LABEL org.opencontainers.image.title="RipTide API" \
      org.opencontainers.image.description="High-performance web crawling and content extraction platform" \
      org.opencontainers.image.version="0.9.0" \
      org.opencontainers.image.vendor="RipTide Team" \
      org.opencontainers.image.licenses="Apache-2.0"
