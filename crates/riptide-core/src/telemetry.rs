use anyhow::{Context, Result};
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
    pub fn init() -> Result<Self> {
        info!("Initializing telemetry system");

        // Initialize tracing subscriber with OpenTelemetry layer
        // This also initializes OpenTelemetry internally
        init_tracing_subscriber()?;

        // Set up global propagator
        global::set_text_map_propagator(TraceContextPropagator::new());

        // Get the global tracer for internal use
        let tracer = global::tracer("riptide-crawler");

        let sanitizer = DataSanitizer::new();
        let sla_monitor = SlaMonitor::new();
        let resource_tracker = ResourceTracker::new();

        info!("Telemetry system initialized successfully");

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
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,riptide=debug"));

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
        let patterns = vec![
            // API Keys and tokens
            (
                Regex::new(
                    r#"(?i)(api[_-]?key|token|secret|password|auth)[\s=:]+([a-zA-Z0-9+/=-]{20,})"#,
                )
                .unwrap(),
                "$1=***REDACTED***".to_string(),
            ),
            // Authorization headers
            (
                Regex::new(
                    r#"(?i)(authorization|bearer)["':\s=]*["']?([a-zA-Z0-9+/=._-]{20,})["']?"#,
                )
                .unwrap(),
                "$1: ***REDACTED***".to_string(),
            ),
            // Email addresses (PII)
            (
                Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap(),
                "***EMAIL_REDACTED***".to_string(),
            ),
            // IP addresses (partial)
            (
                Regex::new(r"\b(\d{1,3}\.\d{1,3}\.\d{1,3}\.)\d{1,3}\b").unwrap(),
                "${1}XXX".to_string(),
            ),
            // Credit card numbers
            (
                Regex::new(r"\b(?:\d{4}[-\s]?){3}\d{4}\b").unwrap(),
                "***CC_REDACTED***".to_string(),
            ),
            // Social security numbers
            (
                Regex::new(r"\b\d{3}-?\d{2}-?\d{4}\b").unwrap(),
                "***SSN_REDACTED***".to_string(),
            ),
            // Phone numbers
            (
                Regex::new(r"\b(?:\+?1[-.\s]?)?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}\b")
                    .unwrap(),
                "***PHONE_REDACTED***".to_string(),
            ),
        ];

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

#[derive(Debug, Clone)]
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
}

impl Default for OperationMetrics {
    fn default() -> Self {
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

        // TODO: Implement proper percentile calculation with histogram
        // For now, use simple approximation
        if metrics.total_requests.is_multiple_of(20) {
            metrics.p95_duration = Duration::from_nanos(
                (metrics.total_duration.as_nanos() as f64 * 0.95) as u64
                    / metrics.total_requests,
            );
            metrics.p99_duration = Duration::from_nanos(
                (metrics.total_duration.as_nanos() as f64 * 0.99) as u64
                    / metrics.total_requests,
            );
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
    #[allow(dead_code)] // TODO: use for system metrics collection
    system: sysinfo::System,
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
            system: sysinfo::System::new_all(),
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

        ResourceUsage {
            cpu_usage_percent: cpu_usage,
            memory_usage_bytes: used_memory,
            memory_usage_percent: memory_percent,
            disk_usage_bytes: 0, // TODO: Implement disk usage tracking
            network_rx_bytes: network_rx,
            network_tx_bytes: network_tx,
            open_file_descriptors: 0, // TODO: Implement FD tracking
            load_average: (load_avg.one, load_avg.five, load_avg.fifteen),
        }
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
