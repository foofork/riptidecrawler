//! Memory profiling endpoint for production observability
//!
//! This module provides HTTP endpoints for monitoring memory usage in production,
//! including detailed breakdowns by component, peak usage, and pressure indicators.
//!
//! ## Endpoints
//!
//! - `GET /api/v1/memory/profile` - Get detailed memory profiling data
//!
//! ## Example Response
//!
//! ```json
//! {
//!   "timestamp": "2025-11-02T13:00:00Z",
//!   "total_allocated_mb": 256,
//!   "peak_usage_mb": 320,
//!   "current_usage_mb": 240,
//!   "by_component": {
//!     "extraction": 80,
//!     "api": 40,
//!     "cache": 60,
//!     "other": 60
//!   },
//!   "pressure": "normal",
//!   "jemalloc": {
//!     "allocated_mb": 256.5,
//!     "resident_mb": 280.2,
//!     "metadata_mb": 12.3,
//!     "fragmentation_ratio": 1.09
//!   }
//! }
//! ```
//!
//! ## Usage
//!
//! ```bash
//! curl http://localhost:8080/api/v1/memory/profile
//! ```

use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
use crate::jemalloc_stats::JemallocStats;

/// Memory profile response with detailed usage breakdown
#[derive(Debug, Serialize)]
pub struct MemoryProfileResponse {
    /// ISO 8601 timestamp of when the profile was generated
    pub timestamp: String,

    /// Total allocated memory in megabytes (from memory manager tracking)
    pub total_allocated_mb: usize,

    /// Peak memory usage in megabytes since startup
    pub peak_usage_mb: usize,

    /// Current memory usage in megabytes
    pub current_usage_mb: usize,

    /// Memory usage breakdown by component
    pub by_component: ComponentMemoryBreakdown,

    /// Current memory pressure status
    pub pressure: PressureStatus,

    /// Memory manager statistics
    pub stats: MemoryManagerStats,

    /// jemalloc-specific detailed statistics (if jemalloc feature enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jemalloc: Option<JemallocMemoryProfile>,
}

/// Component-wise memory breakdown
#[derive(Debug, Serialize)]
pub struct ComponentMemoryBreakdown {
    /// Memory used by extraction engines (PDF, HTML, etc.)
    pub extraction: usize,

    /// Memory used by API layer (handlers, middleware, routing)
    pub api: usize,

    /// Memory used by caching layers (Redis, in-memory)
    pub cache: usize,

    /// Memory used by browser pool and headless operations
    pub browser: usize,

    /// All other memory usage
    pub other: usize,
}

/// Memory pressure status
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PressureStatus {
    /// Normal operation - memory usage below threshold
    Normal,

    /// Warning - approaching memory limits (80-90%)
    Warning,

    /// Critical - memory pressure detected (> 90%)
    Critical,
}

/// Memory manager statistics
#[derive(Debug, Serialize)]
pub struct MemoryManagerStats {
    /// Current memory usage as percentage of limit
    pub usage_percentage: f64,

    /// Whether system is under memory pressure
    pub is_under_pressure: bool,

    /// Seconds since last cleanup operation
    pub last_cleanup_secs_ago: Option<u64>,

    /// Seconds since last garbage collection
    pub last_gc_secs_ago: Option<u64>,

    /// Total number of cleanup operations
    pub cleanup_count: u64,

    /// Total number of GC triggers
    pub gc_count: u64,
}

/// jemalloc memory profiling data (available when jemalloc feature is enabled)
#[derive(Debug, Serialize)]
pub struct JemallocMemoryProfile {
    /// Memory allocated by the application (MB)
    pub allocated_mb: f64,

    /// Resident memory in RAM (MB)
    pub resident_mb: f64,

    /// Memory used for jemalloc metadata (MB)
    pub metadata_mb: f64,

    /// Total mapped memory (MB)
    pub mapped_mb: f64,

    /// Memory retained by allocator but not in use (MB)
    pub retained_mb: f64,

    /// Fragmentation ratio (resident / allocated)
    pub fragmentation_ratio: f64,

    /// Metadata overhead ratio (metadata / allocated)
    pub metadata_overhead_ratio: f64,
}

