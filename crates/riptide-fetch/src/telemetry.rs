use anyhow::{Context, Result};
use hdrhistogram::Histogram;
use opentelemetry::trace::{TraceError, TracerProvider};
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{Config, Sampler};
use opentelemetry_sdk::{runtime, Resource};
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};
use regex::Regex;
use std::collections::HashMap;

// Export telemetry macros
#[macro_export]
macro_rules! telemetry_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*);
    };
}

#[macro_export]
macro_rules! telemetry_span {
    ($name:expr) => {
        tracing::info_span!($name)
    };
    ($name:expr, $($field:tt)*) => {
        tracing::info_span!($name, $($field)*)
    };
}
use std::sync::Arc;
use std::time::Duration;
use tracing::info;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, Registry};

/// Comprehensive telemetry and observability system for RipTide Crawler
///
/// This module provides:
/// - OpenTelemetry integration for distributed tracing
/// - Structured logging with sensitive data sanitization
/// - Performance metrics collection
/// - SLA monitoring and alerting
/// - Resource usage tracking
pub struct TelemetrySystem {
    tracer: Arc<opentelemetry::global::BoxedTracer>,
    sanitizer: DataSanitizer,
    sla_monitor: SlaMonitor,
    resource_tracker: ResourceTracker,
}

impl TelemetrySystem {
    /// Initialize the telemetry system with OpenTelemetry
    /// Note: This will set up the global tracing subscriber, so it should only be called once
    pub fn init() -> Result<Self> {
        // Initialize tracing subscriber with OpenTelemetry layer
        // This also initializes OpenTelemetry internally
        init_tracing_subscriber()?;

        // Now we can safely use tracing
        info!("Telemetry system initialized");

        // Set up global propagator
        global::set_text_map_propagator(TraceContextPropagator::new());

        // Get the global tracer for internal use
        let tracer = global::tracer("riptide-crawler");

        let sanitizer = DataSanitizer::new();
        let sla_monitor = SlaMonitor::new();
        let resource_tracker = ResourceTracker::new();

        Ok(Self {
            tracer: Arc::new(tracer),
            sanitizer,
            sla_monitor,
            resource_tracker,
        })
    }

    /// Get a reference to the tracer for creating spans
    pub fn tracer(&self) -> &opentelemetry::global::BoxedTracer {
        self.tracer.as_ref()
    }

    /// Sanitize sensitive data from logs and traces
    pub fn sanitize_data(&self, data: &str) -> String {
        self.sanitizer.sanitize(data)
    }

    /// Record SLA metrics for monitoring
    pub fn record_sla_metric(&mut self, operation: &str, duration: Duration, success: bool) {
        self.sla_monitor.record_metric(operation, duration, success);
    }

    /// Get current SLA status
    pub fn get_sla_status(&self) -> SlaStatus {
        self.sla_monitor.get_status()
    }

    /// Get current resource usage
    pub fn get_resource_usage(&self) -> ResourceUsage {
        self.resource_tracker.get_usage()
    }

    /// Shutdown telemetry system gracefully
    pub fn shutdown(&self) {
        info!("Shutting down telemetry system");
        global::shutdown_tracer_provider();
    }
}

/// Initialize OpenTelemetry with OTLP exporter
fn init_opentelemetry() -> Result<opentelemetry_sdk::trace::Tracer> {
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let resource = Resource::new(vec![
        KeyValue::new(SERVICE_NAME, "riptide-crawler"),
        KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
        KeyValue::new(
            "service.environment",
            std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
        ),
        KeyValue::new(
            "service.instance.id",
            std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string()),
        ),
    ]);

    // Create OTLP exporter
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(otlp_endpoint)
        .with_timeout(Duration::from_secs(3));

    // Create span exporter
    let span_exporter = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            Config::default()
                .with_resource(resource)
                .with_sampler(Sampler::TraceIdRatioBased(
                    std::env::var("OTEL_TRACE_SAMPLE_RATE")
                        .unwrap_or_else(|_| "0.1".to_string())
                        .parse()
                        .unwrap_or(0.1),
                )),
        )
        .install_batch(runtime::Tokio)
        .context("Failed to initialize OpenTelemetry tracer")?;

    // The install_batch call returns the tracer provider
    // We need to get a tracer from it
    let tracer = span_exporter.tracer("riptide-crawler");

    Ok(tracer)
}

