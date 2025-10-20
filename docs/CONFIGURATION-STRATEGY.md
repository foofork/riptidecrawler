# RipTide Configuration Strategy

**Generated:** 2025-10-20
**Status:** Architectural Decision Document

## Executive Summary

This document provides a comprehensive configuration strategy for the RipTide EventMesh project, categorizing hardcoded values into appropriate configuration mechanisms (.env, config.toml, or code constants) based on Rust best practices.

---

## Configuration Decision Matrix

### Classification Criteria

| Type | Use Case | Examples | Rationale |
|------|----------|----------|-----------|
| **Environment Variables (.env)** | Deployment-specific, secrets, runtime overrides | URLs, ports, API keys, host addresses | Changes per environment; security-sensitive |
| **TOML Configuration** | Structured settings, feature flags, complex policies | Timeouts, retry policies, limits, algorithms | Requires complex structure; changes occasionally |
| **Code Constants** | True invariants, protocol limits, algorithm parameters | Magic numbers, protocol constants, WASM limits | Never changes; part of business logic |

---

## 📋 Detailed Configuration Catalog

### 1. RipTide API Configuration

#### 1.1 Resource Management
| Value | Current | Type | Config Name | Rationale |
|-------|---------|------|-------------|-----------|
| Max concurrent renders | 10 | TOML | `api.resources.max_concurrent_renders` | Tunable per deployment scale |
| Max concurrent PDF | 2 | TOML | `api.resources.max_concurrent_pdf` | Hardware-dependent limit |
| Max concurrent WASM | 4 | TOML | `api.resources.max_concurrent_wasm` | Resource constraint |
| Global timeout | 30s | TOML | `api.resources.global_timeout_secs` | Deployment-specific |
| Cleanup interval | 60s | TOML | `api.resources.cleanup_interval_secs` | Operational tuning |
| Health check interval | 30s | TOML | `api.resources.health_check_interval_secs` | Monitoring frequency |

#### 1.2 Performance Limits
| Value | Current | Type | Config Name | Rationale |
|-------|---------|------|-------------|-----------|
| Render timeout | 3s | TOML | `api.performance.render_timeout_secs` | **Critical requirement** - configurable for testing |
| PDF timeout | 10s | TOML | `api.performance.pdf_timeout_secs` | Document complexity varies |
| WASM timeout | 5s | TOML | `api.performance.wasm_timeout_secs` | Execution safety limit |
| HTTP timeout | 10s | TOML | `api.performance.http_timeout_secs` | Network variability |
| Memory cleanup threshold | 512 MB | TOML | `api.performance.memory_cleanup_threshold_mb` | Memory management |
| Degradation threshold | 0.8 | TOML | `api.performance.degradation_threshold` | Performance monitoring |

#### 1.3 Rate Limiting
| Value | Current | Type | Config Name | Rationale |
|-------|---------|------|-------------|-----------|
| Requests per second | 1.5 | ENV | `RIPTIDE_RATE_LIMIT_RPS` | **Critical requirement** - varies by deployment |
| Jitter factor | 0.1 | TOML | `api.rate_limiting.jitter_factor` | Algorithm parameter |
| Burst capacity | 3 | TOML | `api.rate_limiting.burst_capacity_per_host` | Traffic pattern tuning |
| Window duration | 60s | TOML | `api.rate_limiting.window_duration_secs` | Rate calculation period |
| Max tracked hosts | 10000 | TOML | `api.rate_limiting.max_tracked_hosts` | Memory vs. accuracy tradeoff |

#### 1.4 Memory Management
| Value | Current | Type | Config Name | Rationale |
|-------|---------|------|-------------|-----------|
| Max memory per request | 256 MB | TOML | `api.memory.max_memory_per_request_mb` | Safety limit |
| Global memory limit | 2048 MB | ENV | `RIPTIDE_MEMORY_LIMIT_MB` | Deployment-specific |
| Memory soft limit | 400 MB | TOML | `api.memory.memory_soft_limit_mb` | Warning threshold |
| Memory hard limit | 500 MB | TOML | `api.memory.memory_hard_limit_mb` | Rejection threshold |
| Pressure threshold | 0.85 | TOML | `api.memory.pressure_threshold` | Algorithm parameter |
| GC trigger threshold | 1024 MB | TOML | `api.memory.gc_trigger_threshold_mb` | Performance tuning |