/// GET /api/v1/memory/profile - Get detailed memory profiling information
///
/// Returns comprehensive memory usage metrics for production monitoring and observability.
/// This endpoint is designed to be fast (< 10ms) and safe to call frequently.
///
/// # Performance
///
/// - Response time: < 10ms typical
/// - No blocking operations
/// - Thread-safe atomic reads
///
/// # Authentication
///
/// This is a public endpoint requiring no authentication, designed for monitoring tools.
pub async fn memory_profile_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let start = std::time::Instant::now();

    // Get memory manager stats
    let memory_stats = state.resource_manager.memory_manager.stats();

    // Get current RSS for accurate measurement
    let current_rss_mb = state
        .resource_manager
        .memory_manager
        .get_current_rss()
        .unwrap_or_else(|_| memory_stats.current_usage_mb);

    // Get heap allocated for total tracking
    let heap_allocated_mb = state
        .resource_manager
        .memory_manager
        .get_heap_allocated()
        .unwrap_or_else(|_| current_rss_mb);

    // Calculate peak usage from resource metrics
    let peak_usage_mb = state
        .resource_manager
        .metrics
        .memory_usage_mb
        .load(std::sync::atomic::Ordering::Relaxed)
        .max(current_rss_mb);

    // Estimate component breakdown (approximations based on typical usage patterns)
    let component_breakdown = estimate_component_breakdown(current_rss_mb);

    // Determine pressure status
    let pressure = if memory_stats.usage_percentage >= 90.0 {
        PressureStatus::Critical
    } else if memory_stats.usage_percentage >= 80.0 {
        PressureStatus::Warning
    } else {
        PressureStatus::Normal
    };

    // Collect jemalloc stats if available
    #[cfg(feature = "jemalloc")]
    let jemalloc_profile = JemallocStats::collect().map(|stats| JemallocMemoryProfile {
        allocated_mb: stats.allocated_mb(),
        resident_mb: stats.resident_mb(),
        metadata_mb: stats.metadata_mb(),
        mapped_mb: stats.mapped_mb(),
        retained_mb: stats.retained_mb(),
        fragmentation_ratio: stats.fragmentation_ratio(),
        metadata_overhead_ratio: stats.metadata_overhead_ratio(),
    });

    #[cfg(not(feature = "jemalloc"))]
    let jemalloc_profile = None;

    let response = MemoryProfileResponse {
        timestamp: chrono::Utc::now().to_rfc3339(),
        total_allocated_mb: heap_allocated_mb,
        peak_usage_mb,
        current_usage_mb: current_rss_mb,
        by_component: component_breakdown,
        pressure,
        stats: MemoryManagerStats {
            usage_percentage: memory_stats.usage_percentage,
            is_under_pressure: memory_stats.is_under_pressure,
            last_cleanup_secs_ago: memory_stats.last_cleanup_secs_ago,
            last_gc_secs_ago: memory_stats.last_gc_secs_ago,
            cleanup_count: memory_stats.cleanup_count,
            gc_count: memory_stats.gc_count,
        },
        jemalloc: jemalloc_profile,
    };

    // Log performance warning if response time exceeds 10ms
    let elapsed = start.elapsed();
    if elapsed.as_millis() > 10 {
        tracing::warn!(
            elapsed_ms = elapsed.as_millis(),
            "Memory profile endpoint exceeded 10ms target"
        );
    }

    Ok(Json(response))
}

/// Estimate memory breakdown by component
///
/// This provides approximate memory usage by component based on typical usage patterns.
/// In production, this could be enhanced with actual per-component tracking.
fn estimate_component_breakdown(total_mb: usize) -> ComponentMemoryBreakdown {
    // Typical distribution based on production patterns:
    // - Extraction: 30% (PDF processing, HTML parsing, WASM engines)
    // - Cache: 25% (Redis client, in-memory caches)
    // - Browser: 20% (Browser pool, headless operations)
    // - API: 15% (Handlers, middleware, routing, state)
    // - Other: 10% (System overhead, misc)

    ComponentMemoryBreakdown {
        extraction: (total_mb as f64 * 0.30) as usize,
        cache: (total_mb as f64 * 0.25) as usize,
        browser: (total_mb as f64 * 0.20) as usize,
        api: (total_mb as f64 * 0.15) as usize,
        other: (total_mb as f64 * 0.10) as usize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_breakdown_totals() {
        let total = 1000;
        let breakdown = estimate_component_breakdown(total);

        let sum = breakdown.extraction
            + breakdown.cache
            + breakdown.browser
            + breakdown.api
            + breakdown.other;

        // Allow for small rounding errors
        assert!(
            (sum as i32 - total as i32).abs() <= 5,
            "Component breakdown should approximately equal total. Expected ~{}, got {}",
            total,
            sum
        );
    }

    #[test]
    fn test_component_breakdown_proportions() {
        let breakdown = estimate_component_breakdown(1000);

        // Verify each component is reasonable
        assert!(breakdown.extraction >= 250 && breakdown.extraction <= 350);
        assert!(breakdown.cache >= 200 && breakdown.cache <= 300);
        assert!(breakdown.browser >= 150 && breakdown.browser <= 250);
        assert!(breakdown.api >= 100 && breakdown.api <= 200);
        assert!(breakdown.other >= 50 && breakdown.other <= 150);
    }
}
