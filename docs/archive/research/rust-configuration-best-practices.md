# Rust Configuration Best Practices Research Report

**Project**: RipTide Event Mesh / Message Broker System
**Date**: 2025-10-20
**Focus**: Configuration management for event mesh, message broker, and distributed systems

---

## Executive Summary

This research provides comprehensive guidelines for configuration management in Rust-based distributed systems, specifically tailored for event mesh/message broker architectures with WebAssembly support and service discovery. Based on analysis of the RipTide codebase and industry best practices, this report recommends a **layered configuration approach** combining environment variables, structured config files, and code constants.

**Key Recommendations:**
1. Use **environment variables** for secrets and deployment-specific values
2. Use **TOML config files** for complex, hierarchical operational settings
3. Use **code constants** for system invariants and compile-time values
4. Implement **config-rs** or **figment** for unified configuration loading
5. Never commit secrets; use environment variables or secret management systems

---

## Table of Contents

1. [Configuration Strategy by Data Type](#1-configuration-strategy-by-data-type)
2. [Security Best Practices for Sensitive Data](#2-security-best-practices-for-sensitive-data)
3. [Configuration Crate Comparison](#3-configuration-crate-comparison)
4. [12-Factor App Principles for Rust Services](#4-12-factor-app-principles-for-rust-services)
5. [System-Specific Recommendations](#5-system-specific-recommendations)
6. [Implementation Roadmap](#6-implementation-roadmap)
7. [Code Examples](#7-code-examples)

---

## 1. Configuration Strategy by Data Type

### 1.1 Environment Variables (.env)

**Use For:**
- ✅ API keys and secrets (e.g., `OPENAI_API_KEY`, `SERPER_API_KEY`)
- ✅ Database connection strings (`REDIS_URL`, `POSTGRES_URL`)
- ✅ Service URLs that change per environment (`API_BASE_URL`)
- ✅ Feature flags for deployment control (`ENABLE_TELEMETRY`)
- ✅ Deployment-specific identifiers (`NODE_ID`, `CLUSTER_NAME`)

**Why:**
- Easy to change without recompiling or modifying files
- Standard practice for container deployments (Docker, Kubernetes)
- Supported by most secret management systems
- Clear separation between code and configuration

**Best Practices:**
```bash
# .env.example (committed to git)
REDIS_URL=redis://localhost:6379
ANTHROPIC_API_KEY=your_key_here
OPENAI_API_KEY=your_key_here
ENABLE_TELEMETRY=true

# .env (NOT committed, in .gitignore)
REDIS_URL=redis://prod-redis:6379
ANTHROPIC_API_KEY=sk-ant-actual-secret-key
OPENAI_API_KEY=sk-actual-secret-key
ENABLE_TELEMETRY=true
```

**Current Codebase Status:**
- ✅ Already using environment variables for API keys (intelligence config)
- ✅ Good pattern in `riptide-intelligence/src/config.rs` lines 273-326
- ⚠️ Some hardcoded URLs could be moved to environment variables

### 1.2 TOML Configuration Files

**Use For:**
- ✅ Complex hierarchical settings (thread pools, timeouts, limits)
- ✅ Domain-specific rules (gate thresholds, routing policies)
- ✅ Multi-tenant configurations
- ✅ Performance tuning parameters
- ✅ Circuit breaker and reliability settings
- ✅ Feature weights and scoring algorithms

**Why:**
- Human-readable and version-controlled
- Supports comments for documentation
- Type-safe deserialization with `serde`
- Easy to validate before deployment
- Natural fit for Rust ecosystem

**Best Practices:**
```toml
# config/eventmesh.toml
[performance]
max_concurrent_connections = 10000
request_timeout_secs = 30
worker_threads = 8

[message_broker]
max_message_size_bytes = 10485760  # 10MB
retention_seconds = 86400  # 24 hours
delivery_guarantee = "at_least_once"

[circuit_breaker]
failure_threshold = 50
min_requests = 5
recovery_timeout_secs = 60

# Environment variables can override these:
# EVENTMESH_PERFORMANCE_MAX_CONCURRENT_CONNECTIONS=20000
```

**Current Codebase Status:**
- ✅ Excellent TOML example: `config/gate_thresholds.toml.example`
- ✅ Good pattern for A/B testing and domain overrides
- ⚠️ Missing centralized TOML loading with environment overrides

### 1.3 YAML Configuration Files

**Use For:**
- ✅ CI/CD pipeline definitions (GitHub Actions)
- ✅ Kubernetes/Docker Compose configurations
- ✅ OpenAPI specifications
- ✅ Monitoring dashboards and alerts
- ⚠️ Optional for application config (TOML preferred in Rust)

**Why:**
- Standard for DevOps tooling
- Better for lists and complex nested structures
- Widely supported in cloud-native ecosystem

**Current Codebase Status:**
- ✅ Already used for monitoring: `config/monitoring/*.yaml`
- ✅ OpenAPI specs in YAML
- ℹ️ Keep YAML for infrastructure, TOML for application

### 1.4 Code Constants

**Use For:**
- ✅ System invariants (max URL length: 2048, max header size: 8KB)
- ✅ Protocol constants (HTTP status codes, content types)
- ✅ Compile-time feature flags
- ✅ Algorithm constants (magic numbers with explanations)
- ✅ Default values that rarely change

**Why:**
- Zero runtime overhead
- Compiler optimization opportunities
- Type safety and IDE support
- Clear signal: "this shouldn't change"

**Best Practices:**
```rust
// src/constants.rs
/// Maximum URL length per RFC 2616 recommendations
pub const MAX_URL_LENGTH: usize = 2048;

/// Maximum HTTP header size (8KB)
pub const MAX_HEADER_SIZE: usize = 8192;

/// Allowed content types for extraction
pub const ALLOWED_CONTENT_TYPES: &[&str] = &[
    "text/html",
    "application/xhtml+xml",
    "text/plain",
];

// Feature flags at compile time
#[cfg(feature = "wasm-extraction")]
pub const WASM_ENABLED: bool = true;
```

**Current Codebase Status:**
- ✅ Good constants in `riptide-config/src/validation.rs`
- ✅ System limits defined as constants
- ✅ Clear, documented constant usage

---

## 2. Security Best Practices for Sensitive Data

### 2.1 Never Commit Secrets

**Rules:**
1. ❌ **Never** hardcode API keys in source code
2. ❌ **Never** commit `.env` files with real credentials
3. ✅ **Always** use `.env.example` with placeholder values
4. ✅ **Always** add `.env` to `.gitignore`
5. ✅ **Always** use secret management in production

**Gitignore Pattern:**
```gitignore
# Environment files with secrets
.env
.env.local
.env.*.local
secrets/
*.secret
credentials.json
```

### 2.2 Secret Management Solutions

**Development:**
- `.env` files (local only, gitignored)
- `dotenvy` crate for loading

**Production Options:**

| Solution | Use Case | Integration |
|----------|----------|-------------|
| **Kubernetes Secrets** | Kubernetes deployments | Mount as env vars or files |
| **HashiCorp Vault** | Enterprise security | API client or sidecar |
| **AWS Secrets Manager** | AWS environments | SDK integration |
| **Azure Key Vault** | Azure environments | SDK integration |
| **Google Secret Manager** | GCP environments | SDK integration |
| **Docker Secrets** | Docker Swarm | Mount at `/run/secrets/` |

**Example: Kubernetes Secret**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: riptide-secrets
type: Opaque
stringData:
  ANTHROPIC_API_KEY: "sk-ant-..."
  OPENAI_API_KEY: "sk-..."
  REDIS_URL: "redis://prod-redis:6379"
```

### 2.3 Secrets in Rust Code

**Recommended Pattern:**
```rust
use std::env;

pub struct SecretConfig {
    pub anthropic_api_key: String,
    pub openai_api_key: String,
    pub redis_url: String,
}

impl SecretConfig {
    /// Load secrets from environment, fail fast if missing
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            anthropic_api_key: env::var("ANTHROPIC_API_KEY")
                .map_err(|_| ConfigError::MissingSecret("ANTHROPIC_API_KEY"))?,
            openai_api_key: env::var("OPENAI_API_KEY")
                .map_err(|_| ConfigError::MissingSecret("OPENAI_API_KEY"))?,
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        })
    }
}

// Prevent accidental logging of secrets
impl std::fmt::Debug for SecretConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretConfig")
            .field("anthropic_api_key", &"<redacted>")
            .field("openai_api_key", &"<redacted>")
            .field("redis_url", &self.redis_url)
            .finish()
    }
}
```

### 2.4 Secret Rotation

**Best Practices:**
1. Support hot-reloading of credentials without restart
2. Use short-lived tokens when possible
3. Implement graceful degradation if secrets become invalid
4. Log secret rotation events (but never log the secrets themselves)

**Example:**
```rust
pub struct RotatingSecret {
    value: Arc<RwLock<String>>,
    refresh_interval: Duration,
}

impl RotatingSecret {
    pub async fn start_rotation(&self, loader: impl SecretLoader) {
        let value = self.value.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(self.refresh_interval).await;
                if let Ok(new_secret) = loader.load().await {
                    *value.write().await = new_secret;
                    tracing::info!("Secret rotated successfully");
                }
            }
        });
    }
}
```

---

## 3. Configuration Crate Comparison

### 3.1 Recommended Crates

| Crate | Best For | Pros | Cons |
|-------|----------|------|------|
| **config** (config-rs) | Large apps, multiple sources | Layered config, many formats, environment overlays | Heavier dependency |
| **figment** | Rocket-based apps, profiles | Profile-based, type-safe, excellent errors | Rocket ecosystem focus |
| **envy** | Simple env parsing | Minimal, fast, serde-based | Environment variables only |
| **dotenvy** | .env file loading | Drop-in dotenv replacement, maintained | Just .env loading, not full config |
| **serde + std::env** | Custom solutions | Full control, no dependencies | More boilerplate |

### 3.2 Detailed Analysis

#### config-rs

**When to Use:**
- You need to merge multiple configuration sources
- You want environment variable overrides of TOML/YAML
- You need profile support (dev, staging, prod)
- You have hierarchical configuration

**Example:**
```rust
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub intelligence: IntelligenceConfig,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let env = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".into());

        Config::builder()
            // Start with defaults
            .add_source(File::with_name("config/default"))
            // Layer environment-specific config
            .add_source(File::with_name(&format!("config/{}", env)).required(false))
            // Local overrides (gitignored)
            .add_source(File::with_name("config/local").required(false))
            // Environment variable overrides with prefix
            .add_source(Environment::with_prefix("RIPTIDE").separator("__"))
            .build()?
            .try_deserialize()
    }
}
```

**Cargo.toml:**
```toml
[dependencies]
config = { version = "0.14", features = ["toml", "yaml"] }
```

#### figment

**When to Use:**
- You're building a Rocket web service
- You need profile-based configuration (similar to Spring Boot)
- You want excellent error messages
- Type safety is critical

**Example:**
```rust
use figment::{Figment, providers::{Format, Toml, Env}};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct EventMeshConfig {
    pub broker: BrokerConfig,
    pub wasm: WasmConfig,
}

