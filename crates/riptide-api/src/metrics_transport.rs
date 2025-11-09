//! Transport-level metrics for RipTide API layer.
//!
//! This module contains metrics specific to the HTTP/WebSocket transport layer:
//! - HTTP request/response metrics
//! - Connection tracking (WebSocket, SSE)
//! - Streaming protocol metrics
//! - System resource metrics (jemalloc)
//!
//! Business domain metrics (extractions, gates, PDF processing, etc.) are in
//! `riptide-facade` BusinessMetrics.

use axum_prometheus::{metrics_exporter_prometheus::PrometheusHandle, PrometheusMetricLayer};
use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry,
};
use std::time::Instant;
use tracing::info;

#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
use crate::jemalloc_stats::JemallocStats;

/// Transport-level metrics for HTTP/WebSocket/SSE protocols
#[derive(Debug)]
pub struct TransportMetrics {
    /// Prometheus registry for metrics
    pub registry: Registry,

    // ===== HTTP Protocol Metrics =====
    /// HTTP request counter
    pub http_requests_total: Counter,
    /// HTTP request duration histogram
    pub http_request_duration: Histogram,
    /// HTTP error counter
    pub http_errors: Counter,

    // ===== Connection Metrics =====
    /// Active HTTP connections gauge
    pub active_connections: Gauge,
    /// Active streaming connections (WebSocket/SSE)
    pub streaming_active_connections: Gauge,
    /// Total streaming connections created
    pub streaming_total_connections: Gauge,

    // ===== Streaming Protocol Metrics =====
    /// Messages sent via streaming
    pub streaming_messages_sent: Counter,
    /// Messages dropped (backpressure/errors)
    pub streaming_messages_dropped: Counter,
    /// Streaming error rate
    pub streaming_error_rate: Gauge,
    /// Streaming memory usage
    pub streaming_memory_usage_bytes: Gauge,
    /// Connection duration
    pub streaming_connection_duration: Histogram,
    /// Bytes transferred
    pub streaming_bytes_total: Counter,
    /// Streaming operation duration
    pub streaming_duration_seconds: HistogramVec,
    /// Streaming errors by type
    pub streaming_errors_total: IntCounterVec,
    /// Streaming throughput
    pub streaming_throughput_bytes_per_sec: Gauge,
    /// Streaming latency by operation
    pub streaming_latency_seconds: HistogramVec,

    // ===== System Resource Metrics (Jemalloc) =====
    /// Total allocated bytes via jemalloc
    pub jemalloc_allocated_bytes: Gauge,
    /// Active bytes in pages
    pub jemalloc_active_bytes: Gauge,
    /// Resident physical memory
    pub jemalloc_resident_bytes: Gauge,
    /// Metadata overhead
    pub jemalloc_metadata_bytes: Gauge,
    /// Total mapped bytes
    pub jemalloc_mapped_bytes: Gauge,
    /// Retained for future allocations
    pub jemalloc_retained_bytes: Gauge,
    /// Fragmentation ratio (active/allocated)
    pub jemalloc_fragmentation_ratio: Gauge,
    /// Metadata overhead ratio
    pub jemalloc_metadata_ratio: Gauge,
}

impl TransportMetrics {
    /// Initialize transport metrics with Prometheus registry
    pub fn new() -> anyhow::Result<Self> {
        let registry = Registry::new();

        // HTTP request metrics
        let http_requests_total = Counter::with_opts(
            Opts::new(
                "riptide_transport_http_requests_total",
                "Total number of HTTP requests",
            )
            .const_label("service", "riptide-transport"),
        )?;

        let http_request_duration = Histogram::with_opts(
            HistogramOpts::new(
                "riptide_transport_http_request_duration_seconds",
                "HTTP request duration",
            )
            .const_label("service", "riptide-transport")
            .buckets(vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ]),
        )?;

        let http_errors = Counter::with_opts(
            Opts::new("riptide_transport_http_errors_total", "HTTP errors")
                .const_label("service", "riptide-transport"),
        )?;

        // Connection metrics
        let active_connections = Gauge::with_opts(
            Opts::new(
                "riptide_transport_active_connections",
                "Number of active HTTP connections",
            )
            .const_label("service", "riptide-transport"),
        )?;

