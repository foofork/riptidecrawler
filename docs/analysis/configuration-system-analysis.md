# Configuration System Analysis

**Date**: 2025-11-04
**Purpose**: Complete analysis of RipTide configuration system to plan server.yaml and precedence implementation
**Status**: Complete

## Executive Summary

RipTide has a **split configuration architecture** with two distinct systems:
1. **riptide-config** crate: Provides `ApiConfig` (authentication, rate limiting, request handling) and `SpiderConfig`
2. **riptide-api** crate: Provides `ApiConfig` (resource controls, performance, memory) and `AppConfig` (application-level)

Both use **environment variable loading** exclusively. There is **no YAML/file-based configuration** support yet, and **no precedence logic** exists for combining multiple configuration sources.

## Configuration Types Inventory

### 1. riptide-config Crate (`/crates/riptide-config/src/`)

#### 1.1 ApiConfig (api.rs) - API Server Configuration
```rust
pub struct ApiConfig {
    pub auth: AuthenticationConfig,
    pub rate_limit: RateLimitConfig,
    pub request: RequestConfig,
    pub bind_address: String, // "0.0.0.0:8080"
}
```

**Sub-configurations:**

**AuthenticationConfig**:
- `require_auth: bool` (default: true)
- `api_keys: Vec<String>` (from `API_KEYS` env var, comma-separated)
- `constant_time_comparison: bool` (default: true)
- `public_paths: Vec<String>` (health endpoints)
- **Validation**: Enforces 32+ char API keys with alphanumeric + no weak patterns

**RateLimitConfig**:
- `max_concurrent_requests: usize` (default: 100, env: `MAX_CONCURRENT_REQUESTS`)
- `requests_per_minute: u32` (default: 60, env: `RATE_LIMIT_PER_MINUTE`)
- `enabled: bool` (default: true, env: `ENABLE_RATE_LIMITING`)
- `custom_limits: Vec<CustomRateLimit>`

**RequestConfig**:
- `timeout: Duration` (default: 30s, env: `REQUEST_TIMEOUT_SECS`)
- `max_payload_size: usize` (default: 50MB, env: `MAX_PAYLOAD_SIZE`)
- `enable_cors: bool` (default: true, env: `ENABLE_CORS`)
- `enable_compression: bool` (default: true, env: `ENABLE_COMPRESSION`)

#### 1.2 SpiderConfig (spider.rs) - Web Crawling Configuration
```rust
pub struct SpiderConfig {
    pub user_agent: String,
    pub timeout: Duration,
    pub delay: Duration,
    pub concurrency: usize,
    pub max_depth: Option<usize>,
    pub max_pages: Option<usize>,
    pub respect_robots: bool,
    pub follow_redirects: bool,
    pub max_redirects: usize,
    pub enable_javascript: bool,
    pub url_processing: UrlProcessingConfig,
    pub performance: PerformanceConfig,
}
```

**Sub-configurations:**

**UrlProcessingConfig**:
- URL normalization settings
- Deduplication with bloom filters
- Exclude patterns and extensions
- Max URL length (default: 2048)

**PerformanceConfig**:
- `max_concurrent_global: usize` (default: 10)
- `max_concurrent_per_host: usize` (default: 2)
- `request_timeout: Duration` (default: 30s)
- Adaptive throttling settings
- Memory/CPU thresholds

**Presets Available**:
- `SpiderPresets::development()`
- `SpiderPresets::high_performance()`
- `SpiderPresets::news_site()`
- `SpiderPresets::ecommerce_site()`
- `SpiderPresets::documentation_site()`
- `SpiderPresets::authenticated_crawling()`

#### 1.3 ValidationConfig (validation.rs) - Security Validation
```rust
pub struct ValidationConfig {
    pub max_url_length: usize, // 2048
    pub max_header_size: usize, // 8192
    pub allowed_content_types: HashSet<String>,
    pub blocked_patterns: HashSet<String>,
    pub allowed_domains: Option<HashSet<String>>,
    pub block_private_ips: bool,
    pub max_content_size: usize, // 20MB
    pub strict_ssl: bool,
}
```

**Validators Available**:
- `CommonValidator` - URL, content type, headers, content size, query validation
- `ContentTypeValidator` - HTML, JSON, XML, PDF detection
- `UrlValidator` - Scheme and suspicious pattern detection
- `SizeValidator` - URL, content, query length validation
- `ParameterValidator` - Positive integers, ranges, non-empty strings, URL lists

### 2. riptide-api Crate (`/crates/riptide-api/src/`)

