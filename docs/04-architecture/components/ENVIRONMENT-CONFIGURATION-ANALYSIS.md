# Environment Configuration Analysis - EventMesh (RipTide)

**Document Version:** 1.0
**Date:** 2025-10-20
**Author:** System Architecture Designer
**Status:** Complete

## Executive Summary

This document provides a comprehensive analysis of environment-specific configuration values in the EventMesh (RipTide) codebase, categorizing them by configuration type (environment variables, compile-time flags, runtime constants) and environment profile (development, staging, production).

## Table of Contents

1. [Configuration Strategy](#configuration-strategy)
2. [Service Discovery & External Endpoints](#service-discovery--external-endpoints)
3. [Feature Flags & Experimental Features](#feature-flags--experimental-features)
4. [Debug & Logging Configuration](#debug--logging-configuration)
5. [Resource Limits & Performance Tuning](#resource-limits--performance-tuning)
6. [Environment Variable Matrix](#environment-variable-matrix)
7. [Compile-Time vs Runtime Configuration](#compile-time-vs-runtime-configuration)
8. [Recommendations](#recommendations)

---

## 1. Configuration Strategy

### Configuration Hierarchy

EventMesh uses a **three-tier configuration strategy**:

```
┌─────────────────────────────────────────┐
│   COMPILE-TIME CONFIGURATION            │
│   (Cargo features, compile-time.toml)   │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│   RUNTIME CONFIGURATION FILES           │
│   (TOML/YAML/JSON config files)         │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│   ENVIRONMENT VARIABLES                 │
│   (Highest precedence, runtime override)│
└─────────────────────────────────────────┘
```

**Priority Order:**
1. **Environment Variables** (highest) - runtime overrides
2. **Config Files** (medium) - environment-specific profiles
3. **Compile-Time Flags** (lowest) - build-time decisions
4. **Code Defaults** (fallback) - hardcoded defaults

---

## 2. Service Discovery & External Endpoints

### 2.1 Core Service Endpoints

| Service | Environment Variable | Default Value | Config Type | Notes |
|---------|---------------------|---------------|-------------|-------|
| **API Server** | `RIPTIDE_API_HOST` | `0.0.0.0` | ENV | Binding interface |
| | `RIPTIDE_API_PORT` | `8080` | ENV | API port |
| | `RIPTIDE_API_URL` | `http://localhost:8080` | ENV | CLI client URL |
| **Redis** | `REDIS_URL` | `redis://localhost:6379/0` | ENV | Cache/persistence |
| **Headless Browser** | `HEADLESS_URL` | `http://localhost:9123` | ENV | Browser service |
| **Health Check** | `HEALTH_CHECK_PORT` | (same as API) | ENV | Optional separate port |

### 2.2 External API Providers

#### Search Providers

| Provider | Environment Variable | Config Type | Notes |
|----------|---------------------|-------------|-------|
| **Search Backend Selection** | `SEARCH_BACKEND` | ENV + Runtime | Values: `serper`, `searxng`, `none` |
| **Serper.dev** | `SERPER_API_KEY` | ENV (Secret) | Required when backend=serper |
| **SearXNG** | `SEARXNG_BASE_URL` | ENV | Required when backend=searxng |
| **Search Timeout** | `SEARCH_TIMEOUT` | ENV | Default: 30s |
| **URL Parsing** | `SEARCH_ENABLE_URL_PARSING` | ENV | Boolean flag |

#### LLM/AI Providers (Intelligence Module)

| Provider | Environment Variables | Config Type | Notes |
|----------|----------------------|-------------|-------|
| **OpenAI** | `OPENAI_API_KEY` | ENV (Secret) | Auto-discovery enabled |
| | `OPENAI_BASE_URL` | ENV | Optional override |
| | `RIPTIDE_PROVIDER_OPENAI_ENABLED` | ENV | Explicit enable |
| | `RIPTIDE_PROVIDER_OPENAI_MODEL` | ENV | Default model selection |
| | `RIPTIDE_PROVIDER_OPENAI_PRIORITY` | ENV | Failover priority |
| **Anthropic** | `ANTHROPIC_API_KEY` | ENV (Secret) | Auto-discovery enabled |
| | `RIPTIDE_PROVIDER_ANTHROPIC_ENABLED` | ENV | Explicit enable |
| **Azure OpenAI** | `AZURE_OPENAI_KEY` | ENV (Secret) | Auto-discovery enabled |
| | `AZURE_OPENAI_ENDPOINT` | ENV | Required for Azure |
| | `RIPTIDE_PROVIDER_AZURE_REGION` | ENV | Azure region |
| **Ollama (Local)** | `OLLAMA_BASE_URL` | ENV | Default: `http://localhost:11434` |
| | | | Auto-discovery via TCP check |

### 2.3 Observability & Telemetry Endpoints

| Component | Environment Variable | Default Value | Config Type |
|-----------|---------------------|---------------|-------------|
| **Telemetry Enable** | `TELEMETRY_ENABLED` | `true` | ENV |
| **Service Name** | `TELEMETRY_SERVICE_NAME` | `riptide-api` | ENV |
| **OTLP Endpoint** | `TELEMETRY_OTLP_ENDPOINT` | `http://localhost:4317` | ENV |
| **Alternative OTEL** | `OTEL_ENDPOINT` | `http://localhost:4317` | ENV |
| **Exporter Type** | `TELEMETRY_EXPORTER_TYPE` | `stdout` | ENV |
| **Prometheus** | `RIPTIDE_PROMETHEUS_ENDPOINT` | (none) | ENV |
| **Metrics Port** | `metrics_port` | `9090` | Config File |

### 2.4 Environment-Specific Endpoint Recommendations

#### Development Environment
```env
RIPTIDE_API_HOST=127.0.0.1
RIPTIDE_API_PORT=8080
REDIS_URL=redis://localhost:6379/0
HEADLESS_URL=http://localhost:9123
TELEMETRY_EXPORTER_TYPE=stdout
OLLAMA_BASE_URL=http://localhost:11434
SEARCH_BACKEND=none
```

#### Staging Environment
```env
RIPTIDE_API_HOST=0.0.0.0
RIPTIDE_API_PORT=8080
REDIS_URL=redis://redis.staging.internal:6379/0
HEADLESS_URL=http://headless-service.staging.internal:9123
TELEMETRY_EXPORTER_TYPE=otlp
TELEMETRY_OTLP_ENDPOINT=http://otel-collector.staging.internal:4317
SEARCH_BACKEND=serper
SERPER_API_KEY=${SERPER_STAGING_KEY}
```

#### Production Environment
```env
RIPTIDE_API_HOST=0.0.0.0
RIPTIDE_API_PORT=8080
REDIS_URL=redis://redis-cluster.prod.internal:6379/0
HEADLESS_URL=http://headless-service.prod.internal:9123
TELEMETRY_EXPORTER_TYPE=otlp
TELEMETRY_OTLP_ENDPOINT=https://otel-collector.prod.internal:4317
RIPTIDE_PROMETHEUS_ENDPOINT=http://prometheus.prod.internal:9090
SEARCH_BACKEND=serper
SERPER_API_KEY=${SERPER_PROD_KEY}
OPENAI_API_KEY=${OPENAI_PROD_KEY}
```

---

## 3. Feature Flags & Experimental Features

### 3.1 Compile-Time Feature Flags

**Location:** `/workspaces/eventmesh/config/feature-flags/compile-time.toml`

#### Performance Features (COMPILE-TIME)

| Feature | Default | Environment | Rationale |
|---------|---------|-------------|-----------|
| `optimized_memory_allocation` | `true` | ALL | Production optimization |
| `simd_acceleration` | `true` | ALL | CPU vectorization |
| `vectorized_processing` | `true` | ALL | SIMD processing |
| `zero_copy_optimization` | `true` | ALL | Memory efficiency |

#### Safety Features (COMPILE-TIME)

| Feature | Default | Dev | Staging | Prod | Rationale |
|---------|---------|-----|---------|------|-----------|
| `golden_tests_enabled` | `true` | ✓ | ✓ | ✓ | Regression detection |
| `regression_detection` | `true` | ✓ | ✓ | ✓ | Quality gates |
| `memory_limit_enforcement` | `true` | ✓ | ✓ | ✓ | OOM protection |
| `performance_monitoring` | `true` | ✓ | ✓ | ✓ | Observability |

#### Core Features (COMPILE-TIME)

| Feature | Default | Dev | Staging | Prod | Rationale |
|---------|---------|-----|---------|------|-----------|
| `search_provider_integration` | `true` | ✓ | ✓ | ✓ | Core feature |
| `event_system_enabled` | `true` | ✓ | ✓ | ✓ | Core feature |
| `telemetry_collection` | `true` | ✓ | ✓ | ✓ | Core feature |
| `opentelemetry_export` | `true` | ✓ | ✓ | ✓ | Core feature |
| `redis_caching` | `true` | ✓ | ✓ | ✓ | Core feature |

#### Refactoring Safety (COMPILE-TIME)

| Feature | Default | Dev | Staging | Prod | Notes |
|---------|---------|-----|---------|------|-------|
| `behavior_capture` | `true` | ✓ | ✓ | ✓ | Snapshot testing |
| `baseline_comparison` | `true` | ✓ | ✓ | ✓ | Regression checks |
| `rollback_support` | `true` | ✓ | ✓ | ✓ | Safe deployment |
| `safe_deployment` | `true` | ✓ | ✓ | ✓ | Canary support |

#### Experimental Features (COMPILE-TIME - DISABLED BY DEFAULT)

| Feature | Default | Dev | Staging | Prod | Status |
|---------|---------|-----|---------|------|--------|
| `wasm_optimization` | `false` | ⚠️ | ✗ | ✗ | Experimental |
| `neural_prediction` | `false` | ⚠️ | ✗ | ✗ | Experimental |
| `adaptive_scaling` | `false` | ⚠️ | ✗ | ✗ | Experimental |
| `cloud_integration` | `false` | ⚠️ | ✗ | ✗ | Experimental |

**Recommendation:** Enable experimental features via Cargo feature flags for dev builds only:
```bash
# Development build with experimental features
cargo build --features experimental-wasm,neural-prediction

# Production build (no experimental features)
cargo build --release
```

### 3.2 Runtime Feature Flags

#### Spider/Crawler Features

| Feature | Environment Variable | Default | Config Type |
|---------|---------------------|---------|-------------|
| **Spider Enable** | `SPIDER_ENABLE` | `false` | ENV + Runtime |
| **Spider Base URL** | `SPIDER_BASE_URL` | (none) | ENV |
| **Max Depth** | `SPIDER_MAX_DEPTH` | `3` | ENV |
| **Max Pages** | `SPIDER_MAX_PAGES` | `100` | ENV |
| **Concurrency** | `SPIDER_CONCURRENCY` | `4` | ENV |
| **Respect Robots** | `SPIDER_RESPECT_ROBOTS` | `true` | ENV |

#### Enhanced Pipeline Features

| Feature | Environment Variable | Default | Config Type |
|---------|---------------------|---------|-------------|
| **Enhanced Pipeline** | `ENHANCED_PIPELINE_ENABLE` | `true` | ENV |
| **Pipeline Metrics** | `ENHANCED_PIPELINE_METRICS` | `true` | ENV |
| **Pipeline Debug** | `ENHANCED_PIPELINE_DEBUG` | `false` | ENV |

#### Authentication & Security

| Feature | Environment Variable | Default | Config Type |
|---------|---------------------|---------|-------------|
| **Require Auth** | `REQUIRE_AUTH` | `false` | ENV |
| **API Keys** | `API_KEYS` | (none) | ENV (Secret) |
| **Enable TLS** | `RIPTIDE_ENABLE_TLS` | `false` | ENV |
| **TLS Cert Path** | `RIPTIDE_TLS_CERT_PATH` | (none) | ENV |
| **TLS Key Path** | `RIPTIDE_TLS_KEY_PATH` | (none) | ENV |

#### Multi-Tenancy & Isolation

| Feature | Environment Variable | Default | Config Type |
|---------|---------------------|---------|-------------|
| **Multi-Tenancy** | `ENABLE_MULTI_TENANCY` | `false` | ENV |
| **Tenant Isolation** | `RIPTIDE_TENANT_ISOLATION_ENABLED` | `true` | ENV |
| **Strict Isolation** | (Config File) | `false` | Config File |

---

## 4. Debug & Logging Configuration

### 4.1 Logging Levels

| Environment Variable | Default | Dev | Staging | Prod |
|---------------------|---------|-----|---------|------|
| `RUST_LOG` | `info` | `debug` | `info` | `warn` |
| `RIPTIDE_LOG_LEVEL` | (inherits RUST_LOG) | `trace` | `info` | `warn` |
| `TELEMETRY_SAMPLING_RATIO` | `1.0` | `1.0` | `0.1` | `0.01` |

### 4.2 Debug Features

| Feature | Environment Variable | Default | Dev | Staging | Prod |
|---------|---------------------|---------|-----|---------|------|
| **Development Mode** | `RIPTIDE_DEV_MODE` | `false` | ✓ | ✗ | ✗ |
| **Verbose CLI** | `RIPTIDE_CLI_VERBOSE` | `false` | ✓ | ✗ | ✗ |
| **Enhanced Debug** | `ENHANCED_PIPELINE_DEBUG` | `false` | ✓ | ✗ | ✗ |
| **Verbose Logging** | (Config File: `debug.verbose_logging`) | `false` | ✓ | ✗ | ✗ |
| **Trace Allocations** | (Config File: `debug.trace_all_allocations`) | `false` | ⚠️ | ✗ | ✗ |
| **Heap Snapshots** | (Config File: `debug.dump_heap_snapshots`) | `false` | ✓ | ✗ | ✗ |

### 4.3 Logging Configuration Files

#### Development (`crates/riptide-performance/config/development.toml`)

```toml
[monitoring]
log_level = "info"
enable_metrics_export = true
enable_tracing = true
trace_sample_rate = 1.0  # 100% sampling

[debug]
verbose_logging = true
trace_all_allocations = false  # Very expensive
dump_heap_snapshots = true
snapshot_interval_secs = 300
```

#### Production (`crates/riptide-performance/config/production.toml`)

```toml
[monitoring]
log_level = "info"
enable_metrics_export = true
enable_tracing = true
trace_sample_rate = 0.1  # 10% sampling

[alerts]
enabled = true
notification_channels = ["log", "otlp"]
alert_cooldown_secs = 300
```

### 4.4 Environment-Specific Logging Recommendations

#### Development
```env
RUST_LOG=debug,hyper=info,tokio=info
RIPTIDE_CLI_VERBOSE=true
RIPTIDE_DEV_MODE=true
TELEMETRY_EXPORTER_TYPE=stdout
TELEMETRY_SAMPLING_RATIO=1.0
```

#### Staging
```env
RUST_LOG=info,riptide=debug
RIPTIDE_CLI_VERBOSE=false
RIPTIDE_DEV_MODE=false
TELEMETRY_EXPORTER_TYPE=otlp
TELEMETRY_SAMPLING_RATIO=0.1
```

#### Production
```env
RUST_LOG=warn,riptide=info
RIPTIDE_CLI_VERBOSE=false
RIPTIDE_DEV_MODE=false
TELEMETRY_EXPORTER_TYPE=otlp
TELEMETRY_SAMPLING_RATIO=0.01
```

---

## 5. Resource Limits & Performance Tuning

### 5.1 Connection Pool Sizes

| Resource | Environment Variable | Default | Dev | Staging | Prod | Notes |
|----------|---------------------|---------|-----|---------|------|-------|
| **Browser Pool Max** | `RIPTIDE_HEADLESS_POOL_SIZE` | `3` | 2 | 3 | 5 | Hard requirement: max=3 |
| **Browser Pool Min** | `RIPTIDE_HEADLESS_MIN_POOL_SIZE` | `1` | 1 | 1 | 2 | Minimum instances |
| **WASM Instances** | `RIPTIDE_MAX_CONCURRENT_WASM` | `4` | 2 | 4 | 8 | Per-worker limit |
| **Worker Pool** | `WORKER_POOL_SIZE` | `4` | 2 | 4 | 8 | CPU-bound |
| **Max Pool Size** | (Config: `browser_pool.max_pool_size`) | `20` | 5 | 10 | 20 | Resource ceiling |

### 5.2 Concurrency Limits

| Resource | Environment Variable | Default | Dev | Staging | Prod | Notes |
|----------|---------------------|---------|-----|---------|------|-------|
| **Concurrent Renders** | `RIPTIDE_MAX_CONCURRENT_RENDERS` | `10` | 5 | 10 | 20 | Render operations |
| **Concurrent PDF** | `RIPTIDE_MAX_CONCURRENT_PDF` | `2` | 2 | 2 | 2 | Semaphore requirement |
| **Concurrent WASM** | `RIPTIDE_MAX_CONCURRENT_WASM` | `4` | 2 | 4 | 8 | WASM instances |
| **Max Concurrent Requests** | (Config: `runtime.max_concurrent_requests`) | `100` | 10 | 50 | 1000 | API throttle |
| **Spider Concurrency** | `SPIDER_CONCURRENCY` | `4` | 2 | 4 | 8 | Crawler threads |
| **Stream Max Concurrent** | `STREAM_MAX_CONCURRENT` | `100` | 10 | 50 | 200 | Streaming limit |

### 5.3 Timeout Configuration

| Operation | Environment Variable | Default | Dev | Staging | Prod | Notes |
|-----------|---------------------|---------|-----|---------|------|-------|
| **Render Timeout** | `RIPTIDE_RENDER_TIMEOUT` | `3s` | 5s | 3s | 3s | **Hard cap: 3s** |
| **PDF Timeout** | `RIPTIDE_PDF_TIMEOUT` | `30s` | 60s | 30s | 20s | PDF processing |
| **WASM Timeout** | `RIPTIDE_WASM_TIMEOUT` | `10s` | 30s | 10s | 5s | Module execution |
| **HTTP Timeout** | `RIPTIDE_HTTP_TIMEOUT` | `10s` | 30s | 15s | 10s | Network requests |
| **Global Timeout** | `RIPTIDE_GLOBAL_TIMEOUT` | `30s` | 60s | 30s | 30s | Fallback timeout |
| **Search Timeout** | `SEARCH_TIMEOUT` | `30s` | 60s | 30s | 20s | Search API calls |
| **Browser Launch** | `RIPTIDE_HEADLESS_LAUNCH_TIMEOUT` | `30s` | 60s | 30s | 20s | Browser startup |

**Critical Note:** The `RIPTIDE_RENDER_TIMEOUT` has a **hard requirement of 3 seconds** for performance.

### 5.4 Memory Limits

| Resource | Environment Variable | Default | Dev | Staging | Prod | Notes |
|----------|---------------------|---------|-----|---------|------|-------|
| **Global Memory Limit** | `RIPTIDE_MEMORY_LIMIT_MB` | `2048` MB | 1024 MB | 2048 MB | 4096 MB | Total limit |
| **Memory Soft Limit** | (Config: `memory.memory_soft_limit_mb`) | `400` MB | 300 MB | 400 MB | 600 MB | Warning threshold |
| **Memory Hard Limit** | (Config: `memory.memory_hard_limit_mb`) | `500` MB | 400 MB | 500 MB | 800 MB | Reject threshold |
| **Max Per Request** | `RIPTIDE_MEMORY_MAX_PER_REQUEST_MB` | `256` MB | 128 MB | 256 MB | 512 MB | Request ceiling |
| **Pressure Threshold** | `RIPTIDE_MEMORY_PRESSURE_THRESHOLD` | `0.85` | 0.7 | 0.85 | 0.9 | Trigger GC |
| **GC Trigger** | `RIPTIDE_MEMORY_GC_TRIGGER_MB` | `1024` MB | 512 MB | 1024 MB | 2048 MB | GC threshold |
| **WASM Max Memory** | `RIPTIDE_WASM_MAX_MEMORY_MB` | `128` MB | 64 MB | 128 MB | 256 MB | Per WASM instance |

### 5.5 Buffer Sizes

| Buffer | Environment Variable | Default | Dev | Staging | Prod | Notes |
|--------|---------------------|---------|-----|---------|------|-------|
| **Stream Buffer** | `STREAM_BUFFER_SIZE` | `8192` | 4096 | 8192 | 16384 | Streaming buffer |
| **Stream Max Buffer** | `STREAM_BUFFER_MAX_SIZE` | `65536` | 32768 | 65536 | 131072 | Max buffer size |
| **WS Max Message** | `WS_MAX_MESSAGE_SIZE` | `16777216` | 8 MB | 16 MB | 32 MB | WebSocket limit |
| **PDF Queue Size** | `RIPTIDE_PDF_QUEUE_SIZE` | `50` | 10 | 50 | 100 | PDF backlog |
| **Request Queue** | (Config: `runtime.request_queue_size`) | `1000` | 100 | 1000 | 5000 | API queue depth |

### 5.6 Thread Counts

| Component | Environment Variable | Default | Dev | Staging | Prod | Calculation |
|-----------|---------------------|---------|-----|---------|------|-------------|
| **Worker Pool Size** | `WORKER_POOL_SIZE` | `4` | 2 | 4 | `num_cpus` | CPU-bound workers |
| **Tokio Runtime** | (Tokio default) | `num_cpus` | 4 | 8 | `num_cpus * 2` | Async runtime |

### 5.7 Rate Limiting

| Parameter | Environment Variable | Default | Dev | Staging | Prod | Notes |
|-----------|---------------------|---------|-----|---------|------|-------|
| **Rate Limit Enabled** | `RIPTIDE_RATE_LIMIT_ENABLED` | `true` | false | true | true | Toggle rate limiting |
| **RPS Per Host** | `RIPTIDE_RATE_LIMIT_RPS` | `1.5` | 10.0 | 2.0 | 1.5 | **Requirement: 1.5 RPS** |
| **Jitter Factor** | `RIPTIDE_RATE_LIMIT_JITTER` | `0.1` | 0.0 | 0.1 | 0.1 | 10% jitter |
| **Burst Capacity** | `RIPTIDE_RATE_LIMIT_BURST_CAPACITY` | `3` | 10 | 5 | 3 | Burst allowance |
| **Window Duration** | `RIPTIDE_RATE_LIMIT_WINDOW_SECS` | `60s` | 10s | 60s | 60s | Rate limit window |
| **Max Tracked Hosts** | `RIPTIDE_RATE_LIMIT_MAX_HOSTS` | `10000` | 1000 | 10000 | 50000 | Host tracking limit |

**Critical Note:** The `RIPTIDE_RATE_LIMIT_RPS` has a **hard requirement of 1.5 RPS per host** for production.

### 5.8 Environment-Specific Resource Profiles

#### Development Profile
```env
# Concurrency
RIPTIDE_MAX_CONCURRENT_RENDERS=5
RIPTIDE_MAX_CONCURRENT_PDF=2
RIPTIDE_MAX_CONCURRENT_WASM=2
WORKER_POOL_SIZE=2

# Memory
RIPTIDE_MEMORY_LIMIT_MB=1024
RIPTIDE_MEMORY_MAX_PER_REQUEST_MB=128
RIPTIDE_MEMORY_PRESSURE_THRESHOLD=0.7

# Timeouts (relaxed for debugging)
RIPTIDE_RENDER_TIMEOUT=5
RIPTIDE_PDF_TIMEOUT=60
RIPTIDE_WASM_TIMEOUT=30
RIPTIDE_HTTP_TIMEOUT=30

# Rate Limiting (disabled for dev)
RIPTIDE_RATE_LIMIT_ENABLED=false
```

#### Staging Profile
```env
# Concurrency
RIPTIDE_MAX_CONCURRENT_RENDERS=10
RIPTIDE_MAX_CONCURRENT_PDF=2
RIPTIDE_MAX_CONCURRENT_WASM=4
WORKER_POOL_SIZE=4

# Memory
RIPTIDE_MEMORY_LIMIT_MB=2048
RIPTIDE_MEMORY_MAX_PER_REQUEST_MB=256
RIPTIDE_MEMORY_PRESSURE_THRESHOLD=0.85

# Timeouts (production-like)
RIPTIDE_RENDER_TIMEOUT=3
RIPTIDE_PDF_TIMEOUT=30
RIPTIDE_WASM_TIMEOUT=10
RIPTIDE_HTTP_TIMEOUT=15

# Rate Limiting
RIPTIDE_RATE_LIMIT_ENABLED=true
RIPTIDE_RATE_LIMIT_RPS=2.0
```

#### Production Profile
```env
# Concurrency (scaled for production)
RIPTIDE_MAX_CONCURRENT_RENDERS=20
RIPTIDE_MAX_CONCURRENT_PDF=2
RIPTIDE_MAX_CONCURRENT_WASM=8
WORKER_POOL_SIZE=8

# Memory (generous limits)
RIPTIDE_MEMORY_LIMIT_MB=4096
RIPTIDE_MEMORY_MAX_PER_REQUEST_MB=512
RIPTIDE_MEMORY_PRESSURE_THRESHOLD=0.9

# Timeouts (strict for performance)
RIPTIDE_RENDER_TIMEOUT=3
RIPTIDE_PDF_TIMEOUT=20
RIPTIDE_WASM_TIMEOUT=5
RIPTIDE_HTTP_TIMEOUT=10

# Rate Limiting (strict)
RIPTIDE_RATE_LIMIT_ENABLED=true
RIPTIDE_RATE_LIMIT_RPS=1.5
RIPTIDE_RATE_LIMIT_MAX_HOSTS=50000
```

---

## 6. Environment Variable Matrix

### 6.1 Complete Environment Variable Reference

| Category | Variable Name | Type | Default | Dev | Staging | Prod | Secret |
|----------|--------------|------|---------|-----|---------|------|--------|
| **API Server** | | | | | | | |
| | `RIPTIDE_API_HOST` | String | `0.0.0.0` | `127.0.0.1` | `0.0.0.0` | `0.0.0.0` | No |
| | `RIPTIDE_API_PORT` | Integer | `8080` | `8080` | `8080` | `8080` | No |
| | `RIPTIDE_API_URL` | URL | `http://localhost:8080` | (same) | (staging URL) | (prod URL) | No |
| **Redis** | | | | | | | |
| | `REDIS_URL` | URL | `redis://localhost:6379/0` | (same) | (staging cluster) | (prod cluster) | Yes |
| **Search** | | | | | | | |
| | `SEARCH_BACKEND` | Enum | `serper` | `none` | `serper` | `serper` | No |
| | `SERPER_API_KEY` | String | (none) | (none) | `${STAGING_KEY}` | `${PROD_KEY}` | Yes |
| | `SEARXNG_BASE_URL` | URL | (none) | `http://localhost:8888` | (staging URL) | (none) | No |
| | `SEARCH_TIMEOUT` | Integer | `30` | `60` | `30` | `20` | No |
| **LLM Providers** | | | | | | | |
| | `OPENAI_API_KEY` | String | (none) | (none) | `${STAGING_KEY}` | `${PROD_KEY}` | Yes |
| | `OPENAI_BASE_URL` | URL | (default) | (default) | (default) | (custom) | No |
| | `ANTHROPIC_API_KEY` | String | (none) | (none) | `${STAGING_KEY}` | `${PROD_KEY}` | Yes |
| | `AZURE_OPENAI_KEY` | String | (none) | (none) | `${STAGING_KEY}` | `${PROD_KEY}` | Yes |
| | `AZURE_OPENAI_ENDPOINT` | URL | (none) | (none) | (staging endpoint) | (prod endpoint) | No |
| | `OLLAMA_BASE_URL` | URL | `http://localhost:11434` | (same) | (none) | (none) | No |
| **Telemetry** | | | | | | | |
| | `TELEMETRY_ENABLED` | Boolean | `true` | `true` | `true` | `true` | No |
| | `TELEMETRY_SERVICE_NAME` | String | `riptide-api` | `riptide-dev` | `riptide-staging` | `riptide-prod` | No |
| | `TELEMETRY_OTLP_ENDPOINT` | URL | `http://localhost:4317` | (same) | (staging collector) | (prod collector) | No |
| | `TELEMETRY_EXPORTER_TYPE` | Enum | `stdout` | `stdout` | `otlp` | `otlp` | No |
| | `TELEMETRY_SAMPLING_RATIO` | Float | `1.0` | `1.0` | `0.1` | `0.01` | No |
| | `RIPTIDE_PROMETHEUS_ENDPOINT` | URL | (none) | (none) | (staging URL) | (prod URL) | No |
| **Logging** | | | | | | | |
| | `RUST_LOG` | String | `info` | `debug` | `info` | `warn` | No |
| | `RIPTIDE_LOG_LEVEL` | String | (inherits) | `trace` | `info` | `warn` | No |
| **Authentication** | | | | | | | |
| | `REQUIRE_AUTH` | Boolean | `false` | `false` | `true` | `true` | No |
| | `API_KEYS` | CSV | (none) | (none) | `${STAGING_KEYS}` | `${PROD_KEYS}` | Yes |
| | `RIPTIDE_ENABLE_TLS` | Boolean | `false` | `false` | `true` | `true` | No |
| | `RIPTIDE_TLS_CERT_PATH` | Path | (none) | (none) | `/certs/staging.pem` | `/certs/prod.pem` | No |
| | `RIPTIDE_TLS_KEY_PATH` | Path | (none) | (none) | `/certs/staging.key` | `/certs/prod.key` | Yes |
| **Concurrency** | | | | | | | |
| | `RIPTIDE_MAX_CONCURRENT_RENDERS` | Integer | `10` | `5` | `10` | `20` | No |
| | `RIPTIDE_MAX_CONCURRENT_PDF` | Integer | `2` | `2` | `2` | `2` | No |
| | `RIPTIDE_MAX_CONCURRENT_WASM` | Integer | `4` | `2` | `4` | `8` | No |
| | `WORKER_POOL_SIZE` | Integer | `4` | `2` | `4` | `8` | No |
| | `STREAM_MAX_CONCURRENT` | Integer | `100` | `10` | `50` | `200` | No |
| **Memory** | | | | | | | |
| | `RIPTIDE_MEMORY_LIMIT_MB` | Integer | `2048` | `1024` | `2048` | `4096` | No |
| | `RIPTIDE_MEMORY_MAX_PER_REQUEST_MB` | Integer | `256` | `128` | `256` | `512` | No |
| | `RIPTIDE_MEMORY_PRESSURE_THRESHOLD` | Float | `0.85` | `0.7` | `0.85` | `0.9` | No |
| | `RIPTIDE_MEMORY_GC_TRIGGER_MB` | Integer | `1024` | `512` | `1024` | `2048` | No |
| | `RIPTIDE_WASM_MAX_MEMORY_MB` | Integer | `128` | `64` | `128` | `256` | No |
| **Timeouts** | | | | | | | |
| | `RIPTIDE_RENDER_TIMEOUT` | Integer (sec) | `3` | `5` | `3` | `3` | No |
| | `RIPTIDE_PDF_TIMEOUT` | Integer (sec) | `30` | `60` | `30` | `20` | No |
| | `RIPTIDE_WASM_TIMEOUT` | Integer (sec) | `10` | `30` | `10` | `5` | No |
| | `RIPTIDE_HTTP_TIMEOUT` | Integer (sec) | `10` | `30` | `15` | `10` | No |
| | `RIPTIDE_GLOBAL_TIMEOUT` | Integer (sec) | `30` | `60` | `30` | `30` | No |
| **Rate Limiting** | | | | | | | |
| | `RIPTIDE_RATE_LIMIT_ENABLED` | Boolean | `true` | `false` | `true` | `true` | No |
| | `RIPTIDE_RATE_LIMIT_RPS` | Float | `1.5` | `10.0` | `2.0` | `1.5` | No |
| | `RIPTIDE_RATE_LIMIT_JITTER` | Float | `0.1` | `0.0` | `0.1` | `0.1` | No |
| | `RIPTIDE_RATE_LIMIT_BURST_CAPACITY` | Integer | `3` | `10` | `5` | `3` | No |
| | `RIPTIDE_RATE_LIMIT_MAX_HOSTS` | Integer | `10000` | `1000` | `10000` | `50000` | No |
| **Browser Pool** | | | | | | | |
| | `RIPTIDE_HEADLESS_POOL_SIZE` | Integer | `3` | `2` | `3` | `5` | No |
| | `RIPTIDE_HEADLESS_MIN_POOL_SIZE` | Integer | `1` | `1` | `1` | `2` | No |
| | `RIPTIDE_HEADLESS_IDLE_TIMEOUT` | Integer (sec) | `300` | `60` | `300` | `600` | No |
| | `RIPTIDE_HEADLESS_LAUNCH_TIMEOUT` | Integer (sec) | `30` | `60` | `30` | `20` | No |
| **Spider** | | | | | | | |
| | `SPIDER_ENABLE` | Boolean | `false` | `true` | `false` | `false` | No |
| | `SPIDER_MAX_DEPTH` | Integer | `3` | `5` | `3` | `2` | No |
| | `SPIDER_MAX_PAGES` | Integer | `100` | `50` | `100` | `200` | No |
| | `SPIDER_CONCURRENCY` | Integer | `4` | `2` | `4` | `8` | No |
| | `SPIDER_RESPECT_ROBOTS` | Boolean | `true` | `false` | `true` | `true` | No |
| **Features** | | | | | | | |
| | `ENHANCED_PIPELINE_ENABLE` | Boolean | `true` | `true` | `true` | `true` | No |
| | `ENHANCED_PIPELINE_METRICS` | Boolean | `true` | `true` | `true` | `true` | No |
| | `ENHANCED_PIPELINE_DEBUG` | Boolean | `false` | `true` | `false` | `false` | No |
| | `ENABLE_MULTI_TENANCY` | Boolean | `false` | `false` | `true` | `true` | No |
| | `RIPTIDE_TENANT_ISOLATION_ENABLED` | Boolean | `true` | `false` | `true` | `true` | No |
| | `ENABLE_COMPRESSION` | Boolean | `true` | `false` | `true` | `true` | No |

---

## 7. Compile-Time vs Runtime Configuration

### 7.1 Configuration Decision Matrix

| Configuration Type | When to Use | Examples | Benefits | Drawbacks |
|-------------------|-------------|----------|----------|-----------|
| **Compile-Time Constants** | Core algorithms, performance flags, feature gates | SIMD acceleration, memory allocator | Zero runtime overhead, compiler optimizations | Requires rebuild to change |
| **Compile-Time Features (Cargo)** | Optional dependencies, platform-specific code | `experimental-wasm`, `redis-caching` | Dead code elimination, smaller binaries | Requires separate builds per environment |
| **Runtime Config Files** | Environment-specific profiles, complex config | `development.toml`, `production.toml` | Easy environment switching, no rebuild | Slight parsing overhead, disk dependency |
| **Environment Variables** | Secrets, deployment-specific values, overrides | API keys, endpoints, resource limits | Secure secret injection, runtime flexibility | No type checking, string parsing |

### 7.2 Recommended Configuration Strategy

#### Use Compile-Time Configuration For:

1. **Performance Optimizations**
   - SIMD acceleration
   - Zero-copy optimizations
   - Vectorized processing
   - Memory allocator selection (jemalloc)

2. **Feature Gates**
   - Major feature toggles (search integration, event system)
   - Experimental features (neural prediction, adaptive scaling)
   - Platform-specific code

3. **Safety Features**
   - Golden test enforcement
   - Regression detection
   - Memory limit enforcement

**Implementation:**
```toml
# config/feature-flags/compile-time.toml
[performance]
simd_acceleration = true
zero_copy_optimization = true

[experimental]
neural_prediction = false
```

#### Use Runtime Config Files For:

1. **Environment Profiles**
   - Development vs staging vs production settings
   - Resource limit profiles
   - Logging configurations

2. **Complex Structured Configuration**
   - LLM provider configurations (multiple providers with nested settings)
   - Monitoring configurations (thresholds, intervals, channels)
   - Backpressure configurations

3. **Tunable Performance Parameters**
   - Connection pool sizes
   - Buffer sizes
   - Timeout values

**Implementation:**
```toml
# crates/riptide-performance/config/production.toml
[memory_profiling]
enabled = true
sampling_interval_secs = 30

[thresholds]
warning_threshold_mb = 650
critical_threshold_mb = 750

[resource_limits]
max_concurrent_requests = 1000
rate_limit_per_second = 100
```

#### Use Environment Variables For:

1. **Secrets & Credentials**
   - API keys (OpenAI, Anthropic, Serper)
   - Database passwords
   - TLS certificates/keys
   - Authentication tokens

2. **Deployment-Specific Values**
   - Service discovery endpoints (Redis URL, headless URL)
   - External API endpoints
   - Telemetry collectors
   - CDN URLs

3. **Runtime Overrides**
   - Resource limit overrides for specific deployments
   - Debug mode toggles
   - Feature flag overrides

**Implementation:**
```env
# Production .env (secrets managed by secret manager)
OPENAI_API_KEY=${SECRET_MANAGER:openai-prod-key}
REDIS_URL=redis://redis-cluster.prod.internal:6379/0
TELEMETRY_OTLP_ENDPOINT=https://otel-collector.prod.internal:4317
RIPTIDE_MAX_CONCURRENT_RENDERS=20  # Override for high-traffic instance
```

### 7.3 Configuration Precedence Implementation

**Current Implementation in `ApiConfig::from_env()`:**

```rust
pub fn from_env() -> Self {
    let mut config = Self::default(); // 1. Start with code defaults

    // 2. Load from config files (environment-specific TOML)
    // (Not yet implemented in current code)

    // 3. Override with environment variables (highest precedence)
    if let Ok(val) = std::env::var("RIPTIDE_MAX_CONCURRENT_RENDERS") {
        if let Ok(val) = val.parse() {
            config.resources.max_concurrent_renders = val;
        }
    }
    // ... more overrides ...

    config
}
```

**Recommended Enhancement:**

```rust
pub fn from_env() -> Self {
    // 1. Load compile-time defaults
    let mut config = Self::default();

    // 2. Load from environment-specific config file
    let env = std::env::var("RIPTIDE_ENV").unwrap_or_else(|_| "development".to_string());
    let config_path = format!("config/{}.toml", env);
    if let Ok(file_config) = Self::from_file(&config_path) {
        config.merge(file_config);
    }

    // 3. Override with environment variables (highest precedence)
    config.apply_env_overrides();

    config
}
```

---

## 8. Recommendations

### 8.1 Critical Actions Required

1. **Add Missing Environment-Specific Configuration Files**

   **Status:** Partially implemented
   **Priority:** HIGH

   **Current State:**
   - ✅ `/crates/riptide-performance/config/development.toml`
   - ✅ `/crates/riptide-performance/config/production.toml`
   - ✅ `/config/feature-flags/compile-time.toml`
   - ✅ `/configs/resource_management.toml`
   - ❌ Missing: `/config/api/development.toml`
   - ❌ Missing: `/config/api/staging.toml`
   - ❌ Missing: `/config/api/production.toml`

   **Action Items:**
   ```
   ├── config/
   │   ├── api/
   │   │   ├── development.toml      # Create
   │   │   ├── staging.toml          # Create
   │   │   └── production.toml       # Create
   │   ├── intelligence/
   │   │   ├── development.yaml      # Optional
   │   │   ├── staging.yaml          # Optional
   │   │   └── production.yaml       # Optional
   │   └── feature-flags/
   │       └── compile-time.toml     # ✅ Exists
   ```

2. **Implement Configuration File Loading**

   **Status:** Not implemented
   **Priority:** HIGH

   **Current Code:**
   ```rust
   // crates/riptide-api/src/config.rs
   impl ApiConfig {
       pub fn from_env() -> Self {
           let mut config = Self::default();
           // Only reads environment variables
           // TODO: Add config file loading
       }
   }
   ```

   **Recommended Implementation:**
   ```rust
   impl ApiConfig {
       pub fn from_env() -> Result<Self, ConfigError> {
           // 1. Detect environment
           let env = std::env::var("RIPTIDE_ENV")
               .unwrap_or_else(|_| "development".to_string());

           // 2. Load base configuration from file
           let config_path = format!("config/api/{}.toml", env);
           let mut config = if Path::new(&config_path).exists() {
               Self::from_file(&config_path)?
           } else {
               Self::default()
           };

           // 3. Override with environment variables
           config.apply_env_overrides()?;

           // 4. Validate
           config.validate()?;

           Ok(config)
       }

       fn from_file(path: &str) -> Result<Self, ConfigError> {
           let content = std::fs::read_to_string(path)?;
           toml::from_str(&content).map_err(Into::into)
       }
   }
   ```

3. **Standardize Secret Management**

   **Status:** Ad-hoc environment variables
   **Priority:** CRITICAL (for production)

   **Current Issues:**
   - Secrets stored in plain text `.env` files
   - No secret rotation mechanism
   - No audit trail for secret access

   **Recommended Solution:**
   - **Development:** `.env` files (acceptable for local dev)
   - **Staging/Production:** Secret manager integration

   **Implementation:**
   ```rust
   // config/secrets.rs
   pub enum SecretSource {
       EnvVar(String),           // For development
       AwsSecretsManager(String), // For AWS deployments
       HashicorpVault(String),    // For Vault deployments
       AzureKeyVault(String),     // For Azure deployments
   }

   impl SecretSource {
       pub async fn get(&self) -> Result<String, SecretError> {
           match self {
               Self::EnvVar(key) => {
                   std::env::var(key).map_err(|_| SecretError::NotFound)
               }
               Self::AwsSecretsManager(arn) => {
                   // AWS SDK integration
                   aws_secrets_manager::get_secret(arn).await
               }
               Self::HashicorpVault(path) => {
                   // Vault API integration
                   vault_client::read_secret(path).await
               }
               Self::AzureKeyVault(uri) => {
                   // Azure SDK integration
                   azure_keyvault::get_secret(uri).await
               }
           }
       }
   }
   ```

   **Usage:**
   ```env
   # Development
   OPENAI_API_KEY=sk-dev-test-key

   # Production (using secret reference)
   OPENAI_API_KEY=aws-sm:arn:aws:secretsmanager:us-east-1:123456789:secret:openai-prod-key
   REDIS_URL=vault:secret/data/redis/prod/url
   ```

4. **Add Service Discovery Integration**

   **Status:** Not implemented
   **Priority:** MEDIUM (for multi-instance deployments)

   **Current State:**
   - Static URLs configured via environment variables
   - No health checking of dependent services
   - No automatic failover

   **Recommended Implementation:**
   ```rust
   // config/service_discovery.rs
   pub trait ServiceDiscovery {
       async fn resolve_endpoint(&self, service_name: &str) -> Result<Url, DiscoveryError>;
       async fn watch_endpoint(&self, service_name: &str) -> impl Stream<Item = Url>;
   }

   pub struct ConsulServiceDiscovery { /* ... */ }
   pub struct StaticServiceDiscovery { /* ... */ } // For development

   // Usage in ApiConfig
   impl ApiConfig {
       pub async fn resolve_redis_url(&self) -> Result<String, ConfigError> {
           if let Some(discovery) = &self.service_discovery {
               let url = discovery.resolve_endpoint("redis").await?;
               Ok(url.to_string())
           } else {
               Ok(std::env::var("REDIS_URL")
                   .unwrap_or_else(|_| "redis://localhost:6379/0".to_string()))
           }
       }
   }
   ```

5. **Add CDN Configuration Support**

   **Status:** Not implemented
   **Priority:** LOW (future enhancement)

   **Recommended Configuration:**
   ```toml
   # config/api/production.toml
   [cdn]
   enabled = true
   base_url = "https://cdn.riptide.example.com"
   static_assets_path = "/static"
   max_age_seconds = 31536000  # 1 year

   [cdn.origins]
   primary = "https://origin-1.riptide.internal"
   fallback = "https://origin-2.riptide.internal"
   ```

   ```env
   # Environment override
   RIPTIDE_CDN_ENABLED=true
   RIPTIDE_CDN_BASE_URL=https://cdn.riptide.example.com
   ```

### 8.2 Configuration Best Practices

1. **Environment Variable Naming Convention**

   **Current Standard:** `RIPTIDE_<COMPONENT>_<SETTING>`

   ✅ **Good Examples:**
   - `RIPTIDE_API_HOST`
   - `RIPTIDE_MAX_CONCURRENT_RENDERS`
   - `RIPTIDE_MEMORY_LIMIT_MB`

   ❌ **Avoid:**
   - Mixing naming conventions (e.g., `REDIS_URL` vs `RIPTIDE_REDIS_URL`)
   - Inconsistent units (e.g., some timeouts in seconds, others in milliseconds)
   - Unclear abbreviations (e.g., `RIPTIDE_MCR` instead of `RIPTIDE_MAX_CONCURRENT_RENDERS`)

2. **Configuration Validation**

   **Current State:** Validation in `ApiConfig::validate()`

   **Recommendations:**
   - Validate configuration at startup, not runtime
   - Provide clear error messages with suggested fixes
   - Fail fast on invalid configuration

   **Example:**
   ```rust
   pub fn validate(&self) -> Result<(), ConfigError> {
       // Validate with helpful error messages
       if self.resources.max_concurrent_renders == 0 {
           return Err(ConfigError::Invalid {
               field: "RIPTIDE_MAX_CONCURRENT_RENDERS".to_string(),
               value: "0".to_string(),
               reason: "Must be greater than 0".to_string(),
               suggestion: "Try setting RIPTIDE_MAX_CONCURRENT_RENDERS=10".to_string(),
           });
       }

       // Validate dependent settings
       if self.headless.min_pool_size > self.headless.max_pool_size {
           return Err(ConfigError::Inconsistent {
               fields: vec![
                   "RIPTIDE_HEADLESS_MIN_POOL_SIZE".to_string(),
                   "RIPTIDE_HEADLESS_MAX_POOL_SIZE".to_string(),
               ],
               reason: "Minimum pool size cannot exceed maximum pool size".to_string(),
           });
       }

       Ok(())
   }
   ```

3. **Configuration Documentation**

   **Current State:** Comments in `.env.example`, code comments

   **Recommendations:**
   - Generate configuration reference documentation from code
   - Include environment-specific examples
   - Document validation rules and constraints

   **Example Tool:**
   ```bash
   # Generate configuration documentation
   cargo run --bin config-doc-generator > docs/configuration/REFERENCE.md
   ```

4. **Configuration Testing**

   **Recommendations:**
   - Unit test configuration loading and validation
   - Integration test environment-specific profiles
   - Test secret resolution in CI/CD

   **Example:**
   ```rust
   #[cfg(test)]
   mod tests {
       #[test]
       fn test_load_development_config() {
           std::env::set_var("RIPTIDE_ENV", "development");
           let config = ApiConfig::from_env().unwrap();
           assert_eq!(config.performance.render_timeout_secs, 5);
       }

       #[test]
       fn test_load_production_config() {
           std::env::set_var("RIPTIDE_ENV", "production");
           let config = ApiConfig::from_env().unwrap();
           assert_eq!(config.performance.render_timeout_secs, 3);
       }

       #[test]
       fn test_env_override_config_file() {
           std::env::set_var("RIPTIDE_ENV", "production");
           std::env::set_var("RIPTIDE_RENDER_TIMEOUT", "10");
           let config = ApiConfig::from_env().unwrap();
           // Env var should override config file
           assert_eq!(config.performance.render_timeout_secs, 10);
       }
   }
   ```

### 8.3 Deployment Checklist

#### Development Environment Setup
```bash
# 1. Copy example configuration
cp .env.example .env

# 2. Set development-specific overrides
export RIPTIDE_ENV=development
export RUST_LOG=debug
export RIPTIDE_RATE_LIMIT_ENABLED=false
export SEARCH_BACKEND=none

# 3. Start local services
docker-compose up -d redis

# 4. Run application
cargo run --bin riptide-api
```

#### Staging Environment Setup
```bash
# 1. Set environment
export RIPTIDE_ENV=staging

# 2. Configure secrets (using secret manager)
export SERPER_API_KEY=$(aws secretsmanager get-secret-value --secret-id serper-staging-key --query SecretString --output text)
export OPENAI_API_KEY=$(aws secretsmanager get-secret-value --secret-id openai-staging-key --query SecretString --output text)

# 3. Configure service endpoints
export REDIS_URL=redis://redis.staging.internal:6379/0
export HEADLESS_URL=http://headless-service.staging.internal:9123
export TELEMETRY_OTLP_ENDPOINT=http://otel-collector.staging.internal:4317

# 4. Enable auth and security
export REQUIRE_AUTH=true
export RIPTIDE_ENABLE_TLS=true

# 5. Run with production-like settings
cargo run --release --bin riptide-api
```

#### Production Environment Setup
```bash
# 1. Set environment
export RIPTIDE_ENV=production

# 2. Configure secrets (using secret manager)
export SERPER_API_KEY=$(vault kv get -field=value secret/riptide/prod/serper)
export OPENAI_API_KEY=$(vault kv get -field=value secret/riptide/prod/openai)
export API_KEYS=$(vault kv get -field=value secret/riptide/prod/api-keys)

# 3. Configure production endpoints
export REDIS_URL=redis://redis-cluster.prod.internal:6379/0
export HEADLESS_URL=http://headless-service.prod.internal:9123
export TELEMETRY_OTLP_ENDPOINT=https://otel-collector.prod.internal:4317
export RIPTIDE_PROMETHEUS_ENDPOINT=http://prometheus.prod.internal:9090

# 4. Enable all security features
export REQUIRE_AUTH=true
export RIPTIDE_ENABLE_TLS=true
export ENABLE_MULTI_TENANCY=true
export RIPTIDE_TENANT_ISOLATION_ENABLED=true

# 5. Set production resource limits
export RIPTIDE_MAX_CONCURRENT_RENDERS=20
export RIPTIDE_MEMORY_LIMIT_MB=4096
export RIPTIDE_RATE_LIMIT_RPS=1.5

# 6. Set strict logging
export RUST_LOG=warn,riptide=info
export TELEMETRY_SAMPLING_RATIO=0.01

# 7. Run production build
cargo run --release --bin riptide-api
```

---

## Appendix A: Configuration File Examples

### A.1 Development API Configuration

**File:** `config/api/development.toml`

```toml
[resources]
max_concurrent_renders = 5
max_concurrent_pdf = 2
max_concurrent_wasm = 2
global_timeout_secs = 60
cleanup_interval_secs = 60
enable_monitoring = true
health_check_interval_secs = 30

[performance]
render_timeout_secs = 5  # Relaxed for debugging
pdf_timeout_secs = 60
wasm_timeout_secs = 30
http_timeout_secs = 30
memory_cleanup_threshold_mb = 256
auto_cleanup_on_timeout = true
degradation_threshold = 0.8

[rate_limiting]
enabled = false  # Disabled for development
requests_per_second_per_host = 10.0
jitter_factor = 0.0
burst_capacity_per_host = 10
window_duration_secs = 10
cleanup_interval_secs = 300
max_tracked_hosts = 1000

[memory]
max_memory_per_request_mb = 128
global_memory_limit_mb = 1024
memory_soft_limit_mb = 300
memory_hard_limit_mb = 400
pressure_threshold = 0.7
auto_gc = true
gc_trigger_threshold_mb = 512
monitoring_interval_secs = 30
enable_leak_detection = true
enable_proactive_monitoring = true

[headless]
max_pool_size = 2
min_pool_size = 1
idle_timeout_secs = 60
health_check_interval_secs = 60
max_pages_per_browser = 10
restart_threshold = 5
enable_recycling = true
launch_timeout_secs = 60
max_retries = 3

[pdf]
max_concurrent = 2
processing_timeout_secs = 60
max_file_size_mb = 50
enable_streaming = true
queue_size = 10
queue_timeout_secs = 60

[wasm]
instances_per_worker = 1
module_timeout_secs = 30
max_memory_mb = 64
enable_recycling = false
health_check_interval_secs = 120
max_operations_per_instance = 10000
restart_threshold = 10

[search]
backend = "none"  # No search provider for dev
timeout_secs = 60
enable_url_parsing = true
circuit_breaker_failure_threshold = 50
circuit_breaker_min_requests = 5
circuit_breaker_recovery_timeout_secs = 60
```

### A.2 Production API Configuration

**File:** `config/api/production.toml`

```toml
[resources]
max_concurrent_renders = 20
max_concurrent_pdf = 2
max_concurrent_wasm = 8
global_timeout_secs = 30
cleanup_interval_secs = 60
enable_monitoring = true
health_check_interval_secs = 30

[performance]
render_timeout_secs = 3  # STRICT: Hard requirement
pdf_timeout_secs = 20
wasm_timeout_secs = 5
http_timeout_secs = 10
memory_cleanup_threshold_mb = 512
auto_cleanup_on_timeout = true
degradation_threshold = 0.8

[rate_limiting]
enabled = true
requests_per_second_per_host = 1.5  # STRICT: Hard requirement
jitter_factor = 0.1
burst_capacity_per_host = 3
window_duration_secs = 60
cleanup_interval_secs = 300
max_tracked_hosts = 50000

[memory]
max_memory_per_request_mb = 512
global_memory_limit_mb = 4096
memory_soft_limit_mb = 600
memory_hard_limit_mb = 800
pressure_threshold = 0.9
auto_gc = true
gc_trigger_threshold_mb = 2048
monitoring_interval_secs = 30
enable_leak_detection = true
enable_proactive_monitoring = true

[headless]
max_pool_size = 5  # Scaled for production
min_pool_size = 2
idle_timeout_secs = 600
health_check_interval_secs = 60
max_pages_per_browser = 10
restart_threshold = 5
enable_recycling = true
launch_timeout_secs = 20
max_retries = 3

[pdf]
max_concurrent = 2
processing_timeout_secs = 20
max_file_size_mb = 100
enable_streaming = true
queue_size = 100
queue_timeout_secs = 60

[wasm]
instances_per_worker = 1
module_timeout_secs = 5
max_memory_mb = 256
enable_recycling = false
health_check_interval_secs = 120
max_operations_per_instance = 10000
restart_threshold = 10

[search]
backend = "serper"
timeout_secs = 20
enable_url_parsing = true
circuit_breaker_failure_threshold = 50
circuit_breaker_min_requests = 5
circuit_breaker_recovery_timeout_secs = 60
```

---

## Appendix B: Environment Variable Quick Reference

### B.1 Copy-Paste Environment Configurations

#### Development `.env`
```env
# === DEVELOPMENT ENVIRONMENT ===

# API
RIPTIDE_API_HOST=127.0.0.1
RIPTIDE_API_PORT=8080
RIPTIDE_API_URL=http://localhost:8080

# Services
REDIS_URL=redis://localhost:6379/0
HEADLESS_URL=http://localhost:9123

# Search (disabled for dev)
SEARCH_BACKEND=none

# Logging
RUST_LOG=debug,hyper=info,tokio=info
RIPTIDE_CLI_VERBOSE=true
RIPTIDE_DEV_MODE=true

# Telemetry
TELEMETRY_ENABLED=true
TELEMETRY_SERVICE_NAME=riptide-dev
TELEMETRY_EXPORTER_TYPE=stdout
TELEMETRY_SAMPLING_RATIO=1.0

# Concurrency (low for dev)
RIPTIDE_MAX_CONCURRENT_RENDERS=5
RIPTIDE_MAX_CONCURRENT_PDF=2
RIPTIDE_MAX_CONCURRENT_WASM=2
WORKER_POOL_SIZE=2

# Memory (low for dev)
RIPTIDE_MEMORY_LIMIT_MB=1024
RIPTIDE_MEMORY_MAX_PER_REQUEST_MB=128

# Timeouts (relaxed for debugging)
RIPTIDE_RENDER_TIMEOUT=5
RIPTIDE_PDF_TIMEOUT=60
RIPTIDE_WASM_TIMEOUT=30
RIPTIDE_HTTP_TIMEOUT=30

# Rate Limiting (disabled for dev)
RIPTIDE_RATE_LIMIT_ENABLED=false

# Spider (enabled for dev testing)
SPIDER_ENABLE=true
SPIDER_MAX_DEPTH=5
SPIDER_MAX_PAGES=50
SPIDER_CONCURRENCY=2
```

#### Staging `.env`
```env
# === STAGING ENVIRONMENT ===

# API
RIPTIDE_API_HOST=0.0.0.0
RIPTIDE_API_PORT=8080
RIPTIDE_API_URL=https://api-staging.riptide.example.com

# Services (staging internal endpoints)
REDIS_URL=redis://redis.staging.internal:6379/0
HEADLESS_URL=http://headless-service.staging.internal:9123

# Search (staging provider)
SEARCH_BACKEND=serper
SERPER_API_KEY=${STAGING_SERPER_KEY}  # Managed by secret manager
SEARCH_TIMEOUT=30

# LLM Providers (staging keys)
OPENAI_API_KEY=${STAGING_OPENAI_KEY}
ANTHROPIC_API_KEY=${STAGING_ANTHROPIC_KEY}

# Logging
RUST_LOG=info,riptide=debug
RIPTIDE_CLI_VERBOSE=false
RIPTIDE_DEV_MODE=false

# Telemetry (staging collector)
TELEMETRY_ENABLED=true
TELEMETRY_SERVICE_NAME=riptide-staging
TELEMETRY_EXPORTER_TYPE=otlp
TELEMETRY_OTLP_ENDPOINT=http://otel-collector.staging.internal:4317
TELEMETRY_SAMPLING_RATIO=0.1

# Authentication
REQUIRE_AUTH=true
API_KEYS=${STAGING_API_KEYS}
RIPTIDE_ENABLE_TLS=true
RIPTIDE_TLS_CERT_PATH=/certs/staging.pem
RIPTIDE_TLS_KEY_PATH=/certs/staging.key

# Concurrency (moderate for staging)
RIPTIDE_MAX_CONCURRENT_RENDERS=10
RIPTIDE_MAX_CONCURRENT_PDF=2
RIPTIDE_MAX_CONCURRENT_WASM=4
WORKER_POOL_SIZE=4

# Memory (moderate for staging)
RIPTIDE_MEMORY_LIMIT_MB=2048
RIPTIDE_MEMORY_MAX_PER_REQUEST_MB=256

# Timeouts (production-like)
RIPTIDE_RENDER_TIMEOUT=3
RIPTIDE_PDF_TIMEOUT=30
RIPTIDE_WASM_TIMEOUT=10
RIPTIDE_HTTP_TIMEOUT=15

# Rate Limiting (enabled, slightly relaxed)
RIPTIDE_RATE_LIMIT_ENABLED=true
RIPTIDE_RATE_LIMIT_RPS=2.0

# Multi-Tenancy
ENABLE_MULTI_TENANCY=true
RIPTIDE_TENANT_ISOLATION_ENABLED=true
```

#### Production `.env`
```env
# === PRODUCTION ENVIRONMENT ===

# API
RIPTIDE_API_HOST=0.0.0.0
RIPTIDE_API_PORT=8080
RIPTIDE_API_URL=https://api.riptide.example.com

# Services (production internal endpoints)
REDIS_URL=redis://redis-cluster.prod.internal:6379/0
HEADLESS_URL=http://headless-service.prod.internal:9123

# Search (production provider)
SEARCH_BACKEND=serper
SERPER_API_KEY=${PROD_SERPER_KEY}  # Managed by secret manager
SEARCH_TIMEOUT=20

# LLM Providers (production keys)
OPENAI_API_KEY=${PROD_OPENAI_KEY}
ANTHROPIC_API_KEY=${PROD_ANTHROPIC_KEY}

# Logging (strict)
RUST_LOG=warn,riptide=info
RIPTIDE_CLI_VERBOSE=false
RIPTIDE_DEV_MODE=false

# Telemetry (production collector)
TELEMETRY_ENABLED=true
TELEMETRY_SERVICE_NAME=riptide-prod
TELEMETRY_EXPORTER_TYPE=otlp
TELEMETRY_OTLP_ENDPOINT=https://otel-collector.prod.internal:4317
TELEMETRY_SAMPLING_RATIO=0.01
RIPTIDE_PROMETHEUS_ENDPOINT=http://prometheus.prod.internal:9090

# Authentication (strict)
REQUIRE_AUTH=true
API_KEYS=${PROD_API_KEYS}
RIPTIDE_ENABLE_TLS=true
RIPTIDE_TLS_CERT_PATH=/certs/prod.pem
RIPTIDE_TLS_KEY_PATH=/certs/prod.key

# Concurrency (high for production)
RIPTIDE_MAX_CONCURRENT_RENDERS=20
RIPTIDE_MAX_CONCURRENT_PDF=2
RIPTIDE_MAX_CONCURRENT_WASM=8
WORKER_POOL_SIZE=8
STREAM_MAX_CONCURRENT=200

# Memory (generous for production)
RIPTIDE_MEMORY_LIMIT_MB=4096
RIPTIDE_MEMORY_MAX_PER_REQUEST_MB=512
RIPTIDE_MEMORY_PRESSURE_THRESHOLD=0.9

# Timeouts (strict for performance)
RIPTIDE_RENDER_TIMEOUT=3  # HARD REQUIREMENT
RIPTIDE_PDF_TIMEOUT=20
RIPTIDE_WASM_TIMEOUT=5
RIPTIDE_HTTP_TIMEOUT=10
RIPTIDE_GLOBAL_TIMEOUT=30

# Rate Limiting (strict)
RIPTIDE_RATE_LIMIT_ENABLED=true
RIPTIDE_RATE_LIMIT_RPS=1.5  # HARD REQUIREMENT
RIPTIDE_RATE_LIMIT_MAX_HOSTS=50000

# Multi-Tenancy (enabled)
ENABLE_MULTI_TENANCY=true
RIPTIDE_TENANT_ISOLATION_ENABLED=true

# Compression
ENABLE_COMPRESSION=true

# Spider (disabled in production)
SPIDER_ENABLE=false
```

---

## Conclusion

This document provides a comprehensive architecture analysis of environment-specific configuration in the EventMesh (RipTide) system. Key findings:

1. **Three-tier configuration strategy** is in place but not fully implemented
2. **Service discovery and external endpoints** are configured via environment variables
3. **Feature flags** use compile-time TOML configuration effectively
4. **Debug/logging levels** are environment-driven with appropriate defaults
5. **Resource limits** have comprehensive environment variable support with clear requirements

**Immediate Action Items:**
1. Implement config file loading in `ApiConfig::from_env()`
2. Add missing environment-specific TOML configuration files
3. Integrate secret manager for staging/production
4. Document configuration validation rules
5. Add configuration testing in CI/CD pipeline

**Architecture Strengths:**
- ✅ Clear separation between compile-time and runtime configuration
- ✅ Comprehensive environment variable support
- ✅ Strong validation with hard requirements enforced
- ✅ Environment-specific performance tuning profiles

**Architecture Gaps:**
- ❌ Config file loading not implemented (relying only on env vars)
- ❌ Secret management is ad-hoc (plain text `.env` files)
- ❌ No service discovery integration
- ❌ Configuration testing coverage is minimal

---

**Document Status:** COMPLETE
**Next Review:** When implementing config file loading or secret management
**Owner:** System Architecture Team