impl EventMeshConfig {
    pub fn load() -> Result<Self, figment::Error> {
        Figment::new()
            .merge(Toml::file("config/base.toml"))
            .merge(Toml::file("config/production.toml").nested())
            .merge(Env::prefixed("EVENTMESH_"))
            .extract()
    }
}
```

#### dotenvy

**When to Use:**
- You only need .env file support
- You want minimal dependencies
- You're following 12-factor strictly

**Example:**
```rust
use dotenvy::dotenv;

fn main() {
    // Load .env file into environment
    dotenv().ok();

    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY must be set");
}
```

### 3.3 Recommendation for RipTide

**Primary: config-rs**
- Best fit for complex event mesh architecture
- Handles TOML + environment variables elegantly
- Supports existing file structure
- Industry standard for Rust web services

**Secondary: dotenvy**
- For simple .env loading in development
- Lightweight addition for local development

**Implementation:**
```toml
[dependencies]
config = { version = "0.14", features = ["toml"] }
dotenvy = "0.15"  # Optional, for .env files
```

---

## 4. 12-Factor App Principles for Rust Services

### 4.1 Core Principles Applied

#### I. Codebase
- ✅ One codebase tracked in Git, many deploys
- ✅ Workspace structure supports multiple deployable units

#### II. Dependencies
- ✅ Explicit in `Cargo.toml`
- ✅ `Cargo.lock` for reproducible builds
- ✅ No reliance on system packages

#### III. Config
- **Store config in the environment**
- ❌ **Current issue**: Some config in code, some in env
- ✅ **Recommendation**: Layered approach (TOML + env overrides)

```rust
// ❌ Bad: Hardcoded
const REDIS_URL: &str = "redis://localhost:6379";