#### 2.1 ApiConfig (config.rs) - Resource & Performance Configuration
```rust
pub struct ApiConfig {
    pub resources: ResourceConfig,
    pub performance: PerformanceConfig,
    pub rate_limiting: RateLimitingConfig,
    pub memory: MemoryConfig,
    pub headless: HeadlessConfig,
    pub pdf: PdfConfig,
    #[cfg(feature = "wasm-extractor")]
    pub wasm: WasmConfig,
    pub search: SearchProviderConfig,
}
```

**ResourceConfig** (7 fields):
- `max_concurrent_renders: usize` (default: 10, env: `RIPTIDE_MAX_CONCURRENT_RENDERS`)
- `max_concurrent_pdf: usize` (default: 2, env: `RIPTIDE_MAX_CONCURRENT_PDF`)
- `max_concurrent_wasm: usize` (default: 4, env: `RIPTIDE_MAX_CONCURRENT_WASM`)
- `global_timeout_secs: u64` (default: 30, env: `RIPTIDE_GLOBAL_TIMEOUT_SECS`)
- `cleanup_interval_secs: u64` (default: 60, env: `RIPTIDE_CLEANUP_INTERVAL_SECS`)
- `enable_monitoring: bool` (default: true, env: `RIPTIDE_ENABLE_MONITORING`)
- `health_check_interval_secs: u64` (default: 30, env: `RIPTIDE_HEALTH_CHECK_INTERVAL_SECS`)

**PerformanceConfig** (7 fields):
- `render_timeout_secs: u64` (default: 3, env: `RIPTIDE_RENDER_TIMEOUT_SECS`)
- `pdf_timeout_secs: u64` (default: 10, env: `RIPTIDE_PDF_TIMEOUT_SECS`)
- `wasm_timeout_secs: u64` (default: 5, env: `RIPTIDE_WASM_TIMEOUT_SECS`)
- `http_timeout_secs: u64` (default: 10, env: `RIPTIDE_HTTP_TIMEOUT_SECS`)
- `memory_cleanup_threshold_mb: usize` (default: 512, env: `RIPTIDE_MEMORY_CLEANUP_THRESHOLD_MB`)
- `auto_cleanup_on_timeout: bool` (default: true, env: `RIPTIDE_AUTO_CLEANUP_ON_TIMEOUT`)
- `degradation_threshold: f64` (default: 0.8, env: `RIPTIDE_DEGRADATION_THRESHOLD`)

**RateLimitingConfig** (7 fields):
- `enabled: bool` (default: true, env: `RIPTIDE_RATE_LIMIT_ENABLED`)
- `requests_per_second_per_host: f64` (default: 1.5, env: `RIPTIDE_RATE_LIMIT_RPS`)
- `jitter_factor: f64` (default: 0.1, env: `RIPTIDE_RATE_LIMIT_JITTER`)
- `burst_capacity_per_host: u32` (default: 3, env: `RIPTIDE_RATE_LIMIT_BURST_CAPACITY`)
- `window_duration_secs: u64` (default: 60, env: `RIPTIDE_RATE_LIMIT_WINDOW_SECS`)
- `cleanup_interval_secs: u64` (default: 300, env: `RIPTIDE_RATE_LIMIT_CLEANUP_INTERVAL_SECS`)
- `max_tracked_hosts: usize` (default: 10000, env: `RIPTIDE_RATE_LIMIT_MAX_TRACKED_HOSTS`)

**MemoryConfig** (10 fields):
- `max_memory_per_request_mb: usize` (default: 256, env: `RIPTIDE_MAX_MEMORY_PER_REQUEST_MB`)
- `global_memory_limit_mb: usize` (default: 2048, env: `RIPTIDE_GLOBAL_MEMORY_LIMIT_MB`)
- `memory_soft_limit_mb: usize` (default: 400, env: `RIPTIDE_MEMORY_SOFT_LIMIT_MB`)
- `memory_hard_limit_mb: usize` (default: 500, env: `RIPTIDE_MEMORY_HARD_LIMIT_MB`)
- `pressure_threshold: f64` (default: 0.85, env: `RIPTIDE_MEMORY_PRESSURE_THRESHOLD`)
- `auto_gc: bool` (default: true, env: `RIPTIDE_MEMORY_AUTO_GC`)
- `gc_trigger_threshold_mb: usize` (default: 1024, env: `RIPTIDE_MEMORY_GC_TRIGGER_THRESHOLD_MB`)
- `monitoring_interval_secs: u64` (default: 30, env: `RIPTIDE_MEMORY_MONITORING_INTERVAL_SECS`)
- `enable_leak_detection: bool` (default: true, env: `RIPTIDE_MEMORY_ENABLE_LEAK_DETECTION`)
- `enable_proactive_monitoring: bool` (default: true, env: `RIPTIDE_MEMORY_ENABLE_PROACTIVE_MONITORING`)

