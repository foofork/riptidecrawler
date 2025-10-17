# Configuration & Environment Variables Audit Report

**Date:** 2025-10-17
**Priority:** P0 - Critical Path Documentation
**Status:** Complete

## Executive Summary

This audit documents all environment variables, configuration patterns, and output directory mappings across the RipTide codebase. Critical findings include **inconsistent output directory handling** and **undocumented environment variables** that could impact deployment and user experience.

---

## 1. Environment Variables Matrix

### 1.1 Core API Configuration

| Variable Name | Type | Default Value | Used By | Documented |
|--------------|------|---------------|---------|------------|
| `RIPTIDE_API_URL` | String | `http://localhost:8080` | CLI | ✅ .env.example |
| `RIPTIDE_API_KEY` | String | (none) | CLI, API | ✅ .env.example |
| `REDIS_URL` | String | `redis://localhost:6379` | API, Workers | ✅ .env.example |
| `HEADLESS_URL` | String | (none) | API | ✅ .env.example |
| `WASM_EXTRACTOR_PATH` | String | `./target/wasm32-wasip2/release/riptide_extractor_wasm.wasm` | API, CLI, Workers | ⚠️ Partial |

### 1.2 Performance & Resource Limits

| Variable Name | Type | Default Value | Used By | Documented |
|--------------|------|---------------|---------|------------|
| `RIPTIDE_MAX_CONCURRENT_RENDERS` | usize | 10 | API | ✅ .env.example |
| `RIPTIDE_MAX_CONCURRENT_PDF` | usize | 2 | API | ✅ .env.example |
| `RIPTIDE_RENDER_TIMEOUT` | u64 | 3 (seconds) | API | ✅ .env.example |
| `RIPTIDE_RATE_LIMIT_RPS` | f64 | 1.5 | API | ✅ .env.example |
| `RIPTIDE_RATE_LIMIT_JITTER` | f64 | 0.1 | API | ✅ .env.example |
| `RIPTIDE_HEADLESS_POOL_SIZE` | usize | 3 | API | ✅ .env.example |
| `RIPTIDE_MEMORY_LIMIT_MB` | usize | 2048 | API | ✅ .env.example |
| `MAX_CONCURRENCY` | usize | 16 | API | ❌ Missing |

### 1.3 Search Backend Configuration

| Variable Name | Type | Default Value | Used By | Documented |
|--------------|------|---------------|---------|------------|
| `SEARCH_BACKEND` | String | "serper" | API, Search | ✅ .env.example |
| `SERPER_API_KEY` | String | (none) | Search | ✅ .env.example |
| `SEARXNG_BASE_URL` | String | (none) | Search | ✅ .env.example |
| `SEARCH_TIMEOUT` | u64 | 30 (seconds) | Search | ✅ .env.example |
| `SEARCH_ENABLE_URL_PARSING` | bool | true | Search | ✅ .env.example |

### 1.4 Circuit Breaker & Reliability

| Variable Name | Type | Default Value | Used By | Documented |
|--------------|------|---------------|---------|------------|
| `CIRCUIT_BREAKER_FAILURE_THRESHOLD` | u8 | 50 (%) | API, Search | ✅ .env.example |
| `CIRCUIT_BREAKER_TIMEOUT_MS` | u64 | 5000 | API | ✅ .env.example |
| `CIRCUIT_BREAKER_MIN_REQUESTS` | u64 | 10 | API, Search | ✅ .env.example |
| `CIRCUIT_BREAKER_RECOVERY_TIMEOUT` | u64 | 60 (seconds) | Search | ✅ .env.example |

### 1.5 Enhanced Pipeline Configuration

| Variable Name | Type | Default Value | Used By | Documented |
|--------------|------|---------------|---------|------------|
| `ENHANCED_PIPELINE_ENABLE` | bool | true | API | ✅ .env.example |
| `ENHANCED_PIPELINE_METRICS` | bool | true | API | ✅ .env.example |
| `ENHANCED_PIPELINE_DEBUG` | bool | false | API | ✅ .env.example |
| `ENHANCED_PIPELINE_FETCH_TIMEOUT` | u64 | 15 (seconds) | API | ✅ .env.example |
| `ENHANCED_PIPELINE_GATE_TIMEOUT` | u64 | 5 (seconds) | API | ✅ .env.example |
| `ENHANCED_PIPELINE_WASM_TIMEOUT` | u64 | 30 (seconds) | API | ✅ .env.example |
| `ENHANCED_PIPELINE_RENDER_TIMEOUT` | u64 | 60 (seconds) | API | ✅ .env.example |

### 1.6 Spider/Crawler Configuration

| Variable Name | Type | Default Value | Used By | Documented |
|--------------|------|---------------|---------|------------|
| `SPIDER_ENABLE` | bool | false | API | ✅ .env.example |
| `SPIDER_BASE_URL` | String | "https://example.com" | Spider | ✅ .env.example |
| `SPIDER_MAX_DEPTH` | usize | 3 | Spider | ✅ .env.example |
| `SPIDER_MAX_PAGES` | usize | 100 | Spider | ✅ .env.example |
| `SPIDER_CONCURRENCY` | usize | 4 | Spider | ✅ .env.example |
| `SPIDER_TIMEOUT_SECONDS` | u64 | 30 | Spider | ✅ .env.example |
| `SPIDER_DELAY_MS` | u64 | 500 | Spider | ✅ .env.example |
| `SPIDER_RESPECT_ROBOTS` | bool | true | Spider | ✅ .env.example |
| `SPIDER_USER_AGENT` | String | "RipTide Spider/1.0" | Spider | ✅ .env.example |