/// Initialize tracing subscriber with OpenTelemetry layer
fn init_tracing_subscriber() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,cranelift_codegen=warn"));

    // Initialize OpenTelemetry tracer for distributed tracing
    let tracer = init_opentelemetry()?;

    // Create OpenTelemetry layer
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Create subscriber with both OpenTelemetry and fmt layers
    let subscriber = Registry::default()
        .with(env_filter)
        .with(telemetry_layer)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .json(),
        );

    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set tracing subscriber")?;

    Ok(())
}

/// Data sanitizer for removing sensitive information from logs and traces
pub struct DataSanitizer {
    patterns: Vec<(Regex, String)>,
}

impl DataSanitizer {
    pub fn new() -> Self {
        // Helper function to create regex patterns safely
        // These patterns are compile-time constants and should always be valid,
        // but we handle the error case to satisfy clippy's unwrap_used lint
        let create_pattern = |pattern: &str, replacement: &str| -> Option<(Regex, String)> {
            Regex::new(pattern)
                .ok()
                .map(|r| (r, replacement.to_string()))
        };

        let patterns = vec![
            // API Keys and tokens
            create_pattern(
                r#"(?i)(api[_-]?key|token|secret|password|auth)[\s=:]+([a-zA-Z0-9+/=-]{20,})"#,
                "$1=***REDACTED***",
            ),
            // Authorization headers
            create_pattern(
                r#"(?i)(authorization|bearer)["':\s=]*["']?([a-zA-Z0-9+/=._-]{20,})["']?"#,
                "$1: ***REDACTED***",
            ),
            // Email addresses (PII)
            create_pattern(
                r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b",
                "***EMAIL_REDACTED***",
            ),
            // IP addresses (partial)
            create_pattern(r"\b(\d{1,3}\.\d{1,3}\.\d{1,3}\.)\d{1,3}\b", "${1}XXX"),
            // Credit card numbers
            create_pattern(r"\b(?:\d{4}[-\s]?){3}\d{4}\b", "***CC_REDACTED***"),
            // Social security numbers
            create_pattern(r"\b\d{3}-?\d{2}-?\d{4}\b", "***SSN_REDACTED***"),
            // Phone numbers
            create_pattern(
                r"\b(?:\+?1[-.\s]?)?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}\b",
                "***PHONE_REDACTED***",
            ),
        ]
        .into_iter()
        .flatten()
        .collect();

        Self { patterns }
    }

    /// Sanitize sensitive data from input string
    pub fn sanitize(&self, input: &str) -> String {
        let mut result = input.to_string();

        for (pattern, replacement) in &self.patterns {
            result = pattern.replace_all(&result, replacement).to_string();
        }

        result
    }

    /// Sanitize a hashmap of key-value pairs
    pub fn sanitize_map(&self, map: &HashMap<String, String>) -> HashMap<String, String> {
        map.iter()
            .map(|(k, v)| {
                let sanitized_key = self.sanitize(k);
                let sanitized_value = self.sanitize(v);
                (sanitized_key, sanitized_value)
            })
            .collect()
    }
}

impl Default for DataSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

/// SLA monitoring for tracking performance against service level agreements
pub struct SlaMonitor {
    metrics: HashMap<String, OperationMetrics>,
    sla_thresholds: HashMap<String, SlaThreshold>,
}

