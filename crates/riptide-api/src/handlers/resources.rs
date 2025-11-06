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

#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
use crate::jemalloc_stats::JemallocStats;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jemalloc: Option<JemallocMemoryStats>,
}

/// Jemalloc-specific memory statistics
#[derive(Debug, Serialize)]
pub struct JemallocMemoryStats {
    pub allocated_mb: f64,
    pub resident_mb: f64,
    pub metadata_mb: f64,
    pub mapped_mb: f64,
    pub retained_mb: f64,
    pub fragmentation_ratio: f64,
    pub metadata_overhead_ratio: f64,
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

    #[cfg(feature = "browser")]
    let (total_capacity, in_use, available) = match &state.resource_manager.browser_pool {
        Some(pool) => {
            let stats = pool.get_stats().await;
            (stats.total_capacity, stats.in_use, stats.available)
        }
        None => (0, 0, 0), // No local pool when using headless service
    };

    #[cfg(not(feature = "browser"))]
    let (total_capacity, in_use, available) = (0, 0, 0);

    Ok(Json(ResourceStatusResponse {
        browser_pool: BrowserPoolStatus {
            total_capacity,
            in_use,
            available,
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
        memory: {
            // Collect jemalloc stats if available
            #[cfg(feature = "jemalloc")]
            let jemalloc_stats = JemallocStats::collect().map(|stats| JemallocMemoryStats {
                allocated_mb: stats.allocated_mb(),
                resident_mb: stats.resident_mb(),
                metadata_mb: stats.metadata_mb(),
                mapped_mb: stats.mapped as f64 / (1024.0 * 1024.0),
                retained_mb: stats.retained as f64 / (1024.0 * 1024.0),
                fragmentation_ratio: stats.fragmentation_ratio(),
                metadata_overhead_ratio: stats.metadata_overhead_ratio(),
            });

            #[cfg(not(feature = "jemalloc"))]
            let jemalloc_stats = None;

            // Update metrics with latest jemalloc stats
            #[cfg(feature = "jemalloc")]
            if jemalloc_stats.is_some() {
                state.metrics.update_jemalloc_stats();
            }

            MemoryStatus {
                current_usage_mb: resource_status.memory_usage_mb,
                pressure_detected: resource_status.memory_pressure,
                jemalloc: jemalloc_stats,
            }
        },
        performance: PerformanceStatus {
            timeout_count: resource_status.timeout_count,
            degradation_score: resource_status.degradation_score,
        },
    }))
}

/// Get browser pool specific status
pub async fn get_browser_pool_status(
    State(_state): State<AppState>,
) -> Result<Json<BrowserPoolStatus>, StatusCode> {
    #[cfg(feature = "browser")]
    let (total_capacity, in_use, available) = match &_state.resource_manager.browser_pool {
        Some(pool) => {
            let stats = pool.get_stats().await;
            (stats.total_capacity, stats.in_use, stats.available)
        }
        None => (0, 0, 0), // No local pool when using headless service
    };

    #[cfg(not(feature = "browser"))]
    let (total_capacity, in_use, available) = (0, 0, 0);

    Ok(Json(BrowserPoolStatus {
        total_capacity,
        in_use,
        available,
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

    // Collect jemalloc stats if available
    #[cfg(feature = "jemalloc")]
    let jemalloc_stats = JemallocStats::collect().map(|stats| JemallocMemoryStats {
        allocated_mb: stats.allocated_mb(),
        resident_mb: stats.resident_mb(),
        metadata_mb: stats.metadata_mb(),
        mapped_mb: stats.mapped as f64 / (1024.0 * 1024.0),
        retained_mb: stats.retained as f64 / (1024.0 * 1024.0),
        fragmentation_ratio: stats.fragmentation_ratio(),
        metadata_overhead_ratio: stats.metadata_overhead_ratio(),
    });

    #[cfg(not(feature = "jemalloc"))]
    let jemalloc_stats = None;

    // Update metrics with latest jemalloc stats
    #[cfg(feature = "jemalloc")]
    if jemalloc_stats.is_some() {
        state.metrics.update_jemalloc_stats();
    }

    Ok(Json(MemoryStatus {
        current_usage_mb: resource_status.memory_usage_mb,
        pressure_detected: resource_status.memory_pressure,
        jemalloc: jemalloc_stats,
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

/// Get memory leak detection report
pub async fn get_memory_leaks(
    State(state): State<AppState>,
) -> Result<Json<crate::resource_manager::memory_manager::LeakReport>, StatusCode> {
    let report = state.resource_manager.memory_manager.detect_leaks();
    Ok(Json(report))
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