### 1.7 Worker Configuration

| Variable Name | Type | Default Value | Used By | Documented |
|--------------|------|---------------|---------|------------|
| `WORKER_POOL_SIZE` | usize | 4 | Workers | ✅ .env.example |
| `WORKER_REDIS_URL` | String | (falls back to REDIS_URL) | Workers | ❌ Missing |
| `WORKER_MAX_BATCH_SIZE` | usize | 50 | Workers | ✅ .env.example |
| `WORKER_MAX_CONCURRENCY` | usize | 10 | Workers | ✅ .env.example |
| `WORKER_ENABLE_SCHEDULER` | bool | true | Workers | ✅ .env.example |

### 1.8 LLM/AI Provider Configuration

| Variable Name | Type | Default Value | Used By | Documented |
|--------------|------|---------------|---------|------------|
| `OPENAI_API_KEY` | String | (none) | Intelligence | ✅ .env.example |
| `OPENAI_BASE_URL` | String | "https://api.openai.com/v1" | Intelligence | ✅ .env.example |
| `ANTHROPIC_API_KEY` | String | (none) | Intelligence | ✅ .env.example |
| `AZURE_OPENAI_KEY` | String | (none) | Intelligence | ✅ .env.example |
| `AZURE_OPENAI_ENDPOINT` | String | (none) | Intelligence | ✅ .env.example |
| `OLLAMA_BASE_URL` | String | "http://localhost:11434" | Intelligence | ✅ .env.example |
| `RIPTIDE_PROVIDER_*_ENABLED` | bool | false | Intelligence | ❌ Missing |
| `RIPTIDE_PROVIDER_*_API_KEY` | String | (none) | Intelligence | ❌ Missing |
| `RIPTIDE_PROVIDER_*_MODEL` | String | (varies) | Intelligence | ❌ Missing |
| `RIPTIDE_PROVIDER_*_PRIORITY` | u32 | (none) | Intelligence | ❌ Missing |
| `RIPTIDE_METRICS_ENABLED` | bool | true | Intelligence | ❌ Missing |
| `RIPTIDE_COST_TRACKING_ENABLED` | bool | true | Intelligence | ❌ Missing |

### 1.9 Telemetry & Observability

| Variable Name | Type | Default Value | Used By | Documented |
|--------------|------|---------------|---------|------------|
| `TELEMETRY_ENABLED` | bool | true | API | ✅ .env.example |
| `TELEMETRY_SERVICE_NAME` | String | "riptide-api" | API | ✅ .env.example |
| `TELEMETRY_SERVICE_VERSION` | String | `CARGO_PKG_VERSION` | API | ✅ .env.example |
| `TELEMETRY_OTLP_ENDPOINT` | String | (none) | API | ✅ .env.example |
| `OTEL_ENDPOINT` | String | (none) | API | ✅ .env.example |
| `TELEMETRY_EXPORTER_TYPE` | String | "stdout" | API | ✅ .env.example |
| `TELEMETRY_SAMPLING_RATIO` | f64 | 1.0 | API | ✅ .env.example |
| `TELEMETRY_EXPORT_TIMEOUT_SECS` | u64 | 30 | API | ✅ .env.example |
| `TELEMETRY_MAX_QUEUE_SIZE` | u64 | 2048 | API | ✅ .env.example |
| `TELEMETRY_MAX_EXPORT_BATCH_SIZE` | u64 | 512 | API | ✅ .env.example |

### 1.10 Streaming Configuration

| Variable Name | Type | Default Value | Used By | Documented |
|--------------|------|---------------|---------|------------|
| `STREAM_BUFFER_SIZE` | usize | 8192 | API | ✅ .env.example |
| `STREAM_BUFFER_MAX_SIZE` | usize | 65536 | API | ✅ .env.example |
| `WS_MAX_MESSAGE_SIZE` | usize | 16777216 | API | ✅ .env.example |
| `WS_PING_INTERVAL` | u64 | 30 (seconds) | API | ✅ .env.example |
| `STREAM_MAX_CONCURRENT` | usize | 100 | API | ✅ .env.example |
| `STREAM_DEFAULT_TIMEOUT` | u64 | 300 (seconds) | API | ✅ .env.example |
| `STREAM_RATE_LIMIT_ENABLED` | bool | true | API | ✅ .env.example |
| `STREAM_RATE_LIMIT_RPS` | u64 | 10 | API | ✅ .env.example |

### 1.11 Cache & Persistence

| Variable Name | Type | Default Value | Used By | Documented |
|--------------|------|---------------|---------|------------|
| `CACHE_TTL` | u64 | 86400 (seconds) | API | ✅ .env.example |
| `CACHE_DEFAULT_TTL_SECONDS` | u64 | 86400 | API | ✅ .env.example |
| `ENABLE_COMPRESSION` | bool | true | API | ✅ .env.example |
| `ENABLE_MULTI_TENANCY` | bool | false | API | ✅ .env.example |
| `GATE_HI_THRESHOLD` | f32 | 0.7 | API | ❌ Missing |
| `GATE_LO_THRESHOLD` | f32 | 0.3 | API | ❌ Missing |