**HeadlessConfig** (9 fields):
- `max_pool_size: usize` (default: 3, env: `RIPTIDE_HEADLESS_MAX_POOL_SIZE`)
- `min_pool_size: usize` (default: 1, env: `RIPTIDE_HEADLESS_MIN_POOL_SIZE`)
- `idle_timeout_secs: u64` (default: 300, env: `RIPTIDE_HEADLESS_IDLE_TIMEOUT_SECS`)
- `health_check_interval_secs: u64` (default: 60, env: `RIPTIDE_HEADLESS_HEALTH_CHECK_INTERVAL_SECS`)
- `max_pages_per_browser: usize` (default: 10, env: `RIPTIDE_HEADLESS_MAX_PAGES_PER_BROWSER`)
- `restart_threshold: u32` (default: 5, env: `RIPTIDE_HEADLESS_RESTART_THRESHOLD`)
- `enable_recycling: bool` (default: true, env: `RIPTIDE_HEADLESS_ENABLE_RECYCLING`)
- `launch_timeout_secs: u64` (default: 30, env: `RIPTIDE_HEADLESS_LAUNCH_TIMEOUT_SECS`)
- `max_retries: u32` (default: 3, env: `RIPTIDE_HEADLESS_MAX_RETRIES`)

**PdfConfig** (6 fields):
- `max_concurrent: usize` (default: 2, env: `RIPTIDE_PDF_MAX_CONCURRENT`)
- `processing_timeout_secs: u64` (default: 30, env: `RIPTIDE_PDF_PROCESSING_TIMEOUT_SECS`)
- `max_file_size_mb: usize` (default: 100, env: `RIPTIDE_PDF_MAX_FILE_SIZE_MB`)
- `enable_streaming: bool` (default: true, env: `RIPTIDE_PDF_ENABLE_STREAMING`)
- `queue_size: usize` (default: 50, env: `RIPTIDE_PDF_QUEUE_SIZE`)
- `queue_timeout_secs: u64` (default: 60, env: `RIPTIDE_PDF_QUEUE_TIMEOUT_SECS`)

**WasmConfig** (7 fields, feature-gated):
- `instances_per_worker: usize` (default: 1, env: `RIPTIDE_WASM_INSTANCES_PER_WORKER`)
- `module_timeout_secs: u64` (default: 10, env: `RIPTIDE_WASM_MODULE_TIMEOUT_SECS`)
- `max_memory_mb: usize` (default: 128, env: `RIPTIDE_WASM_MAX_MEMORY_MB`)
- `enable_recycling: bool` (default: false, env: `RIPTIDE_WASM_ENABLE_RECYCLING`)
- `health_check_interval_secs: u64` (default: 120, env: `RIPTIDE_WASM_HEALTH_CHECK_INTERVAL_SECS`)
- `max_operations_per_instance: u64` (default: 10000, env: `RIPTIDE_WASM_MAX_OPERATIONS_PER_INSTANCE`)
- `restart_threshold: u32` (default: 10, env: `RIPTIDE_WASM_RESTART_THRESHOLD`)

**SearchProviderConfig** (6 fields):
- `backend: String` (default: "serper", env: `RIPTIDE_SEARCH_BACKEND`)
- `timeout_secs: u64` (default: 30, env: `RIPTIDE_SEARCH_TIMEOUT_SECS`)
- `enable_url_parsing: bool` (default: true, env: `RIPTIDE_SEARCH_ENABLE_URL_PARSING`)
- `circuit_breaker_failure_threshold: u32` (default: 50, env: `RIPTIDE_SEARCH_CIRCUIT_BREAKER_FAILURE_THRESHOLD`)
- `circuit_breaker_min_requests: u32` (default: 5, env: `RIPTIDE_SEARCH_CIRCUIT_BREAKER_MIN_REQUESTS`)
- `circuit_breaker_recovery_timeout_secs: u64` (default: 60, env: `RIPTIDE_SEARCH_CIRCUIT_BREAKER_RECOVERY_TIMEOUT_SECS`)

#### 2.2 AppConfig (state.rs) - Application-Level Configuration
```rust
pub struct AppConfig {
    pub redis_url: String,
    pub wasm_path: String,
    pub max_concurrency: usize,
    pub cache_ttl: u64,
    pub gate_hi_threshold: f32,
    pub gate_lo_threshold: f32,
    pub headless_url: Option<String>,
    pub session_config: SessionConfig,
    pub spider_config: Option<SpiderConfig>,
    pub worker_config: WorkerServiceConfig,
    pub event_bus_config: EventBusConfig,
    pub circuit_breaker_config: CircuitBreakerConfig,
    pub reliability_config: ReliabilityConfig,
    pub monitoring_config: MonitoringConfig,
    pub enhanced_pipeline_config: EnhancedPipelineConfig,
    #[cfg(feature = "wasm-extractor")]
    pub cache_warming_config: CacheWarmingConfig,
    pub engine_selection_config: EngineSelectionConfig,
}
```

