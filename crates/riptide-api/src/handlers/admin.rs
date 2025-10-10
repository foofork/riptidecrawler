/*!
# Admin Endpoints for Persistence Layer Management

This module provides comprehensive administrative endpoints for managing:
- Multi-tenant operations (CRUD, usage, billing)
- Cache management (warming, invalidation, statistics)
- State management (hot reload, checkpoints, restore)

All endpoints are protected and should require admin authentication.

## Endpoint Summary

### Tenant Management (7 endpoints)
- POST   /admin/tenants           - Create new tenant
- GET    /admin/tenants           - List all tenants
- GET    /admin/tenants/:id       - Get tenant details
- PUT    /admin/tenants/:id       - Update tenant
- DELETE /admin/tenants/:id       - Delete tenant
- GET    /admin/tenants/:id/usage - Get tenant usage stats
- GET    /admin/tenants/:id/billing - Get billing information

### Cache Management (3 endpoints)
- POST   /admin/cache/warm        - Warm cache with keys
- POST   /admin/cache/invalidate  - Invalidate cache entries
- GET    /admin/cache/stats       - Get cache statistics

### State Management (3 endpoints)
- POST   /admin/state/reload      - Hot reload configuration
- POST   /admin/state/checkpoint  - Create state checkpoint
- POST   /admin/state/restore/:id - Restore from checkpoint
*/

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;

// ============================================================================
// Request/Response Models
// ============================================================================

/// Request to create a new tenant
#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    /// Tenant name
    pub name: String,
    /// Owner email address
    pub owner_email: String,
    /// Owner full name
    pub owner_name: String,
    /// Organization name (optional)
    pub owner_organization: Option<String>,
    /// Billing plan: "free", "basic", "professional", "enterprise"
    pub billing_plan: String,
    /// Custom resource quotas (optional)
    pub quotas: Option<HashMap<String, u64>>,
}

/// Request to update tenant configuration
#[derive(Debug, Deserialize)]
pub struct UpdateTenantRequest {
    /// New tenant name (optional)
    pub name: Option<String>,
    /// Updated quotas (optional)
    pub quotas: Option<HashMap<String, u64>>,
    /// Enable/disable encryption (optional)
    pub encryption_enabled: Option<bool>,
    /// Custom settings (optional)
    pub settings: Option<HashMap<String, String>>,
}

/// Tenant response
#[derive(Debug, Serialize)]
pub struct TenantResponse {
    /// Tenant ID
    pub tenant_id: String,
    /// Tenant name
    pub name: String,
    /// Status (active, suspended, deleted)
    pub status: String,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
}

/// Tenant usage statistics response
#[derive(Debug, Serialize)]
pub struct TenantUsageResponse {
    /// Tenant ID
    pub tenant_id: String,
    /// Total operations performed
    pub operation_count: u64,
    /// Total data transferred (bytes)
    pub data_bytes: u64,
    /// Total compute time (milliseconds)
    pub compute_time_ms: u64,
    /// Storage used (bytes)
    pub storage_bytes: u64,
    /// Current period start
    pub period_start: String,
    /// Current period end
    pub period_end: String,
}

/// Tenant billing information response
#[derive(Debug, Serialize)]
pub struct TenantBillingResponse {
    /// Tenant ID
    pub tenant_id: String,
    /// Billing plan name
    pub plan: String,
    /// Current billing period usage
    pub current_usage: TenantUsageResponse,
    /// Estimated cost for current period
    pub estimated_cost: f64,
    /// Currency (e.g., "USD")
    pub currency: String,
}

/// Cache warming request
#[derive(Debug, Deserialize)]
pub struct WarmCacheRequest {
    /// Cache keys to warm
    pub keys: Vec<String>,
    /// Optional tenant ID for tenant-specific warming
    pub tenant_id: Option<String>,
}

/// Cache invalidation request
#[derive(Debug, Deserialize)]
pub struct InvalidateCacheRequest {
    /// Cache keys to invalidate
    pub keys: Vec<String>,
    /// Optional tenant ID for tenant-specific invalidation
    pub tenant_id: Option<String>,
}

/// Cache statistics response
#[derive(Debug, Serialize)]
pub struct CacheStatsResponse {
    /// Total cache entries
    pub total_entries: usize,
    /// Cache hit rate (0-1)
    pub hit_rate: f64,
    /// Average access time (milliseconds)
    pub avg_access_time_ms: f64,
    /// Memory usage (bytes)
    pub memory_usage_bytes: usize,
    /// Number of evictions
    pub evictions: usize,
}