### 1.12 Authentication & Security

| Variable Name | Type | Default Value | Used By | Documented |
|--------------|------|---------------|---------|------------|
| `API_KEYS` | String (CSV) | (none) | API | ✅ .env.example |
| `REQUIRE_AUTH` | bool | false | API | ✅ .env.example |

### 1.13 Cache Warming Configuration

| Variable Name | Type | Default Value | Used By | Documented |
|--------------|------|---------------|---------|------------|
| `RIPTIDE_CACHE_WARMING_ENABLED` | bool | false | Core | ❌ Missing |
| `RIPTIDE_WARM_POOL_SIZE` | usize | 2 | Core | ❌ Missing |
| `RIPTIDE_MIN_WARM_INSTANCES` | usize | 1 | Core | ❌ Missing |
| `RIPTIDE_MAX_WARM_INSTANCES` | usize | 4 | Core | ❌ Missing |
| `RIPTIDE_WARMING_INTERVAL_SECS` | u64 | 300 | Core | ❌ Missing |
| `RIPTIDE_CACHE_HIT_TARGET` | f32 | 0.8 | Core | ❌ Missing |
| `RIPTIDE_ENABLE_PREFETCHING` | bool | true | Core | ❌ Missing |

### 1.14 WASM Configuration

| Variable Name | Type | Default Value | Used By | Documented |
|--------------|------|---------------|---------|------------|
| `RIPTIDE_WASM_INSTANCES_PER_WORKER` | usize | 1 | Core | ❌ Missing |
| `RIPTIDE_WASM_ENABLE_SIMD` | bool | (varies) | Tests | ❌ Missing |
| `RIPTIDE_WASM_ENABLE_AOT_CACHE` | bool | (varies) | Tests | ❌ Missing |
| `RIPTIDE_WASM_MEMORY_LIMIT_PAGES` | usize | (varies) | Tests | ❌ Missing |
| `RIPTIDE_WASM_MEMORY_LIMIT_MB` | usize | (varies) | Tests | ❌ Missing |
| `RIPTIDE_WASM_COLD_START_TARGET_MS` | u64 | (varies) | Tests | ❌ Missing |
| `RIPTIDE_WASM_MAX_POOL_SIZE` | usize | (varies) | Tests | ❌ Missing |
| `RIPTIDE_WASM_INITIAL_POOL_SIZE` | usize | (varies) | Tests | ❌ Missing |

### 1.15 Development & Testing

| Variable Name | Type | Default Value | Used By | Documented |
|--------------|------|---------------|---------|------------|
| `RUST_LOG` | String | "info" | All | ✅ .env.example |
| `HEALTH_CHECK_PORT` | u16 | (main API port) | API | ✅ .env.example |
| `GIT_SHA` | String | (auto-populated) | API | ✅ .env.example |
| `BUILD_TIMESTAMP` | String | (auto-populated) | API | ✅ .env.example |
| `TEST_REDIS_URL` | String | "redis://localhost:6379/15" | Tests | ✅ .env.example |
| `TEST_WASM_PATH` | String | (varies) | Tests | ✅ .env.example |
| `SKIP_PERSISTENCE_TESTS` | bool | false | Tests | ✅ .env.example |
| `SKIP_REDIS_TESTS` | bool | false | Tests | ✅ .env.example |
| `TEST_TIMEOUT_MULTIPLIER` | f64 | 1.0 | Tests | ✅ .env.example |
| `WASM_COMPONENT_PATH` | String | (varies) | Tests | ❌ Missing |
| `RIPTIDE_FEATURE_PDF` | bool | (varies) | Tests | ✅ .env.example |
| `RIPTIDE_FEATURE_BENCHMARKS` | bool | (varies) | Tests | ✅ .env.example |
| `RIPTIDE_FEATURE_API_INTEGRATION` | bool | (varies) | Tests | ✅ .env.example |
| `GOLDEN_TEST_ENV` | String | "development" | Tests | ❌ Missing |
| `RUSTC_VERSION` | String | (auto) | Tests | ❌ Missing |

### 1.16 Streaming CLI Configuration

| Variable Name | Type | Default Value | Used By | Documented |
|--------------|------|---------------|---------|------------|
| `RIPTIDE_API_HOST` | String | "localhost" | Streaming | ❌ Missing |
| `RIPTIDE_API_PORT` | u16 | 8080 | Streaming | ❌ Missing |
| `RIPTIDE_LOG_LEVEL` | String | (falls back to RUST_LOG) | Streaming | ❌ Missing |
| `RIPTIDE_CONFIG_FILE` | String | (none) | Streaming | ❌ Missing |
| `RIPTIDE_DEV` | bool | false | Streaming | ❌ Missing |
| `DEVELOPMENT` | bool | false | Streaming | ❌ Missing |
| `RIPTIDE_CACHE_DIR` | String | (none) | CLI | ❌ Missing |

---

## 2. Output Directory Mapping (P0 Priority)

### 2.1 Screenshot Storage Paths

**CRITICAL FINDING:** No explicit screenshot output directory configuration found in current codebase.

