# RipTide Persistence Layer Integration Architecture

**Author**: System Architecture Designer
**Date**: 2025-10-10
**Status**: Design Phase
**Version**: 1.0.0

## Executive Summary

This document outlines the comprehensive architecture for integrating the `riptide-persistence` crate into the `riptide-api` service. The integration replaces direct Redis usage with a sophisticated multi-tenant caching layer featuring advanced capabilities including cache warming, hot config reload, and distributed state management.

### Key Benefits

1. **<5ms Cache Access**: High-performance cache with sub-5ms SLA
2. **Multi-Tenancy**: Complete tenant isolation with resource quotas
3. **State Management**: Hot reload, checkpointing, and graceful shutdown
4. **Cache Warming**: Intelligent pre-warming for top URLs
5. **Spillover Management**: Automatic disk spillover for memory pressure
6. **Distributed Coordination**: Multi-instance synchronization

## Current State Analysis

### Existing Redis Usage

**Location**: `/workspaces/eventmesh/crates/riptide-api/src/state.rs`

```rust
// Current implementation
pub struct AppState {
    pub cache: Arc<tokio::sync::Mutex<CacheManager>>,
    // ... other fields
}

// CacheManager from riptide-core/cache
impl CacheManager {
    pub async fn new(redis_url: &str) -> Result<Self>;
    pub async fn get(&mut self, key: &str) -> Result<Option<String>>;
    pub async fn set(&mut self, key: &str, value: &str, ttl: u64) -> Result<()>;
}
```

**Pain Points**:
- Direct Redis client usage without abstraction
- No multi-tenancy support
- Manual TTL management
- No cache warming
- No state management for hot reload
- Single-tenant architecture