#[derive(Debug)]
pub struct OperationMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub total_duration: Duration,
    pub max_duration: Duration,
    pub min_duration: Duration,
    pub p95_duration: Duration,
    pub p99_duration: Duration,
    pub error_count: u64,
    pub timeout_count: u64,
    // HDR Histogram for accurate percentile calculation
    // Tracks latencies from 1ns to 1 hour with 3 significant digits
    latency_histogram: Histogram<u64>,
}

impl Clone for OperationMetrics {
    fn clone(&self) -> Self {
        Self {
            total_requests: self.total_requests,
            successful_requests: self.successful_requests,
            total_duration: self.total_duration,
            max_duration: self.max_duration,
            min_duration: self.min_duration,
            p95_duration: self.p95_duration,
            p99_duration: self.p99_duration,
            error_count: self.error_count,
            timeout_count: self.timeout_count,
            latency_histogram: self.latency_histogram.clone(),
        }
    }
}

impl Default for OperationMetrics {
    fn default() -> Self {
        // Create histogram tracking 1ns to 1 hour (3_600_000_000_000 ns) with 3 sig figs
        let histogram = Histogram::<u64>::new_with_bounds(1, 3_600_000_000_000, 3)
            .expect("Failed to create histogram with valid bounds");

        Self {
            total_requests: 0,
            successful_requests: 0,
            total_duration: Duration::ZERO,
            max_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            p95_duration: Duration::ZERO,
            p99_duration: Duration::ZERO,
            error_count: 0,
            timeout_count: 0,
            latency_histogram: histogram,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SlaThreshold {
    pub max_latency_p95: Duration,
    pub max_latency_p99: Duration,
    pub min_availability: f64,
    pub max_error_rate: f64,
}

impl Default for SlaThreshold {
    fn default() -> Self {
        Self {
            max_latency_p95: Duration::from_millis(2000),
            max_latency_p99: Duration::from_millis(5000),
            min_availability: 99.5,
            max_error_rate: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SlaStatus {
    pub overall_compliance: bool,
    pub operations: HashMap<String, OperationSlaStatus>,
}

#[derive(Debug, Clone)]
pub struct OperationSlaStatus {
    pub operation: String,
    pub compliant: bool,
    pub availability: f64,
    pub error_rate: f64,
    pub p95_latency: Duration,
    pub p99_latency: Duration,
    pub violations: Vec<String>,
}

impl SlaMonitor {
    pub fn new() -> Self {
        let mut sla_thresholds = HashMap::new();

        // Define SLA thresholds for different operations
        sla_thresholds.insert(
            "http_fetch".to_string(),
            SlaThreshold {
                max_latency_p95: Duration::from_millis(1500),
                max_latency_p99: Duration::from_millis(3000),
                min_availability: 99.0,
                max_error_rate: 2.0,
            },
        );

        sla_thresholds.insert(
            "content_extraction".to_string(),
            SlaThreshold {
                max_latency_p95: Duration::from_millis(500),
                max_latency_p99: Duration::from_millis(1000),
                min_availability: 99.5,
                max_error_rate: 0.5,
            },
        );

        sla_thresholds.insert(
            "cache_operation".to_string(),
            SlaThreshold {
                max_latency_p95: Duration::from_millis(50),
                max_latency_p99: Duration::from_millis(100),
                min_availability: 99.9,
                max_error_rate: 0.1,
            },
        );

        Self {
            metrics: HashMap::new(),
            sla_thresholds,
        }
    }

    pub fn record_metric(&mut self, operation: &str, duration: Duration, success: bool) {
        let metrics = self.metrics.entry(operation.to_string()).or_default();

        metrics.total_requests += 1;
        if success {
            metrics.successful_requests += 1;
        } else {
            metrics.error_count += 1;
        }

        metrics.total_duration += duration;
        metrics.max_duration = metrics.max_duration.max(duration);
        metrics.min_duration = metrics.min_duration.min(duration);

        // Record duration in histogram for accurate percentile calculation
        let duration_ns = duration.as_nanos() as u64;
        // Clamp value to histogram bounds (1ns to 1 hour)
        let clamped_duration = duration_ns.max(1).min(3_600_000_000_000);

        if let Err(e) = metrics.latency_histogram.record(clamped_duration) {
            tracing::warn!(
                operation = operation,
                duration_ns = duration_ns,
                error = ?e,
                "Failed to record latency in histogram"
            );
        }

        // Calculate percentiles from histogram (every 10 requests for efficiency)
        if metrics.total_requests % 10 == 0 && metrics.latency_histogram.len() > 0 {
            // P95 percentile
            let p95_ns = metrics.latency_histogram.value_at_percentile(95.0);
            metrics.p95_duration = Duration::from_nanos(p95_ns);

            // P99 percentile
            let p99_ns = metrics.latency_histogram.value_at_percentile(99.0);
            metrics.p99_duration = Duration::from_nanos(p99_ns);
        }
    }

    pub fn get_status(&self) -> SlaStatus {
        let mut overall_compliance = true;
        let mut operations = HashMap::new();

        for (operation, metrics) in &self.metrics {
            let default_threshold = SlaThreshold::default();
            let threshold = self
                .sla_thresholds
                .get(operation)
                .unwrap_or(&default_threshold);

            let availability = if metrics.total_requests > 0 {
                (metrics.successful_requests as f64 / metrics.total_requests as f64) * 100.0
            } else {
                100.0
            };

            let error_rate = if metrics.total_requests > 0 {
                (metrics.error_count as f64 / metrics.total_requests as f64) * 100.0
            } else {
                0.0
            };

            let mut violations = Vec::new();
            let mut compliant = true;

            if availability < threshold.min_availability {
                violations.push(format!(
                    "Availability {:.2}% below threshold {:.2}%",
                    availability, threshold.min_availability
                ));
                compliant = false;
            }

            if error_rate > threshold.max_error_rate {
                violations.push(format!(
                    "Error rate {:.2}% above threshold {:.2}%",
                    error_rate, threshold.max_error_rate
                ));
                compliant = false;
            }

            if metrics.p95_duration > threshold.max_latency_p95 {
                violations.push(format!(
                    "P95 latency {:?} above threshold {:?}",
                    metrics.p95_duration, threshold.max_latency_p95
                ));
                compliant = false;
            }

            if metrics.p99_duration > threshold.max_latency_p99 {
                violations.push(format!(
                    "P99 latency {:?} above threshold {:?}",
                    metrics.p99_duration, threshold.max_latency_p99
                ));
                compliant = false;
            }

            if !compliant {
                overall_compliance = false;
            }

            operations.insert(
                operation.clone(),
                OperationSlaStatus {
                    operation: operation.clone(),
                    compliant,
                    availability,
                    error_rate,
                    p95_latency: metrics.p95_duration,
                    p99_latency: metrics.p99_duration,
                    violations,
                },
            );
        }

        SlaStatus {
            overall_compliance,
            operations,
        }
    }
}

impl Default for SlaMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource usage tracking for system monitoring
pub struct ResourceTracker {
    _system_info_placeholder: (),
}

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_usage_percent: f32,
    pub memory_usage_bytes: u64,
    pub memory_usage_percent: f32,
    pub disk_usage_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub open_file_descriptors: u64,
    pub load_average: (f64, f64, f64), // 1min, 5min, 15min
}

impl ResourceTracker {
    pub fn new() -> Self {
        Self {
            _system_info_placeholder: (),
        }
    }

    pub fn get_usage(&self) -> ResourceUsage {
        let mut system = sysinfo::System::new();
        system.refresh_all();

        let cpu_usage = system.global_cpu_usage();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let memory_percent = if total_memory > 0 {
            (used_memory as f32 / total_memory as f32) * 100.0
        } else {
            0.0
        };

        // Get load average (Unix-like systems only)
        let load_avg = sysinfo::System::load_average();

        // Network monitoring removed in sysinfo 0.32
        // Using placeholder values for now
        let network_rx = 0;
        let network_tx = 0;

        // Get disk usage for current working directory
        let disk_usage_bytes = Self::get_disk_usage();

        // Get open file descriptor count
        let open_file_descriptors = Self::get_file_descriptor_count();

        ResourceUsage {
            cpu_usage_percent: cpu_usage,
            memory_usage_bytes: used_memory,
            memory_usage_percent: memory_percent,
            disk_usage_bytes,
            network_rx_bytes: network_rx,
            network_tx_bytes: network_tx,
            open_file_descriptors,
            load_average: (load_avg.one, load_avg.five, load_avg.fifteen),
        }
    }

    /// Get disk usage for current working directory
    ///
    /// Uses platform-specific methods to determine disk space usage:
    /// - Linux: Uses statvfs to get filesystem statistics
    /// - macOS: Uses statvfs to get filesystem statistics
    /// - Windows: Uses GetDiskFreeSpaceEx to get volume statistics
    ///
    /// Returns the total disk space used in bytes, or 0 on error.
    fn get_disk_usage() -> u64 {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;

            // Try to get disk usage from filesystem stats
            if let Ok(metadata) = std::fs::metadata(".") {
                // Get the device ID (currently unused, but kept for future enhancements)
                let _dev = metadata.dev();

                // Try using statvfs to get filesystem stats
                #[cfg(target_os = "linux")]
                {
                    use std::ffi::CString;
                    use std::mem::MaybeUninit;

                    if let Ok(path) = CString::new(".") {
                        unsafe {
                            let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
                            if libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) == 0 {
                                let stat = stat.assume_init();
                                let total_blocks = stat.f_blocks;
                                let free_blocks = stat.f_bfree;
                                let used_blocks = total_blocks.saturating_sub(free_blocks);
                                let block_size = stat.f_frsize;
                                return used_blocks * block_size;
                            }
                        }
                    }
                }

                #[cfg(target_os = "macos")]
                {
                    use std::ffi::CString;
                    use std::mem::MaybeUninit;

                    if let Ok(path) = CString::new(".") {
                        unsafe {
                            let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
                            if libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) == 0 {
                                let stat = stat.assume_init();
                                let total_blocks = stat.f_blocks;
                                let free_blocks = stat.f_bfree;
                                let used_blocks = total_blocks.saturating_sub(free_blocks);
                                let block_size = stat.f_frsize as u64;
                                return used_blocks * block_size;
                            }
                        }
                    }
                }
            }
        }

        #[cfg(windows)]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use std::path::Path;

            if let Ok(current_dir) = std::env::current_dir() {
                let path = current_dir.to_str().unwrap_or(".");
                let wide: Vec<u16> = OsStr::new(path)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();

                unsafe {
                    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
                    let mut total_bytes: u64 = 0;
                    let mut free_bytes: u64 = 0;

                    if GetDiskFreeSpaceExW(
                        wide.as_ptr(),
                        std::ptr::null_mut(),
                        &mut total_bytes,
                        &mut free_bytes,
                    ) != 0
                    {
                        return total_bytes.saturating_sub(free_bytes);
                    }
                }
            }
        }

        // Fallback: return 0 if unable to determine
        0
    }

    /// Get file descriptor count for current process
    ///
    /// Platform-specific implementations:
    /// - Linux: Counts entries in /proc/self/fd directory
    /// - macOS: Counts entries in /dev/fd directory or uses proc_pidinfo
    /// - Windows: Uses GetProcessHandleCount
    ///
    /// Returns the number of open file descriptors/handles, or 0 on error.
    fn get_file_descriptor_count() -> u64 {
        #[cfg(target_os = "linux")]
        {
            // On Linux, count files in /proc/self/fd
            if let Ok(entries) = std::fs::read_dir("/proc/self/fd") {
                return entries.count() as u64;
            }
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, try /dev/fd first
            if let Ok(entries) = std::fs::read_dir("/dev/fd") {
                return entries.count() as u64;
            }

            // Fallback: use proc_pidinfo if available
            // This would require the libproc crate, so we'll skip for now
        }

        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::HANDLE;
            use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessHandleCount};

            unsafe {
                let process: HANDLE = GetCurrentProcess();
                let mut handle_count: u32 = 0;

                if GetProcessHandleCount(process, &mut handle_count) != 0 {
                    return handle_count as u64;
                }
            }
        }