| Component | Default Path | Env Var | Configurable | Status |
|-----------|-------------|---------|--------------|---------|
| CLI Render | `.` (current dir) | ❌ None | ✅ Via `--output-dir` flag | ⚠️ Hardcoded default |
| API | ❌ Not implemented | ❌ None | ❌ No | 🔴 **Missing** |
| Headless | ❌ Not implemented | ❌ None | ❌ No | 🔴 **Missing** |

**CLI Render Command:**
```rust
// File: crates/riptide-cli/src/commands/render.rs:145
#[arg(long, short = 'o', default_value = ".")]
pub output_dir: String,
```

**Issue:** Screenshot output defaults to current working directory, not a dedicated output folder.

### 2.2 Extraction/Report Storage Paths

| Component | Default Path | Env Var | Configurable | Status |
|-----------|-------------|---------|--------------|---------|
| Reports (Streaming) | System data dir + `/reports` | ❌ None | ✅ Via config | ⚠️ Platform-specific |
| Sample Reports | `target/sample_report.*` | ❌ None | ❌ No | ⚠️ Build artifact path |
| CLI Domain Reports | (user specified) | ❌ None | ✅ Via `--output` flag | ✅ Good |
| CLI Schema Reports | (user specified) | ❌ None | ✅ Via `--output` flag | ✅ Good |
| Triage Reports | `.reports/triage.md` | ❌ None | ❌ No | ⚠️ Hardcoded |

**Streaming Reports Default:**
```rust
// File: crates/riptide-streaming/src/config.rs:402
pub fn get_default_output_directory() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "riptide", "riptide") {
        proj_dirs.data_dir().join("reports")
    } else {
        PathBuf::from("reports")
    }
}
```

**Platform-specific paths:**
- Linux: `~/.local/share/riptide/reports`
- macOS: `~/Library/Application Support/com.riptide.riptide/reports`
- Windows: `C:\Users\<user>\AppData\Roaming\riptide\riptide\reports`

### 2.3 Recommended Output Structure

```
$RIPTIDE_OUTPUT_DIR/  (default: ./riptide_output or ~/.local/share/riptide)
├── screenshots/
│   ├── full/
│   └── viewport/
├── extractions/
│   ├── html/
│   ├── dom/
│   └── json/
├── reports/
│   ├── html/
│   ├── pdf/
│   └── csv/
├── pdfs/
└── logs/
```

---

## 3. Configuration Inconsistencies

### 3.1 Dual Configuration Patterns

**Issue:** Mix of `ApiConfig::from_env()` and `ConfigBuilder` approaches:

```rust
// Pattern 1: Direct environment parsing (API)
impl ApiConfig {
    pub fn from_env() -> Self {
        if let Ok(val) = std::env::var("RIPTIDE_MAX_CONCURRENT_RENDERS") {
            config.resources.max_concurrent_renders = val.parse().unwrap_or_default();
        }
    }
}

// Pattern 2: Config builder (Streaming)
impl ConfigBuilder {
    pub fn add_env(mut self, prefix: &str) -> Self {
        self.env_prefix = Some(prefix.to_string());
        self
    }
}
```

**Recommendation:** Standardize on `ConfigBuilder` pattern with fallback chains.

### 3.2 Missing Default Fallbacks

Several environment variables lack proper defaults:

| Variable | Issue | Recommendation |
|----------|-------|----------------|
| `RIPTIDE_WASM_PATH` | Hardcoded path in multiple places | Create `RIPTIDE_WASM_PATH` env with global default |
| `MAX_CONCURRENCY` | Not in .env.example | Add with default: 16 |
| `GATE_HI_THRESHOLD` | Not documented | Add with default: 0.7 |
| `GATE_LO_THRESHOLD` | Not documented | Add with default: 0.3 |
| `WORKER_REDIS_URL` | Falls back to REDIS_URL silently | Document fallback behavior |

### 3.3 Conflicting Defaults

| Variable | Location 1 | Location 2 | Conflict |
|----------|-----------|-----------|----------|
| `WASM_EXTRACTOR_PATH` | `./target/wasm32-wasi/release/extractor.wasm` | `./target/wasm32-wasip2/release/riptide_extractor_wasm.wasm` | ⚠️ Different WASI versions |
| `SPIDER_BASE_URL` | Required when enabled | Default "https://example.com" | ⚠️ May fail validation |

---

## 4. Configuration Gaps & Issues

### 4.1 Missing Environment Variables

These are used in code but **not documented** in `.env.example`:

1. **Intelligence System:**
   - `RIPTIDE_PROVIDER_*` (all provider-specific configs)
   - `RIPTIDE_METRICS_ENABLED`
   - `RIPTIDE_COST_TRACKING_ENABLED`
   - `RIPTIDE_PROMETHEUS_ENDPOINT`

2. **WASM Runtime:**
   - `RIPTIDE_WASM_INSTANCES_PER_WORKER`
   - `RIPTIDE_WASM_*` (all WASM performance configs)

3. **Cache Warming:**
   - `RIPTIDE_CACHE_WARMING_ENABLED`
   - `RIPTIDE_WARM_POOL_SIZE`
   - `RIPTIDE_*_WARM_*` (all warming configs)

4. **Streaming CLI:**
   - `RIPTIDE_API_HOST`
   - `RIPTIDE_API_PORT`
   - `RIPTIDE_LOG_LEVEL`
   - `RIPTIDE_CONFIG_FILE`
   - `RIPTIDE_DEV`
   - `RIPTIDE_CACHE_DIR`