// ✅ Good: From environment
let redis_url = env::var("REDIS_URL")
    .unwrap_or_else(|_| "redis://localhost:6379".to_string());

// ✅ Better: Structured config with defaults
#[derive(Deserialize)]
struct RedisConfig {
    #[serde(default = "default_redis_url")]
    url: String,
}

fn default_redis_url() -> String {
    "redis://localhost:6379".to_string()
}
```

#### IV. Backing Services
- ✅ Treat backing services as attached resources
- ✅ Redis, LLM APIs configured via URLs
- ⚠️ Improve by making all service URLs configurable

#### V. Build, Release, Run
- ✅ Separate build (cargo build) from run
- ✅ Docker images for immutable releases
- ⚠️ Consider versioned config releases

#### VI. Processes
- ✅ Stateless processes
- ✅ State in Redis (persistence layer)
- ✅ Shared-nothing architecture

#### VII. Port Binding
- ✅ Self-contained HTTP service
- ⚠️ Make port configurable via environment

```rust
let port = env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string())
    .parse::<u16>()?;
```

#### VIII. Concurrency
- ✅ Horizontal scaling via process replication
- ✅ Tokio async runtime
- ✅ Worker pools for concurrent processing

#### IX. Disposability
- ✅ Fast startup
- ⚠️ Improve graceful shutdown with state preservation

```rust
// Implement graceful shutdown
let shutdown = async {
    tokio::signal::ctrl_c().await.ok();
    tracing::info!("Shutdown signal received");

    // Drain in-flight requests
    shutdown_tx.send(()).ok();

    // Persist state if needed
    state.persist().await.ok();
};
```

#### X. Dev/Prod Parity
- ✅ Same Rust version across environments
- ⚠️ Ensure Redis version parity
- ✅ Docker for consistent environments

#### XI. Logs
- ✅ Structured logging with `tracing`
- ✅ OpenTelemetry integration
- ✅ Logs to stdout (container-friendly)

#### XII. Admin Processes
- ✅ Admin tasks as separate commands (CLI)
- ✅ Database migrations via separate tool
- ⚠️ Document common admin operations

### 4.2 Environment Variable Naming Convention

**Recommended Pattern:**
```bash
# Format: {SERVICE}_{COMPONENT}_{SETTING}
RIPTIDE_SERVER_PORT=3000
RIPTIDE_REDIS_URL=redis://localhost:6379
RIPTIDE_INTELLIGENCE_MAX_RETRIES=3
RIPTIDE_WASM_MAX_MEMORY_MB=128