        // Fallback: return 0 if unable to determine
        0
    }
}

impl Default for ResourceTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Telemetry-aware error type with automatic sanitization
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("OpenTelemetry error: {0}")]
    OpenTelemetry(#[from] TraceError),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Resource tracking error: {0}")]
    ResourceTracking(String),

    #[error("SLA monitoring error: {0}")]
    SlaMonitoring(String),
}

// Note: telemetry_span and telemetry_info macros are defined at the top of the file
// to avoid duplicate definitions

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_sanitizer() {
        let sanitizer = DataSanitizer::new();

        // Test API key sanitization
        let input = "api_key=sk-1234567890abcdef1234567890abcdef";
        let sanitized = sanitizer.sanitize(input);
        println!("DEBUG: Input: '{}', Sanitized: '{}'", input, sanitized);
        // The regex replaces the whole matched part with $1=***REDACTED***
        // Since the pattern captures (api_key) and the value, it should produce api_key=***REDACTED***
        assert!(sanitized.contains("=***REDACTED***"));
        assert!(!sanitized.contains("sk-1234567890abcdef1234567890abcdef"));

        // Test email sanitization
        let input = "Contact us at user@example.com for support";
        let sanitized = sanitizer.sanitize(input);
        assert!(sanitized.contains("***EMAIL_REDACTED***"));
        assert!(!sanitized.contains("user@example.com"));

        // Test IP partial sanitization
        let input = "Server IP: 192.168.1.100";
        let sanitized = sanitizer.sanitize(input);
        assert!(sanitized.contains("192.168.1.XXX"));
        assert!(!sanitized.contains("192.168.1.100"));
    }

    #[test]
    fn test_sla_monitor() {
        let mut monitor = SlaMonitor::new();

        // Record some metrics
        monitor.record_metric("test_op", Duration::from_millis(100), true);
        monitor.record_metric("test_op", Duration::from_millis(200), true);
        monitor.record_metric("test_op", Duration::from_millis(300), false);

        let status = monitor.get_status();
        assert!(status.operations.contains_key("test_op"));

        let op_status = &status.operations["test_op"];
        assert_eq!(op_status.operation, "test_op");
        assert!((op_status.availability - 66.67).abs() < 0.1);
        assert!((op_status.error_rate - 33.33).abs() < 0.1);
    }

    #[test]
    fn test_resource_tracker() {
        let tracker = ResourceTracker::new();
        let usage = tracker.get_usage();

        // Basic sanity checks
        assert!(usage.cpu_usage_percent >= 0.0);
        assert!(usage.memory_usage_bytes > 0);
        assert!(usage.memory_usage_percent >= 0.0);
        assert!(usage.memory_usage_percent <= 100.0);
    }
}

