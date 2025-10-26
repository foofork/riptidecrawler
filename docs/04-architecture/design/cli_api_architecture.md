# CLI → API Migration Architecture Design

**Version:** 1.0
**Status:** Design Phase
**Date:** 2025-10-17
**Author:** System Architecture Team

---

## Executive Summary

This document outlines the comprehensive architecture for migrating the RipTide CLI from direct library usage to an API-first approach with intelligent fallback. The hybrid architecture enables:

- **Centralized resource management** (browser pool, rate limiting, memory)
- **API-first execution** with automatic fallback to direct mode
- **Backward compatibility** for offline/development scenarios
- **Standardized output directory** structure across all operations
- **Zero breaking changes** for existing users

---

## Table of Contents

1. [Current Architecture](#1-current-architecture)
2. [Target Hybrid Architecture](#2-target-hybrid-architecture)
3. [Execution Mode Decision Logic](#3-execution-mode-decision-logic)
4. [Resource Management Strategy](#4-resource-management-strategy)
5. [API Client Implementation Pattern](#5-api-client-implementation-pattern)
6. [Backward Compatibility](#6-backward-compatibility)
7. [Output Directory Standardization](#7-output-directory-standardization)
8. [Component Interactions](#8-component-interactions)
9. [Implementation Phases](#9-implementation-phases)
10. [Testing Strategy](#10-testing-strategy)
11. [Migration Guide](#11-migration-guide)

---

## 1. Current Architecture

### Current State (v1.0)

```
┌─────────────────────────────────────────────────┐
│              RipTide CLI                        │
│  (riptide-cli)                                  │
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌──────────────────────────────────────────┐  │
│  │  Direct Library Dependencies             │  │
│  │                                          │  │
│  │  • riptide-headless (browser pool)      │  │
│  │  • riptide-extraction (WASM)            │  │
│  │  • riptide-stealth                      │  │
│  │  • riptide-pdf                          │  │
│  │  • Direct HTTP client                   │  │
│  └──────────────────────────────────────────┘  │
│                                                 │
│  Issues:                                        │
│  • Duplicate browser instances (CLI + API)      │
│  • No centralized rate limiting                 │
│  • No memory pressure handling                  │
│  • Resource exhaustion risks                    │
└─────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│          RipTide API Server                     │
│  (riptide-api)                                  │
├─────────────────────────────────────────────────┤
│                                                 │
│  • Browser pool (3 max instances)               │
│  • Rate limiter (1.5 RPS)                       │
│  • Memory manager (2GB global limit)            │
│  • Resource guards                              │
│  • Performance monitoring                       │
└─────────────────────────────────────────────────┘
```

### Problems Identified

1. **Resource Duplication**
   - CLI creates its own browser instances
   - API server manages separate browser pool
   - Total memory footprint: 2x-3x higher than needed

2. **No Coordination**
   - Rate limiting only applies to API requests
   - CLI bypasses all resource controls
   - Memory pressure not communicated

3. **Inconsistent Behavior**
   - Different timeout handling (CLI: 30s, API: 3s configurable)
   - Different stealth implementations
   - Different output formats

---

## 2. Target Hybrid Architecture

### Architectural Principles

1. **API-First with Intelligent Fallback**
   - Primary: CLI → HTTP → API Server → riptide-headless
   - Fallback: CLI → riptide-headless (direct)

2. **Single Source of Truth**
   - API server owns ALL resource management
   - CLI becomes a thin client in API mode
   - Configuration cascades: env vars → CLI flags → defaults

3. **Zero Breaking Changes**
   - Existing CLI commands work unchanged
   - Backward compatibility via direct mode
   - Graceful degradation when API unavailable

### High-Level Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                        RipTide CLI (Hybrid Mode)               │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │           Execution Mode Selector                        │ │
│  │                                                          │ │
│  │  if RIPTIDE_API_URL set:                                │ │
│  │    1. Try API request                                   │ │
│  │    2. If fails/timeout → fallback to direct mode        │ │
│  │  else:                                                   │ │
│  │    Direct mode only                                     │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
│  ┌─────────────────────┐      ┌─────────────────────────────┐ │
│  │   API Client Mode   │      │   Direct Mode (Fallback)    │ │
│  ├─────────────────────┤      ├─────────────────────────────┤ │
│  │                     │      │                             │ │
│  │ • HTTP requests     │      │ • riptide-headless (local)  │ │
│  │ • JSON responses    │      │ • riptide-extraction (WASM) │ │
│  │ • Resource info     │      │ • Local browser instances   │ │
│  │ • Rate limit aware  │      │ • No coordination           │ │
│  │ • No local browser  │      │ • Offline capable           │ │
│  └─────────────────────┘      └─────────────────────────────┘ │
│          │                                   │                 │
│          │                                   │                 │
└──────────┼───────────────────────────────────┼─────────────────┘
           │                                   │
           ▼                                   ▼
    ┌──────────────┐                  ┌────────────────┐
    │   HTTP/REST  │                  │  Direct Call   │
    └──────────────┘                  └────────────────┘
           │                                   │
           ▼                                   ▼
┌────────────────────────────────────────────────────────────────┐
│                     RipTide API Server                         │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │             Resource Manager (Centralized)               │ │
│  │                                                          │ │
│  │  • Browser Pool: 3 max instances (reused)               │ │
│  │  • Rate Limiter: 1.5 RPS global                         │ │
│  │  • Memory Monitor: 2GB limit                            │ │
│  │  • Performance Tracker                                  │ │
│  │  • Timeout Controller: 3s default                       │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                    Core Services                         │ │
│  │                                                          │ │
│  │  • riptide-headless (shared pool)                       │ │
│  │  • riptide-extraction (WASM)                            │ │
│  │  • riptide-stealth                                      │ │
│  │  • riptide-pdf                                          │ │
│  └──────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
```

---

## 3. Execution Mode Decision Logic

### Configuration Priority (Highest to Lowest)

1. **CLI Flags** (explicit user intent)
   ```bash
   riptide render --url https://example.com --direct  # Force direct mode
   riptide render --url https://example.com --api-url http://custom:8080  # Override API URL
   ```

2. **Environment Variables** (persistent config)
   ```bash
   export RIPTIDE_API_URL=http://localhost:8080
   export RIPTIDE_API_KEY=secret_key_123
   export RIPTIDE_EXECUTION_MODE=api-first  # or "direct-only"
   ```

3. **Configuration File** (project settings)
   ```yaml
   # ~/.config/riptide/config.yml
   api:
     url: http://localhost:8080
     key: ${RIPTIDE_API_KEY}
     enabled: true
   execution:
     mode: api-first  # api-first | direct-only | auto
     fallback_on_error: true
     timeout_ms: 5000
   ```

4. **Defaults**
   - No API URL → Direct mode only
   - API URL set → API-first with fallback

### Decision Flow

```
┌─────────────────────────────────────────────────┐
│         Start CLI Command Execution            │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
        ┌────────────────────┐
        │ Check --direct flag│
        └────────┬───────────┘
                 │
         ┌───────▼────────┐
         │ Flag present?  │
         └───┬────────┬───┘
             │        │
            Yes       No
             │        │
             ▼        ▼
        ┌─────────┐  ┌──────────────────────┐
        │ Direct  │  │ Check RIPTIDE_API_URL│
        │  Mode   │  └──────┬───────────────┘
        └─────────┘         │
                            ▼
                   ┌────────────────┐
                   │  URL set?      │
                   └─┬─────────┬────┘
                     │         │
                    Yes        No
                     │         │
                     ▼         ▼
            ┌────────────┐  ┌─────────┐
            │ API-First  │  │ Direct  │
            │   Mode     │  │  Mode   │
            └──────┬─────┘  └─────────┘
                   │
                   ▼
        ┌──────────────────────┐
        │  Try API Request     │
        └──────┬───────────────┘
               │
               ▼
        ┌──────────────┐
        │   Success?   │
        └─┬──────────┬─┘
          │          │
         Yes         No
          │          │
          ▼          ▼
     ┌────────┐  ┌─────────────────┐
     │ Return │  │ Fallback enabled?│
     │ Result │  └────┬──────────┬──┘
     └────────┘       │          │
                     Yes         No
                      │          │
                      ▼          ▼
              ┌─────────────┐  ┌────────┐
              │Execute      │  │ Return │
              │Direct Mode  │  │ Error  │
              └─────────────┘  └────────┘
```

### Implementation (Rust)

```rust
// crates/riptide-cli/src/execution.rs

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    /// API-first with fallback to direct on failure
    ApiFallback,
    /// API-only, fail if API unavailable
    ApiOnly,
    /// Direct mode only (no API)
    DirectOnly,
    /// Auto-detect based on environment
    Auto,
}

#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub mode: ExecutionMode,
    pub api_url: Option<String>,
    pub api_key: Option<String>,
    pub fallback_enabled: bool,
    pub api_timeout_ms: u64,
}

impl ExecutionConfig {
    /// Load configuration from environment, CLI args, and config file
    pub fn load(cli_args: &CliArgs) -> Result<Self> {
        // Priority 1: CLI flags
        if cli_args.direct {
            return Ok(Self {
                mode: ExecutionMode::DirectOnly,
                api_url: None,
                api_key: None,
                fallback_enabled: false,
                api_timeout_ms: 5000,
            });
        }

        // Priority 2: Explicit API URL from CLI
        if let Some(api_url) = &cli_args.api_url {
            return Ok(Self {
                mode: ExecutionMode::ApiFallback,
                api_url: Some(api_url.clone()),
                api_key: cli_args.api_key.clone(),
                fallback_enabled: cli_args.no_fallback.map(|v| !v).unwrap_or(true),
                api_timeout_ms: cli_args.api_timeout_ms.unwrap_or(5000),
            });
        }

        // Priority 3: Environment variables
        if let Ok(api_url) = std::env::var("RIPTIDE_API_URL") {
            let api_key = std::env::var("RIPTIDE_API_KEY").ok();
            let fallback_enabled = std::env::var("RIPTIDE_FALLBACK_ENABLED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true);

            return Ok(Self {
                mode: ExecutionMode::ApiFallback,
                api_url: Some(api_url),
                api_key,
                fallback_enabled,
                api_timeout_ms: 5000,
            });
        }

        // Priority 4: Configuration file
        if let Ok(config) = ConfigFile::load() {
            if let Some(api_config) = config.api {
                if api_config.enabled {
                    return Ok(Self {
                        mode: ExecutionMode::ApiFallback,
                        api_url: Some(api_config.url),
                        api_key: api_config.key,
                        fallback_enabled: api_config.fallback_on_error,
                        api_timeout_ms: api_config.timeout_ms,
                    });
                }
            }
        }

        // Default: Direct mode only
        Ok(Self {
            mode: ExecutionMode::DirectOnly,
            api_url: None,
            api_key: None,
            fallback_enabled: false,
            api_timeout_ms: 5000,
        })
    }
}

/// Execute with automatic mode selection
pub async fn execute_with_mode<T, F, Fut>(
    config: &ExecutionConfig,
    api_fn: F,
    direct_fn: impl FnOnce() -> Fut,
) -> Result<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    match &config.mode {
        ExecutionMode::DirectOnly => {
            tracing::info!("Executing in direct mode (API disabled)");
            direct_fn().await
        }
        ExecutionMode::ApiOnly => {
            tracing::info!("Executing in API-only mode");
            api_fn().await.context("API execution failed and fallback is disabled")
        }
        ExecutionMode::ApiFallback => {
            tracing::info!("Attempting API execution with fallback enabled");
            match api_fn().await {
                Ok(result) => {
                    tracing::info!("API execution succeeded");
                    Ok(result)
                }
                Err(e) if config.fallback_enabled => {
                    tracing::warn!(
                        error = %e,
                        "API execution failed, falling back to direct mode"
                    );
                    direct_fn().await
                }
                Err(e) => {
                    tracing::error!(error = %e, "API execution failed and fallback is disabled");
                    Err(e)
                }
            }
        }
        ExecutionMode::Auto => {
            if config.api_url.is_some() {
                execute_with_mode(
                    &ExecutionConfig {
                        mode: ExecutionMode::ApiFallback,
                        ..config.clone()
                    },
                    api_fn,
                    direct_fn,
                )
                .await
            } else {
                direct_fn().await
            }
        }
    }
}
```

---

## 4. Resource Management Strategy

### Current API Server Resource Management

The API server already implements comprehensive resource management:

```rust
// From: crates/riptide-api/src/handlers/render/handlers.rs

pub async fn render(
    State(state): State<AppState>,
    session_ctx: SessionContext,
    Json(body): Json<RenderRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Resource acquisition with multiple protection layers
    let resource_guard = match state
        .resource_manager
        .acquire_render_resources(&body.url)
        .await
    {
        Ok(ResourceResult::Success(guard)) => guard,
        Ok(ResourceResult::Timeout) => return Err(ApiError::timeout(...)),
        Ok(ResourceResult::ResourceExhausted) => return Err(ApiError::service_unavailable(...)),
        Ok(ResourceResult::RateLimited { retry_after }) => return Err(ApiError::rate_limited(...)),
        Ok(ResourceResult::MemoryPressure) => return Err(ApiError::service_unavailable(...)),
        // ... error handling
    };

    // Resource guard ensures cleanup on drop
    // ...
}
```

### Resource Types and Limits

| Resource | Limit | Location | Purpose |
|----------|-------|----------|---------|
| **Browser Instances** | 3 max | API Server | Prevent memory exhaustion |
| **Request Rate** | 1.5 RPS | API Server | Prevent resource thrashing |
| **Global Memory** | 2GB | API Server | System stability |
| **Render Timeout** | 3s (configurable) | API Server | Prevent hung requests |
| **WASM Instances** | 10 max | API Server | Memory efficiency |

### CLI Resource Behavior

#### API Mode (Recommended)
```
CLI Command → HTTP Request → API Server → Resource Manager
                                            ↓
                                    ┌───────────────────┐
                                    │ Resource Guard    │
                                    ├───────────────────┤
                                    │ • Browser Pool    │
                                    │ • Rate Limiter    │
                                    │ • Memory Check    │
                                    │ • Timeout Control │
                                    └───────────────────┘
                                            ↓
                                    Shared Resources
                                    (1 browser pool for all clients)
```

**Benefits:**
- No duplicate browser instances
- Centralized rate limiting
- Memory pressure awareness
- Performance metrics collection

#### Direct Mode (Fallback)
```
CLI Command → Direct Library Call → Local Resources
                                     ↓
                              ┌─────────────────┐
                              │ Local Instance  │
                              ├─────────────────┤
                              │ • New browser   │
                              │ • No rate limit │
                              │ • No coordination│
                              │ • Local timeout │
                              └─────────────────┘
```

**Use Cases:**
- Offline development
- API server unavailable
- Testing/debugging
- Bypassing rate limits (dev only)

### Resource Coordination Protocol

```rust
// API Client sends resource-aware requests

#[derive(Serialize)]
struct ApiRenderRequest {
    url: String,
    mode: RenderMode,

    // Resource hints
    priority: RequestPriority,  // low, medium, high, critical
    timeout_override: Option<u64>,
    bypass_rate_limit: bool,  // Requires special permission

    // Resource preferences
    prefer_cached: bool,
    max_wait_time: Option<u64>,
}

// API Server responds with resource information

#[derive(Deserialize)]
struct ApiRenderResponse {
    url: String,
    content: String,

    // Resource metadata
    resource_info: ResourceInfo {
        browser_pool_utilization: f32,  // 0.0 - 1.0
        rate_limit_remaining: u32,
        memory_pressure: MemoryPressure,  // Low, Medium, High
        queue_depth: u32,
    },

    // Performance metrics
    metrics: RenderMetrics {
        api_overhead_ms: u64,
        browser_time_ms: u64,
        queue_wait_ms: u64,
        total_time_ms: u64,
    },
}
```

---

## 5. API Client Implementation Pattern

### Command Execution Pattern

Each CLI command follows this pattern:

```rust
// crates/riptide-cli/src/commands/render.rs

pub async fn execute(args: RenderArgs, output_format: &str) -> Result<()> {
    // 1. Load execution configuration
    let exec_config = ExecutionConfig::load_from_env_and_args(&args)?;

    // 2. Execute with mode selection
    let result = execute_with_mode(
        &exec_config,
        || execute_via_api(&args),      // API function
        || execute_direct(&args),        // Direct function
    ).await?;

    // 3. Handle output (same for both modes)
    output_result(&result, output_format)?;

    Ok(())
}

/// Execute via API server
async fn execute_via_api(args: &RenderArgs) -> Result<RenderResult> {
    let client = RipTideClient::new()?;

    // Build API request
    let request = ApiRenderRequest {
        url: args.url.clone(),
        mode: parse_render_mode(&args)?,
        wait_condition: parse_wait_condition(&args.wait)?,
        screenshot: parse_screenshot_mode(&args.screenshot)?,
        stealth_level: parse_stealth_level(&args.stealth)?,
        session_id: args.session.clone(),

        // Output options
        capture_html: args.html,
        capture_dom: args.dom,
        capture_pdf: args.pdf,
        capture_har: args.har,

        // Resource hints
        priority: RequestPriority::Medium,
        timeout_override: args.timeout,
    };

    // Send request with timeout
    let api_timeout = Duration::from_secs(args.timeout.unwrap_or(30));
    let response = tokio::time::timeout(
        api_timeout,
        client.post("/api/v1/render")
            .json(&request)
            .send()
    ).await??;

    // Handle API response
    if response.status().is_success() {
        let api_result: ApiRenderResponse = response.json().await?;

        // Save artifacts to local filesystem
        save_artifacts_from_api(&api_result, args).await?;

        Ok(RenderResult::from_api(api_result))
    } else {
        let error_body = response.text().await?;
        Err(anyhow::anyhow!("API request failed: {}", error_body))
    }
}

/// Execute directly (fallback/offline mode)
async fn execute_direct(args: &RenderArgs) -> Result<RenderResult> {
    // Current implementation - unchanged
    // Uses local riptide-headless, riptide-extraction, etc.

    let launcher = HeadlessLauncher::with_config(...).await?;
    let session = launcher.launch_page(&args.url, ...).await?;

    // ... existing direct execution logic ...

    Ok(RenderResult { ... })
}

/// Save API-returned artifacts to local filesystem
async fn save_artifacts_from_api(
    api_result: &ApiRenderResponse,
    args: &RenderArgs,
) -> Result<()> {
    let output_dir = get_output_directory(args)?;
    let prefix = generate_file_prefix(&args.url);

    // Save HTML if requested
    if args.html && let Some(ref html) = api_result.artifacts.html {
        let path = output_dir.join("html").join(format!("{}.html", prefix));
        fs::write(&path, html).await?;
        println!("HTML saved to: {}", path.display());
    }

    // Save screenshot if returned
    if let Some(ref screenshot_base64) = api_result.artifacts.screenshot {
        let screenshot_bytes = base64::decode(screenshot_base64)?;
        let path = output_dir.join("screenshots").join(format!("{}.png", prefix));
        fs::write(&path, screenshot_bytes).await?;
        println!("Screenshot saved to: {}", path.display());
    }

    // Save PDF if requested
    if args.pdf && let Some(ref pdf_base64) = api_result.artifacts.pdf {
        let pdf_bytes = base64::decode(pdf_base64)?;
        let path = output_dir.join("pdf").join(format!("{}.pdf", prefix));
        fs::write(&path, pdf_bytes).await?;
        println!("PDF saved to: {}", path.display());
    }

    // Save DOM if requested
    if args.dom && let Some(ref dom_json) = api_result.artifacts.dom {
        let path = output_dir.join("dom").join(format!("{}.json", prefix));
        fs::write(&path, dom_json).await?;
        println!("DOM saved to: {}", path.display());
    }

    Ok(())
}
```

### Client Configuration

```rust
// crates/riptide-cli/src/client.rs

pub struct RipTideClient {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl RipTideClient {
    pub fn new() -> Result<Self> {
        let base_url = std::env::var("RIPTIDE_API_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        let api_key = std::env::var("RIPTIDE_API_KEY").ok();

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(10)
            .build()?;

        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut builder = self.client.post(&url);

        if let Some(ref key) = self.api_key {
            builder = builder.header("X-API-Key", key);
        }

        builder
    }

    pub async fn health_check(&self) -> Result<bool> {
        let response = tokio::time::timeout(
            Duration::from_secs(2),
            self.client.get(&format!("{}/healthz", self.base_url)).send()
        ).await??;

        Ok(response.status().is_success())
    }
}
```

---

## 6. Backward Compatibility

### Compatibility Matrix

| Scenario | v1.0 (Current) | v2.0 (Hybrid) | Breaking? |
|----------|---------------|---------------|-----------|
| **Offline usage** | ✅ Works | ✅ Works (direct mode) | ❌ No |
| **No API URL set** | ✅ Works | ✅ Works (direct mode) | ❌ No |
| **API URL set** | ⚠️ Ignored | ✅ Uses API + fallback | ❌ No |
| **Existing scripts** | ✅ Works | ✅ Works unchanged | ❌ No |
| **Output paths** | ✅ Works | ✅ Enhanced (standardized) | ❌ No |
| **CLI flags** | ✅ Works | ✅ Works + new flags | ❌ No |

### Migration Path

#### Phase 1: Add API Support (No Breaking Changes)
- Add `--api-url` and `--api-key` flags
- Add `RIPTIDE_API_URL` environment variable support
- API mode is **opt-in** via explicit flag or env var
- Default behavior unchanged (direct mode)

#### Phase 2: Enable API-First (Opt-In)
- Add `RIPTIDE_EXECUTION_MODE=api-first` option
- Still requires `RIPTIDE_API_URL` to be set
- Fallback to direct mode on API failure
- Users must explicitly opt-in

#### Phase 3: API-First Default (Future)
- If API server is running locally, use it by default
- Auto-detect via health check on localhost:8080
- Still maintain direct mode as fallback
- Clear messaging about mode being used

### Deprecation Timeline

| Version | Change | Impact |
|---------|--------|--------|
| **v2.0.0** | Add API client support (opt-in) | ✅ No breaking changes |
| **v2.1.0** | Enable API-first mode (opt-in) | ✅ No breaking changes |
| **v2.2.0** | Deprecate duplicate implementations | ⚠️ Warnings only |
| **v3.0.0** | API-first becomes default | ⚠️ Breaking (with fallback) |

### Backward Compatibility Guarantees

```rust
// Compatibility layer

/// Ensures v1.0 commands work unchanged in v2.0
pub mod compat {
    /// Old-style command execution (direct mode only)
    pub async fn execute_v1_render(args: RenderArgs) -> Result<()> {
        // Force direct mode for v1 compatibility
        let exec_config = ExecutionConfig {
            mode: ExecutionMode::DirectOnly,
            ..Default::default()
        };

        execute_with_config(args, exec_config).await
    }

    /// Detect if running in compatibility mode
    pub fn is_compat_mode() -> bool {
        std::env::var("RIPTIDE_COMPAT_MODE")
            .map(|v| v == "v1" || v == "true")
            .unwrap_or(false)
    }
}
```

---

## 7. Output Directory Standardization

### Standardized Directory Structure

```
$RIPTIDE_OUTPUT_DIR/  (default: ~/.local/share/riptide/output or platform-specific)
├── screenshots/           # PNG images from render command
│   ├── example_com.png
│   ├── github_com_repo.png
│   └── .gitignore
├── html/                  # Extracted/rendered HTML
│   ├── example_com.html
│   ├── github_com_repo.html
│   └── .gitignore
├── pdf/                   # Generated PDFs
│   ├── example_com.pdf
│   ├── invoice_123.pdf
│   └── .gitignore
├── dom/                   # DOM JSON trees
│   ├── example_com.json
│   ├── github_com_repo.json
│   └── .gitignore
├── reports/               # Streaming reports and crawl data
│   ├── crawl_20251017_143022.json
│   ├── deepsearch_results.ndjson
│   └── .gitignore
├── artifacts/             # Test artifacts and debug output
│   ├── har/              # HTTP Archive files
│   │   └── example_com.har
│   ├── traces/           # Performance traces
│   │   └── render_trace_123.json
│   ├── cookies/          # Cookie jars
│   │   └── session_abc.json
│   └── storage/          # localStorage/sessionStorage
│       └── session_abc_storage.json
├── temp/                  # Temporary files (auto-cleanup)
│   └── .gitkeep
└── .riptide_meta.json    # Metadata and index
```

### Output Directory Resolution

```rust
// crates/riptide-cli/src/config.rs

use std::path::PathBuf;

/// Get output directory with priority: CLI arg > env var > default
pub fn get_output_directory(args: &RenderArgs) -> PathBuf {
    // Priority 1: CLI argument
    if let Some(ref dir) = args.output_dir {
        return PathBuf::from(dir);
    }

    // Priority 2: Environment variable
    if let Ok(dir) = std::env::var("RIPTIDE_OUTPUT_DIR") {
        return PathBuf::from(dir);
    }

    // Priority 3: Platform-specific default
    get_default_output_directory()
}

/// Get platform-specific default output directory
pub fn get_default_output_directory() -> PathBuf {
    if let Some(data_dir) = dirs::data_dir() {
        data_dir.join("riptide").join("output")
    } else {
        // Fallback to current directory
        PathBuf::from("./riptide_output")
    }
}

/// Ensure output subdirectory exists
pub fn ensure_output_subdir(subdir: &str) -> Result<PathBuf> {
    let base = get_default_output_directory();
    let path = base.join(subdir);

    fs::create_dir_all(&path)?;

    // Create .gitignore in subdirectory
    let gitignore_path = path.join(".gitignore");
    if !gitignore_path.exists() {
        fs::write(&gitignore_path, "*\n!.gitignore\n")?;
    }

    Ok(path)
}

/// Map file type to subdirectory
pub fn get_output_subdir_for_type(file_type: &str) -> &'static str {
    match file_type {
        "screenshot" | "png" => "screenshots",
        "html" => "html",
        "pdf" => "pdf",
        "dom" | "json" => "dom",
        "har" => "artifacts/har",
        "trace" => "artifacts/traces",
        "report" | "crawl" => "reports",
        "cookie" => "artifacts/cookies",
        "storage" => "artifacts/storage",
        _ => "temp",
    }
}
```

### Artifact Metadata Tracking

```rust
// .riptide_meta.json structure

{
  "version": "2.0.0",
  "output_root": "/home/user/.local/share/riptide/output",
  "artifacts": [
    {
      "id": "render_example_com_20251017_143022",
      "type": "render",
      "command": "riptide render --url https://example.com --html --screenshot",
      "timestamp": "2025-10-17T14:30:22Z",
      "files": [
        {
          "path": "html/example_com.html",
          "size_bytes": 45621,
          "checksum": "sha256:abc123..."
        },
        {
          "path": "screenshots/example_com.png",
          "size_bytes": 128456,
          "checksum": "sha256:def456..."
        }
      ],
      "execution_mode": "api",
      "api_endpoint": "http://localhost:8080/api/v1/render",
      "duration_ms": 1234
    }
  ],
  "cleanup_policy": {
    "temp_retention_hours": 24,
    "max_total_size_mb": 10240,
    "auto_cleanup_enabled": true
  }
}
```

---

## 8. Component Interactions

### Sequence Diagram: API-First Execution

```
┌─────┐          ┌─────┐          ┌──────────┐          ┌──────────────┐
│ User│          │ CLI │          │API Client│          │  API Server  │
└──┬──┘          └──┬──┘          └────┬─────┘          └──────┬───────┘
   │                │                  │                        │
   │ riptide render │                  │                        │
   │  --url=...     │                  │                        │
   ├───────────────>│                  │                        │
   │                │                  │                        │
   │                │ Load Config      │                        │
   │                ├─────────────────>│                        │
   │                │                  │                        │
   │                │ Check API URL?   │                        │
   │                │<─────────────────┤                        │
   │                │                  │                        │
   │                │ Health Check     │                        │
   │                ├─────────────────>│                        │
   │                │                  │                        │
   │                │                  │ GET /healthz           │
   │                │                  ├───────────────────────>│
   │                │                  │                        │
   │                │                  │ 200 OK                 │
   │                │                  │<───────────────────────┤
   │                │                  │                        │
   │                │ API Available    │                        │
   │                │<─────────────────┤                        │
   │                │                  │                        │
   │                │ Build Request    │                        │
   │                ├─────────────────>│                        │
   │                │                  │                        │
   │                │                  │ POST /api/v1/render    │
   │                │                  │ {url, mode, ...}       │
   │                │                  ├───────────────────────>│
   │                │                  │                        │
   │                │                  │ Acquire Resources      │
   │                │                  │ (browser, rate limit)  │
   │                │                  │<───────────────────────┤
   │                │                  │                        │
   │                │                  │ Execute Render         │
   │                │                  │<───────────────────────┤
   │                │                  │                        │
   │                │                  │ 200 OK                 │
   │                │                  │ {content, artifacts}   │
   │                │                  │<───────────────────────┤
   │                │                  │                        │
   │                │ API Response     │                        │
   │                │<─────────────────┤                        │
   │                │                  │                        │
   │                │ Save Artifacts   │                        │
   │                │ to Local FS      │                        │
   │                ├─────────────────>│                        │
   │                │                  │                        │
   │                │ Display Results  │                        │
   │                │<─────────────────┤                        │
   │                │                  │                        │
   │ Result Output  │                  │                        │
   │<───────────────┤                  │                        │
   │                │                  │                        │
```

### Sequence Diagram: Fallback to Direct Mode

```
┌─────┐          ┌─────┐          ┌──────────┐          ┌──────────────┐
│ User│          │ CLI │          │API Client│          │  API Server  │
└──┬──┘          └──┬──┘          └────┬─────┘          └──────┬───────┘
   │                │                  │                        │
   │ riptide render │                  │                        │
   │  --url=...     │                  │                        │
   ├───────────────>│                  │                        │
   │                │                  │                        │
   │                │ Load Config      │                        │
   │                ├─────────────────>│                        │
   │                │                  │                        │
   │                │ Check API URL?   │                        │
   │                │<─────────────────┤                        │
   │                │                  │                        │
   │                │ Health Check     │                        │
   │                ├─────────────────>│                        │
   │                │                  │                        │
   │                │                  │ GET /healthz           │
   │                │                  ├─────────────────X      │
   │                │                  │                        │
   │                │                  │ Connection Timeout     │
   │                │                  │<───────────────────────┤
   │                │                  │                        │
   │                │ API Unavailable  │                        │
   │                │<─────────────────┤                        │
   │                │                  │                        │
   │                │ Fallback Enabled?│                        │
   │                ├─────────────────>│                        │
   │                │                  │                        │
   │                │ Yes, Execute     │                        │
   │                │ Direct Mode      │                        │
   │                │<─────────────────┤                        │
   │                │                  │                        │
   │                │ Init Local       │                        │
   │                │ Browser          │                        │
   │                ├─────────────────────────────────────────────┐
   │                │                                             │
   │                │ Launch Headless Browser (Local)            │
   │                │<────────────────────────────────────────────┘
   │                │                  │                        │
   │                │ Execute Render   │                        │
   │                │ (Direct)         │                        │
   │                ├─────────────────────────────────────────────┐
   │                │                                             │
   │                │ Render Page, Extract Content, Save Files   │
   │                │<────────────────────────────────────────────┘
   │                │                  │                        │
   │                │ Display Results  │                        │
   │                │ (⚠️ Fallback Used)│                        │
   │                │<─────────────────┤                        │
   │                │                  │                        │
   │ Result Output  │                  │                        │
   │ + Warning      │                  │                        │
   │<───────────────┤                  │                        │
   │                │                  │                        │
```

### Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         INPUT LAYER                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐              │
│  │ CLI Flags  │  │  Env Vars  │  │Config File │              │
│  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘              │
│        │               │               │                      │
│        └───────────────┴───────────────┘                      │
│                        │                                       │
│                        ▼                                       │
│              ┌─────────────────────┐                          │
│              │  Execution Config   │                          │
│              │  (Merged Settings)  │                          │
│              └──────────┬──────────┘                          │
│                         │                                       │
└─────────────────────────┼───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                      DECISION LAYER                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│                  ┌───────────────┐                             │
│                  │  Mode Selector│                             │
│                  └───────┬───────┘                             │
│                          │                                       │
│              ┌───────────┴───────────┐                         │
│              │                       │                         │
│              ▼                       ▼                         │
│      ┌───────────────┐       ┌──────────────┐                 │
│      │  API Mode     │       │ Direct Mode  │                 │
│      │  (Primary)    │       │ (Fallback)   │                 │
│      └───────┬───────┘       └──────┬───────┘                 │
│              │                       │                         │
└──────────────┼───────────────────────┼─────────────────────────┘
               │                       │
               ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                     EXECUTION LAYER                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────┐       ┌──────────────────────────┐   │
│  │   API Client        │       │   Direct Executor        │   │
│  ├─────────────────────┤       ├──────────────────────────┤   │
│  │ • HTTP Request      │       │ • Local Browser Launch   │   │
│  │ • JSON Payload      │       │ • WASM Extraction        │   │
│  │ • Auth Headers      │       │ • Local File I/O         │   │
│  │ • Timeout Control   │       │ • No Coordination        │   │
│  └──────────┬──────────┘       └─────────┬────────────────┘   │
│             │                            │                     │
│             ▼                            ▼                     │
│  ┌─────────────────────┐       ┌──────────────────────────┐   │
│  │  API Server         │       │  Local Resources         │   │
│  ├─────────────────────┤       ├──────────────────────────┤   │
│  │ • Resource Manager  │       │  • riptide-headless      │   │
│  │ • Browser Pool (3)  │       │  • riptide-extraction    │   │
│  │ • Rate Limiter      │       │  • riptide-stealth       │   │
│  │ • Memory Monitor    │       │  • Local browser process │   │
│  └──────────┬──────────┘       └─────────┬────────────────┘   │
│             │                            │                     │
└─────────────┼────────────────────────────┼─────────────────────┘
              │                            │
              ▼                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                      OUTPUT LAYER                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │           Standardized Output Directory                 │   │
│  │                                                         │   │
│  │  $RIPTIDE_OUTPUT_DIR/                                  │   │
│  │    ├── screenshots/                                     │   │
│  │    ├── html/                                            │   │
│  │    ├── pdf/                                             │   │
│  │    ├── dom/                                             │   │
│  │    ├── reports/                                         │   │
│  │    ├── artifacts/                                       │   │
│  │    └── temp/                                            │   │
│  │                                                         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              User-Facing Output                         │   │
│  │                                                         │   │
│  │  • Console output (text/json/table)                    │   │
│  │  • File paths                                           │   │
│  │  • Performance metrics                                  │   │
│  │  • Resource utilization                                 │   │
│  │  • Error messages                                       │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 9. Implementation Phases

### Phase 0: Preparation (P0 - Week 1)

**Goal:** Foundation and analysis

- [ ] Audit current CLI commands and identify API candidates
- [ ] Document existing resource usage patterns
- [ ] Create feature flag system for gradual rollout
- [ ] Set up integration test infrastructure

**Deliverables:**
- Command audit spreadsheet
- Resource usage baseline metrics
- Feature flag configuration
- Test plan document

---

### Phase 1: API Client Foundation (P0 - Week 2-3)

**Goal:** Basic API client with health checks

**Tasks:**
- [ ] Implement `ExecutionConfig` with priority resolution
- [ ] Create `RipTideClient` with health check support
- [ ] Add `--api-url` and `--api-key` CLI flags
- [ ] Implement `RIPTIDE_API_URL` environment variable support
- [ ] Add connection timeout and retry logic

**Implementation:**
```rust
// crates/riptide-cli/src/client.rs
pub struct RipTideClient { ... }

impl RipTideClient {
    pub fn new() -> Result<Self> { ... }
    pub async fn health_check(&self) -> Result<bool> { ... }
    pub fn post(&self, path: &str) -> RequestBuilder { ... }
}
```

**Testing:**
- Unit tests for config resolution
- Integration tests for health checks
- Error handling tests (timeout, connection refused)

**Success Criteria:**
- CLI can detect and connect to API server
- Health check completes in <500ms
- Graceful fallback when API unavailable

---

### Phase 2: Render Command Migration (P0 - Week 4-5)

**Goal:** Migrate `riptide render` to API-first

**Tasks:**
- [ ] Implement `execute_via_api()` for render command
- [ ] Add artifact download and local file saving
- [ ] Maintain `execute_direct()` as fallback
- [ ] Add `--direct` flag to force local execution
- [ ] Implement automatic fallback on API failure

**API Request/Response:**
```rust
#[derive(Serialize)]
struct ApiRenderRequest {
    url: String,
    mode: RenderMode,
    wait_condition: WaitCondition,
    screenshot: ScreenshotMode,
    // ... all render options
}

#[derive(Deserialize)]
struct ApiRenderResponse {
    url: String,
    content: String,
    artifacts: RenderArtifacts,
    resource_info: ResourceInfo,
    metrics: RenderMetrics,
}
```

**Testing:**
- API mode: Full render with screenshot/HTML/DOM/PDF
- Direct mode: Same tests for fallback
- Fallback trigger: Kill API server mid-request
- Output verification: Check all artifacts saved correctly

**Success Criteria:**
- API mode produces identical output to direct mode
- Fallback activates within 2 seconds of API failure
- All artifacts downloaded and saved to correct paths
- Resource metadata displayed in output

---

### Phase 3: Extract & Crawl Commands (P1 - Week 6-7)

**Goal:** Migrate extraction and crawling to API

**Tasks:**
- [ ] Migrate `riptide extract` command
- [ ] Migrate `riptide crawl` command
- [ ] Add streaming support for crawl results
- [ ] Implement progress reporting from API
- [ ] Add batch extraction support

**API Endpoints:**
```
POST /api/v1/extract
POST /api/v1/crawl
GET  /api/v1/crawl/stream  (Server-Sent Events)
```

**Testing:**
- Extract: Various extraction methods (wasm, css, llm)
- Crawl: Multi-page crawls with depth limits
- Streaming: Real-time progress updates
- Error handling: Network interruptions

**Success Criteria:**
- Extraction produces same results via API
- Crawl progress streams in real-time
- Fallback handles partial failures gracefully

---

### Phase 4: Output Directory Standardization (P1 - Week 8)

**Goal:** Unified output structure

**Tasks:**
- [ ] Implement `get_output_directory()` with priority resolution
- [ ] Create standardized subdirectories
- [ ] Add `.gitignore` files to each subdirectory
- [ ] Implement artifact metadata tracking
- [ ] Add cleanup utilities for temp files

**Directory Structure:**
```
$RIPTIDE_OUTPUT_DIR/
  ├── screenshots/
  ├── html/
  ├── pdf/
  ├── dom/
  ├── reports/
  ├── artifacts/
  ├── temp/
  └── .riptide_meta.json
```

**Testing:**
- Output directory creation on all platforms
- File organization for each command
- Metadata generation and updates
- Cleanup of old temp files

**Success Criteria:**
- All outputs organized in standard structure
- Platform-specific defaults work correctly
- Metadata accurately tracks all artifacts

---

### Phase 5: Session & Job Commands (P1 - Week 9)

**Goal:** Session and job management via API

**Tasks:**
- [ ] Migrate `riptide session` commands
- [ ] Migrate `riptide job` commands
- [ ] Remove `riptide job-local` (use direct mode instead)
- [ ] Add session state synchronization
- [ ] Implement job result streaming

**API Integration:**
- Session commands → `/sessions/*` endpoints
- Job commands → `/workers/jobs/*` endpoints

**Testing:**
- Session creation and management
- Job submission and status polling
- Result retrieval and artifact download

---

### Phase 6: Advanced Features (P2 - Week 10-11)

**Goal:** Full feature parity with API

**Tasks:**
- [ ] Migrate PDF, Stealth, Domain, Schema commands
- [ ] Add metrics streaming from API
- [ ] Implement cache warming via API
- [ ] Add validation and system check via API
- [ ] Implement performance profiling endpoints

**Commands:**
```bash
riptide pdf ...         # → /pdf/*
riptide stealth ...     # → /stealth/*
riptide domain ...      # → /domain/*
riptide schema ...      # → /api/v1/tables/schema
riptide metrics tail    # → /metrics (streaming)
riptide cache warm ...  # → /admin/cache/warm
```

**Testing:**
- End-to-end tests for all commands
- Performance comparison (API vs Direct)
- Resource usage validation

---

### Phase 7: Documentation & Migration Tools (P2 - Week 12)

**Goal:** User-facing documentation and migration support

**Tasks:**
- [ ] Write migration guide
- [ ] Create API vs Direct mode comparison docs
- [ ] Add troubleshooting guide
- [ ] Implement `riptide migrate-config` command
- [ ] Add `riptide doctor` diagnostic tool

**Documentation:**
- Migration guide (this document)
- API endpoint reference
- Configuration reference
- Troubleshooting FAQ

**Migration Tools:**
```bash
# Analyze current setup and suggest migration path
riptide doctor

# Migrate v1 config to v2
riptide migrate-config --from v1 --to v2

# Test API connectivity
riptide test-api --url http://localhost:8080
```

---

## 10. Testing Strategy

### Test Categories

#### 1. Unit Tests

**Scope:** Individual functions and components

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_config_priority() {
        // Test CLI flags override env vars
        let args = CliArgs {
            direct: true,
            api_url: None,
            ..Default::default()
        };

        std::env::set_var("RIPTIDE_API_URL", "http://localhost:8080");

        let config = ExecutionConfig::load(&args).unwrap();
        assert_eq!(config.mode, ExecutionMode::DirectOnly);
    }

    #[test]
    fn test_output_directory_resolution() {
        // Test output dir priority
        let args = RenderArgs {
            output_dir: Some("/custom/path".to_string()),
            ..Default::default()
        };

        let dir = get_output_directory(&args);
        assert_eq!(dir, PathBuf::from("/custom/path"));
    }
}
```

#### 2. Integration Tests

**Scope:** CLI ↔ API interactions

```rust
#[tokio::test]
async fn test_render_api_fallback() {
    // Start API server
    let api = spawn_test_api_server().await;

    // Set API URL
    std::env::set_var("RIPTIDE_API_URL", api.url());

    // Execute render command
    let result = execute_render(RenderArgs {
        url: "https://example.com".to_string(),
        html: true,
        ..Default::default()
    }).await;

    assert!(result.is_ok());
    assert!(result.unwrap().used_api);

    // Kill API server
    api.shutdown().await;

    // Execute should fallback to direct mode
    let result = execute_render(RenderArgs {
        url: "https://example.com".to_string(),
        html: true,
        ..Default::default()
    }).await;

    assert!(result.is_ok());
    assert!(!result.unwrap().used_api);
}
```

#### 3. End-to-End Tests

**Scope:** Full CLI workflows

```bash
#!/bin/bash
# E2E test script

# Test 1: API mode with all features
export RIPTIDE_API_URL=http://localhost:8080
riptide render --url https://example.com \
  --html --screenshot full --dom --pdf \
  --stealth high \
  --output-dir /tmp/riptide-test

# Verify outputs
test -f /tmp/riptide-test/html/example_com.html || exit 1
test -f /tmp/riptide-test/screenshots/example_com.png || exit 1
test -f /tmp/riptide-test/dom/example_com.json || exit 1
test -f /tmp/riptide-test/pdf/example_com.pdf || exit 1

# Test 2: Direct mode fallback
unset RIPTIDE_API_URL
riptide render --url https://example.com --html

# Should succeed with direct mode
test $? -eq 0 || exit 1
```

#### 4. Performance Tests

**Benchmark API overhead**

```rust
#[tokio::test]
async fn bench_api_vs_direct() {
    // Benchmark API mode
    let start = Instant::now();
    execute_via_api(&render_args).await.unwrap();
    let api_time = start.elapsed();

    // Benchmark direct mode
    let start = Instant::now();
    execute_direct(&render_args).await.unwrap();
    let direct_time = start.elapsed();

    // API should have <100ms overhead
    let overhead = api_time.saturating_sub(direct_time);
    assert!(overhead < Duration::from_millis(100),
        "API overhead too high: {:?}", overhead);
}
```

#### 5. Resource Tests

**Verify resource management**

```rust
#[tokio::test]
async fn test_browser_pool_coordination() {
    let api = spawn_test_api_server().await;

    // Spawn 5 concurrent render requests
    let handles: Vec<_> = (0..5).map(|i| {
        tokio::spawn(execute_via_api(RenderArgs {
            url: format!("https://example{}.com", i),
            ..Default::default()
        }))
    }).collect();

    // Wait for all
    let results = join_all(handles).await;

    // Check browser pool stayed within limits
    let pool_stats = api.get_browser_pool_stats().await;
    assert!(pool_stats.max_instances_used <= 3,
        "Browser pool exceeded limit: {}", pool_stats.max_instances_used);

    // All should succeed (some queued)
    assert!(results.iter().all(|r| r.is_ok()));
}
```

### Test Coverage Goals

| Component | Target Coverage |
|-----------|----------------|
| Execution mode selection | 95% |
| API client | 90% |
| Command implementations | 85% |
| Output handling | 90% |
| Error handling | 95% |
| Resource management | 90% |

---

## 11. Migration Guide

### For Users

#### Quick Start (v2.0)

**No changes required!** Your existing commands work unchanged.

```bash
# v1.0 command (still works in v2.0)
riptide render --url https://example.com --html

# Exactly the same behavior - uses direct mode by default
```

#### Opt-In to API Mode

```bash
# Option 1: Environment variable (recommended)
export RIPTIDE_API_URL=http://localhost:8080
riptide render --url https://example.com --html

# Option 2: CLI flag (per-command)
riptide render --url https://example.com --html \
  --api-url http://localhost:8080

# Option 3: Config file (~/.config/riptide/config.yml)
api:
  url: http://localhost:8080
  enabled: true
```

#### Verify Your Setup

```bash
# Check if API server is reachable
riptide health --api-url http://localhost:8080

# Test API mode
export RIPTIDE_API_URL=http://localhost:8080
riptide render --url https://example.com --html -v

# Output should show: "Executing via API server: http://localhost:8080"
```

#### Troubleshooting

```bash
# Force direct mode (bypass API)
riptide render --url https://example.com --direct

# Check execution mode
riptide doctor

# Output:
# ✓ API URL: http://localhost:8080
# ✓ API reachable: Yes
# ✓ Execution mode: API-first with fallback
# ✓ Output directory: /home/user/.local/share/riptide/output
```

### For Developers

#### Update Your Scripts

**Old (v1.0):**
```bash
#!/bin/bash
riptide render --url "$URL" --html --output-dir ./output
```

**New (v2.0) - Recommended:**
```bash
#!/bin/bash
# Set API URL for coordinated execution
export RIPTIDE_API_URL=${RIPTIDE_API_URL:-http://localhost:8080}

# Same command, but now uses API if available
riptide render --url "$URL" --html --output-dir ./output

# Fallback to direct mode on API failure (automatic)
```

#### CI/CD Integration

**GitHub Actions Example:**

```yaml
name: Web Scraping

on: [push]

jobs:
  scrape:
    runs-on: ubuntu-latest

    services:
      riptide-api:
        image: riptide/api:latest
        ports:
          - 8080:8080
        env:
          REDIS_URL: redis://redis:6379

    steps:
      - uses: actions/checkout@v3

      - name: Install RipTide CLI
        run: cargo install riptide-cli

      - name: Configure API
        run: |
          echo "RIPTIDE_API_URL=http://localhost:8080" >> $GITHUB_ENV

      - name: Run scraping
        run: |
          riptide render --url https://example.com --html

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: scrape-results
          path: ~/.local/share/riptide/output/
```

#### Docker Compose Setup

```yaml
version: '3.8'

services:
  riptide-api:
    image: riptide/api:latest
    ports:
      - "8080:8080"
    environment:
      - REDIS_URL=redis://redis:6379
      - WASM_PATH=/opt/riptide/extractor.wasm
    volumes:
      - browser-data:/opt/riptide/browser

  redis:
    image: redis:7-alpine

  cli-runner:
    image: riptide/cli:latest
    environment:
      - RIPTIDE_API_URL=http://riptide-api:8080
    volumes:
      - ./scripts:/scripts
      - ./output:/output
    command: /scripts/run-scraping.sh

volumes:
  browser-data:
```

---

## Appendix A: Configuration Reference

### Environment Variables

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `RIPTIDE_API_URL` | API server base URL | None | `http://localhost:8080` |
| `RIPTIDE_API_KEY` | API authentication key | None | `secret_key_123` |
| `RIPTIDE_EXECUTION_MODE` | Execution mode | `auto` | `api-first`, `direct-only` |
| `RIPTIDE_FALLBACK_ENABLED` | Enable fallback to direct mode | `true` | `true`, `false` |
| `RIPTIDE_OUTPUT_DIR` | Output directory path | Platform-specific | `/custom/output` |
| `RIPTIDE_API_TIMEOUT_MS` | API request timeout | `5000` | `10000` |

### CLI Flags

| Flag | Description | Commands |
|------|-------------|----------|
| `--api-url <URL>` | Override API server URL | All |
| `--api-key <KEY>` | Override API key | All |
| `--direct` | Force direct mode (skip API) | All |
| `--no-fallback` | Disable fallback to direct mode | All |
| `--output-dir <PATH>` | Override output directory | render, crawl, extract |

### Configuration File

**Location:** `~/.config/riptide/config.yml`

```yaml
# API Configuration
api:
  url: http://localhost:8080
  key: ${RIPTIDE_API_KEY}  # Read from environment
  enabled: true
  fallback_on_error: true
  timeout_ms: 5000

# Execution Configuration
execution:
  mode: api-first  # auto, api-first, api-only, direct-only

# Output Configuration
output:
  base_dir: ~/.local/share/riptide/output
  subdirs:
    screenshots: screenshots
    html: html
    pdf: pdf
    dom: dom
    reports: reports
  cleanup:
    temp_retention_hours: 24
    max_total_size_mb: 10240
    auto_cleanup_enabled: true

# Resource Preferences
resources:
  prefer_api_for:
    - render
    - crawl
    - extract
  always_direct:
    - validate
    - system-check
```

---

## Appendix B: Error Codes and Handling

### API Error Codes

| Code | Meaning | CLI Behavior |
|------|---------|--------------|
| `200` | Success | Return result |
| `400` | Bad Request | Show error, exit 1 |
| `401` | Unauthorized | Check API key, exit 1 |
| `429` | Rate Limited | Show retry time, fallback or exit |
| `500` | Server Error | Fallback to direct mode |
| `503` | Service Unavailable | Fallback to direct mode |
| `504` | Gateway Timeout | Fallback to direct mode |

### Fallback Triggers

| Condition | Action |
|-----------|--------|
| Connection refused | Immediate fallback |
| Connection timeout (>2s) | Immediate fallback |
| HTTP 5xx error | Fallback after 1 retry |
| Rate limit (429) | Fallback if `--no-wait` set |
| Resource exhausted (503) | Fallback or queue |

---

## Appendix C: Performance Benchmarks

### Expected Performance

| Metric | API Mode | Direct Mode | Overhead |
|--------|----------|-------------|----------|
| Simple render | 1.2s | 1.1s | +100ms |
| Complex render | 3.5s | 3.4s | +100ms |
| Extraction | 0.5s | 0.4s | +100ms |
| Crawl (10 pages) | 8.2s | 8.5s | -300ms* |

*Crawl is faster via API due to browser pool reuse

### Resource Utilization

| Scenario | API Mode | Direct Mode | Savings |
|----------|----------|-------------|---------|
| 10 concurrent renders | 1 browser pool (3 instances) | 10 browser instances | 70% memory |
| Memory footprint | 600MB (shared) | 2GB (10x200MB) | 70% reduction |
| CPU usage | 40% (coordinated) | 80% (parallel) | 50% reduction |

---

## Conclusion

This hybrid architecture provides:

1. **✅ Zero Breaking Changes** - Existing commands work unchanged
2. **✅ Opt-In Migration** - Users choose when to adopt API mode
3. **✅ Resource Efficiency** - 70% memory savings via API mode
4. **✅ Intelligent Fallback** - Automatic direct mode on API failure
5. **✅ Standardized Output** - Unified directory structure
6. **✅ Future-Proof** - Foundation for advanced features

The implementation phases are designed for incremental delivery with minimal risk. Users can migrate at their own pace, and developers benefit from centralized resource management immediately upon adoption.

---

**Next Steps:**
1. Review and approve architecture design
2. Begin Phase 0 preparation tasks
3. Set up feature flags and testing infrastructure
4. Start Phase 1 implementation (API client foundation)

---

*Document Version: 1.0*
*Last Updated: 2025-10-17*
*Status: Ready for Review*
