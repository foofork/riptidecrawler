//! Resource monitoring and status endpoints
//!
//! Provides comprehensive visibility into system resource utilization:
//! - Browser pool status
//! - Rate limiting metrics
//! - Memory usage
//! - Performance indicators

use crate::state::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

/// Complete resource status overview
#[derive(Debug, Serialize)]
pub struct ResourceStatusResponse {
    pub browser_pool: BrowserPoolStatus,
    pub rate_limiter: RateLimiterStatus,
    pub pdf_semaphore: SemaphoreStatus,
    pub memory: MemoryStatus,
    pub performance: PerformanceStatus,
}

/// Browser pool status
#[derive(Debug, Serialize)]
pub struct BrowserPoolStatus {
    pub total_capacity: usize,
    pub in_use: usize,
    pub available: usize,
    pub waiting: usize,
}

/// Rate limiter status
#[derive(Debug, Serialize)]
pub struct RateLimiterStatus {
    pub total_hits: u64,
    pub enabled: bool,
}

/// Semaphore status
#[derive(Debug, Serialize)]
pub struct SemaphoreStatus {
    pub total_permits: usize,
    pub available_permits: usize,
    pub in_use: usize,
}

/// Memory status
#[derive(Debug, Serialize)]
pub struct MemoryStatus {
    pub current_usage_mb: usize,
    pub pressure_detected: bool,
}

/// Performance status
#[derive(Debug, Serialize)]
pub struct PerformanceStatus {
    pub timeout_count: u64,
    pub degradation_score: f64,
}

/// Get comprehensive resource status
pub async fn get_resource_status(
    State(state): State<AppState>,
) -> Result<Json<ResourceStatusResponse>, StatusCode> {
    let resource_status = state.resource_manager.get_resource_status().await;
    let pool_stats = state.resource_manager.browser_pool.get_stats().await;

    Ok(Json(ResourceStatusResponse {
        browser_pool: BrowserPoolStatus {
            total_capacity: pool_stats.total_capacity,
            in_use: pool_stats.in_use,
            available: pool_stats.available,
            waiting: 0, // Field not available in PoolStats
        },
        rate_limiter: RateLimiterStatus {
            total_hits: resource_status.rate_limit_hits,
            enabled: state.api_config.rate_limiting.enabled,
        },
        pdf_semaphore: SemaphoreStatus {
            total_permits: resource_status.pdf_total,
            available_permits: resource_status.pdf_available,
            in_use: resource_status.pdf_total - resource_status.pdf_available,
        },
        memory: MemoryStatus {
            current_usage_mb: resource_status.memory_usage_mb,
            pressure_detected: resource_status.memory_pressure,
        },
        performance: PerformanceStatus {
            timeout_count: resource_status.timeout_count,
            degradation_score: resource_status.degradation_score,
        },
    }))
}

/// Get browser pool specific status
pub async fn get_browser_pool_status(
    State(state): State<AppState>,
) -> Result<Json<BrowserPoolStatus>, StatusCode> {
    let pool_stats = state.resource_manager.browser_pool.get_stats().await;

    Ok(Json(BrowserPoolStatus {
        total_capacity: pool_stats.total_capacity,
        in_use: pool_stats.in_use,
        available: pool_stats.available,
        waiting: 0, // Field not available in PoolStats
    }))
}

/// Get rate limiter status
pub async fn get_rate_limiter_status(
    State(state): State<AppState>,
) -> Result<Json<RateLimiterStatus>, StatusCode> {
    let resource_status = state.resource_manager.get_resource_status().await;

    Ok(Json(RateLimiterStatus {
        total_hits: resource_status.rate_limit_hits,
        enabled: state.api_config.rate_limiting.enabled,
    }))
}

/// Get memory status
pub async fn get_memory_status(
    State(state): State<AppState>,
) -> Result<Json<MemoryStatus>, StatusCode> {
    let resource_status = state.resource_manager.get_resource_status().await;

    Ok(Json(MemoryStatus {
        current_usage_mb: resource_status.memory_usage_mb,
        pressure_detected: resource_status.memory_pressure,
    }))
}

/// Get performance metrics
pub async fn get_performance_status(
    State(state): State<AppState>,
) -> Result<Json<PerformanceStatus>, StatusCode> {
    let resource_status = state.resource_manager.get_resource_status().await;

    Ok(Json(PerformanceStatus {
        timeout_count: resource_status.timeout_count,
        degradation_score: resource_status.degradation_score,
    }))
}

/// Get PDF semaphore status
pub async fn get_pdf_semaphore_status(
    State(state): State<AppState>,
) -> Result<Json<SemaphoreStatus>, StatusCode> {
    let resource_status = state.resource_manager.get_resource_status().await;

    Ok(Json(SemaphoreStatus {
        total_permits: resource_status.pdf_total,
        available_permits: resource_status.pdf_available,
        in_use: resource_status.pdf_total - resource_status.pdf_available,
    }))
}