# Secrets: No service prefix for portability
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...
SERPER_API_KEY=...

# Feature flags
ENABLE_TELEMETRY=true
ENABLE_WASM_AOT_CACHE=true
DEBUG=false
```

---

## 5. System-Specific Recommendations

### 5.1 Event Mesh / Message Broker

**Configuration Priorities:**
1. **Throughput Settings**: TOML (complex tuning)
2. **Connection Limits**: TOML with env override
3. **Message Retention**: TOML with env override
4. **Broker URLs**: Environment variables
5. **Partition Strategies**: TOML (algorithmic)

**Example Structure:**
```toml
# config/eventmesh.toml
[broker]
max_partitions = 100
message_retention_hours = 24
max_message_size_mb = 10
compression_codec = "lz4"

[delivery]
guarantee = "at_least_once"  # Options: at_most_once, at_least_once, exactly_once
timeout_ms = 5000
max_retries = 3

[scaling]
min_workers = 2
max_workers = 16
scale_up_threshold = 0.8
scale_down_threshold = 0.2
```

**Environment Overrides:**
```bash
RIPTIDE_BROKER_MAX_PARTITIONS=200
RIPTIDE_DELIVERY_GUARANTEE=exactly_once
```

### 5.2 WebAssembly-Enabled Services

**Configuration Priorities:**
1. **Memory Limits**: Code constants (safety-critical)
2. **Instance Pools**: TOML
3. **Timeout Values**: TOML with env override
4. **AOT Cache**: Environment variable (filesystem path)

**Example:**
```rust
// constants.rs - Safety-critical limits
pub const WASM_MAX_MEMORY_PAGES: u32 = 256;  // 16MB
pub const WASM_MAX_TABLE_SIZE: u32 = 1000;
pub const WASM_MAX_STACK_SIZE: usize = 1024 * 1024;  // 1MB