#### 1.5 Browser Pool
| Value | Current | Type | Config Name | Rationale |
|-------|---------|------|-------------|-----------|
| Max pool size | 3 | TOML | `api.headless.max_pool_size` | **Critical requirement** - resource constraint |
| Min pool size | 1 | TOML | `api.headless.min_pool_size` | Always have one ready |
| Idle timeout | 300s | TOML | `api.headless.idle_timeout_secs` | Resource cleanup |
| Launch timeout | 30s | TOML | `api.headless.launch_timeout_secs` | Browser startup time |
| Max retries | 3 | TOML | `api.headless.max_retries` | Reliability parameter |

#### 1.6 Search Provider
| Value | Current | Type | Config Name | Rationale |
|-------|---------|------|-------------|-----------|
| Search backend | "serper" | ENV | `RIPTIDE_SEARCH_BACKEND` | Deployment choice |
| Search timeout | 30s | TOML | `api.search.timeout_secs` | Provider variability |
| Circuit breaker threshold | 50% | TOML | `api.search.circuit_breaker_failure_threshold` | Reliability tuning |
| Min requests before open | 5 | TOML | `api.search.circuit_breaker_min_requests` | Statistical significance |
| Recovery timeout | 60s | TOML | `api.search.circuit_breaker_recovery_timeout_secs` | Backoff period |

---

### 2. RipTide Intelligence Configuration

#### 2.1 Circuit Breaker
| Value | Current | Type | Config Name | Rationale |
|-------|---------|------|-------------|-----------|
| Failure threshold | 5 | TOML | `intelligence.circuit_breaker.failure_threshold` | Reliability tuning |
| Failure window | 60s | TOML | `intelligence.circuit_breaker.failure_window_secs` | Time-based tracking |
| Min request threshold | 10 | TOML | `intelligence.circuit_breaker.min_request_threshold` | Statistical validity |
| Recovery timeout | 30s | TOML | `intelligence.circuit_breaker.recovery_timeout_secs` | Backoff period |
| Max repair attempts | 1 | **CONSTANT** | N/A | **Hard requirement** - never changes |
| Success rate threshold | 0.7 | TOML | `intelligence.circuit_breaker.success_rate_threshold` | Recovery criteria |
| Half-open max requests | 3 | TOML | `intelligence.circuit_breaker.half_open_max_requests` | Test traffic limit |

#### 2.2 Timeout Configuration
| Value | Current | Type | Config Name | Rationale |
|-------|---------|------|-------------|-----------|
| Default timeout | 5s | TOML | `intelligence.timeout.default_secs` | LLM call safety |
| Completion timeout | 5s | TOML | `intelligence.timeout.completion_secs` | Generation time |
| Embedding timeout | 3s | TOML | `intelligence.timeout.embedding_secs` | Faster operation |
| Health check timeout | 2s | TOML | `intelligence.timeout.health_check_secs` | Quick validation |
| Strict completion | 3s | TOML | `intelligence.timeout.strict.completion_secs` | Aggressive limit |
| Relaxed completion | 10s | TOML | `intelligence.timeout.relaxed.completion_secs` | Lenient limit |

#### 2.3 Health Monitor
| Value | Current | Type | Config Name | Rationale |
|-------|---------|------|-------------|-----------|
| Check interval | 30s | TOML | `intelligence.health.interval_secs` | Monitoring frequency |
| Check timeout | 10s | TOML | `intelligence.health.timeout_secs` | Health check duration |
| Failure threshold | 3 | TOML | `intelligence.health.failure_threshold` | Degradation detection |
| Success threshold | 2 | TOML | `intelligence.health.success_threshold` | Recovery confirmation |
| Degraded threshold | 10.0% | TOML | `intelligence.health.degraded_threshold_pct` | Warning level |
| Critical threshold | 50.0% | TOML | `intelligence.health.critical_threshold_pct` | Alert level |