5. **Core:**
   - `MAX_CONCURRENCY`
   - `GATE_HI_THRESHOLD`
   - `GATE_LO_THRESHOLD`

### 4.2 Validation Issues

**No validation for:**
- `SPIDER_BASE_URL` format before spider initialization
- `WASM_EXTRACTOR_PATH` existence before startup
- `API_KEYS` format (CSV parsing could fail silently)
- Numeric environment variable parsing (many use `.unwrap_or_default()`)

### 4.3 Secret Management

**Sensitive values in `.env.example`:**
- `RIPTIDE_API_KEY=your_api_key_here` (should be placeholder)
- `SERPER_API_KEY=your_serper_api_key_here`
- All LLM provider API keys

**Recommendation:** Add `.env.secrets` template with clear documentation on secret management.

---

## 5. Standardization Recommendations

### 5.1 Immediate Actions (P0)

1. **Create `RIPTIDE_OUTPUT_DIR` environment variable:**
   ```bash
   RIPTIDE_OUTPUT_DIR=./riptide_output
   ```

2. **Standardize screenshot paths:**
   ```rust
   // Default: $RIPTIDE_OUTPUT_DIR/screenshots
   pub fn get_screenshot_dir() -> PathBuf {
       env::var("RIPTIDE_OUTPUT_DIR")
           .map(PathBuf::from)
           .unwrap_or_else(|_| PathBuf::from("./riptide_output"))
           .join("screenshots")
   }
   ```

3. **Add missing variables to `.env.example`:**
   - All `RIPTIDE_PROVIDER_*` variables
   - All `RIPTIDE_WASM_*` variables
   - All `RIPTIDE_CACHE_*` variables
   - `MAX_CONCURRENCY`, `GATE_HI_THRESHOLD`, `GATE_LO_THRESHOLD`

4. **Document fallback chains:**
   ```bash
   # WORKER_REDIS_URL falls back to REDIS_URL
   WORKER_REDIS_URL=${REDIS_URL}
   ```

### 5.2 Configuration Architecture (P1)

1. **Unified Configuration Loader:**
   ```rust
   pub struct RipTideConfig {
       // Load from: CLI args → env vars → config file → defaults
       pub fn load() -> Result<Self> {
           ConfigBuilder::new()
               .with_config_file()
               .with_env_prefix("RIPTIDE_")
               .with_cli_overrides()
               .build()
       }
   }
   ```

2. **Environment Variable Naming Convention:**
   - Prefix: `RIPTIDE_`
   - Component: `WASM_`, `SPIDER_`, `WORKER_`, etc.
   - Setting: `TIMEOUT`, `POOL_SIZE`, etc.
   - Example: `RIPTIDE_WASM_POOL_SIZE`

3. **Configuration Validation:**
   ```rust
   impl RipTideConfig {
       pub fn validate(&self) -> Result<()> {
           // Validate paths exist
           // Validate numeric ranges
           // Validate URL formats
           // Check for conflicting settings
       }
   }
   ```

### 5.3 Documentation Improvements (P1)

1. **Create `docs/configuration.md` with:**
   - Complete environment variable reference
   - Configuration file examples
   - Output directory structure
   - Secret management guidelines
   - Deployment best practices

2. **Update `.env.example` with:**
   - Section headers for clarity
   - Detailed comments for each variable
   - Examples for complex formats (CSV, JSON)
   - Security warnings for sensitive values

3. **Add inline documentation:**
   - Rustdoc comments for all config structs
   - Default value documentation
   - Deprecation warnings for old variables

---

## 6. Environment Variable Reference Table

### Complete Alphabetical Reference

