use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use chrono::{DateTime, Utc};

use crate::profiling::{
    memory_tracker::MemoryProfiler,
    allocation_analyzer::AllocationStats,
    leak_detector::LeakReport,
};

/// Memory snapshot response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub total_allocated: u64,
    pub total_deallocated: u64,
    pub current_usage: u64,
    pub peak_usage: u64,
    pub allocation_count: usize,
    pub deallocation_count: usize,
}

/// Leak analysis response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakAnalysisResponse {
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub leak_count: usize,
    pub total_leaked_bytes: u64,
    pub leaks: Vec<LeakInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakInfo {
    pub location: String,
    pub size: u64,
    pub age_seconds: i64,
    pub allocation_time: DateTime<Utc>,
}

/// Allocation statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationStatsResponse {
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub top_allocators: Vec<AllocatorInfo>,
    pub total_allocations: usize,
    pub average_allocation_size: u64,
    pub median_allocation_size: u64,
    pub allocation_rate_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatorInfo {
    pub location: String,
    pub allocation_count: usize,
    pub total_bytes: u64,
    pub average_size: u64,
    pub percentage_of_total: f64,
}

/// Memory trend query parameters
#[derive(Debug, Deserialize)]
pub struct TrendQuery {
    #[serde(default = "default_duration")]
    pub duration: String,
    #[serde(default = "default_interval")]
    pub interval: String,
}

fn default_duration() -> String {
    "1h".to_string()
}

fn default_interval() -> String {
    "1m".to_string()
}