        let streaming_active_connections = Gauge::with_opts(
            Opts::new(
                "riptide_transport_streaming_active_connections",
                "Active streaming connections (WebSocket/SSE)",
            )
            .const_label("service", "riptide-transport"),
        )?;

        let streaming_total_connections = Gauge::with_opts(
            Opts::new(
                "riptide_transport_streaming_total_connections",
                "Total streaming connections created",
            )
            .const_label("service", "riptide-transport"),
        )?;

        // Streaming metrics
        let streaming_messages_sent = Counter::with_opts(
            Opts::new(
                "riptide_transport_streaming_messages_sent_total",
                "Total streaming messages sent",
            )
            .const_label("service", "riptide-transport"),
        )?;

        let streaming_messages_dropped = Counter::with_opts(
            Opts::new(
                "riptide_transport_streaming_messages_dropped_total",
                "Total streaming messages dropped",
            )
            .const_label("service", "riptide-transport"),
        )?;

        let streaming_error_rate = Gauge::with_opts(
            Opts::new(
                "riptide_transport_streaming_error_rate",
                "Streaming error rate (0.0 to 1.0)",
            )
            .const_label("service", "riptide-transport"),
        )?;

        let streaming_memory_usage_bytes = Gauge::with_opts(
            Opts::new(
                "riptide_transport_streaming_memory_usage_bytes",
                "Streaming memory usage in bytes",
            )
            .const_label("service", "riptide-transport"),
        )?;

        let streaming_connection_duration = Histogram::with_opts(
            HistogramOpts::new(
                "riptide_transport_streaming_connection_duration_seconds",
                "Streaming connection duration",
            )
            .const_label("service", "riptide-transport")
            .buckets(vec![
                1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 600.0, 1800.0, 3600.0,
            ]),
        )?;

        let streaming_bytes_total = Counter::with_opts(
            Opts::new(
                "riptide_transport_streaming_bytes_total",
                "Total bytes transferred via streaming",
            )
            .const_label("service", "riptide-transport"),
        )?;