#[test]
fn test_histogram_percentile_accuracy() {
    let mut monitor = SlaMonitor::new();

    // Record a distribution with known percentiles
    // Create 100 requests with latencies from 1ms to 100ms
    for i in 1..=100 {
        monitor.record_metric("percentile_test", Duration::from_millis(i), true);
    }

    let status = monitor.get_status();
    let op_status = &status.operations["percentile_test"];

    // P95 should be around 95ms (95th value in sorted list)
    // Allow some tolerance due to histogram bucketing
    let p95_ms = op_status.p95_latency.as_millis();
    assert!(
        p95_ms >= 90 && p95_ms <= 100,
        "P95 latency {} ms should be near 95ms",
        p95_ms
    );

    // P99 should be around 99ms
    let p99_ms = op_status.p99_latency.as_millis();
    assert!(
        p99_ms >= 95 && p99_ms <= 105,
        "P99 latency {} ms should be near 99ms",
        p99_ms
    );
}

#[test]
fn test_histogram_with_outliers() {
    let mut monitor = SlaMonitor::new();

    // Record mostly fast requests with a few outliers
    // 95 fast requests at 10ms
    for _ in 0..95 {
        monitor.record_metric("outlier_test", Duration::from_millis(10), true);
    }

    // 5 slow outliers at 1000ms
    for _ in 0..5 {
        monitor.record_metric("outlier_test", Duration::from_millis(1000), true);
    }

    let status = monitor.get_status();
    let op_status = &status.operations["outlier_test"];

    // P95 should still be around 10ms (not affected by the 5% outliers)
    let p95_ms = op_status.p95_latency.as_millis();
    assert!(
        p95_ms >= 8 && p95_ms <= 20,
        "P95 latency {} ms should be near 10ms despite outliers",
        p95_ms
    );

    // P99 should be closer to the outlier values
    let p99_ms = op_status.p99_latency.as_millis();
    assert!(
        p99_ms >= 100,
        "P99 latency {} ms should reflect the outliers",
        p99_ms
    );
}