#### 2.4 Failover Configuration
| Value | Current | Type | Config Name | Rationale |
|-------|---------|------|-------------|-----------|
| Max retries | 3 | TOML | `intelligence.failover.max_retries` | Redundancy level |
| Retry delay | 500ms | TOML | `intelligence.failover.retry_delay_ms` | Backoff timing |
| Failback delay | 60s | TOML | `intelligence.failover.failback_delay_secs` | Recovery period |
| Health check threshold | 3 | TOML | `intelligence.failover.health_check_threshold` | Stability requirement |

#### 2.5 Provider Configuration
| Value | Current | Type | Config Name | Rationale |
|-------|---------|------|-------------|-----------|
| OpenAI API key | - | ENV | `OPENAI_API_KEY` | **Secret** - never in code |
| Anthropic API key | - | ENV | `ANTHROPIC_API_KEY` | **Secret** |
| Azure endpoint | - | ENV | `AZURE_OPENAI_ENDPOINT` | Deployment-specific |
| Azure API version | "2023-12-01-preview" | TOML | `intelligence.providers.azure.api_version` | Version tracking |
| Ollama base URL | "http://localhost:11434" | ENV | `OLLAMA_BASE_URL` | Local development |
| AWS region | - | ENV | `AWS_REGION` | Deployment-specific |
| Google project ID | - | ENV | `GOOGLE_VERTEX_PROJECT_ID` | GCP configuration |
| Google location | "us-central1" | TOML | `intelligence.providers.google.default_location` | Default region |

---

### 3. RipTide Persistence Configuration

#### 3.1 Redis/DragonflyDB
| Value | Current | Type | Config Name | Rationale |
|-------|---------|------|-------------|-----------|
| Redis URL | "redis://localhost:6379" | ENV | `RIPTIDE_REDIS_URL` | Connection string |
| Connection timeout | N/A | ENV | `RIPTIDE_REDIS_TIMEOUT_MS` | Network variability |
| Pool size | N/A | ENV | `RIPTIDE_REDIS_POOL_SIZE` | Concurrency requirement |
| Default TTL | 3600s | TOML | `persistence.cache.default_ttl_seconds` | Cache policy |
| Max entry size | 10 MB | TOML | `persistence.cache.max_entry_size_bytes` | Memory protection |
| Compression threshold | 1024 bytes | TOML | `persistence.cache.compression_threshold_bytes` | Performance optimization |
| Warming batch size | 100 | TOML | `persistence.cache.warming_batch_size` | Startup performance |

#### 3.2 Cache Configuration
| Value | Current | Type | Config Name | Rationale |
|-------|---------|------|-------------|-----------|
| Key prefix | "test" | ENV | `RIPTIDE_CACHE_PREFIX` | Environment isolation |
| Cache version | "v1" | TOML | `persistence.cache.version` | Schema versioning |
| Enable compression | false | TOML | `persistence.cache.enable_compression` | Feature flag |
| Enable warming | false | TOML | `persistence.cache.enable_warming` | Startup optimization |
| Eviction policy | LRU | TOML | `persistence.cache.eviction_policy` | Cache algorithm |

---

### 4. Cross-Cutting Configuration

#### 4.1 Logging & Telemetry
| Value | Current | Type | Config Name | Rationale |
|-------|---------|------|-------------|-----------|
| Log level | N/A | ENV | `RUST_LOG` | Standard Rust practice |
| OTLP endpoint | N/A | ENV | `OTEL_EXPORTER_OTLP_ENDPOINT` | Observability backend |
| Service name | N/A | ENV | `OTEL_SERVICE_NAME` | Telemetry identification |

#### 4.2 Network Configuration
| Value | Current | Type | Config Name | Rationale |
|-------|---------|------|-------------|-----------|
| Listen address | N/A | ENV | `RIPTIDE_LISTEN_ADDR` | Binding configuration |
| Listen port | N/A | ENV | `RIPTIDE_PORT` | Port assignment |
| TLS enabled | N/A | ENV | `RIPTIDE_TLS_ENABLED` | Security flag |
| TLS cert path | N/A | ENV | `RIPTIDE_TLS_CERT` | Certificate location |
| TLS key path | N/A | ENV | `RIPTIDE_TLS_KEY` | Key location |

#### 4.3 Worker Configuration
| Value | Current | Type | Config Name | Rationale |
|-------|---------|------|-------------|-----------|
| Worker threads | N/A | ENV | `RIPTIDE_WORKER_THREADS` | CPU allocation |
| Blocking threads | N/A | ENV | `RIPTIDE_BLOCKING_THREADS` | I/O thread pool |