**Environment Variables**:
- `REDIS_URL` (default: "redis://localhost:6379")
- `WASM_EXTRACTOR_PATH` (default: "./target/wasm32-wasip2/release/riptide_extractor_wasm.wasm")
- `MAX_CONCURRENCY` (default: 16)
- `CACHE_TTL` (default: 3600)
- `GATE_HI_THRESHOLD` (default: 0.7)
- `GATE_LO_THRESHOLD` (default: 0.3)
- `HEADLESS_URL` (optional)
- `SPIDER_ENABLE` (default: false)
- `SPIDER_BASE_URL`, `SPIDER_USER_AGENT`, `SPIDER_TIMEOUT_SECONDS`, etc.

## Loading Mechanisms

### Environment Variable Loading

#### riptide-config Pattern (api.rs)
```rust
impl ApiConfig {
    pub fn from_env() -> Self {
        Self {
            auth: AuthenticationConfig::from_env(),
            rate_limit: RateLimitConfig::from_env(),
            request: RequestConfig::from_env(),
            bind_address: std::env::var("BIND_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
        }
    }
}

impl AuthenticationConfig {
    pub fn from_env() -> Self {
        let api_keys: Vec<String> = std::env::var("API_KEYS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        // Validates API keys on load!
        for key in &api_keys {
            if let Err(e) = validation::validate_api_key(key) {
                panic!("Invalid API key: {}", e);
            }
        }
        // ...
    }
}
```

#### riptide-api Pattern (config.rs)
```rust
impl ApiConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Manual parsing for each field (59 env vars total!)
        if let Ok(val) = std::env::var("RIPTIDE_MAX_CONCURRENT_RENDERS") {
            if let Ok(val) = val.parse() {
                config.resources.max_concurrent_renders = val;
            }
        }
        // ... repeated 58 more times

        config
    }
}
```

#### EnvConfigLoader Utility (env.rs)
```rust
pub struct EnvConfigLoader {
    prefix: Option<String>,
    required: Vec<String>,
    defaults: HashMap<String, String>,
}

impl EnvConfigLoader {
    pub fn new() -> Self;
    pub fn with_prefix(prefix: impl Into<String>) -> Self;
    pub fn require(var: impl Into<String>) -> Self;
    pub fn default(var: impl Into<String>, value: impl Into<String>) -> Self;

    // Type-safe getters
    pub fn get(&self, var: &str) -> Result<String, EnvError>;
    pub fn get_int(&self, var: &str) -> Result<i64, EnvError>;
    pub fn get_uint(&self, var: &str) -> Result<u64, EnvError>;
    pub fn get_bool(&self, var: &str) -> Result<bool, EnvError>;
    pub fn get_duration(&self, var: &str) -> Result<Duration, EnvError>; // "30s", "5m", "1h"
    pub fn get_list(&self, var: &str) -> Result<Vec<String>, EnvError>;

    pub fn load_all(&self) -> HashMap<String, String>;
    pub fn validate(&self) -> Result<(), EnvError>;
}
```

**Duration parsing supports**: `"30s"`, `"5m"`, `"1h"`, `"500ms"`, or plain numbers (seconds)

### Builder Patterns

#### DefaultConfigBuilder (builder.rs)
```rust
pub struct DefaultConfigBuilder<T> {
    fields: HashMap<String, ConfigValue>,
    required_fields: Vec<String>,
    defaults: HashMap<String, ConfigValue>,
}

pub enum ConfigValue {
    String(String),
    Integer(i64),
    UnsignedInteger(u64),
    Float(f64),
    Boolean(bool),
    Duration(Duration),
    OptionalString(Option<String>),
    OptionalInteger(Option<i64>),
    StringList(Vec<String>),
}

impl<T> DefaultConfigBuilder<T> {
    pub fn new() -> Self;
    pub fn require_field(&mut self, field: &str) -> &mut Self;
    pub fn default_value(&mut self, field: &str, value: ConfigValue) -> &mut Self;
    pub fn set_field(&mut self, field: &str, value: ConfigValue) -> &mut Self;
    pub fn from_env_with_prefix(&mut self, prefix: &str) -> &mut Self;
    pub fn validate_required_fields(&self) -> BuilderResult<()>;
}
```