/// Checkpoint information response
#[derive(Debug, Serialize)]
pub struct CheckpointResponse {
    /// Checkpoint ID
    pub checkpoint_id: String,
    /// Creation timestamp
    pub created_at: String,
    /// Size in bytes
    pub size_bytes: usize,
    /// Description
    pub description: String,
}

// ============================================================================
// Tenant Management Endpoints
// ============================================================================

/// Create a new tenant
///
/// POST /admin/tenants
pub async fn create_tenant(
    State(state): State<AppState>,
    Json(req): Json<CreateTenantRequest>,
) -> ApiResult<Json<TenantResponse>> {
    tracing::info!(
        tenant_name = %req.name,
        billing_plan = %req.billing_plan,
        "Creating new tenant"
    );

    // Get persistence adapter
    let adapter = state
        .persistence_adapter
        .as_ref()
        .ok_or_else(|| ApiError::internal("Persistence layer not initialized"))?;

    // Parse billing plan
    let billing_plan = match req.billing_plan.as_str() {
        "free" => riptide_persistence::BillingPlan::Free,
        "basic" => riptide_persistence::BillingPlan::Basic,
        "professional" => riptide_persistence::BillingPlan::Professional,
        "enterprise" => riptide_persistence::BillingPlan::Enterprise,
        _ => {
            return Err(ApiError::validation(
                "Invalid billing plan. Must be: free, basic, professional, enterprise",
            ))
        }
    };

    // Create tenant configuration
    let tenant_config = riptide_persistence::TenantConfig {
        tenant_id: String::new(), // Generated by manager
        name: req.name.clone(),
        quotas: req.quotas.unwrap_or_default(),
        isolation_level: riptide_persistence::config::TenantIsolationLevel::Strong,
        encryption_enabled: true,
        settings: HashMap::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Create tenant owner
    let owner = riptide_persistence::TenantOwner {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.owner_name,
        email: req.owner_email,
        organization: req.owner_organization,
    };

    // Create tenant via adapter
    let tenant_id = adapter
        .create_tenant(tenant_config, owner, billing_plan)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to create tenant: {}", e)))?;

    tracing::info!(tenant_id = %tenant_id, "Tenant created successfully");

    Ok(Json(TenantResponse {
        tenant_id,
        name: req.name,
        status: "active".to_string(),
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    }))
}

/// List all tenants
///
/// GET /admin/tenants
pub async fn list_tenants(State(state): State<AppState>) -> ApiResult<Json<Vec<TenantResponse>>> {
    tracing::debug!("Listing all tenants");

    let adapter = state
        .persistence_adapter
        .as_ref()
        .ok_or_else(|| ApiError::internal("Persistence layer not initialized"))?;

    let tenants = adapter
        .list_tenants()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to list tenants: {}", e)))?;

    let response: Vec<TenantResponse> = tenants
        .into_iter()
        .map(|t| TenantResponse {
            tenant_id: t.tenant_id,
            name: t.name,
            status: format!("{:?}", t.status),
            created_at: t.created_at.to_rfc3339(),
            updated_at: t.updated_at.to_rfc3339(),
        })
        .collect();

    tracing::debug!(count = response.len(), "Tenants listed successfully");
    Ok(Json(response))
}

/// Get tenant details
///
/// GET /admin/tenants/:id
pub async fn get_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> ApiResult<Json<TenantResponse>> {
    tracing::debug!(tenant_id = %tenant_id, "Getting tenant details");

    let adapter = state
        .persistence_adapter
        .as_ref()
        .ok_or_else(|| ApiError::internal("Persistence layer not initialized"))?;

    let tenant = adapter
        .get_tenant(&tenant_id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get tenant: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Tenant not found"))?;

    Ok(Json(TenantResponse {
        tenant_id: tenant.tenant_id,
        name: tenant.name,
        status: format!("{:?}", tenant.status),
        created_at: tenant.created_at.to_rfc3339(),
        updated_at: tenant.updated_at.to_rfc3339(),
    }))
}

/// Update tenant configuration
///
/// PUT /admin/tenants/:id
pub async fn update_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(req): Json<UpdateTenantRequest>,
) -> ApiResult<Json<TenantResponse>> {
    tracing::info!(tenant_id = %tenant_id, "Updating tenant");

    let adapter = state
        .persistence_adapter
        .as_ref()
        .ok_or_else(|| ApiError::internal("Persistence layer not initialized"))?;

    // Get current tenant config
    let current = adapter
        .get_tenant(&tenant_id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get tenant: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Tenant not found"))?;

    // Build updated config
    let mut updated_config = riptide_persistence::TenantConfig {
        tenant_id: tenant_id.clone(),
        name: req.name.unwrap_or(current.name.clone()),
        quotas: req.quotas.unwrap_or_default(),
        isolation_level: riptide_persistence::config::TenantIsolationLevel::Strong,
        encryption_enabled: req.encryption_enabled.unwrap_or(true),
        settings: req.settings.unwrap_or_default(),
        created_at: current.created_at,
        updated_at: Utc::now(),
    };

    // Update tenant
    adapter
        .update_tenant(&tenant_id, updated_config.clone())
        .await
        .map_err(|e| ApiError::internal(format!("Failed to update tenant: {}", e)))?;

    tracing::info!(tenant_id = %tenant_id, "Tenant updated successfully");

    Ok(Json(TenantResponse {
        tenant_id,
        name: updated_config.name,
        status: format!("{:?}", current.status),
        created_at: current.created_at.to_rfc3339(),
        updated_at: updated_config.updated_at.to_rfc3339(),
    }))
}

/// Delete tenant
///
/// DELETE /admin/tenants/:id
pub async fn delete_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> ApiResult<StatusCode> {
    tracing::warn!(tenant_id = %tenant_id, "Deleting tenant");

    let adapter = state
        .persistence_adapter
        .as_ref()
        .ok_or_else(|| ApiError::internal("Persistence layer not initialized"))?;

    adapter
        .delete_tenant(&tenant_id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to delete tenant: {}", e)))?;

    tracing::info!(tenant_id = %tenant_id, "Tenant deleted successfully");
    Ok(StatusCode::NO_CONTENT)
}

/// Get tenant usage statistics
///
/// GET /admin/tenants/:id/usage
pub async fn get_tenant_usage(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> ApiResult<Json<TenantUsageResponse>> {
    tracing::debug!(tenant_id = %tenant_id, "Getting tenant usage");

    let adapter = state
        .persistence_adapter
        .as_ref()
        .ok_or_else(|| ApiError::internal("Persistence layer not initialized"))?;

    let usage = adapter
        .get_tenant_usage(&tenant_id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get tenant usage: {}", e)))?;

    let now = Utc::now();
    let period_start = now - chrono::Duration::days(30); // Last 30 days

    Ok(Json(TenantUsageResponse {
        tenant_id,
        operation_count: usage.total_operations,
        data_bytes: usage.total_data_bytes,
        compute_time_ms: usage.total_compute_ms,
        storage_bytes: usage.total_storage_bytes,
        period_start: period_start.to_rfc3339(),
        period_end: now.to_rfc3339(),
    }))
}

/// Get tenant billing information
///
/// GET /admin/tenants/:id/billing
pub async fn get_tenant_billing(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> ApiResult<Json<TenantBillingResponse>> {
    tracing::debug!(tenant_id = %tenant_id, "Getting tenant billing");

    let adapter = state
        .persistence_adapter
        .as_ref()
        .ok_or_else(|| ApiError::internal("Persistence layer not initialized"))?;

    let billing_info = adapter
        .get_tenant_billing(&tenant_id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get billing info: {}", e)))?;

    let usage = adapter
        .get_tenant_usage(&tenant_id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get tenant usage: {}", e)))?;

    let now = Utc::now();
    let period_start = now - chrono::Duration::days(30);

    Ok(Json(TenantBillingResponse {
        tenant_id: tenant_id.clone(),
        plan: format!("{:?}", billing_info.plan),
        current_usage: TenantUsageResponse {
            tenant_id,
            operation_count: usage.total_operations,
            data_bytes: usage.total_data_bytes,
            compute_time_ms: usage.total_compute_ms,
            storage_bytes: usage.total_storage_bytes,
            period_start: period_start.to_rfc3339(),
            period_end: now.to_rfc3339(),
        },
        estimated_cost: billing_info.current_period_cost,
        currency: "USD".to_string(),
    }))
}

// ============================================================================
// Cache Management Endpoints
// ============================================================================

/// Warm cache with specified keys
///
/// POST /admin/cache/warm
pub async fn warm_cache(
    State(state): State<AppState>,
    Json(req): Json<WarmCacheRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    tracing::info!(key_count = req.keys.len(), "Warming cache");

    let adapter = state
        .persistence_adapter
        .as_ref()
        .ok_or_else(|| ApiError::internal("Persistence layer not initialized"))?;

    let warmed_count = adapter
        .warm_cache(req.keys.clone())
        .await
        .map_err(|e| ApiError::internal(format!("Cache warming failed: {}", e)))?;

    tracing::info!(
        requested = req.keys.len(),
        warmed = warmed_count,
        "Cache warming completed"
    );

    Ok(Json(serde_json::json!({
        "status": "success",
        "keys_requested": req.keys.len(),
        "keys_warmed": warmed_count
    })))
}

/// Invalidate cache entries
///
/// POST /admin/cache/invalidate
pub async fn invalidate_cache(
    State(state): State<AppState>,
    Json(req): Json<InvalidateCacheRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    tracing::info!(key_count = req.keys.len(), "Invalidating cache entries");

    let adapter = state
        .persistence_adapter
        .as_ref()
        .ok_or_else(|| ApiError::internal("Persistence layer not initialized"))?;

    let mut invalidated = 0;
    for key in &req.keys {
        if adapter
            .invalidate(key, req.tenant_id.as_deref())
            .await
            .is_ok()
        {
            invalidated += 1;
        }
    }

    tracing::info!(
        requested = req.keys.len(),
        invalidated = invalidated,
        "Cache invalidation completed"
    );

    Ok(Json(serde_json::json!({
        "status": "success",
        "keys_requested": req.keys.len(),
        "keys_invalidated": invalidated
    })))
}

/// Get cache statistics
///
/// GET /admin/cache/stats
pub async fn get_cache_stats(State(state): State<AppState>) -> ApiResult<Json<CacheStatsResponse>> {
    tracing::debug!("Getting cache statistics");

    let adapter = state
        .persistence_adapter
        .as_ref()
        .ok_or_else(|| ApiError::internal("Persistence layer not initialized"))?;

    let stats = adapter
        .get_cache_stats()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get cache stats: {}", e)))?;

    Ok(Json(CacheStatsResponse {
        total_entries: stats.total_entries,
        hit_rate: stats.hit_rate,
        avg_access_time_ms: stats.avg_access_time_ms,
        memory_usage_bytes: stats.memory_usage_bytes,
        evictions: stats.eviction_count,
    }))
}

// ============================================================================
// State Management Endpoints
// ============================================================================

/// Reload configuration from disk
///
/// POST /admin/state/reload
pub async fn reload_state(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    tracing::info!("Reloading configuration");

    let adapter = state
        .persistence_adapter
        .as_ref()
        .ok_or_else(|| ApiError::internal("Persistence layer not initialized"))?;

    adapter
        .reload_config()
        .await
        .map_err(|e| ApiError::internal(format!("Config reload failed: {}", e)))?;

    tracing::info!("Configuration reloaded successfully");

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Configuration reloaded successfully",
        "timestamp": Utc::now().to_rfc3339()
    })))
}

/// Create state checkpoint
///
/// POST /admin/state/checkpoint
pub async fn create_checkpoint(
    State(state): State<AppState>,
) -> ApiResult<Json<CheckpointResponse>> {
    tracing::info!("Creating state checkpoint");

    let adapter = state
        .persistence_adapter
        .as_ref()
        .ok_or_else(|| ApiError::internal("Persistence layer not initialized"))?;

    let checkpoint = adapter
        .create_checkpoint()
        .await
        .map_err(|e| ApiError::internal(format!("Checkpoint creation failed: {}", e)))?;

    tracing::info!(checkpoint_id = %checkpoint.id, "Checkpoint created successfully");

    Ok(Json(CheckpointResponse {
        checkpoint_id: checkpoint.id,
        created_at: checkpoint.timestamp.to_rfc3339(),
        size_bytes: checkpoint.data.len(),
        description: checkpoint.description,
    }))
}

/// Restore from checkpoint
///
/// POST /admin/state/restore/:id
pub async fn restore_checkpoint(
    State(state): State<AppState>,
    Path(checkpoint_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    tracing::info!(checkpoint_id = %checkpoint_id, "Restoring from checkpoint");

    let adapter = state
        .persistence_adapter
        .as_ref()
        .ok_or_else(|| ApiError::internal("Persistence layer not initialized"))?;

    adapter
        .restore_checkpoint(&checkpoint_id)
        .await
        .map_err(|e| ApiError::internal(format!("Checkpoint restore failed: {}", e)))?;

    tracing::info!(checkpoint_id = %checkpoint_id, "Checkpoint restored successfully");

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "State restored successfully",
        "checkpoint_id": checkpoint_id,
        "timestamp": Utc::now().to_rfc3339()
    })))
}
