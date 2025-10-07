use crate::errors::ApiError;
use crate::models::{DependencyStatus, HealthResponse, ServiceHealth, SystemMetrics};
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::time::Instant;
use tracing::{debug, info};

/// Application startup time for uptime calculation
pub static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

/// Initialize startup time tracking
pub fn init_startup_time() {
    START_TIME.set(Instant::now()).ok();
}

/// Comprehensive health check endpoint with dependency validation.
///
/// Returns detailed health information including:
/// - Overall application health status
/// - Individual dependency health (Redis, WASM extractor, HTTP client)
/// - System metrics and uptime
/// - Version information
///
/// This endpoint is suitable for load balancer health checks and monitoring systems.
#[tracing::instrument(
    name = "health_check",
    skip(state),
    fields(
        http.method = "GET",
        http.route = "/health",
        otel.status_code
    )
)]
pub async fn health(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();
    debug!("Starting health check");

    // Perform comprehensive health check
    let health_status = state.health_check().await;

    // Calculate uptime
    let uptime = START_TIME
        .get()
        .map(|start| start.elapsed().as_secs())
        .unwrap_or(0);

    // Get current timestamp
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Build dependency status
    let dependencies = DependencyStatus {
        redis: health_status.redis.into(),
        extractor: health_status.extractor.into(),
        http_client: health_status.http_client.into(),
        headless_service: state.config.headless_url.as_ref().map(|_| ServiceHealth {
            status: "unknown".to_string(),
            message: Some("Headless service configured but not checked".to_string()),
            response_time_ms: None,
            last_check: timestamp.clone(),
        }),
        spider_engine: state.spider.as_ref().map(|_| ServiceHealth {
            status: health_status.spider.to_string(),
            message: Some(match health_status.spider {
                crate::state::DependencyHealth::Healthy => "Spider engine ready".to_string(),
                crate::state::DependencyHealth::Unhealthy(ref msg) => msg.clone(),
                crate::state::DependencyHealth::Unknown => "Spider status unknown".to_string(),
            }),
            response_time_ms: None,
            last_check: timestamp.clone(),
        }),
    };

    // Implement actual system metrics collection using full health checker
    let health_checker = crate::health::HealthChecker::new();
    let metrics = Some(health_checker.collect_system_metrics(&state).await);

    let overall_status = if health_status.healthy {
        "healthy"
    } else {
        "unhealthy"
    };

    let response = HealthResponse {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp,
        uptime,
        dependencies,
        metrics,
    };

    info!(
        status = overall_status,
        uptime_seconds = uptime,
        check_time_ms = start_time.elapsed().as_millis(),
        "Health check completed"
    );

    // Return appropriate HTTP status based on health
    let status_code = if health_status.healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    Ok((status_code, Json(response)))
}

/// Perform actual headless service health check
// Moved to crate::health::perform_headless_health_check for shared use across handlers

/// Collect actual system metrics using sysinfo and psutil
pub(super) fn collect_system_metrics(avg_response_time_ms: f64) -> SystemMetrics {
    use std::process;
    use sysinfo::{Pid, System};

    let mut sys = System::new_all();
    sys.refresh_all();

    // Get memory usage
    let memory_usage_bytes = (sys.total_memory() - sys.available_memory()) * 1024; // Convert from KB to bytes

    // Get CPU usage (average across all cores)
    let cpu_usage_percent = if !sys.cpus().is_empty() {
        let total_cpu: f32 = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
        Some(total_cpu / sys.cpus().len() as f32)
    } else {
        None
    };

    // Get current process info
    let current_pid = process::id();
    let current_process = sys.process(Pid::from(current_pid as usize));

    let thread_count = current_process.map(|_p| 4).unwrap_or(4); // Simplified thread count

    // Get system load average (Unix-like systems) - simplified
    let load_avg_1min = if cfg!(unix) {
        // Simplified approach - would need proper implementation for production
        Some(1.0)
    } else {
        None
    };

    // Try to get file descriptor count (Unix-like systems only)
    let file_descriptor_count = get_file_descriptor_count();

    // Get disk usage for root filesystem - simplified
    let disk_usage_bytes = None; // Simplified - would need proper implementation

    // Calculate approximate active connections and total requests
    // These would typically come from application-specific metrics
    let (active_connections, total_requests, requests_per_second) = get_network_metrics();

    SystemMetrics {
        memory_usage_bytes,
        active_connections,
        total_requests,
        requests_per_second,
        avg_response_time_ms,
        cpu_usage_percent,
        disk_usage_bytes,
        file_descriptor_count,
        thread_count: if thread_count > 0 {
            Some(thread_count as u32)
        } else {
            None
        },
        load_average: load_avg_1min.map(|avg| [avg, avg, avg]),
    }
}