#### Fluent Builder Pattern (spider.rs)
```rust
impl SpiderConfig {
    pub fn with_user_agent(mut self, user_agent: String) -> Self;
    pub fn with_timeout(mut self, timeout: Duration) -> Self;
    pub fn with_delay(mut self, delay: Duration) -> Self;
    pub fn with_concurrency(mut self, concurrency: usize) -> Self;
    pub fn with_max_depth(mut self, max_depth: Option<usize>) -> Self;
    pub fn with_max_pages(mut self, max_pages: Option<usize>) -> Self;
    pub fn with_respect_robots(mut self, respect_robots: bool) -> Self;
    pub fn with_javascript(mut self, enable_javascript: bool) -> Self;
}
```

### Validation Capabilities

#### ApiConfig Validation (riptide-api/config.rs)
```rust
impl ApiConfig {
    pub fn validate(&self) -> Result<(), String> {
        // Resource limits
        if self.resources.max_concurrent_renders == 0 {
            return Err("max_concurrent_renders must be greater than 0".to_string());
        }

        // Rate limiting
        if self.rate_limiting.enabled {
            if self.rate_limiting.requests_per_second_per_host <= 0.0 {
                return Err("requests_per_second_per_host must be greater than 0".to_string());
            }
            if self.rate_limiting.jitter_factor < 0.0 || self.rate_limiting.jitter_factor > 1.0 {
                return Err("jitter_factor must be between 0.0 and 1.0".to_string());
            }
        }

        // Memory settings
        if self.memory.pressure_threshold <= 0.0 || self.memory.pressure_threshold > 1.0 {
            return Err("memory pressure_threshold must be between 0.0 and 1.0".to_string());
        }

        // Headless settings
        if self.headless.min_pool_size > self.headless.max_pool_size {
            return Err("headless min_pool_size cannot be greater than max_pool_size".to_string());
        }

        // Search backend validation
        match self.search.backend.as_str() {
            "serper" | "none" | "searxng" => {}
            _ => return Err(format!("Invalid search backend '{}'", self.search.backend)),
        }

        Ok(())
    }
}
```

#### SpiderConfig Validation (spider.rs)
```rust
impl SpiderConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.concurrency == 0 {
            return Err("Concurrency must be greater than 0".to_string());
        }
        if let Some(max_depth) = self.max_depth {
            if max_depth == 0 {
                return Err("Max depth must be greater than 0".to_string());
            }
        }
        if self.timeout.is_zero() {
            return Err("Timeout must be greater than 0".to_string());
        }
        if self.max_redirects > 20 {
            return Err("Max redirects should not exceed 20".to_string());
        }
        Ok(())
    }
}
```

#### ValidationPatterns Utility (builder.rs)
```rust
impl ValidationPatterns {
    pub fn validate_positive_integer(value: i64, field: &str) -> BuilderResult<()>;
    pub fn validate_range(value: f64, min: f64, max: f64, field: &str) -> BuilderResult<()>;
    pub fn validate_url(url: &str, field: &str) -> BuilderResult<()>;
    pub fn validate_non_empty_string(value: &str, field: &str) -> BuilderResult<()>;
    pub fn validate_positive_duration(duration: Duration, field: &str) -> BuilderResult<()>;
}
```

## Current Configuration Flow

### Application Startup (main.rs)
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load AppConfig from environment
    let config = AppConfig::default(); // Uses env vars internally

    // 2. Create metrics and health checker
    let metrics = Arc::new(RipTideMetrics::new()?);
    let health_checker = Arc::new(HealthChecker::new());

    // 3. Initialize AppState (loads ApiConfig from env)
    let app_state = AppState::new(config, metrics.clone(), health_checker.clone()).await?;

    // 4. Start server
    // ...
}
```

### AppState Initialization (state.rs)
```rust
impl AppState {
    pub async fn new_with_telemetry_and_api_config(
        config: AppConfig,
        api_config: ApiConfig,  // Loaded via ApiConfig::from_env()
        metrics: Arc<RipTideMetrics>,
        health_checker: Arc<HealthChecker>,
        telemetry: Option<Arc<TelemetrySystem>>,
    ) -> Result<Self> {
        // Validate API configuration
        api_config.validate()
            .map_err(|e| anyhow::anyhow!("Invalid API configuration: {}", e))?;

        // Initialize components using both configs
        // ...
    }
}
```

## Gaps for server.yaml Support

### 1. No File-Based Configuration Loading
- **Current**: Only environment variables via `std::env::var()`
- **Missing**:
  - YAML file parsing (serde_yaml integration)
  - TOML/JSON support
  - Config file discovery (`/etc/riptide/`, `~/.riptide/`, `./config/`)
  - File watching for hot reload

### 2. No Configuration Precedence Logic
- **Current**: Single source (environment variables)
- **Missing**:
  - Layered configuration merging
  - Precedence ordering (CLI args > env vars > config file > defaults)
  - Override mechanisms
  - Configuration source tracking (which value came from where)

### 3. Limited Configuration Validation
- **Current**: Per-struct validation in `validate()` methods
- **Missing**:
  - Cross-configuration validation (e.g., `riptide-config::ApiConfig` vs `riptide-api::ApiConfig`)
  - Conflict detection between configuration sources
  - Deprecation warnings
  - Schema validation

### 4. No Unified Configuration Management
- **Current**: Split between two crates with different naming conventions
- **Issues**:
  - Two `ApiConfig` types with different purposes
  - Inconsistent env var prefixes (`RIPTIDE_` vs none)
  - No central configuration registry
  - Manual env var parsing in riptide-api (59 fields!)

### 5. No Configuration Documentation
- **Missing**:
  - Auto-generated config reference
  - Example configuration files
  - Migration guides
  - Configuration templates

### 6. No Dynamic Configuration
- **Missing**:
  - Runtime configuration updates
  - Configuration reload without restart
  - Per-tenant configuration
  - Feature flags

## Recommendations for server.yaml Implementation

### 1. Create Unified Configuration Module
```
/crates/riptide-config/src/
├── lib.rs              # Public API
├── loader.rs           # Multi-source loading
├── precedence.rs       # Layering logic
├── validation.rs       # Cross-config validation
├── schema.rs           # JSON Schema generation
└── formats/
    ├── yaml.rs         # YAML parsing
    ├── toml.rs         # TOML parsing
    └── env.rs          # Enhanced env loading