---

## 📝 Recommended .env Template

```bash
# ============================================================================
# RipTide EventMesh Configuration
# ============================================================================

# -----------------------------------------------------------------------------
# Core Service Configuration
# -----------------------------------------------------------------------------
RIPTIDE_LISTEN_ADDR=0.0.0.0
RIPTIDE_PORT=8080
RUST_LOG=info,riptide=debug

# -----------------------------------------------------------------------------
# Resource Limits (Deployment-Specific)
# -----------------------------------------------------------------------------
RIPTIDE_MEMORY_LIMIT_MB=2048
RIPTIDE_RATE_LIMIT_RPS=1.5

# -----------------------------------------------------------------------------
# Search Provider
# -----------------------------------------------------------------------------
RIPTIDE_SEARCH_BACKEND=serper  # Options: serper, none, searxng
SERPER_API_KEY=your_serper_api_key_here

# -----------------------------------------------------------------------------
# LLM Provider API Keys (SECRETS - Never commit!)
# -----------------------------------------------------------------------------
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
AZURE_OPENAI_ENDPOINT=https://your-resource.openai.azure.com
AZURE_OPENAI_API_KEY=...
AWS_REGION=us-east-1
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...
GOOGLE_VERTEX_PROJECT_ID=your-project-id
GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
OLLAMA_BASE_URL=http://localhost:11434

# -----------------------------------------------------------------------------
# Persistence Layer
# -----------------------------------------------------------------------------
RIPTIDE_REDIS_URL=redis://localhost:6379
RIPTIDE_REDIS_POOL_SIZE=10
RIPTIDE_REDIS_TIMEOUT_MS=5000
RIPTIDE_CACHE_PREFIX=riptide

# -----------------------------------------------------------------------------
# Observability
# -----------------------------------------------------------------------------
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=riptide-api
OTEL_RESOURCE_ATTRIBUTES=environment=production,version=1.0.0

# -----------------------------------------------------------------------------
# TLS Configuration (Optional)
# -----------------------------------------------------------------------------
RIPTIDE_TLS_ENABLED=false
RIPTIDE_TLS_CERT=/path/to/cert.pem
RIPTIDE_TLS_KEY=/path/to/key.pem

# -----------------------------------------------------------------------------
# Worker Pool Configuration
# -----------------------------------------------------------------------------
RIPTIDE_WORKER_THREADS=8
RIPTIDE_BLOCKING_THREADS=16
```

---

## 🔧 Recommended config.toml Structure