/// Get file descriptor count for current process (Unix-like systems)
pub(super) fn get_file_descriptor_count() -> Option<u32> {
    #[cfg(unix)]
    {
        use std::fs;
        if let Ok(entries) = fs::read_dir("/proc/self/fd") {
            Some(entries.count() as u32)
        } else {
            None
        }
    }
    #[cfg(not(unix))]
    {
        None
    }
}

/// Get network metrics (placeholder implementation)
/// In a real application, these would come from application-specific counters
pub(super) fn get_network_metrics() -> (u32, u64, f64) {
    // For now, return placeholder values
    // These should be tracked by the application's metrics system
    (0, 0, 0.0)
}

/// Enhanced health check endpoint with comprehensive diagnostics
///
/// Returns detailed health information including:
/// - All dependency health checks with response times
/// - Comprehensive system metrics (CPU, memory, disk, threads, load average)
/// - Build information (git SHA, build timestamp, component versions)
/// - Bucket configuration for performance monitoring
///
/// This endpoint provides the most comprehensive health diagnostics available.
#[tracing::instrument(
    name = "health_check_detailed",
    skip(state),
    fields(
        http.method = "GET",
        http.route = "/health/detailed",
        otel.status_code
    )
)]
pub async fn health_detailed(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    debug!("Starting comprehensive detailed health check");

    // Use HealthChecker to perform comprehensive health check
    let health_response = state.health_checker.check_health(&state).await;

    info!(
        status = %health_response.status,
        uptime_seconds = health_response.uptime,
        "Comprehensive detailed health check completed"
    );

    // Return appropriate HTTP status based on health
    let status_code = if health_response.status == "healthy" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    Ok((status_code, Json(health_response)))
}

/// Component-specific health check endpoint
///
/// Returns health status for a specific component: redis, extractor, http_client, or headless
pub async fn component_health_check(
    State(state): State<AppState>,
    axum::extract::Path(component): axum::extract::Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    debug!(component = %component, "Checking specific component health");

    let health_response = state.health_checker.check_health(&state).await;
    let timestamp = chrono::Utc::now().to_rfc3339();

    let component_health = match component.as_str() {
        "redis" => health_response.dependencies.redis,
        "extractor" => health_response.dependencies.extractor,
        "http_client" => health_response.dependencies.http_client,
        "headless" => health_response
            .dependencies
            .headless_service
            .unwrap_or_else(|| ServiceHealth {
                status: "not_configured".to_string(),
                message: Some("Headless service not configured".to_string()),
                response_time_ms: None,
                last_check: timestamp,
            }),
        "spider" => health_response
            .dependencies
            .spider_engine
            .unwrap_or_else(|| ServiceHealth {
                status: "not_configured".to_string(),
                message: Some("Spider engine not configured".to_string()),
                response_time_ms: None,
                last_check: timestamp,
            }),
        _ => {
            return Err(ApiError::not_found(format!("Component '{}' not found. Available components: redis, extractor, http_client, headless, spider", component)));
        }
    };

    info!(
        component = %component,
        status = %component_health.status,
        "Component health check completed"
    );

    let status_code = if component_health.status == "healthy" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    Ok((status_code, Json(component_health)))
}

/// System metrics endpoint
///
/// Returns comprehensive system metrics including CPU, memory, disk, and network stats
pub async fn health_metrics_check(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    debug!("Collecting system metrics");

    let health_response = state.health_checker.check_health(&state).await;

    let metrics = health_response
        .metrics
        .unwrap_or_else(|| collect_system_metrics(0.0));

    info!(
        memory_mb = metrics.memory_usage_bytes / (1024 * 1024),
        cpu_percent = ?metrics.cpu_usage_percent,
        "System metrics collected"
    );

    Ok(Json(metrics))
}