#[test]
fn test_histogram_percentile_stability() {
    let mut monitor = SlaMonitor::new();

    // Record requests in batches and verify percentiles are stable
    for batch in 1..=5 {
        for i in 1..=20 {
            monitor.record_metric("stability_test", Duration::from_millis(i), true);
        }

        if batch >= 2 {
            // After second batch, percentiles should be relatively stable
            let status = monitor.get_status();
            let op_status = &status.operations["stability_test"];

            let p95_ms = op_status.p95_latency.as_millis();
            assert!(
                p95_ms >= 17 && p95_ms <= 21,
                "P95 should be stable around 19ms in batch {}, got {}ms",
                batch,
                p95_ms
            );
        }
    }
}

#[test]
fn test_histogram_extreme_values() {
    let mut monitor = SlaMonitor::new();

    // Test with very small durations
    for _ in 0..50 {
        monitor.record_metric("extreme_test", Duration::from_nanos(100), true);
    }

    // Test with very large durations (but within histogram bounds)
    for _ in 0..50 {
        monitor.record_metric("extreme_test", Duration::from_secs(60), true);
    }

    let status = monitor.get_status();
    let op_status = &status.operations["extreme_test"];

    // Percentiles should handle extreme ranges
    assert!(op_status.p95_latency > Duration::ZERO);
    assert!(op_status.p99_latency > Duration::ZERO);
    assert!(op_status.p99_latency >= op_status.p95_latency);
}

#[test]
fn test_histogram_percentile_ordering() {
    let mut monitor = SlaMonitor::new();

    // Record various latencies
    for i in (10..=100).step_by(10) {
        for _ in 0..10 {
            monitor.record_metric("ordering_test", Duration::from_millis(i), true);
        }
    }

    let status = monitor.get_status();
    let op_status = &status.operations["ordering_test"];

    // Verify percentile ordering: P95 <= P99
    assert!(
        op_status.p95_latency <= op_status.p99_latency,
        "P95 ({:?}) should be <= P99 ({:?})",
        op_status.p95_latency,
        op_status.p99_latency
    );

    // Verify percentiles are within reasonable range
    // Note: histogram bucketing may cause slight variations above max due to precision
    assert!(
        op_status.p95_latency >= Duration::from_millis(10),
        "P95 should be >= minimum value"
    );
    assert!(
        op_status.p99_latency <= Duration::from_millis(120),
        "P99 should be near maximum value (with tolerance for histogram bucketing)"
    );
}