```toml
# ============================================================================
# RipTide Configuration File
# ============================================================================
# This file contains structured configuration for timeouts, limits, and
# complex policies that don't change frequently across environments.

[api.resources]
max_concurrent_renders = 10
max_concurrent_pdf = 2
max_concurrent_wasm = 4
global_timeout_secs = 30
cleanup_interval_secs = 60
enable_monitoring = true
health_check_interval_secs = 30

[api.performance]
render_timeout_secs = 3
pdf_timeout_secs = 10
wasm_timeout_secs = 5
http_timeout_secs = 10
memory_cleanup_threshold_mb = 512
auto_cleanup_on_timeout = true
degradation_threshold = 0.8

[api.rate_limiting]
enabled = true
jitter_factor = 0.1
burst_capacity_per_host = 3
window_duration_secs = 60
cleanup_interval_secs = 300
max_tracked_hosts = 10000

[api.memory]
max_memory_per_request_mb = 256
memory_soft_limit_mb = 400
memory_hard_limit_mb = 500
pressure_threshold = 0.85
auto_gc = true
gc_trigger_threshold_mb = 1024
monitoring_interval_secs = 30
enable_leak_detection = true
enable_proactive_monitoring = true

[api.headless]
max_pool_size = 3
min_pool_size = 1
idle_timeout_secs = 300
health_check_interval_secs = 60
max_pages_per_browser = 10
restart_threshold = 5
enable_recycling = true
launch_timeout_secs = 30
max_retries = 3

[api.pdf]
max_concurrent = 2
processing_timeout_secs = 30
max_file_size_mb = 100
enable_streaming = true
queue_size = 50
queue_timeout_secs = 60

[api.wasm]
instances_per_worker = 1
module_timeout_secs = 10
max_memory_mb = 128
enable_recycling = false
health_check_interval_secs = 120
max_operations_per_instance = 10000
restart_threshold = 10

[api.search]
timeout_secs = 30
enable_url_parsing = true
circuit_breaker_failure_threshold = 50
circuit_breaker_min_requests = 5
circuit_breaker_recovery_timeout_secs = 60

[intelligence.circuit_breaker]
failure_threshold = 5
failure_window_secs = 60
min_request_threshold = 10
recovery_timeout_secs = 30
success_rate_threshold = 0.7
half_open_max_requests = 3

[intelligence.circuit_breaker.strict]
failure_threshold = 3
failure_window_secs = 30
min_request_threshold = 5
recovery_timeout_secs = 15
success_rate_threshold = 0.8
half_open_max_requests = 2

[intelligence.circuit_breaker.lenient]
failure_threshold = 10
failure_window_secs = 120
min_request_threshold = 20
recovery_timeout_secs = 60
success_rate_threshold = 0.6
half_open_max_requests = 5

[intelligence.timeout]
default_secs = 5
completion_secs = 5
embedding_secs = 3
health_check_secs = 2

[intelligence.timeout.strict]
completion_secs = 3
embedding_secs = 2
health_check_secs = 1

[intelligence.timeout.relaxed]
completion_secs = 10
embedding_secs = 5
health_check_secs = 3

[intelligence.health]
interval_secs = 30
timeout_secs = 10
failure_threshold = 3
success_threshold = 2
degraded_threshold_pct = 10.0
critical_threshold_pct = 50.0

[intelligence.failover]
max_retries = 3
retry_delay_ms = 500
failback_delay_secs = 60
health_check_threshold = 3
circuit_breaker_enabled = true
load_balancing_enabled = true

[intelligence.providers.azure]
api_version = "2023-12-01-preview"

[intelligence.providers.google]
default_location = "us-central1"

[persistence.cache]
default_ttl_seconds = 3600
max_entry_size_bytes = 10000000
enable_compression = false
compression_threshold_bytes = 1024
enable_warming = false
warming_batch_size = 100
version = "v1"
eviction_policy = "LRU"  # Options: LRU, LFU, FIFO

[persistence.cache.compression]
algorithm = "lz4"  # Options: lz4, gzip, zstd, none
```

---

## 🚫 Values That Should REMAIN as Code Constants

### Reason: These are true invariants that are part of the business logic or protocol specifications

```rust
// Circuit breaker hard requirement
pub const MAX_REPAIR_ATTEMPTS: u32 = 1;  // PR requirement - never exceeds 1

// WASM runtime constants
pub const WASM_MAX_STACK_SIZE: usize = 1024 * 1024;  // 1MB stack
pub const WASM_MAX_MEMORY_PAGES: u32 = 256;  // 16MB max

// Protocol constants
pub const HTTP2_MAX_CONCURRENT_STREAMS: u32 = 100;
pub const WEBSOCKET_MAX_MESSAGE_SIZE: usize = 1024 * 1024;  // 1MB

// Algorithm parameters (unless you want to tune them)
pub const BLOOM_FILTER_ERROR_RATE: f64 = 0.01;  // 1% false positive
pub const RATE_LIMITER_DEFAULT_JITTER: f64 = 0.1;  // 10% jitter

// Safety limits (architectural decisions)
pub const MAX_CONCURRENT_CONNECTIONS: usize = 10_000;
pub const MAX_REQUEST_BODY_SIZE: usize = 10 * 1024 * 1024;  // 10MB
```

---

## 🔨 Implementation Recommendations

### 1. Use `dotenvy` for Environment Variables

```rust
// In main.rs or lib.rs
use dotenvy::dotenv;

fn main() {
    // Load .env file if present (ignored in production)
    dotenv().ok();

    // Access environment variables
    let redis_url = std::env::var("RIPTIDE_REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
}
```

### 2. Use `config` crate for TOML Configuration