// config.toml - Operational tuning
[wasm]
instances_per_worker = 1
module_timeout_secs = 10
enable_aot_cache = true
cache_size_mb = 100
health_check_interval_secs = 120
```

**Environment Variables:**
```bash
WASM_CACHE_DIR=/var/cache/riptide/wasm
WASM_MODULE_TIMEOUT=10
```

### 5.3 Distributed Systems with Service Discovery

**Configuration Priorities:**
1. **Node Identity**: Environment variable (unique per instance)
2. **Cluster Topology**: TOML
3. **Service Endpoints**: Environment variables
4. **Consensus Parameters**: TOML
5. **Health Check Intervals**: TOML

**Example:**
```toml
# config/distributed.toml
[cluster]
name = "riptide-production"
topology = "mesh"  # Options: mesh, star, ring

[consensus]
algorithm = "raft"
election_timeout_ms = 1000
heartbeat_interval_ms = 100
min_quorum_size = 3

[service_discovery]
provider = "consul"  # Options: consul, etcd, kubernetes
refresh_interval_secs = 30
health_check_path = "/health"
```

**Environment Variables:**
```bash
# Unique per instance
NODE_ID=node-prod-01
NODE_REGION=us-east-1
NODE_AVAILABILITY_ZONE=us-east-1a

# Service discovery
CONSUL_ADDR=http://consul.internal:8500
CLUSTER_JOIN_ADDRESSES=node1:8080,node2:8080,node3:8080
```

### 5.4 Performance-Critical Configurations

**Strategy: Compile-Time + Runtime Hybrid**

**Compile-Time (Feature Flags):**
```toml
# Cargo.toml
[features]
default = ["wasm-extraction", "simd-optimizations"]
wasm-extraction = ["wasmtime"]
simd-optimizations = ["faster-simd"]
telemetry = ["opentelemetry", "tracing-opentelemetry"]
```

**Runtime (TOML):**
```toml
[performance]
# Thread pool sizing
worker_threads = 8  # CPU cores
blocking_threads = 8  # For blocking I/O

# Connection pooling
max_connections = 1000
connection_timeout_ms = 5000
idle_timeout_secs = 300

# Buffer sizes
read_buffer_size_kb = 64
write_buffer_size_kb = 64