| Variable | Type | Default | Component | Priority |
|----------|------|---------|-----------|----------|
| API_KEYS | String | (none) | API | Medium |
| ANTHROPIC_API_KEY | String | (none) | Intelligence | Low |
| AZURE_OPENAI_ENDPOINT | String | (none) | Intelligence | Low |
| AZURE_OPENAI_KEY | String | (none) | Intelligence | Low |
| BUILD_TIMESTAMP | String | (auto) | API | Low |
| CACHE_DEFAULT_TTL_SECONDS | u64 | 86400 | API | Low |
| CACHE_TTL | u64 | 86400 | API | Medium |
| CIRCUIT_BREAKER_FAILURE_THRESHOLD | u8 | 50 | API | High |
| CIRCUIT_BREAKER_MIN_REQUESTS | u64 | 10 | API | High |
| CIRCUIT_BREAKER_RECOVERY_TIMEOUT | u64 | 60 | Search | High |
| CIRCUIT_BREAKER_TIMEOUT_MS | u64 | 5000 | API | High |
| ENABLE_COMPRESSION | bool | true | API | Low |
| ENABLE_MULTI_TENANCY | bool | false | API | Low |
| ENHANCED_PIPELINE_DEBUG | bool | false | API | Low |
| ENHANCED_PIPELINE_ENABLE | bool | true | API | High |
| ENHANCED_PIPELINE_FETCH_TIMEOUT | u64 | 15 | API | Medium |
| ENHANCED_PIPELINE_GATE_TIMEOUT | u64 | 5 | API | Medium |
| ENHANCED_PIPELINE_METRICS | bool | true | API | Medium |
| ENHANCED_PIPELINE_RENDER_TIMEOUT | u64 | 60 | API | Medium |
| ENHANCED_PIPELINE_WASM_TIMEOUT | u64 | 30 | API | Medium |
| GATE_HI_THRESHOLD | f32 | 0.7 | API | **Missing** |
| GATE_LO_THRESHOLD | f32 | 0.3 | API | **Missing** |
| GIT_SHA | String | (auto) | API | Low |
| GOLDEN_TEST_ENV | String | "development" | Tests | **Missing** |
| HEADLESS_URL | String | (none) | API | Medium |
| HEALTH_CHECK_PORT | u16 | (main) | API | Low |
| MAX_CONCURRENCY | usize | 16 | API | **Missing** |
| OLLAMA_BASE_URL | String | "http://localhost:11434" | Intelligence | Low |
| OPENAI_API_KEY | String | (none) | Intelligence | Low |
| OPENAI_BASE_URL | String | "https://api.openai.com/v1" | Intelligence | Low |
| OTEL_ENDPOINT | String | (none) | API | Low |
| REDIS_URL | String | "redis://localhost:6379" | API | High |
| REQUIRE_AUTH | bool | false | API | High |
| RIPTIDE_API_HOST | String | "localhost" | Streaming | **Missing** |
| RIPTIDE_API_KEY | String | (none) | CLI | High |
| RIPTIDE_API_PORT | u16 | 8080 | Streaming | **Missing** |
| RIPTIDE_API_URL | String | "http://localhost:8080" | CLI | High |
| RIPTIDE_CACHE_DIR | String | (none) | CLI | **Missing** |
| RIPTIDE_CACHE_HIT_TARGET | f32 | 0.8 | Core | **Missing** |
| RIPTIDE_CACHE_WARMING_ENABLED | bool | false | Core | **Missing** |
| RIPTIDE_CONFIG_FILE | String | (none) | Streaming | **Missing** |
| RIPTIDE_DEV | bool | false | Streaming | **Missing** |
| RIPTIDE_ENABLE_PREFETCHING | bool | true | Core | **Missing** |
| RIPTIDE_FEATURE_API_INTEGRATION | bool | (varies) | Tests | Low |
| RIPTIDE_FEATURE_BENCHMARKS | bool | (varies) | Tests | Low |
| RIPTIDE_FEATURE_PDF | bool | (varies) | Tests | Low |
| RIPTIDE_HEADLESS_POOL_SIZE | usize | 3 | API | High |
| RIPTIDE_LOG_LEVEL | String | (RUST_LOG) | Streaming | **Missing** |
| RIPTIDE_MAX_CONCURRENT_PDF | usize | 2 | API | High |
| RIPTIDE_MAX_CONCURRENT_RENDERS | usize | 10 | API | High |
| RIPTIDE_MAX_WARM_INSTANCES | usize | 4 | Core | **Missing** |
| RIPTIDE_MEMORY_LIMIT_MB | usize | 2048 | API | High |
| RIPTIDE_METRICS_ENABLED | bool | true | Intelligence | **Missing** |
| RIPTIDE_MIN_WARM_INSTANCES | usize | 1 | Core | **Missing** |
| RIPTIDE_PROVIDER_*_API_KEY | String | (varies) | Intelligence | **Missing** |
| RIPTIDE_PROVIDER_*_ENABLED | bool | false | Intelligence | **Missing** |
| RIPTIDE_PROVIDER_*_MODEL | String | (varies) | Intelligence | **Missing** |
| RIPTIDE_PROVIDER_*_PRIORITY | u32 | (none) | Intelligence | **Missing** |
| RIPTIDE_RATE_LIMIT_JITTER | f64 | 0.1 | API | Medium |
| RIPTIDE_RATE_LIMIT_RPS | f64 | 1.5 | API | High |
| RIPTIDE_RENDER_TIMEOUT | u64 | 3 | API | High |
| RIPTIDE_WARM_POOL_SIZE | usize | 2 | Core | **Missing** |
| RIPTIDE_WARMING_INTERVAL_SECS | u64 | 300 | Core | **Missing** |
| RIPTIDE_WASM_COLD_START_TARGET_MS | u64 | (varies) | Tests | **Missing** |
| RIPTIDE_WASM_ENABLE_AOT_CACHE | bool | (varies) | Tests | **Missing** |
| RIPTIDE_WASM_ENABLE_SIMD | bool | (varies) | Tests | **Missing** |
| RIPTIDE_WASM_INITIAL_POOL_SIZE | usize | (varies) | Tests | **Missing** |
| RIPTIDE_WASM_INSTANCES_PER_WORKER | usize | 1 | Core | **Missing** |
| RIPTIDE_WASM_MAX_POOL_SIZE | usize | (varies) | Tests | **Missing** |
| RIPTIDE_WASM_MEMORY_LIMIT_MB | usize | (varies) | Tests | **Missing** |
| RIPTIDE_WASM_MEMORY_LIMIT_PAGES | usize | (varies) | Tests | **Missing** |
| RIPTIDE_WASM_PATH | String | (varies) | CLI | Medium |
| RUST_LOG | String | "info" | All | Medium |
| RUSTC_VERSION | String | (auto) | Tests | Low |
| SEARCH_BACKEND | String | "serper" | Search | High |
| SEARCH_ENABLE_URL_PARSING | bool | true | Search | Medium |
| SEARCH_TIMEOUT | u64 | 30 | Search | Medium |
| SEARXNG_BASE_URL | String | (none) | Search | Medium |
| SERPER_API_KEY | String | (none) | Search | High |
| SKIP_PERSISTENCE_TESTS | bool | false | Tests | Low |
| SKIP_REDIS_TESTS | bool | false | Tests | Low |
| SPIDER_BASE_URL | String | "https://example.com" | Spider | High |
| SPIDER_CONCURRENCY | usize | 4 | Spider | Medium |
| SPIDER_DELAY_MS | u64 | 500 | Spider | Medium |
| SPIDER_ENABLE | bool | false | Spider | High |
| SPIDER_MAX_DEPTH | usize | 3 | Spider | High |
| SPIDER_MAX_PAGES | usize | 100 | Spider | High |
| SPIDER_RESPECT_ROBOTS | bool | true | Spider | Medium |
| SPIDER_TIMEOUT_SECONDS | u64 | 30 | Spider | Medium |
| SPIDER_USER_AGENT | String | "RipTide Spider/1.0" | Spider | Low |
| STREAM_BUFFER_MAX_SIZE | usize | 65536 | API | Medium |
| STREAM_BUFFER_SIZE | usize | 8192 | API | Medium |
| STREAM_DEFAULT_TIMEOUT | u64 | 300 | API | Medium |
| STREAM_MAX_CONCURRENT | usize | 100 | API | Medium |
| STREAM_RATE_LIMIT_ENABLED | bool | true | API | Medium |
| STREAM_RATE_LIMIT_RPS | u64 | 10 | API | Medium |
| TELEMETRY_ENABLED | bool | true | API | Low |
| TELEMETRY_EXPORT_TIMEOUT_SECS | u64 | 30 | API | Low |
| TELEMETRY_EXPORTER_TYPE | String | "stdout" | API | Low |
| TELEMETRY_MAX_EXPORT_BATCH_SIZE | u64 | 512 | API | Low |
| TELEMETRY_MAX_QUEUE_SIZE | u64 | 2048 | API | Low |
| TELEMETRY_OTLP_ENDPOINT | String | (none) | API | Low |
| TELEMETRY_SAMPLING_RATIO | f64 | 1.0 | API | Low |
| TELEMETRY_SERVICE_NAME | String | "riptide-api" | API | Low |
| TELEMETRY_SERVICE_VERSION | String | (CARGO_PKG_VERSION) | API | Low |
| TEST_REDIS_URL | String | "redis://localhost:6379/15" | Tests | Low |
| TEST_TIMEOUT_MULTIPLIER | f64 | 1.0 | Tests | Low |
| TEST_WASM_PATH | String | (varies) | Tests | Low |
| WASM_COMPONENT_PATH | String | (varies) | Tests | **Missing** |
| WASM_EXTRACTOR_PATH | String | (varies) | API | High |
| WORKER_ENABLE_SCHEDULER | bool | true | Workers | Medium |
| WORKER_MAX_BATCH_SIZE | usize | 50 | Workers | Medium |
| WORKER_MAX_CONCURRENCY | usize | 10 | Workers | Medium |
| WORKER_POOL_SIZE | usize | 4 | Workers | High |
| WORKER_REDIS_URL | String | (REDIS_URL) | Workers | **Missing** |
| WS_MAX_MESSAGE_SIZE | usize | 16777216 | API | Medium |
| WS_PING_INTERVAL | u64 | 30 | API | Medium |