```rust
use config::{Config, File, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub api: ApiConfig,
    pub intelligence: IntelligenceConfig,
    pub persistence: PersistenceConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let cfg = Config::builder()
            // Start with default config
            .add_source(File::with_name("config/default"))
            // Layer environment-specific config (e.g., config/production.toml)
            .add_source(
                File::with_name(&format!("config/{}", std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string())))
                    .required(false)
            )
            // Override with environment variables (prefix: RIPTIDE)
            .add_source(Environment::with_prefix("RIPTIDE").separator("__"))
            .build()?;

        cfg.try_deserialize()
    }
}
```

### 3. Configuration Validation

```rust
impl AppConfig {
    pub fn validate(&self) -> Result<(), String> {
        // Validate resource limits
        if self.api.headless.max_pool_size == 0 {
            return Err("Browser pool size must be > 0".to_string());
        }

        // Validate rate limiting
        if self.api.rate_limiting.enabled && self.api.rate_limiting.jitter_factor > 1.0 {
            return Err("Jitter factor must be <= 1.0".to_string());
        }

        // Validate circuit breaker
        if self.intelligence.circuit_breaker.failure_threshold == 0 {
            return Err("Failure threshold must be > 0".to_string());
        }

        Ok(())
    }
}
```

### 4. Configuration Merging Strategy

**Precedence (highest to lowest):**
1. Environment variables (runtime overrides)
2. Environment-specific TOML (e.g., `config/production.toml`)
3. Default TOML (`config/default.toml`)
4. Code defaults (as fallback)

### 5. Secret Management Best Practices

```rust
// NEVER do this
const API_KEY: &str = "sk-abc123...";  // ❌ WRONG

// Instead, use environment variables
let api_key = std::env::var("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY must be set");  // ✅ CORRECT

// For production, use secret management services
// - AWS Secrets Manager
// - HashiCorp Vault
// - Azure Key Vault
// - GCP Secret Manager
```

---

## 📊 Configuration Migration Plan

### Phase 1: Add Environment Variable Support (Week 1)
- Add `dotenvy` dependency
- Update `main.rs` to load `.env` files
- Document all environment variables in `.env.example`
- Update existing hardcoded secrets to use env vars

### Phase 2: Implement TOML Configuration (Week 2)
- Add `config` crate dependency
- Create `config/` directory structure
- Define Rust structs matching TOML schema
- Migrate timeout and limit values to TOML
- Add configuration validation

### Phase 3: Refactor Existing Code (Week 3)
- Update `ApiConfig::from_env()` to use new system
- Update all crates to accept configuration objects
- Remove hardcoded values from code
- Add configuration tests

### Phase 4: Documentation & Examples (Week 4)
- Document configuration options in README
- Provide example configurations for different scenarios
- Create deployment guides
- Add configuration troubleshooting guide

---

## 🎯 Configuration Testing Strategy

### Unit Tests
```rust
#[test]
fn test_config_validation() {
    let mut config = AppConfig::default();
    assert!(config.validate().is_ok());

    config.api.headless.max_pool_size = 0;
    assert!(config.validate().is_err());
}

#[test]
fn test_env_var_override() {
    std::env::set_var("RIPTIDE__API__HEADLESS__MAX_POOL_SIZE", "5");
    let config = AppConfig::load().unwrap();
    assert_eq!(config.api.headless.max_pool_size, 5);
}
```

### Integration Tests
- Test configuration loading from different sources
- Test environment-specific configuration files
- Test validation of invalid configurations
- Test default fallback behavior

---

## 🔒 Security Considerations

1. **Never commit secrets** - Use `.gitignore` for `.env` files
2. **Use secret management** - For production deployments
3. **Validate all inputs** - Especially URLs and file paths
4. **Audit configuration changes** - Log configuration at startup
5. **Principle of least privilege** - Only expose necessary configuration

---

## 📚 Additional Resources

- [config-rs documentation](https://docs.rs/config/)
- [dotenvy documentation](https://docs.rs/dotenvy/)
- [Rust security best practices](https://anssi-fr.github.io/rust-guide/)
- [12-Factor App Configuration](https://12factor.net/config)

---

## Conclusion

This configuration strategy provides a **clear separation of concerns**:
- **Environment variables** for deployment-specific and secret values
- **TOML files** for structured, occasionally-changing configuration
- **Code constants** for true invariants and business logic

This approach maximizes **flexibility**, **security**, and **maintainability** while following Rust ecosystem best practices.