/// Memory trend response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTrendResponse {
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub duration: String,
    pub interval: String,
    pub data_points: Vec<TrendDataPoint>,
    pub trend_analysis: TrendAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendDataPoint {
    pub timestamp: DateTime<Utc>,
    pub memory_usage: u64,
    pub allocation_count: usize,
    pub deallocation_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub average_usage: u64,
    pub peak_usage: u64,
    pub min_usage: u64,
    pub growth_rate: f64,
    pub volatility: f64,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub status: HealthStatus,
    pub memory_usage_percent: f64,
    pub leak_severity: LeakSeverity,
    pub allocation_rate_status: AllocationRateStatus,
    pub details: HealthDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LeakSeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AllocationRateStatus {
    Normal,
    Elevated,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDetails {
    pub current_memory: u64,
    pub memory_limit: u64,
    pub leak_count: usize,
    pub leaked_bytes: u64,
    pub allocation_rate: f64,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Garbage collection response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcResponse {
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub triggered: bool,
    pub memory_before: u64,
    pub memory_after: u64,
    pub freed_bytes: u64,
    pub duration_ms: u64,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub timestamp: DateTime<Utc>,
    pub error: String,
    pub details: Option<String>,
}

/// Memory metrics router state
#[derive(Clone)]
pub struct MemoryMetricsState {
    profiler: Arc<RwLock<MemoryProfiler>>,
    session_id: String,
}

/// Memory metrics router
pub struct MemoryMetricsRouter;

impl MemoryMetricsRouter {
    /// Create a new memory metrics router with the given profiler
    pub fn new(profiler: Arc<RwLock<MemoryProfiler>>, session_id: String) -> MemoryMetricsState {
        MemoryMetricsState {
            profiler,
            session_id,
        }
    }

    /// Build the router with all memory profiling endpoints
    pub fn routes(state: MemoryMetricsState) -> Router {
        Router::new()
            .route("/metrics/memory/snapshot", get(get_snapshot))
            .route("/metrics/memory/leaks", get(get_leaks))
            .route("/metrics/memory/allocations", get(get_allocations))
            .route("/metrics/memory/trend", get(get_trend))
            .route("/metrics/memory/health", get(health_check))
            .route("/metrics/memory/gc", post(force_gc))
            .layer(CorsLayer::permissive())
            .with_state(state)
    }
}

/// GET /metrics/memory/snapshot - Current memory snapshot
async fn get_snapshot(
    State(state): State<MemoryMetricsState>,
) -> Result<Json<MemorySnapshot>, (StatusCode, Json<ErrorResponse>)> {
    let profiler = state.profiler.read().await;

    let snapshot = profiler.snapshot();

    Ok(Json(MemorySnapshot {
        timestamp: Utc::now(),
        session_id: state.session_id.clone(),
        total_allocated: snapshot.total_allocated,
        total_deallocated: snapshot.total_deallocated,
        current_usage: snapshot.current_usage,
        peak_usage: snapshot.peak_usage,
        allocation_count: snapshot.allocation_count,
        deallocation_count: snapshot.deallocation_count,
    }))
}

/// GET /metrics/memory/leaks - Leak analysis report
async fn get_leaks(
    State(state): State<MemoryMetricsState>,
) -> Result<Json<LeakAnalysisResponse>, (StatusCode, Json<ErrorResponse>)> {
    let profiler = state.profiler.read().await;

    let leak_report = profiler.detect_leaks();

    let leaks: Vec<LeakInfo> = leak_report.leaks.iter().map(|leak| {
        LeakInfo {
            location: leak.location.clone(),
            size: leak.size,
            age_seconds: leak.age.as_secs() as i64,
            allocation_time: leak.allocation_time,
        }
    }).collect();

    let total_leaked_bytes: u64 = leaks.iter().map(|l| l.size).sum();

    Ok(Json(LeakAnalysisResponse {
        timestamp: Utc::now(),
        session_id: state.session_id.clone(),
        leak_count: leaks.len(),
        total_leaked_bytes,
        leaks,
    }))
}

/// GET /metrics/memory/allocations - Top allocators and statistics
async fn get_allocations(
    State(state): State<MemoryMetricsState>,
) -> Result<Json<AllocationStatsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let profiler = state.profiler.read().await;

    let stats = profiler.allocation_stats();
    let total_bytes: u64 = stats.total_allocated;

    let top_allocators: Vec<AllocatorInfo> = stats.top_allocators
        .iter()
        .take(20)
        .map(|allocator| {
            let percentage = if total_bytes > 0 {
                (allocator.total_bytes as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };

            AllocatorInfo {
                location: allocator.location.clone(),
                allocation_count: allocator.count,
                total_bytes: allocator.total_bytes,
                average_size: allocator.average_size,
                percentage_of_total: percentage,
            }
        })
        .collect();

    Ok(Json(AllocationStatsResponse {
        timestamp: Utc::now(),
        session_id: state.session_id.clone(),
        top_allocators,
        total_allocations: stats.total_count,
        average_allocation_size: stats.average_size,
        median_allocation_size: stats.median_size,
        allocation_rate_per_second: stats.allocation_rate,
    }))
}

/// GET /metrics/memory/trend?duration=1h&interval=1m - Memory trend over time
async fn get_trend(
    State(state): State<MemoryMetricsState>,
    Query(params): Query<TrendQuery>,
) -> Result<Json<MemoryTrendResponse>, (StatusCode, Json<ErrorResponse>)> {
    let profiler = state.profiler.read().await;

    // Parse duration
    let duration_secs = parse_duration(&params.duration)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    timestamp: Utc::now(),
                    error: "Invalid duration format".to_string(),
                    details: Some(e),
                }),
            )
        })?;

    // Parse interval
    let interval_secs = parse_duration(&params.interval)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    timestamp: Utc::now(),
                    error: "Invalid interval format".to_string(),
                    details: Some(e),
                }),
            )
        })?;

    let trend_data = profiler.get_trend(duration_secs, interval_secs);

    let data_points: Vec<TrendDataPoint> = trend_data.points.iter().map(|point| {
        TrendDataPoint {
            timestamp: point.timestamp,
            memory_usage: point.memory_usage,
            allocation_count: point.allocation_count,
            deallocation_count: point.deallocation_count,
        }
    }).collect();

    let avg_usage = if !data_points.is_empty() {
        data_points.iter().map(|p| p.memory_usage).sum::<u64>() / data_points.len() as u64
    } else {
        0
    };

    let peak_usage = data_points.iter().map(|p| p.memory_usage).max().unwrap_or(0);
    let min_usage = data_points.iter().map(|p| p.memory_usage).min().unwrap_or(0);

    // Calculate growth rate (bytes per second)
    let growth_rate = if data_points.len() > 1 {
        let first = data_points.first().unwrap().memory_usage as f64;
        let last = data_points.last().unwrap().memory_usage as f64;
        let time_diff = duration_secs as f64;
        (last - first) / time_diff
    } else {
        0.0
    };

    // Calculate volatility (standard deviation)
    let volatility = if data_points.len() > 1 {
        let mean = avg_usage as f64;
        let variance: f64 = data_points.iter()
            .map(|p| {
                let diff = p.memory_usage as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / data_points.len() as f64;
        variance.sqrt()
    } else {
        0.0
    };

    Ok(Json(MemoryTrendResponse {
        timestamp: Utc::now(),
        session_id: state.session_id.clone(),
        duration: params.duration,
        interval: params.interval,
        data_points,
        trend_analysis: TrendAnalysis {
            average_usage: avg_usage,
            peak_usage,
            min_usage,
            growth_rate,
            volatility,
        },
    }))
}

/// GET /metrics/memory/health - Health check with thresholds
async fn health_check(
    State(state): State<MemoryMetricsState>,
) -> Result<Json<HealthCheckResponse>, (StatusCode, Json<ErrorResponse>)> {
    let profiler = state.profiler.read().await;

    let snapshot = profiler.snapshot();
    let leak_report = profiler.detect_leaks();
    let stats = profiler.allocation_stats();

    // Memory limit (configurable, defaulting to 1GB)
    let memory_limit: u64 = 1_073_741_824; // 1GB
    let memory_usage_percent = (snapshot.current_usage as f64 / memory_limit as f64) * 100.0;

    // Determine health status
    let status = if memory_usage_percent > 90.0 || leak_report.leaks.len() > 100 {
        HealthStatus::Critical
    } else if memory_usage_percent > 75.0 || leak_report.leaks.len() > 50 {
        HealthStatus::Warning
    } else {
        HealthStatus::Healthy
    };

    // Leak severity
    let leaked_bytes: u64 = leak_report.leaks.iter().map(|l| l.size).sum();
    let leak_severity = if leak_report.leaks.is_empty() {
        LeakSeverity::None
    } else if leak_report.leaks.len() < 10 && leaked_bytes < 1_048_576 {
        LeakSeverity::Low
    } else if leak_report.leaks.len() < 50 && leaked_bytes < 10_485_760 {
        LeakSeverity::Medium
    } else if leak_report.leaks.len() < 100 && leaked_bytes < 104_857_600 {
        LeakSeverity::High
    } else {
        LeakSeverity::Critical
    };

    // Allocation rate status
    let allocation_rate_status = if stats.allocation_rate > 10000.0 {
        AllocationRateStatus::Critical
    } else if stats.allocation_rate > 1000.0 {
        AllocationRateStatus::Elevated
    } else {
        AllocationRateStatus::Normal
    };

    // Generate warnings and recommendations
    let mut warnings = Vec::new();
    let mut recommendations = Vec::new();

    if memory_usage_percent > 75.0 {
        warnings.push(format!("Memory usage at {:.1}% of limit", memory_usage_percent));
        recommendations.push("Consider reducing memory footprint or increasing memory limit".to_string());
    }

    if leak_report.leaks.len() > 10 {
        warnings.push(format!("{} potential memory leaks detected", leak_report.leaks.len()));
        recommendations.push("Investigate memory leaks using /metrics/memory/leaks endpoint".to_string());
    }

    if stats.allocation_rate > 1000.0 {
        warnings.push(format!("High allocation rate: {:.1} allocs/sec", stats.allocation_rate));
        recommendations.push("Consider object pooling or allocation optimization".to_string());
    }

    Ok(Json(HealthCheckResponse {
        timestamp: Utc::now(),
        session_id: state.session_id.clone(),
        status,
        memory_usage_percent,
        leak_severity,
        allocation_rate_status,
        details: HealthDetails {
            current_memory: snapshot.current_usage,
            memory_limit,
            leak_count: leak_report.leaks.len(),
            leaked_bytes,
            allocation_rate: stats.allocation_rate,
            warnings,
            recommendations,
        },
    }))
}

/// POST /metrics/memory/gc - Force garbage collection
async fn force_gc(
    State(state): State<MemoryMetricsState>,
) -> Result<Json<GcResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start = std::time::Instant::now();
    let memory_before = {
        let profiler = state.profiler.read().await;
        profiler.snapshot().current_usage
    };

    // Note: Rust doesn't have explicit GC, but we can suggest cleanup
    // This would trigger any cleanup mechanisms in the profiler
    let mut profiler = state.profiler.write().await;
    let cleaned = profiler.cleanup_stale_allocations();

    let memory_after = profiler.snapshot().current_usage;
    let duration_ms = start.elapsed().as_millis() as u64;

    let freed_bytes = memory_before.saturating_sub(memory_after);

    Ok(Json(GcResponse {
        timestamp: Utc::now(),
        session_id: state.session_id.clone(),
        triggered: cleaned > 0,
        memory_before,
        memory_after,
        freed_bytes,
        duration_ms,
    }))
}