```

### 2. Configuration Precedence Strategy
```
Priority (highest to lowest):
1. Runtime overrides (API calls, feature flags)
2. CLI arguments (--config-file, --bind-address)
3. Environment variables (RIPTIDE_*, API_KEYS, etc.)
4. User config file (~/.config/riptide/server.yaml)
5. System config file (/etc/riptide/server.yaml)
6. Project config file (./config/server.yaml)
7. Built-in defaults (Default trait implementations)
```

### 3. Merge Strategy
- **Scalar values**: Last-wins (highest precedence source)
- **Arrays**: Extend or replace (configurable via `merge_strategy`)
- **Objects**: Deep merge with override tracking
- **Example**:
  ```yaml
  # system.yaml
  rate_limiting:
    enabled: true
    requests_per_minute: 60

  # user.yaml (overrides)
  rate_limiting:
    requests_per_minute: 120  # Override
    # enabled: true inherited from system.yaml
  ```

### 4. Proposed server.yaml Structure
```yaml
# /etc/riptide/server.yaml
version: "1.0"
metadata:
  environment: production

server:
  bind_address: "0.0.0.0:8080"
  enable_cors: true
  enable_compression: true

authentication:
  require_auth: true
  api_keys:
    - "${API_KEY_1}"  # Support env var expansion
    - "${API_KEY_2}"
  constant_time_comparison: true

rate_limiting:
  enabled: true
  requests_per_minute: 60
  max_concurrent_requests: 100
  per_host:
    requests_per_second: 1.5
    burst_capacity: 3

resources:
  max_concurrent_renders: 10
  max_concurrent_pdf: 2
  max_concurrent_wasm: 4
  global_timeout: 30s

performance:
  render_timeout: 3s
  pdf_timeout: 10s
  http_timeout: 10s
  auto_cleanup_on_timeout: true

memory:
  global_limit_mb: 2048
  soft_limit_mb: 400
  hard_limit_mb: 500
  auto_gc: true
  enable_proactive_monitoring: true

headless:
  max_pool_size: 3
  min_pool_size: 1
  idle_timeout: 5m
  launch_timeout: 30s

spider:
  enabled: false
  # Only loaded if enabled: true
  config:
    user_agent: "RipTide Spider/1.0"
    timeout: 30s
    concurrency: 4
    max_depth: 10
    respect_robots: true

search:
  backend: serper  # serper, searxng, none
  timeout: 30s
  circuit_breaker:
    failure_threshold: 50
    recovery_timeout: 60s

cache:
  redis_url: "redis://localhost:6379"
  ttl: 1h

# Advanced options
advanced:
  wasm_path: "./target/wasm32-wasip2/release/riptide_extractor_wasm.wasm"
  gate_hi_threshold: 0.7
  gate_lo_threshold: 0.3