# Caching
enable_hot_cache = true
hot_cache_size_mb = 512
cache_ttl_secs = 3600
```

**Code Constants (Hot Path):**
```rust
// Zero-cost abstractions for performance-critical paths
pub const FAST_PATH_BUFFER_SIZE: usize = 8192;
pub const MAX_INLINE_SIZE: usize = 128;
pub const BATCH_SIZE: usize = 100;
```

---

## 6. Implementation Roadmap

### Phase 1: Immediate Improvements (Week 1)

**Priority: Security & Foundation**

1. **Add .env support with dotenvy**
   ```bash
   cargo add dotenvy
   ```

2. **Create .env.example**
   - Document all required environment variables
   - Add to repository (without secrets)

3. **Update .gitignore**
   ```gitignore
   .env
   .env.local
   secrets/
   ```

4. **Audit secret usage**
   - Find all hardcoded secrets
   - Move to environment variables
   - Add Debug redaction for secret types

**Deliverables:**
- [ ] `.env.example` file
- [ ] Updated `.gitignore`
- [ ] Secret audit report
- [ ] Updated README with configuration docs

### Phase 2: Structured Configuration (Week 2-3)

**Priority: Maintainability & Scalability**

1. **Implement config-rs**
   ```bash
   cargo add config --features toml
   ```

2. **Create configuration structure**
   ```
   config/
   ├── default.toml           # Base config
   ├── development.toml       # Dev overrides
   ├── production.toml        # Prod overrides
   ├── eventmesh.toml        # Event mesh specific
   ├── gate_thresholds.toml  # (existing)
   └── local.toml.example    # Local overrides template
   ```

3. **Implement unified config loader**
   ```rust
   // crates/riptide-config/src/loader.rs
   pub struct ConfigLoader;

   impl ConfigLoader {
       pub fn load() -> Result<RiptideConfig, ConfigError> {
           // Load order: default -> environment -> local -> env vars
       }
   }
   ```

**Deliverables:**
- [ ] `riptide-config` crate enhancements
- [ ] TOML configuration files
- [ ] Migration guide
- [ ] Configuration validation

### Phase 3: Advanced Features (Week 4)

**Priority: Operations & Reliability**

1. **Hot-reload support**
   - Watch config files for changes
   - Reload without restart
   - Validation before apply

2. **Configuration profiles**
   - Dev, staging, production profiles
   - Profile-specific overrides
   - Profile selection via environment

3. **Configuration documentation**
   - Auto-generate docs from schema
   - Configuration reference guide
   - Example configurations

**Deliverables:**
- [ ] Hot-reload implementation
- [ ] Profile system
- [ ] Configuration docs
- [ ] Monitoring dashboard for config

---

## 7. Code Examples

### 7.1 Complete Configuration System

**File: `crates/riptide-config/src/unified.rs`**

```rust
use config::{Config, ConfigError, Environment, File};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unified configuration for RipTide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiptideConfig {
    pub server: ServerConfig,
    pub eventmesh: EventMeshConfig,
    pub intelligence: IntelligenceConfig,
    pub persistence: PersistenceConfig,
    pub wasm: WasmConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMeshConfig {
    pub max_partitions: u32,
    pub message_retention_hours: u32,
    pub compression_codec: String,
}

// ... other config sections

impl RiptideConfig {
    /// Load configuration from all sources with proper precedence
    pub fn load() -> Result<Self, ConfigError> {
        // Load .env file (optional, for local development)
        dotenv().ok();

        // Determine environment
        let env = std::env::var("RUST_ENV")
            .unwrap_or_else(|_| "development".to_string());

        // Build configuration with layering
        let config = Config::builder()
            // 1. Start with defaults
            .add_source(File::with_name("config/default"))

            // 2. Environment-specific overrides
            .add_source(
                File::with_name(&format!("config/{}", env))
                    .required(false)
            )

            // 3. Local overrides (gitignored)
            .add_source(
                File::with_name("config/local")
                    .required(false)
            )

            // 4. Environment variables (highest priority)
            .add_source(
                Environment::with_prefix("RIPTIDE")
                    .separator("__")
                    .try_parsing(true)
            )
            .build()?;

        let config: RiptideConfig = config.try_deserialize()?;

        // Validate before returning
        config.validate()?;

        Ok(config)
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Port range
        if self.server.port == 0 {
            return Err(ConfigError::Message(
                "Server port must be greater than 0".into()
            ));
        }

        // Worker threads
        if self.server.workers == 0 {
            return Err(ConfigError::Message(
                "Worker threads must be greater than 0".into()
            ));
        }

        // Event mesh
        if self.eventmesh.max_partitions == 0 {
            return Err(ConfigError::Message(
                "Max partitions must be greater than 0".into()
            ));
        }

        // WASM memory limits
        if self.wasm.max_memory_mb > 2048 {
            return Err(ConfigError::Message(
                "WASM max memory cannot exceed 2GB".into()
            ));
        }

        Ok(())
    }

    /// Load from specific config file (for testing)
    pub fn from_file(path: impl Into<PathBuf>) -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::from(path.into()))
            .build()?;

        config.try_deserialize()
    }
}

impl Default for RiptideConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
                workers: num_cpus::get(),
            },
            eventmesh: EventMeshConfig {
                max_partitions: 100,
                message_retention_hours: 24,
                compression_codec: "lz4".to_string(),
            },
            intelligence: IntelligenceConfig::default(),
            persistence: PersistenceConfig::default(),
            wasm: WasmConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}
```

### 7.2 Secret Management

**File: `crates/riptide-config/src/secrets.rs`**

```rust
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecretError {
    #[error("Required secret not found: {0}")]
    Missing(String),

    #[error("Invalid secret format: {0}")]
    Invalid(String),
}

/// Configuration for secrets (API keys, credentials)
pub struct Secrets {
    anthropic_api_key: String,
    openai_api_key: String,
    serper_api_key: Option<String>,
    redis_url: String,
}

impl Secrets {
    /// Load secrets from environment variables
    pub fn from_env() -> Result<Self, SecretError> {
        Ok(Self {
            anthropic_api_key: env::var("ANTHROPIC_API_KEY")
                .map_err(|_| SecretError::Missing("ANTHROPIC_API_KEY".into()))?,

            openai_api_key: env::var("OPENAI_API_KEY")
                .map_err(|_| SecretError::Missing("OPENAI_API_KEY".into()))?,

            serper_api_key: env::var("SERPER_API_KEY").ok(),

            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        })
    }