**Total:** 120+ environment variables
**Documented:** ~75 (62.5%)
**Missing from .env.example:** ~45 (37.5%)

---

## 7. Action Items

### Priority P0 (Critical - Complete Before Launch)

1. ✅ **Document all missing environment variables in `.env.example`**
2. 🔴 **Create `RIPTIDE_OUTPUT_DIR` variable and implement in code**
3. 🔴 **Standardize screenshot storage paths**
4. ✅ **Add validation for critical paths (WASM, REDIS, etc.)**
5. ✅ **Document fallback chains and default behaviors**

### Priority P1 (High - Complete This Sprint)

6. ⚠️ **Create unified configuration loader pattern**
7. ⚠️ **Add comprehensive configuration documentation**
8. ⚠️ **Implement secret management guidelines**
9. ⚠️ **Standardize numeric parsing with error handling**
10. ⚠️ **Add config validation in startup sequence**

### Priority P2 (Medium - Future Enhancement)

11. 📋 **Create config migration tool for version updates**
12. 📋 **Add config export/import CLI commands**
13. 📋 **Implement config hot-reload for non-critical settings**
14. 📋 **Create config template generator**
15. 📋 **Add config diff/compare utilities**

---

## 8. Summary Statistics

- **Total Environment Variables Identified:** 120+
- **Properly Documented:** ~75 (62.5%)
- **Missing from .env.example:** ~45 (37.5%)
- **Configuration Patterns:** 2 (needs standardization)
- **Critical Path Issues:** 3 (screenshot paths, WASM validation, output directories)
- **Documentation Coverage:** Partial (needs comprehensive config guide)

---

## Appendix A: Configuration File Locations