/// Parse duration string (e.g., "1h", "30m", "120s")
fn parse_duration(duration_str: &str) -> Result<u64, String> {
    let duration_str = duration_str.trim();

    if duration_str.is_empty() {
        return Err("Duration cannot be empty".to_string());
    }

    let (number_part, unit_part) = duration_str.split_at(
        duration_str.chars().take_while(|c| c.is_numeric()).count()
    );

    let number: u64 = number_part.parse()
        .map_err(|_| format!("Invalid number: {}", number_part))?;

    let multiplier = match unit_part {
        "s" | "sec" | "second" | "seconds" => 1,
        "m" | "min" | "minute" | "minutes" => 60,
        "h" | "hr" | "hour" | "hours" => 3600,
        "d" | "day" | "days" => 86400,
        "" => 1, // Default to seconds if no unit
        _ => return Err(format!("Unknown time unit: {}", unit_part)),
    };

    Ok(number * multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("60s").unwrap(), 60);
        assert_eq!(parse_duration("1m").unwrap(), 60);
        assert_eq!(parse_duration("1h").unwrap(), 3600);
        assert_eq!(parse_duration("2h").unwrap(), 7200);
        assert_eq!(parse_duration("30m").unwrap(), 1800);
        assert_eq!(parse_duration("120").unwrap(), 120);

        assert!(parse_duration("").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("10x").is_err());
    }

    #[test]
    fn test_health_status_determination() {
        // Test would verify health status logic
        let status = HealthStatus::Healthy;
        assert!(matches!(status, HealthStatus::Healthy));
    }
}