```

### 5. Implementation Plan

**Phase 1: YAML Support** (Week 1)
- Add `serde_yaml` dependency
- Create `YamlConfigLoader` in `riptide-config`
- Support basic YAML parsing with defaults
- Add `--config-file` CLI argument

**Phase 2: Precedence Logic** (Week 2)
- Implement `ConfigLayer` trait
- Create `ConfigMerger` with deep merge
- Add source tracking (`ConfigSource` enum)
- Support multiple config file locations

**Phase 3: Validation & Migration** (Week 3)
- Cross-configuration validation
- Deprecation warnings for old env vars
- Generate migration guide
- Add schema validation

**Phase 4: Documentation & Tooling** (Week 4)
- Auto-generate config reference from code
- Create example configs for each preset
- Add `riptide config validate` command
- Add `riptide config show` command (with source info)

### 6. Backward Compatibility
- Keep all existing env var support
- Default to env vars if no config file found
- Warn on deprecated configurations
- Provide migration tool: `riptide config migrate`

## Environment Variable Summary

### riptide-config Crate (10 vars)
- `API_KEYS` (comma-separated)
- `REQUIRE_AUTH` (bool)
- `BIND_ADDRESS` (string)
- `MAX_CONCURRENT_REQUESTS` (usize)
- `RATE_LIMIT_PER_MINUTE` (u32)
- `ENABLE_RATE_LIMITING` (bool)
- `REQUEST_TIMEOUT_SECS` (u64)
- `MAX_PAYLOAD_SIZE` (usize)
- `ENABLE_CORS` (bool)
- `ENABLE_COMPRESSION` (bool)

### riptide-api Crate (59+ vars)
**Resource Config (7)**:
- `RIPTIDE_MAX_CONCURRENT_RENDERS`
- `RIPTIDE_MAX_CONCURRENT_PDF`
- `RIPTIDE_MAX_CONCURRENT_WASM`
- `RIPTIDE_GLOBAL_TIMEOUT_SECS`
- `RIPTIDE_CLEANUP_INTERVAL_SECS`
- `RIPTIDE_ENABLE_MONITORING`
- `RIPTIDE_HEALTH_CHECK_INTERVAL_SECS`

**Performance Config (7)**:
- `RIPTIDE_RENDER_TIMEOUT_SECS`
- `RIPTIDE_PDF_TIMEOUT_SECS`
- `RIPTIDE_WASM_TIMEOUT_SECS`
- `RIPTIDE_HTTP_TIMEOUT_SECS`
- `RIPTIDE_MEMORY_CLEANUP_THRESHOLD_MB`
- `RIPTIDE_AUTO_CLEANUP_ON_TIMEOUT`
- `RIPTIDE_DEGRADATION_THRESHOLD`

**Rate Limiting Config (7)**:
- `RIPTIDE_RATE_LIMIT_ENABLED`
- `RIPTIDE_RATE_LIMIT_RPS`
- `RIPTIDE_RATE_LIMIT_JITTER`
- `RIPTIDE_RATE_LIMIT_BURST_CAPACITY`
- `RIPTIDE_RATE_LIMIT_WINDOW_SECS`
- `RIPTIDE_RATE_LIMIT_CLEANUP_INTERVAL_SECS`
- `RIPTIDE_RATE_LIMIT_MAX_TRACKED_HOSTS`

**Memory Config (10)**:
- `RIPTIDE_MAX_MEMORY_PER_REQUEST_MB`
- `RIPTIDE_GLOBAL_MEMORY_LIMIT_MB`
- `RIPTIDE_MEMORY_SOFT_LIMIT_MB`
- `RIPTIDE_MEMORY_HARD_LIMIT_MB`
- `RIPTIDE_MEMORY_PRESSURE_THRESHOLD`
- `RIPTIDE_MEMORY_AUTO_GC`
- `RIPTIDE_MEMORY_GC_TRIGGER_THRESHOLD_MB`
- `RIPTIDE_MEMORY_MONITORING_INTERVAL_SECS`
- `RIPTIDE_MEMORY_ENABLE_LEAK_DETECTION`
- `RIPTIDE_MEMORY_ENABLE_PROACTIVE_MONITORING`

**Headless Config (9)**:
- `RIPTIDE_HEADLESS_MAX_POOL_SIZE`
- `RIPTIDE_HEADLESS_MIN_POOL_SIZE`
- `RIPTIDE_HEADLESS_IDLE_TIMEOUT_SECS`
- `RIPTIDE_HEADLESS_HEALTH_CHECK_INTERVAL_SECS`
- `RIPTIDE_HEADLESS_MAX_PAGES_PER_BROWSER`
- `RIPTIDE_HEADLESS_RESTART_THRESHOLD`
- `RIPTIDE_HEADLESS_ENABLE_RECYCLING`
- `RIPTIDE_HEADLESS_LAUNCH_TIMEOUT_SECS`
- `RIPTIDE_HEADLESS_MAX_RETRIES`

**PDF Config (6)**:
- `RIPTIDE_PDF_MAX_CONCURRENT`
- `RIPTIDE_PDF_PROCESSING_TIMEOUT_SECS`
- `RIPTIDE_PDF_MAX_FILE_SIZE_MB`
- `RIPTIDE_PDF_ENABLE_STREAMING`
- `RIPTIDE_PDF_QUEUE_SIZE`
- `RIPTIDE_PDF_QUEUE_TIMEOUT_SECS`

**WASM Config (7, feature-gated)**:
- `RIPTIDE_WASM_INSTANCES_PER_WORKER`
- `RIPTIDE_WASM_MODULE_TIMEOUT_SECS`
- `RIPTIDE_WASM_MAX_MEMORY_MB`
- `RIPTIDE_WASM_ENABLE_RECYCLING`
- `RIPTIDE_WASM_HEALTH_CHECK_INTERVAL_SECS`
- `RIPTIDE_WASM_MAX_OPERATIONS_PER_INSTANCE`
- `RIPTIDE_WASM_RESTART_THRESHOLD`

**Search Config (6)**:
- `RIPTIDE_SEARCH_BACKEND`
- `RIPTIDE_SEARCH_TIMEOUT_SECS`
- `RIPTIDE_SEARCH_ENABLE_URL_PARSING`
- `RIPTIDE_SEARCH_CIRCUIT_BREAKER_FAILURE_THRESHOLD`
- `RIPTIDE_SEARCH_CIRCUIT_BREAKER_MIN_REQUESTS`
- `RIPTIDE_SEARCH_CIRCUIT_BREAKER_RECOVERY_TIMEOUT_SECS`

**AppConfig (15+)**:
- `REDIS_URL`
- `WASM_EXTRACTOR_PATH`
- `MAX_CONCURRENCY`
- `CACHE_TTL`
- `GATE_HI_THRESHOLD`
- `GATE_LO_THRESHOLD`
- `HEADLESS_URL`
- `SPIDER_ENABLE`
- `SPIDER_BASE_URL`
- `SPIDER_USER_AGENT`
- `SPIDER_TIMEOUT_SECONDS`
- `SPIDER_DELAY_MS`
- `SPIDER_CONCURRENCY`
- `SPIDER_MAX_DEPTH`
- `SPIDER_MAX_PAGES`
- Plus worker, circuit breaker, and pipeline configs...

## Code Quality Issues

### 1. Two ApiConfig Types
- **riptide-config::ApiConfig**: Authentication, rate limiting, request settings
- **riptide-api::ApiConfig**: Resource, performance, memory settings
- **Impact**: Confusing naming, difficult to merge

### 2. Inconsistent Env Var Naming
- **riptide-config**: No prefix (`API_KEYS`, `BIND_ADDRESS`)
- **riptide-api**: `RIPTIDE_` prefix (`RIPTIDE_MAX_CONCURRENT_RENDERS`)
- **AppConfig**: Mixed (`REDIS_URL`, `SPIDER_ENABLE`)

### 3. Manual Env Var Parsing in riptide-api
- 59 fields parsed manually with copy-paste code
- No use of `EnvConfigLoader` utility
- Error-prone and difficult to maintain

### 4. Validation Scattered
- Some in `from_env()` (panics on invalid API keys)
- Some in `validate()` methods
- Some implicit (type coercion)

### 5. No Configuration Testing
- No tests for env var loading
- No tests for precedence (doesn't exist yet)
- No tests for validation edge cases

## Next Steps

1. **Immediate**: Create `ConfigurationLoader` trait to unify loading
2. **Short-term**: Implement YAML support with `serde_yaml`
3. **Medium-term**: Add precedence logic and file discovery
4. **Long-term**: Consolidate two `ApiConfig` types into coherent hierarchy

## References

### Key Files
- `/crates/riptide-config/src/lib.rs` - Public API
- `/crates/riptide-config/src/api.rs` - API server config (auth, rate limiting)
- `/crates/riptide-config/src/spider.rs` - Spider crawling config
- `/crates/riptide-config/src/env.rs` - Env var loading utilities
- `/crates/riptide-config/src/builder.rs` - Builder patterns
- `/crates/riptide-config/src/validation.rs` - Security validation
- `/crates/riptide-api/src/config.rs` - Resource & performance config
- `/crates/riptide-api/src/state.rs` - AppConfig and initialization
- `/crates/riptide-api/src/main.rs` - Application startup

### Dependencies
- `serde` - Serialization framework (already used)
- `serde_yaml` - YAML support (**to be added**)
- `url` - URL validation (already used)
- `thiserror` - Error types (already used)
- `anyhow` - Error handling (already used)

---

**Document Status**: Complete
**Reviewed By**: Code Quality Analyzer
**Last Updated**: 2025-11-04