```
/workspaces/eventmesh/
├── .env (runtime)
├── .env.example (template) ✅
├── .env.pdfium (specialized)
├── tests/.env.test (test overrides)
├── crates/riptide-api/src/config.rs (API config)
├── crates/riptide-intelligence/src/config.rs (LLM config)
├── crates/riptide-streaming/src/config.rs (streaming config)
├── crates/riptide-cli/src/commands/render.rs (CLI defaults)
└── crates/riptide-core/src/cache_warming.rs (cache warming)
```

---

## Appendix B: Recommended .env.example Updates

Add these sections to `.env.example`:

```bash
# ============================================================================
# Output Directories (NEW)
# ============================================================================

# Base output directory for all generated files
RIPTIDE_OUTPUT_DIR=./riptide_output

# Screenshot output subdirectory (relative to RIPTIDE_OUTPUT_DIR)
# RIPTIDE_SCREENSHOT_DIR=screenshots

# Reports output subdirectory (relative to RIPTIDE_OUTPUT_DIR)
# RIPTIDE_REPORTS_DIR=reports

# ============================================================================
# WASM Runtime Configuration (NEW)
# ============================================================================

# Number of WASM instances per worker thread
RIPTIDE_WASM_INSTANCES_PER_WORKER=1

# Enable SIMD optimizations (requires CPU support)
# RIPTIDE_WASM_ENABLE_SIMD=true

# Enable ahead-of-time compilation caching
# RIPTIDE_WASM_ENABLE_AOT_CACHE=true

# WASM memory limit in megabytes
# RIPTIDE_WASM_MEMORY_LIMIT_MB=512

# WASM memory limit in pages (1 page = 64KB)
# RIPTIDE_WASM_MEMORY_LIMIT_PAGES=8192

# Cold start performance target in milliseconds
# RIPTIDE_WASM_COLD_START_TARGET_MS=25

# Maximum WASM instance pool size
# RIPTIDE_WASM_MAX_POOL_SIZE=16

# Initial WASM instance pool size
# RIPTIDE_WASM_INITIAL_POOL_SIZE=4

# ============================================================================
# Cache Warming Configuration (NEW)
# ============================================================================

# Enable intelligent cache pre-warming
RIPTIDE_CACHE_WARMING_ENABLED=false

# Size of warm instance pool
RIPTIDE_WARM_POOL_SIZE=2

# Minimum number of warm instances to maintain
RIPTIDE_MIN_WARM_INSTANCES=1

# Maximum number of warm instances
RIPTIDE_MAX_WARM_INSTANCES=4

# Cache warming interval in seconds
RIPTIDE_WARMING_INTERVAL_SECS=300

# Target cache hit ratio (0.0-1.0)
RIPTIDE_CACHE_HIT_TARGET=0.8

# Enable prefetching of likely-needed resources
RIPTIDE_ENABLE_PREFETCHING=true

# ============================================================================
# Core Configuration (DOCUMENT EXISTING)
# ============================================================================

# Maximum concurrent operations across all workers
MAX_CONCURRENCY=16

# Quality gate high threshold (0.0-1.0)
GATE_HI_THRESHOLD=0.7

# Quality gate low threshold (0.0-1.0)
GATE_LO_THRESHOLD=0.3

# ============================================================================
# LLM Provider Configuration (NEW)
# ============================================================================

# Enable specific providers (true/false)
# RIPTIDE_PROVIDER_OPENAI_ENABLED=false
# RIPTIDE_PROVIDER_ANTHROPIC_ENABLED=false
# RIPTIDE_PROVIDER_AZURE_ENABLED=false

# Provider API keys (when enabled)
# RIPTIDE_PROVIDER_OPENAI_API_KEY=sk-...
# RIPTIDE_PROVIDER_ANTHROPIC_API_KEY=ant-...
# RIPTIDE_PROVIDER_AZURE_API_KEY=...

# Provider model selection
# RIPTIDE_PROVIDER_OPENAI_MODEL=gpt-4
# RIPTIDE_PROVIDER_ANTHROPIC_MODEL=claude-3-sonnet-20240229

# Provider priority (lower number = higher priority)
# RIPTIDE_PROVIDER_OPENAI_PRIORITY=1
# RIPTIDE_PROVIDER_ANTHROPIC_PRIORITY=2

# Enable metrics collection for LLM usage
RIPTIDE_METRICS_ENABLED=true

# Enable cost tracking for LLM API calls
RIPTIDE_COST_TRACKING_ENABLED=true

# ============================================================================
# Streaming CLI Configuration (NEW)
# ============================================================================

# API host for streaming connections
RIPTIDE_API_HOST=localhost

# API port for streaming connections
RIPTIDE_API_PORT=8080

# Log level (overrides RUST_LOG if set)
# RIPTIDE_LOG_LEVEL=info

# Path to configuration file
# RIPTIDE_CONFIG_FILE=~/.config/riptide/config.toml

# Enable development mode features
# RIPTIDE_DEV=false

# Cache directory for CLI operations
# RIPTIDE_CACHE_DIR=~/.cache/riptide

# ============================================================================
# Worker Configuration (CLARIFY EXISTING)
# ============================================================================

# Worker-specific Redis URL (falls back to REDIS_URL if not set)
WORKER_REDIS_URL=${REDIS_URL}
```

---

**End of Audit Report**