## Target Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      riptide-api                             │
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │              AppState (Refactored)                  │    │
│  │                                                     │    │
│  │  ┌─────────────────────────────────────────────┐  │    │
│  │  │     PersistentCacheManager                   │  │    │
│  │  │  - TTL-based invalidation                    │  │    │
│  │  │  - Compression (LZ4/Zstd)                    │  │    │
│  │  │  - Batch operations                          │  │    │
│  │  │  - Cache warming                             │  │    │
│  │  │  - Performance: <5ms target                  │  │    │
│  │  └─────────────────────────────────────────────┘  │    │
│  │                                                     │    │
│  │  ┌─────────────────────────────────────────────┐  │    │
│  │  │          TenantManager                       │  │    │
│  │  │  - Tenant isolation                          │  │    │
│  │  │  - Resource quotas                           │  │    │
│  │  │  - Billing tracking                          │  │    │
│  │  │  - Access policies                           │  │    │
│  │  └─────────────────────────────────────────────┘  │    │
│  │                                                     │    │
│  │  ┌─────────────────────────────────────────────┐  │    │
│  │  │          StateManager                        │  │    │
│  │  │  - Hot config reload                         │  │    │
│  │  │  - Checkpointing                             │  │    │
│  │  │  - Session management                        │  │    │
│  │  │  - Graceful shutdown                         │  │    │
│  │  └─────────────────────────────────────────────┘  │    │
│  │                                                     │    │
│  └────────────────────────────────────────────────────┘    │
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │           TenantMiddleware                          │    │
│  │  - Extract tenant ID from header                   │    │
│  │  - Validate tenant access                          │    │
│  │  - Enforce quotas                                  │    │
│  │  - Record usage                                    │    │
│  └────────────────────────────────────────────────────┘    │
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │           Admin Handlers                            │    │
│  │  - /admin/tenants (CRUD)                           │    │
│  │  - /admin/cache/* (management)                     │    │
│  │  - /admin/state/* (hot reload)                     │    │
│  └────────────────────────────────────────────────────┘    │
│                                                              │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
                ┌──────────────────────┐
                │   Redis/DragonflyDB   │
                │  - Connection pool    │
                │  - Persistence layer  │
                └──────────────────────┘
```

### Data Flow

#### 1. Request Flow with Multi-Tenancy

```
User Request (X-Tenant-ID: tenant-123)
    │
    ▼
┌─────────────────────┐
│ TenantMiddleware    │
│ - Extract tenant ID │
│ - Validate access   │
│ - Check quotas      │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────────┐
│ Handler (search/extract)│
│ - Process with tenant   │
└──────────┬──────────────┘
           │
           ▼
┌─────────────────────────────┐
│ PersistentCacheManager      │
│ Key: tenant:123:url:hash    │
│ - Tenant isolation          │
│ - TTL enforcement           │
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────┐
│ Redis/DragonflyDB   │
└─────────────────────┘
```

#### 2. Cache Warming Flow

```
Application Startup
    │
    ▼
┌─────────────────────────┐
│ CacheWarmer             │
│ - Load top 1000 URLs    │
│ - Batch prefetch        │
└──────────┬──────────────┘
           │
           ▼
┌─────────────────────────────┐
│ PersistentCacheManager      │
│ - Warm cache entries        │
│ - Track hit rate            │
└─────────────────────────────┘
```

#### 3. Hot Config Reload Flow

```
Config File Change (config/riptide.yml)
    │
    ▼
┌─────────────────────────┐
│ HotReloadWatcher        │
│ - Detect file change    │
└──────────┬──────────────┘
           │
           ▼
┌─────────────────────────────┐
│ StateManager                │
│ - Parse new config          │
│ - Validate changes          │
│ - Apply atomically          │
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────────┐
│ AppState Updated        │
│ - No restart required   │
└─────────────────────────┘
```

## Architecture Decision Records (ADRs)

### ADR-001: Replace Direct Redis Usage with PersistentCacheManager

**Status**: Approved
**Date**: 2025-10-10
**Context**: Current `CacheManager` from `riptide-core` provides basic Redis operations but lacks advanced features like multi-tenancy, cache warming, and performance monitoring.

**Decision**: Integrate `riptide-persistence::PersistentCacheManager` to provide:
- Sub-5ms cache access SLA
- TTL-based automatic invalidation
- Compression for large entries (LZ4/Zstd)
- Batch operations for efficiency
- Cache warming capabilities
- Comprehensive metrics

**Consequences**:
- **Positive**: Performance improvement, better observability, reduced memory usage
- **Negative**: Breaking change to AppState, requires migration
- **Mitigation**: Provide compatibility layer and migration guide

**Alternatives Considered**:
1. **Keep existing CacheManager**: Rejected due to lack of features
2. **Build features into riptide-core**: Rejected to maintain separation of concerns
3. **Use third-party caching library**: Rejected due to tight integration needs

### ADR-002: Implement Multi-Tenancy at Persistence Layer

**Status**: Approved
**Date**: 2025-10-10
**Context**: Need to support multiple tenants with isolation, quotas, and billing.

**Decision**: Use `riptide-persistence::TenantManager` for:
- Complete data isolation via namespaced keys
- Per-tenant resource quotas (memory, operations, storage)
- Billing tracking and usage monitoring
- Security boundaries and access policies

**Consequences**:
- **Positive**: Production-ready multi-tenancy, monetization ready
- **Negative**: Adds complexity to request flow
- **Mitigation**: Middleware handles tenant context transparently

**Key Design Choices**:
- Tenant ID in header: `X-Tenant-ID`
- Cache key format: `tenant:{id}:{namespace}:{key}`
- Default quotas: 100MB memory, 1000 ops/min
- Billing: Per-operation tracking with monthly aggregation

### ADR-003: Add StateManager for Configuration Management

**Status**: Approved
**Date**: 2025-10-10
**Context**: Need hot config reload, checkpointing, and graceful shutdown without service restarts.

**Decision**: Integrate `riptide-persistence::StateManager` for:
- Hot reload of configuration files
- Checkpoint/restore for state preservation
- Session management with disk spillover
- Graceful shutdown with state preservation

**Consequences**:
- **Positive**: Zero-downtime config updates, better reliability
- **Negative**: Additional complexity in initialization
- **Mitigation**: Comprehensive testing and monitoring

**Configuration Hot Reload**:
- Watch: `./config/*.yml`
- Validation before apply
- Atomic config updates
- Notification to consumers

### ADR-004: Cache Warming Strategy

**Status**: Approved
**Date**: 2025-10-10
**Context**: Cold start performance is critical for production workloads.

**Decision**: Implement three-tier cache warming:
1. **Startup Warm**: Top 1000 URLs from access logs
2. **On-Demand**: Admin endpoint `/admin/cache/warm`
3. **Background**: Every 6 hours based on analytics

**Consequences**:
- **Positive**: 85%+ cache hit rate target
- **Negative**: Increased startup time (5-10 seconds)
- **Mitigation**: Async warming, non-blocking

**Warming Sources**:
```rust
pub struct CacheWarmingSource {
    /// Top URLs from analytics
    pub analytics: Vec<String>,
    /// Tenant-specific frequently accessed
    pub tenant_patterns: HashMap<String, Vec<String>>,
    /// Manual warm list
    pub manual: Vec<String>,
}
```

## Component Specifications

### 1. Refactored AppState

**Location**: `/workspaces/eventmesh/crates/riptide-api/src/state.rs`

```rust
use riptide_persistence::{
    PersistentCacheManager, TenantManager, StateManager,
    PersistenceConfig, CacheConfig, TenantConfig, StateConfig
};

#[derive(Clone)]
pub struct AppState {
    // REPLACED: Direct Redis CacheManager
    // OLD: pub cache: Arc<tokio::sync::Mutex<CacheManager>>,

    // NEW: Advanced persistence layer
    pub cache: Arc<PersistentCacheManager>,
    pub tenant_manager: Arc<TenantManager>,
    pub state_manager: Arc<StateManager>,

    // Existing fields remain
    pub http_client: Client,
    pub extractor: Arc<WasmExtractor>,
    pub reliable_extractor: Arc<ReliableExtractor>,
    pub config: AppConfig,
    pub api_config: ApiConfig,
    pub resource_manager: Arc<ResourceManager>,
    pub metrics: Arc<RipTideMetrics>,
    // ... rest unchanged
}

impl AppState {
    pub async fn new(
        config: AppConfig,
        metrics: Arc<RipTideMetrics>,
        health_checker: Arc<HealthChecker>,
    ) -> Result<Self> {
        // Initialize persistence configuration
        let persistence_config = PersistenceConfig {
            redis: RedisConfig {
                url: config.redis_url.clone(),
                pool_size: 10,
                connection_timeout_ms: 5000,
                command_timeout_ms: 5000,
                ..Default::default()
            },
            cache: CacheConfig {
                default_ttl_seconds: config.cache_ttl,
                enable_compression: true,
                compression_threshold_bytes: 1024,
                enable_warming: true,
                ..Default::default()
            },
            tenant: TenantConfig {
                enabled: true,
                isolation_level: TenantIsolationLevel::Strong,
                enable_billing: true,
                ..Default::default()
            },
            state: StateConfig {
                enable_hot_reload: true,
                config_watch_paths: vec!["./config".to_string()],
                checkpoint_interval_seconds: 300, // 5 minutes
                ..Default::default()
            },
            ..Default::default()
        };

        // Initialize persistence layer
        let cache = PersistentCacheManager::new(
            &config.redis_url,
            persistence_config.cache.clone()
        ).await?;

        let tenant_manager = TenantManager::new(
            &config.redis_url,
            persistence_config.tenant.clone(),
            Arc::new(RwLock::new(TenantMetrics::new()))
        ).await?;

        let state_manager = StateManager::new(
            &config.redis_url,
            persistence_config.state.clone()
        ).await?;

        // Enable cache warming
        let warmer = Arc::new(CacheWarmer::new(100));
        cache.enable_warming(warmer.clone());

        // Warm cache on startup
        let warm_keys = Self::get_top_urls_for_warming().await?;
        cache.warm_cache(warm_keys).await?;

        Ok(Self {
            cache: Arc::new(cache),
            tenant_manager: Arc::new(tenant_manager),
            state_manager: Arc::new(state_manager),
            // ... initialize other fields
        })
    }

    async fn get_top_urls_for_warming() -> Result<Vec<String>> {
        // Load from analytics or config
        // For MVP: hardcoded list or from env
        Ok(vec![])
    }
}
```

### 2. Tenant Middleware

**Location**: `/workspaces/eventmesh/crates/riptide-api/src/middleware/tenant.rs`

```rust
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use crate::state::AppState;
use crate::errors::ApiError;

/// Tenant context for request processing
#[derive(Clone, Debug)]
pub struct TenantContext {
    pub tenant_id: String,
    pub is_authenticated: bool,
}

/// Extract tenant ID from request headers
pub async fn tenant_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract tenant ID from header
    let tenant_id = request
        .headers()
        .get("X-Tenant-ID")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::validation("Missing X-Tenant-ID header"))?;

    // Validate tenant exists and is active
    let tenant = state.tenant_manager
        .get_tenant(tenant_id)
        .await
        .map_err(|e| ApiError::internal(format!("Tenant lookup failed: {}", e)))?
        .ok_or_else(|| ApiError::validation("Invalid tenant ID"))?;

    if !matches!(tenant.status, riptide_persistence::TenantStatus::Active) {
        return Err(ApiError::validation("Tenant is not active"));
    }

    // Check rate limits
    let resource = "operations_per_minute";
    state.tenant_manager
        .check_quota(tenant_id, resource, 1)
        .await
        .map_err(|e| ApiError::rate_limited(format!("Quota exceeded: {}", e)))?;

    // Add tenant context to request extensions
    let tenant_ctx = TenantContext {
        tenant_id: tenant_id.to_string(),
        is_authenticated: true,
    };
    request.extensions_mut().insert(tenant_ctx);

    // Process request
    let response = next.run(request).await;

    // Record usage
    let usage = riptide_persistence::ResourceUsageRecord {
        operation_count: 1,
        data_bytes: 0, // Updated by response middleware
        compute_time_ms: 0, // Updated by response middleware
        storage_bytes: 0,
        timestamp: chrono::Utc::now(),
    };

    let _ = state.tenant_manager
        .record_usage(tenant_id, "api_request", usage)
        .await;

    Ok(response)
}

/// Extract tenant context from request
pub fn get_tenant_context(request: &Request) -> Result<&TenantContext, ApiError> {
    request.extensions()
        .get::<TenantContext>()
        .ok_or_else(|| ApiError::internal("Tenant context not found"))
}
```

### 3. Admin Endpoints

**Location**: `/workspaces/eventmesh/crates/riptide-api/src/handlers/admin/mod.rs`

```rust
pub mod tenants;
pub mod cache;
pub mod state;

use axum::{Router, routing::{get, post, put, delete}};
use crate::state::AppState;

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        // Tenant management
        .route("/admin/tenants", get(tenants::list_tenants))
        .route("/admin/tenants", post(tenants::create_tenant))
        .route("/admin/tenants/:id", get(tenants::get_tenant))
        .route("/admin/tenants/:id", put(tenants::update_tenant))
        .route("/admin/tenants/:id", delete(tenants::delete_tenant))
        .route("/admin/tenants/:id/usage", get(tenants::get_tenant_usage))
        .route("/admin/tenants/:id/billing", get(tenants::get_tenant_billing))
        .route("/admin/tenants/:id/suspend", post(tenants::suspend_tenant))

        // Cache management
        .route("/admin/cache/stats", get(cache::get_cache_stats))
        .route("/admin/cache/warm", post(cache::warm_cache))
        .route("/admin/cache/invalidate", post(cache::invalidate_cache))
        .route("/admin/cache/clear", post(cache::clear_cache))

        // State management
        .route("/admin/state/current", get(state::get_current_state))
        .route("/admin/state/reload", post(state::reload_config))
        .route("/admin/state/checkpoint", post(state::create_checkpoint))
        .route("/admin/state/restore", post(state::restore_checkpoint))
}
```

**Location**: `/workspaces/eventmesh/crates/riptide-api/src/handlers/admin/tenants.rs`

```rust
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::errors::{ApiResult, ApiError};
use riptide_persistence::{TenantConfig, TenantOwner, BillingPlan};

#[derive(Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub owner_email: String,
    pub owner_name: String,
    pub billing_plan: String, // "free", "basic", "professional", "enterprise"
    pub quotas: Option<std::collections::HashMap<String, u64>>,
}

#[derive(Serialize)]
pub struct TenantResponse {
    pub tenant_id: String,
    pub name: String,
    pub status: String,
    pub created_at: String,
}

pub async fn create_tenant(
    State(state): State<AppState>,
    Json(req): Json<CreateTenantRequest>,
) -> ApiResult<Json<TenantResponse>> {
    // Parse billing plan
    let billing_plan = match req.billing_plan.as_str() {
        "free" => BillingPlan::Free,
        "basic" => BillingPlan::Basic,
        "professional" => BillingPlan::Professional,
        "enterprise" => BillingPlan::Enterprise,
        _ => return Err(ApiError::validation("Invalid billing plan")),
    };

    // Create tenant configuration
    let tenant_config = TenantConfig {
        tenant_id: String::new(), // Generated by manager
        name: req.name.clone(),
        quotas: req.quotas.unwrap_or_default(),
        isolation_level: riptide_persistence::TenantIsolationLevel::Strong,
        encryption_enabled: true,
        settings: std::collections::HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let owner = TenantOwner {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.owner_name,
        email: req.owner_email,
        organization: None,
    };

    // Create tenant
    let tenant_id = state.tenant_manager
        .create_tenant(tenant_config, owner, billing_plan)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to create tenant: {}", e)))?;

    Ok(Json(TenantResponse {
        tenant_id,
        name: req.name,
        status: "active".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

// Additional handlers: list_tenants, get_tenant, update_tenant, etc.
```

### 4. Cache Integration in Handlers

**Example: Search Handler**

**Location**: `/workspaces/eventmesh/crates/riptide-api/src/handlers/search.rs`

```rust
// OLD IMPLEMENTATION
pub async fn search_handler(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> ApiResult<Json<SearchResponse>> {
    let cache_key = format!("search:{}", req.query);

    // OLD: Direct cache access
    let mut cache = state.cache.lock().await;
    if let Some(cached) = cache.get(&cache_key).await? {
        return Ok(Json(serde_json::from_str(&cached)?));
    }

    // ... perform search
    cache.set(&cache_key, &result_json, 3600).await?;
}

// NEW IMPLEMENTATION
pub async fn search_handler(
    State(state): State<AppState>,
    request: Request,
    Json(req): Json<SearchRequest>,
) -> ApiResult<Json<SearchResponse>> {
    // Extract tenant context
    let tenant_ctx = crate::middleware::tenant::get_tenant_context(&request)?;

    // Tenant-scoped cache key
    let cache_key = format!("search:{}", req.query);

    // NEW: Tenant-aware cache access
    let cached: Option<SearchResponse> = state.cache
        .get(&cache_key, Some(&tenant_ctx.tenant_id))
        .await
        .map_err(|e| ApiError::cache(e.to_string()))?;

    if let Some(result) = cached {
        // Record cache hit in metrics
        state.metrics.record_cache_hit();
        return Ok(Json(result));
    }

    // Perform search with tenant context
    let result = perform_search(&state, &req, tenant_ctx).await?;

    // Store with tenant namespace and TTL
    state.cache
        .set(
            &cache_key,
            &result,
            Some(&tenant_ctx.tenant_id),
            Some(std::time::Duration::from_secs(3600)),
            None, // metadata
        )
        .await
        .map_err(|e| ApiError::cache(e.to_string()))?;

    state.metrics.record_cache_miss();
    Ok(Json(result))
}
```

## Migration Strategy

### Phase 1: Dependency Integration (Sprint 2B - Day 1-2)

**Tasks**:
1. Add `riptide-persistence` to Cargo.toml
2. Update workspace dependencies
3. Resolve any version conflicts
4. Build and ensure compilation

**Deliverables**:
- ✅ Clean build with new dependency
- ✅ No breaking changes to existing handlers

### Phase 2: AppState Refactoring (Sprint 2B - Day 3-5)

**Tasks**:
1. Create new AppState with persistence managers
2. Implement compatibility layer for existing handlers
3. Update initialization logic
4. Test with existing endpoints

**Deliverables**:
- ✅ Refactored AppState
- ✅ Backward compatibility maintained
- ✅ All existing tests passing

### Phase 3: Multi-Tenancy Implementation (Sprint 2B - Day 6-8)

**Tasks**:
1. Implement tenant middleware
2. Update cache keys with tenant namespacing
3. Add tenant validation and quota enforcement
4. Create admin endpoints for tenant management

**Deliverables**:
- ✅ Tenant middleware functional
- ✅ Multi-tenant cache isolation verified
- ✅ Admin API for tenant CRUD

### Phase 4: Cache Warming & State Management (Sprint 2B - Day 9-10)

**Tasks**:
1. Implement cache warming on startup
2. Add hot config reload
3. Implement checkpoint/restore
4. Add graceful shutdown

**Deliverables**:
- ✅ Cache warming active (>85% hit rate)
- ✅ Hot reload functional
- ✅ Checkpoint/restore tested

## Performance Targets

### Cache Performance

| Metric | Target | Current | Improvement |
|--------|--------|---------|-------------|
| Average access time | <5ms | ~15ms | 3x faster |
| p95 access time | <10ms | ~50ms | 5x faster |
| p99 access time | <20ms | ~100ms | 5x faster |
| Cache hit rate | >85% | ~60% | 25% increase |

### Multi-Tenancy Performance

| Metric | Target |
|--------|--------|
| Tenant isolation overhead | <1ms |
| Quota check latency | <2ms |
| Tenant creation time | <100ms |
| Max concurrent tenants | 1000+ |

## Security Considerations

### 1. Tenant Isolation

- **Data Isolation**: Separate namespace per tenant in Redis
- **Key Format**: `riptide:tenant:{id}:{namespace}:{key_hash}`
- **Validation**: Strict tenant ID validation on every request
- **Encryption**: Optional per-tenant encryption keys

### 2. Access Control

- **Authentication**: API key or JWT required
- **Authorization**: RBAC for admin endpoints
- **Quotas**: Hard limits prevent resource exhaustion
- **Rate Limiting**: Per-tenant rate limits

### 3. Audit Logging

```rust
pub struct AuditLog {
    pub timestamp: DateTime<Utc>,
    pub tenant_id: String,
    pub operation: String,
    pub resource: String,
    pub result: String,
    pub metadata: HashMap<String, String>,
}
```

## Monitoring & Observability

### Metrics

```rust
// Cache metrics
cache_operations_total{tenant_id, operation, result}
cache_access_duration_seconds{tenant_id, p50, p95, p99}
cache_hit_rate{tenant_id}
cache_memory_usage_bytes{tenant_id}

// Tenant metrics
tenant_quota_usage{tenant_id, resource}
tenant_operations_total{tenant_id, operation}
tenant_billing_units{tenant_id, period}

// State metrics
config_reload_total{result}
checkpoint_duration_seconds
session_spillover_total{operation}
```

### Alerts

```yaml
- name: CachePerformanceDegraded
  expr: cache_access_duration_seconds{p95} > 0.010
  severity: warning

- name: TenantQuotaExceeded
  expr: tenant_quota_usage > 0.90
  severity: warning

- name: CacheHitRateLow
  expr: cache_hit_rate < 0.70
  severity: info
```

## Testing Strategy

### 1. Unit Tests

- Cache manager operations
- Tenant isolation verification
- Quota enforcement logic
- State management functions

### 2. Integration Tests

- End-to-end request flow with tenants
- Cache warming effectiveness
- Hot config reload
- Checkpoint/restore

### 3. Performance Tests

- Cache access latency benchmarks
- Concurrent tenant load testing
- Memory pressure and spillover
- Distributed coordination

### 4. Security Tests

- Tenant isolation verification
- Cross-tenant access prevention
- Quota bypass attempts
- Encryption validation

## Rollback Plan

### Compatibility Layer

Maintain old interface during transition:

```rust
impl AppState {
    /// Legacy cache access for backward compatibility
    pub async fn get_cache_legacy(&self) -> tokio::sync::MutexGuard<'_, LegacyCacheAdapter> {
        self.legacy_cache.lock().await
    }
}

struct LegacyCacheAdapter {
    cache: Arc<PersistentCacheManager>,
}

impl LegacyCacheAdapter {
    async fn get(&mut self, key: &str) -> Result<Option<String>> {
        self.cache.get(key, None).await
    }

    async fn set(&mut self, key: &str, value: &str, ttl: u64) -> Result<()> {
        self.cache.set(key, value, None, Some(Duration::from_secs(ttl)), None).await
    }
}
```

### Feature Flags

```rust
pub struct FeatureFlags {
    pub enable_multi_tenancy: bool,
    pub enable_cache_warming: bool,
    pub enable_hot_reload: bool,
    pub use_persistence_layer: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            enable_multi_tenancy: std::env::var("ENABLE_MULTI_TENANCY")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            // ... other flags
        }
    }
}
```

## Success Criteria

### Technical Metrics

- ✅ Cache access time <5ms (p95)
- ✅ Cache hit rate >85%
- ✅ Support 100+ concurrent tenants
- ✅ Zero-downtime config reload
- ✅ Graceful shutdown <30 seconds

### Functional Requirements

- ✅ Full multi-tenancy support
- ✅ Admin API for tenant management
- ✅ Cache warming operational
- ✅ Hot config reload working
- ✅ Checkpoint/restore tested

### Quality Gates

- ✅ 100% backward compatibility
- ✅ All existing tests passing
- ✅ New integration tests >90% coverage
- ✅ Performance benchmarks met
- ✅ Security audit passed

## References

### Internal Documentation

- `/workspaces/eventmesh/crates/riptide-persistence/README.md`
- `/workspaces/eventmesh/crates/riptide-api/README.md`
- `/workspaces/eventmesh/docs/phase3/FINAL_STATUS.md`

### External Resources

- [Redis Best Practices](https://redis.io/docs/manual/patterns/)
- [Multi-Tenant Architecture Patterns](https://docs.microsoft.com/en-us/azure/architecture/guide/multitenant/overview)
- [Cache-Aside Pattern](https://docs.microsoft.com/en-us/azure/architecture/patterns/cache-aside)

---

**Next Steps**: Proceed to implementation following the phased approach outlined above.