        let streaming_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "riptide_transport_streaming_duration_seconds",
                "Streaming operation duration by status",
            )
            .const_label("service", "riptide-transport")
            .buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 30.0]),
            &["status"],
        )?;

        let streaming_errors_total = IntCounterVec::new(
            Opts::new(
                "riptide_transport_streaming_errors_total",
                "Streaming errors by type",
            )
            .const_label("service", "riptide-transport"),
            &["error_type"],
        )?;

        let streaming_throughput_bytes_per_sec = Gauge::with_opts(
            Opts::new(
                "riptide_transport_streaming_throughput_bytes_per_sec",
                "Streaming throughput in bytes/second",
            )
            .const_label("service", "riptide-transport"),
        )?;

        let streaming_latency_seconds = HistogramVec::new(
            HistogramOpts::new(
                "riptide_transport_streaming_latency_seconds",
                "Streaming operation latency",
            )
            .const_label("service", "riptide-transport")
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
            &["operation"],
        )?;

        // Jemalloc memory metrics
        let jemalloc_allocated_bytes = Gauge::with_opts(
            Opts::new(
                "riptide_transport_jemalloc_allocated_bytes",
                "Total bytes allocated by the application via jemalloc",
            )
            .const_label("service", "riptide-transport"),
        )?;

        let jemalloc_active_bytes = Gauge::with_opts(
            Opts::new(
                "riptide_transport_jemalloc_active_bytes",
                "Total bytes in active pages allocated by the application",
            )
            .const_label("service", "riptide-transport"),
        )?;

        let jemalloc_resident_bytes = Gauge::with_opts(
            Opts::new(
                "riptide_transport_jemalloc_resident_bytes",
                "Maximum bytes in physically resident data pages mapped",
            )
            .const_label("service", "riptide-transport"),
        )?;

        let jemalloc_metadata_bytes = Gauge::with_opts(
            Opts::new(
                "riptide_transport_jemalloc_metadata_bytes",
                "Total bytes dedicated to jemalloc metadata",
            )
            .const_label("service", "riptide-transport"),
        )?;

        let jemalloc_mapped_bytes = Gauge::with_opts(
            Opts::new(
                "riptide_transport_jemalloc_mapped_bytes",
                "Total bytes in chunks mapped on behalf of the application",
            )
            .const_label("service", "riptide-transport"),
        )?;

        let jemalloc_retained_bytes = Gauge::with_opts(
            Opts::new(
                "riptide_transport_jemalloc_retained_bytes",
                "Total bytes retained for future allocations",
            )
            .const_label("service", "riptide-transport"),
        )?;

        let jemalloc_fragmentation_ratio = Gauge::with_opts(
            Opts::new(
                "riptide_transport_jemalloc_fragmentation_ratio",
                "Memory fragmentation ratio (active/allocated)",
            )
            .const_label("service", "riptide-transport"),
        )?;

        let jemalloc_metadata_ratio = Gauge::with_opts(
            Opts::new(
                "riptide_transport_jemalloc_metadata_ratio",
                "Metadata overhead ratio (metadata/allocated)",
            )
            .const_label("service", "riptide-transport"),
        )?;

        // Register all metrics
        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration.clone()))?;
        registry.register(Box::new(http_errors.clone()))?;
        registry.register(Box::new(active_connections.clone()))?;
        registry.register(Box::new(streaming_active_connections.clone()))?;
        registry.register(Box::new(streaming_total_connections.clone()))?;
        registry.register(Box::new(streaming_messages_sent.clone()))?;
        registry.register(Box::new(streaming_messages_dropped.clone()))?;
        registry.register(Box::new(streaming_error_rate.clone()))?;
        registry.register(Box::new(streaming_memory_usage_bytes.clone()))?;
        registry.register(Box::new(streaming_connection_duration.clone()))?;
        registry.register(Box::new(streaming_bytes_total.clone()))?;
        registry.register(Box::new(streaming_duration_seconds.clone()))?;
        registry.register(Box::new(streaming_errors_total.clone()))?;
        registry.register(Box::new(streaming_throughput_bytes_per_sec.clone()))?;
        registry.register(Box::new(streaming_latency_seconds.clone()))?;
        registry.register(Box::new(jemalloc_allocated_bytes.clone()))?;
        registry.register(Box::new(jemalloc_active_bytes.clone()))?;
        registry.register(Box::new(jemalloc_resident_bytes.clone()))?;
        registry.register(Box::new(jemalloc_metadata_bytes.clone()))?;
        registry.register(Box::new(jemalloc_mapped_bytes.clone()))?;
        registry.register(Box::new(jemalloc_retained_bytes.clone()))?;
        registry.register(Box::new(jemalloc_fragmentation_ratio.clone()))?;
        registry.register(Box::new(jemalloc_metadata_ratio.clone()))?;

        info!("Transport metrics registry initialized with HTTP, streaming, and jemalloc metrics");

        Ok(Self {
            registry,
            http_requests_total,
            http_request_duration,
            http_errors,
            active_connections,
            streaming_active_connections,
            streaming_total_connections,
            streaming_messages_sent,
            streaming_messages_dropped,
            streaming_error_rate,
            streaming_memory_usage_bytes,
            streaming_connection_duration,
            streaming_bytes_total,
            streaming_duration_seconds,
            streaming_errors_total,
            streaming_throughput_bytes_per_sec,
            streaming_latency_seconds,
            jemalloc_allocated_bytes,
            jemalloc_active_bytes,
            jemalloc_resident_bytes,
            jemalloc_metadata_bytes,
            jemalloc_mapped_bytes,
            jemalloc_retained_bytes,
            jemalloc_fragmentation_ratio,
            jemalloc_metadata_ratio,
        })
    }

    /// Update jemalloc memory statistics
    #[cfg(feature = "jemalloc")]
    pub fn update_jemalloc_stats(&self) {
        if let Some(stats) = JemallocStats::collect() {
            self.jemalloc_allocated_bytes.set(stats.allocated as f64);
            self.jemalloc_active_bytes.set(stats.active as f64);
            self.jemalloc_resident_bytes.set(stats.resident as f64);
            self.jemalloc_metadata_bytes.set(stats.metadata as f64);
            self.jemalloc_mapped_bytes.set(stats.mapped as f64);
            self.jemalloc_retained_bytes.set(stats.retained as f64);
            self.jemalloc_fragmentation_ratio
                .set(stats.fragmentation_ratio());
            self.jemalloc_metadata_ratio
                .set(stats.metadata_overhead_ratio());

            tracing::debug!(
                allocated_mb = stats.allocated_mb(),
                resident_mb = stats.resident_mb(),
                metadata_mb = stats.metadata_mb(),
                fragmentation = stats.fragmentation_ratio(),
                "jemalloc memory stats updated"
            );
        }
    }

    /// Record HTTP request
    pub fn record_http_request(&self, _method: &str, _path: &str, status: u16, duration: f64) {
        self.http_requests_total.inc();
        self.http_request_duration.observe(duration);

        if status >= 400 {
            self.http_errors.inc();
        }
    }

    /// Update active connections
    pub fn update_active_connections(&self, count: i64) {
        self.active_connections.set(count as f64);
    }

    /// Update streaming metrics from GlobalStreamingMetrics
    pub fn update_streaming_metrics(
        &self,
        streaming_metrics: &crate::streaming::GlobalStreamingMetrics,
    ) {
        self.streaming_active_connections
            .set(streaming_metrics.active_connections as f64);
        self.streaming_total_connections
            .set(streaming_metrics.total_connections as f64);
        self.streaming_error_rate.set(streaming_metrics.error_rate);
        self.streaming_memory_usage_bytes
            .set(streaming_metrics.memory_usage_bytes as f64);
    }

    /// Record streaming message sent
    pub fn record_streaming_message_sent(&self) {
        self.streaming_messages_sent.inc();
    }

    /// Record streaming message dropped
    pub fn record_streaming_message_dropped(&self) {
        self.streaming_messages_dropped.inc();
    }

    /// Record streaming connection duration
    pub fn record_streaming_connection_duration(&self, duration_seconds: f64) {
        self.streaming_connection_duration.observe(duration_seconds);
    }

    /// Record streaming bytes transferred
    pub fn record_streaming_bytes(&self, bytes: usize) {
        self.streaming_bytes_total.inc_by(bytes as f64);
    }

    /// Record streaming operation duration with status
    pub fn record_streaming_duration(&self, duration_seconds: f64, success: bool) {
        let status = if success { "success" } else { "failure" };
        self.streaming_duration_seconds
            .with_label_values(&[status])
            .observe(duration_seconds);
    }

    /// Record streaming error by type
    pub fn record_streaming_error_by_type(&self, error_type: &str) {
        self.streaming_errors_total
            .with_label_values(&[error_type])
            .inc();
    }

    /// Update streaming throughput
    pub fn update_streaming_throughput(&self, bytes_per_sec: f64) {
        self.streaming_throughput_bytes_per_sec.set(bytes_per_sec);
    }

    /// Record streaming operation latency
    pub fn record_streaming_latency(&self, operation: &str, duration_seconds: f64) {
        self.streaming_latency_seconds
            .with_label_values(&[operation])
            .observe(duration_seconds);
    }
}

/// Phase timing tracker for structured logging and metrics
#[derive(Debug)]
pub struct PhaseTimer {
    start_time: Instant,
    phase_name: String,
}

impl PhaseTimer {
    /// Start timing a phase
    pub fn start(phase_name: String) -> Self {
        info!(phase = %phase_name, "Phase started");
        Self {
            start_time: Instant::now(),
            phase_name,
        }
    }

    /// End timing and log results
    pub fn end(self) {
        let duration = self.start_time.elapsed();
        info!(
            phase = %self.phase_name,
            duration_ms = duration.as_millis(),
            duration_seconds = duration.as_secs_f64(),
            "Phase completed"
        );
    }
}

/// Create Prometheus metric layer for Axum
pub fn create_metrics_layer() -> anyhow::Result<(PrometheusMetricLayer<'static>, PrometheusHandle)>
{
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
    info!("Prometheus metrics layer created");
    Ok((prometheus_layer, metric_handle))
}