    pub fn anthropic_api_key(&self) -> &str {
        &self.anthropic_api_key
    }

    pub fn openai_api_key(&self) -> &str {
        &self.openai_api_key
    }

    pub fn serper_api_key(&self) -> Option<&str> {
        self.serper_api_key.as_deref()
    }

    pub fn redis_url(&self) -> &str {
        &self.redis_url
    }
}

// Prevent accidental logging of secrets
impl std::fmt::Debug for Secrets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Secrets")
            .field("anthropic_api_key", &"<redacted>")
            .field("openai_api_key", &"<redacted>")
            .field("serper_api_key", &if self.serper_api_key.is_some() {
                "<redacted>"
            } else {
                "<not set>"
            })
            .field("redis_url", &self.redis_url)
            .finish()
    }
}

// Never serialize secrets
impl Serialize for Secrets {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Secrets", 1)?;
        state.serialize_field("redacted", &true)?;
        state.end()
    }
}
```

### 7.3 Configuration for Different Deployment Environments

**File: `config/default.toml`**

```toml
# Default configuration - common across all environments

[server]
host = "0.0.0.0"
port = 3000
workers = 8
shutdown_timeout_secs = 30

[eventmesh]
max_partitions = 100
message_retention_hours = 24
max_message_size_mb = 10
compression_codec = "lz4"

[wasm]
instances_per_worker = 1
module_timeout_secs = 10
max_memory_mb = 128
enable_aot_cache = true
health_check_interval_secs = 120

[persistence]
redis_pool_size = 10
connection_timeout_ms = 5000
command_timeout_ms = 5000
enable_pipelining = true

[intelligence]
failover_max_retries = 3
health_check_interval_secs = 60
circuit_breaker_failure_threshold = 50

[security]
enable_rate_limiting = true
max_requests_per_minute = 1000
enable_audit_logging = true
```

**File: `config/production.toml`**

```toml
# Production overrides

[server]
workers = 32  # More workers for production

[eventmesh]
max_partitions = 1000  # Higher capacity
message_retention_hours = 168  # 7 days

[persistence]
redis_pool_size = 50  # Larger pool
enable_pipelining = true

[intelligence]
health_check_interval_secs = 30  # More frequent checks

[security]
enable_rate_limiting = true
max_requests_per_minute = 10000  # Higher throughput
enable_audit_logging = true
audit_retention_days = 90
```

**File: `config/development.toml`**

```toml
# Development overrides

[server]
workers = 2  # Fewer workers for local dev

[eventmesh]
max_partitions = 10  # Limited for testing

[intelligence]
health_check_interval_secs = 300  # Less frequent

[security]
enable_rate_limiting = false  # Easier for testing
enable_audit_logging = false  # Less noise in logs
```

---

## Conclusion

### Key Takeaways

1. **Use the Right Tool for Each Job**
   - Secrets → Environment variables
   - Complex config → TOML files
   - System invariants → Code constants
   - Infrastructure → YAML

2. **Security First**
   - Never commit secrets
   - Use secret management in production
   - Redact secrets from logs and debug output
   - Support secret rotation

3. **Layered Configuration**
   - Start with sensible defaults
   - Layer environment-specific overrides
   - Allow runtime environment variable overrides
   - Validate before use

4. **Follow 12-Factor Principles**
   - Store config in environment
   - Strict separation of config and code
   - Environment parity
   - Treat backing services as attached resources

5. **Operational Excellence**
   - Support hot-reload when safe
   - Provide configuration profiles
   - Document all configuration options
   - Validate configuration at startup

### Next Steps

1. Review current configuration usage in codebase
2. Create `.env.example` with all required variables
3. Implement unified config loader with `config-rs`
4. Create TOML configuration files for each environment
5. Add configuration validation
6. Document configuration options
7. Implement hot-reload for non-critical settings

### References

- [The Twelve-Factor App](https://12factor.net/)
- [config-rs Documentation](https://docs.rs/config/)
- [figment Documentation](https://docs.rs/figment/)
- [Rust Security Best Practices](https://anssi-fr.github.io/rust-guide/)
- [OWASP Secrets Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html)

---

**Report Prepared By**: Research Agent
**For**: RipTide Event Mesh Project
**Date**: 2025-10-20
**Status**: Ready for Implementation
