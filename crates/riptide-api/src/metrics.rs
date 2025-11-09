//! # DEPRECATED MODULE - Sprint 4.5 Metrics Split
//!
//! **This module is deprecated and will be removed in a future release.**
//!
//! Use the new split metrics architecture instead:
//! - `riptide_facade::metrics::BusinessMetrics` for business domain metrics
//!   (extraction quality, gate decisions, PDF processing, cache effectiveness)
//! - `crate::metrics_transport::TransportMetrics` for transport-level metrics
//!   (HTTP requests, WebSocket connections, streaming protocols)
//! - `crate::metrics_integration::CombinedMetrics` for unified /metrics endpoint
//!
//! Migration Guide:
//! ```rust
//! // OLD (deprecated):
//! let metrics = Arc::new(RipTideMetrics::new()?);
//! metrics.gate_decisions_raw.inc();
//! metrics.http_requests_total.inc();
//!
//! // NEW (recommended):
//! let business_metrics = Arc::new(BusinessMetrics::new()?);
//! let transport_metrics = Arc::new(TransportMetrics::new()?);
//! let combined = Arc::new(CombinedMetrics::new(business_metrics.clone(), transport_metrics.clone())?);
//!
//! // Business domain operations use business_metrics:
//! combined.record_gate_decision("raw");
//!
//! // Transport operations use transport_metrics:
//! combined.record_http_request("GET", "/api/extract", 200, 0.150);
//!
//! // Serve unified /metrics endpoint:
//! let all_metrics = combined.gather_all();
//! ```
//!
//! This split provides:
//! - Clear separation of concerns
//! - Better organization and maintainability
//! - Easier testing and mocking
//! - Improved performance through targeted metric collection

#![allow(dead_code)]
#![deprecated(
    since = "4.5.0",
    note = "Use BusinessMetrics + TransportMetrics + CombinedMetrics instead. See module docs for migration guide."
)]

use axum_prometheus::{metrics_exporter_prometheus::PrometheusHandle, PrometheusMetricLayer};
use prometheus::{
    Counter, Gauge, GaugeVec, Histogram, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry,
};
use riptide_pdf::PdfMetricsCollector;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, warn};

#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
use crate::jemalloc_stats::JemallocStats;

/// Metrics collection and management for RipTide API
///
/// **DEPRECATED**: Use `BusinessMetrics` + `TransportMetrics` + `CombinedMetrics` instead
#[deprecated(
    since = "4.5.0",
    note = "Split into BusinessMetrics (facade) and TransportMetrics (API). Use CombinedMetrics for unified endpoint."
)]
#[allow(deprecated)]
#[derive(Debug)]
pub struct RipTideMetrics {
    /// Prometheus registry for metrics
    pub registry: Registry,

    /// HTTP request counter
    pub http_requests_total: Counter,

    /// HTTP request duration histogram
    pub http_request_duration: Histogram,

    /// Active connections gauge
    pub active_connections: Gauge,

    /// Cache hit rate gauge
    pub cache_hit_rate: Gauge,

    /// Phase timing histograms
    pub fetch_phase_duration: Histogram,
    pub gate_phase_duration: Histogram,
    pub wasm_phase_duration: Histogram,
    pub render_phase_duration: Histogram,

    /// Gate decision counters
    pub gate_decisions_raw: Counter,
    pub gate_decisions_probes_first: Counter,
    pub gate_decisions_headless: Counter,
    pub gate_decisions_cached: Counter,

    /// Error counters
    pub errors_total: Counter,
    pub redis_errors: Counter,
    pub wasm_errors: Counter,
    pub http_errors: Counter,

    /// Streaming metrics
    pub streaming_active_connections: Gauge,
    pub streaming_total_connections: Gauge,
    pub streaming_messages_sent: Counter,
    pub streaming_messages_dropped: Counter,
    pub streaming_error_rate: Gauge,
    pub streaming_memory_usage_bytes: Gauge,
    pub streaming_connection_duration: Histogram,
    pub streaming_bytes_total: Counter,
    pub streaming_duration_seconds: HistogramVec,
    pub streaming_errors_total: IntCounterVec,
    pub streaming_throughput_bytes_per_sec: Gauge,
    pub streaming_latency_seconds: HistogramVec,

    /// Spider crawling metrics
    pub spider_crawls_total: Counter,
    pub spider_pages_crawled: Counter,
    pub spider_pages_failed: Counter,
    pub spider_active_crawls: Gauge,
    pub spider_frontier_size: Gauge,
    pub spider_crawl_duration: Histogram,
    pub spider_pages_per_second: Gauge,

    /// PDF processing metrics
    pub pdf_total_processed: Counter,
    pub pdf_total_failed: Counter,
    pub pdf_memory_limit_failures: Counter,
    pub pdf_processing_time: Histogram,
    pub pdf_peak_memory_mb: Gauge,
    pub pdf_pages_per_pdf: Gauge,
    /// Reserved for PDF memory spike handling - will be recorded when feature is implemented
    #[allow(dead_code)]
    pub pdf_memory_spikes_handled: Counter,
    /// Reserved for PDF cleanup operations - will be recorded when feature is implemented
    #[allow(dead_code)]
    pub pdf_cleanup_operations: Counter,

    /// WASM memory metrics (always present but only used with wasm-extractor feature)
    pub wasm_memory_pages: Gauge,
    #[allow(dead_code)] // Only used when wasm-extractor feature is enabled
    pub wasm_grow_failed_total: Counter,
    pub wasm_peak_memory_pages: Gauge,
    pub wasm_cold_start_time_ms: Gauge,
    /// Reserved for WASM AOT cache hit tracking - will be recorded when caching is implemented
    #[allow(dead_code)]
    pub wasm_aot_cache_hits: Counter,
    /// Reserved for WASM AOT cache miss tracking - will be recorded when caching is implemented
    #[allow(dead_code)]
    pub wasm_aot_cache_misses: Counter,

    /// Worker management metrics (Phase 4B Feature 5)
    pub worker_pool_size: Gauge,
    pub worker_pool_healthy: Gauge,
    pub worker_jobs_submitted: Counter,
    pub worker_jobs_completed: Counter,
    pub worker_jobs_failed: Counter,
    pub worker_jobs_retried: Counter,
    /// Reserved for worker processing time tracking - will be recorded when worker metrics are wired up
    #[allow(dead_code)]
    pub worker_processing_time: Histogram,
    pub worker_queue_depth: Gauge,

    // ===== WEEK 1 PHASE 1B: Comprehensive Metrics System (30+ new metrics) =====

    // Gate Decision Enhanced Metrics
    pub gate_decision_total: IntCounterVec, // By decision type (raw/probes_first/headless)
    pub gate_score_histogram: Histogram,    // Score distribution (0.0-1.0)
    pub gate_feature_text_ratio: Histogram, // Text density ratio
    pub gate_feature_script_density: Histogram, // JavaScript density ratio
    pub gate_feature_spa_markers: IntCounterVec, // SPA marker count
    pub gate_decision_duration_ms: Histogram, // Gate latency in milliseconds

    // Extraction Quality Metrics
    pub extraction_quality_score: HistogramVec, // Quality score by mode (0-100)
    pub extraction_quality_success_rate: GaugeVec, // Success rate by mode
    pub extraction_content_length: HistogramVec, // Content length by mode
    pub extraction_links_found: HistogramVec,   // Links count by mode
    pub extraction_images_found: HistogramVec,  // Images count by mode
    pub extraction_has_author: IntCounterVec,   // Author presence by mode
    pub extraction_has_date: IntCounterVec,     // Date presence by mode

    // Extraction Performance
    pub extraction_duration_by_mode: HistogramVec, // Duration by extraction mode
    pub extraction_fallback_triggered: IntCounterVec, // Fallback events (from_mode, to_mode, reason)

    // Pipeline Phase Timing (additional phases)
    /// Reserved for pipeline phase tracking - used via record_pipeline_phase_ms()
    #[allow(dead_code)]
    pub pipeline_phase_gate_analysis_ms: Histogram, // Gate analysis phase
    /// Reserved for pipeline phase tracking - used via record_pipeline_phase_ms()
    #[allow(dead_code)]
    pub pipeline_phase_extraction_ms: Histogram, // Extraction phase

    // Jemalloc Memory Metrics (P2 enhancement)
    pub jemalloc_allocated_bytes: Gauge, // Total allocated bytes
    pub jemalloc_active_bytes: Gauge,    // Active bytes in pages
    pub jemalloc_resident_bytes: Gauge,  // Resident physical memory
    pub jemalloc_metadata_bytes: Gauge,  // Metadata overhead
    pub jemalloc_mapped_bytes: Gauge,    // Total mapped bytes
    pub jemalloc_retained_bytes: Gauge,  // Retained for future allocs
    pub jemalloc_fragmentation_ratio: Gauge, // Active/Allocated ratio
    pub jemalloc_metadata_ratio: Gauge,  // Metadata overhead ratio
}

#[allow(deprecated)]
impl RipTideMetrics {
    /// Initialize metrics with Prometheus registry
    pub fn new() -> anyhow::Result<Self> {
        let registry = Registry::new();

        // HTTP request metrics
        let http_requests_total = Counter::with_opts(
            Opts::new(
                "riptide_http_requests_total",
                "Total number of HTTP requests",
            )
            .const_label("service", "riptide-api"),
        )?;

        let http_request_duration = Histogram::with_opts(
            HistogramOpts::new(
                "riptide_http_request_duration_seconds",
                "HTTP request duration",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ]),
        )?;

        // System metrics
        let active_connections = Gauge::with_opts(
            Opts::new("riptide_active_connections", "Number of active connections")
                .const_label("service", "riptide-api"),
        )?;

        let cache_hit_rate = Gauge::with_opts(
            Opts::new("riptide_cache_hit_rate", "Cache hit rate (0.0 to 1.0)")
                .const_label("service", "riptide-api"),
        )?;

        // Phase timing metrics
        let fetch_phase_duration = Histogram::with_opts(
            HistogramOpts::new(
                "riptide_fetch_phase_duration_seconds",
                "Fetch phase duration",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0]),
        )?;

        let gate_phase_duration = Histogram::with_opts(
            HistogramOpts::new(
                "riptide_gate_phase_duration_seconds",
                "Gate analysis phase duration",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5]),
        )?;

        let wasm_phase_duration = Histogram::with_opts(
            HistogramOpts::new(
                "riptide_wasm_phase_duration_seconds",
                "WASM extraction phase duration",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0]),
        )?;

        let render_phase_duration = Histogram::with_opts(
            HistogramOpts::new(
                "riptide_render_phase_duration_seconds",
                "Render phase duration",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0]),
        )?;

        // Gate decision counters
        let gate_decisions_raw = Counter::with_opts(
            Opts::new("riptide_gate_decisions_raw_total", "Raw gate decisions")
                .const_label("service", "riptide-api"),
        )?;

        let gate_decisions_probes_first = Counter::with_opts(
            Opts::new(
                "riptide_gate_decisions_probes_first_total",
                "Probes first gate decisions",
            )
            .const_label("service", "riptide-api"),
        )?;

        let gate_decisions_headless = Counter::with_opts(
            Opts::new(
                "riptide_gate_decisions_headless_total",
                "Headless gate decisions",
            )
            .const_label("service", "riptide-api"),
        )?;

        let gate_decisions_cached = Counter::with_opts(
            Opts::new(
                "riptide_gate_decisions_cached_total",
                "Cached gate decisions",
            )
            .const_label("service", "riptide-api"),
        )?;

        // Error counters
        let errors_total = Counter::with_opts(
            Opts::new("riptide_errors_total", "Total errors").const_label("service", "riptide-api"),
        )?;

        let redis_errors = Counter::with_opts(
            Opts::new("riptide_redis_errors_total", "Redis errors")
                .const_label("service", "riptide-api"),
        )?;

        let wasm_errors = Counter::with_opts(
            Opts::new("riptide_wasm_errors_total", "WASM errors")
                .const_label("service", "riptide-api"),
        )?;

        let http_errors = Counter::with_opts(
            Opts::new("riptide_http_errors_total", "HTTP errors")
                .const_label("service", "riptide-api"),
        )?;

        // Streaming metrics
        let streaming_active_connections = Gauge::with_opts(
            Opts::new(
                "riptide_streaming_active_connections",
                "Active streaming connections",
            )
            .const_label("service", "riptide-api"),
        )?;

        let streaming_total_connections = Gauge::with_opts(
            Opts::new(
                "riptide_streaming_total_connections",
                "Total streaming connections created",
            )
            .const_label("service", "riptide-api"),
        )?;

        let streaming_messages_sent = Counter::with_opts(
            Opts::new(
                "riptide_streaming_messages_sent_total",
                "Total streaming messages sent",
            )
            .const_label("service", "riptide-api"),
        )?;

        let streaming_messages_dropped = Counter::with_opts(
            Opts::new(
                "riptide_streaming_messages_dropped_total",
                "Total streaming messages dropped",
            )
            .const_label("service", "riptide-api"),
        )?;

        let streaming_error_rate = Gauge::with_opts(
            Opts::new(
                "riptide_streaming_error_rate",
                "Streaming error rate (0.0 to 1.0)",
            )
            .const_label("service", "riptide-api"),
        )?;

        let streaming_memory_usage_bytes = Gauge::with_opts(
            Opts::new(
                "riptide_streaming_memory_usage_bytes",
                "Streaming memory usage in bytes",
            )
            .const_label("service", "riptide-api"),
        )?;

        let streaming_connection_duration = Histogram::with_opts(
            HistogramOpts::new(
                "riptide_streaming_connection_duration_seconds",
                "Streaming connection duration",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![
                0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 300.0,
            ]),
        )?;

        // Enhanced streaming metrics for comprehensive observability
        let streaming_bytes_total = Counter::with_opts(
            Opts::new(
                "riptide_streaming_bytes_total",
                "Total bytes transferred in streaming operations",
            )
            .const_label("service", "riptide-api"),
        )?;

        let streaming_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "riptide_streaming_duration_seconds",
                "Stream processing duration by status (success/failure)",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![
                0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0, 120.0,
            ]),
            &["status"],
        )?;

        let streaming_errors_total = IntCounterVec::new(
            Opts::new(
                "riptide_streaming_errors_total",
                "Total streaming errors by error type",
            )
            .const_label("service", "riptide-api"),
            &["error_type"],
        )?;

        let streaming_throughput_bytes_per_sec = Gauge::with_opts(
            Opts::new(
                "riptide_streaming_throughput_bytes_per_sec",
                "Current streaming throughput in bytes per second",
            )
            .const_label("service", "riptide-api"),
        )?;

        let streaming_latency_seconds = HistogramVec::new(
            HistogramOpts::new(
                "riptide_streaming_latency_seconds",
                "Streaming operation latency percentiles by operation type",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![
                0.0001, 0.0005, 0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0,
            ]),
            &["operation"],
        )?;

        // Spider crawling metrics
        let spider_crawls_total = Counter::with_opts(
            Opts::new(
                "riptide_spider_crawls_total",
                "Total spider crawl operations",
            )
            .const_label("service", "riptide-api"),
        )?;

        let spider_pages_crawled = Counter::with_opts(
            Opts::new(
                "riptide_spider_pages_crawled_total",
                "Total pages crawled by spider",
            )
            .const_label("service", "riptide-api"),
        )?;

        let spider_pages_failed = Counter::with_opts(
            Opts::new(
                "riptide_spider_pages_failed_total",
                "Total pages failed by spider",
            )
            .const_label("service", "riptide-api"),
        )?;

        let spider_active_crawls = Gauge::with_opts(
            Opts::new(
                "riptide_spider_active_crawls",
                "Number of active spider crawls",
            )
            .const_label("service", "riptide-api"),
        )?;

        let spider_frontier_size = Gauge::with_opts(
            Opts::new(
                "riptide_spider_frontier_size",
                "Current spider frontier queue size",
            )
            .const_label("service", "riptide-api"),
        )?;

        let spider_crawl_duration = Histogram::with_opts(
            HistogramOpts::new(
                "riptide_spider_crawl_duration_seconds",
                "Spider crawl operation duration",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![0.1, 1.0, 5.0, 15.0, 30.0, 60.0, 300.0, 900.0, 1800.0]),
        )?;

        let spider_pages_per_second = Gauge::with_opts(
            Opts::new(
                "riptide_spider_pages_per_second",
                "Spider crawl rate in pages per second",
            )
            .const_label("service", "riptide-api"),
        )?;

        // PDF processing metrics
        let pdf_total_processed = Counter::with_opts(
            Opts::new(
                "riptide_pdf_total_processed",
                "Total PDF documents processed",
            )
            .const_label("service", "riptide-api"),
        )?;

        let pdf_total_failed = Counter::with_opts(
            Opts::new("riptide_pdf_total_failed", "Total PDF processing failures")
                .const_label("service", "riptide-api"),
        )?;

        let pdf_memory_limit_failures = Counter::with_opts(
            Opts::new(
                "riptide_pdf_memory_limit_failures",
                "PDF processing failures due to memory limits",
            )
            .const_label("service", "riptide-api"),
        )?;

        let pdf_processing_time = Histogram::with_opts(
            HistogramOpts::new(
                "riptide_pdf_processing_time_seconds",
                "PDF processing time in seconds",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0]),
        )?;

        let pdf_peak_memory_mb = Gauge::with_opts(
            Opts::new(
                "riptide_pdf_peak_memory_mb",
                "PDF processing peak memory usage in MB",
            )
            .const_label("service", "riptide-api"),
        )?;

        let pdf_pages_per_pdf = Gauge::with_opts(
            Opts::new(
                "riptide_pdf_pages_per_pdf",
                "Average pages per PDF processed",
            )
            .const_label("service", "riptide-api"),
        )?;

        let pdf_memory_spikes_handled = Counter::with_opts(
            Opts::new(
                "riptide_pdf_memory_spikes_handled",
                "Number of PDF memory spikes handled",
            )
            .const_label("service", "riptide-api"),
        )?;

        let pdf_cleanup_operations = Counter::with_opts(
            Opts::new(
                "riptide_pdf_cleanup_operations",
                "Number of PDF memory cleanup operations performed",
            )
            .const_label("service", "riptide-api"),
        )?;

        // WASM memory metrics
        let wasm_memory_pages = Gauge::with_opts(
            Opts::new(
                "riptide_wasm_memory_pages",
                "Current WASM memory usage in pages (64KB each)",
            )
            .const_label("service", "riptide-api"),
        )?;

        let wasm_grow_failed_total = Counter::with_opts(
            Opts::new(
                "riptide_wasm_grow_failed_total",
                "Total WASM memory growth failures",
            )
            .const_label("service", "riptide-api"),
        )?;

        let wasm_peak_memory_pages = Gauge::with_opts(
            Opts::new(
                "riptide_wasm_peak_memory_pages",
                "Peak WASM memory usage in pages",
            )
            .const_label("service", "riptide-api"),
        )?;

        let wasm_cold_start_time_ms = Gauge::with_opts(
            Opts::new(
                "riptide_wasm_cold_start_time_ms",
                "WASM cold start time in milliseconds",
            )
            .const_label("service", "riptide-api"),
        )?;

        let wasm_aot_cache_hits = Counter::with_opts(
            Opts::new("riptide_wasm_aot_cache_hits_total", "WASM AOT cache hits")
                .const_label("service", "riptide-api"),
        )?;

        let wasm_aot_cache_misses = Counter::with_opts(
            Opts::new(
                "riptide_wasm_aot_cache_misses_total",
                "WASM AOT cache misses",
            )
            .const_label("service", "riptide-api"),
        )?;

        // Worker management metrics (Phase 4B Feature 5)
        let worker_pool_size = Gauge::with_opts(
            Opts::new(
                "riptide_worker_pool_size",
                "Total number of workers in pool",
            )
            .const_label("service", "riptide-api"),
        )?;

        let worker_pool_healthy = Gauge::with_opts(
            Opts::new(
                "riptide_worker_pool_healthy",
                "Number of healthy workers in pool",
            )
            .const_label("service", "riptide-api"),
        )?;

        let worker_jobs_submitted = Counter::with_opts(
            Opts::new(
                "riptide_worker_jobs_submitted_total",
                "Total number of jobs submitted to workers",
            )
            .const_label("service", "riptide-api"),
        )?;

        let worker_jobs_completed = Counter::with_opts(
            Opts::new(
                "riptide_worker_jobs_completed_total",
                "Total number of jobs completed by workers",
            )
            .const_label("service", "riptide-api"),
        )?;

        let worker_jobs_failed = Counter::with_opts(
            Opts::new(
                "riptide_worker_jobs_failed_total",
                "Total number of failed worker jobs",
            )
            .const_label("service", "riptide-api"),
        )?;

        let worker_jobs_retried = Counter::with_opts(
            Opts::new(
                "riptide_worker_jobs_retried_total",
                "Total number of retried worker jobs",
            )
            .const_label("service", "riptide-api"),
        )?;

        let worker_processing_time = Histogram::with_opts(
            HistogramOpts::new(
                "riptide_worker_processing_time_seconds",
                "Worker job processing time in seconds",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0]),
        )?;

        let worker_queue_depth = Gauge::with_opts(
            Opts::new(
                "riptide_worker_queue_depth",
                "Total number of jobs in worker queues",
            )
            .const_label("service", "riptide-api"),
        )?;

        // ===== WEEK 1 PHASE 1B: Initialize Comprehensive Metrics =====

        // Gate Decision Enhanced Metrics
        let gate_decision_total = IntCounterVec::new(
            Opts::new(
                "riptide_gate_decision_total",
                "Gate decisions by type (raw/probes_first/headless)",
            )
            .const_label("service", "riptide-api"),
            &["decision"],
        )?;

        let gate_score_histogram = Histogram::with_opts(
            HistogramOpts::new("riptide_gate_score", "Gate score distribution (0.0-1.0)")
                .const_label("service", "riptide-api")
                .buckets(vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]),
        )?;

        let gate_feature_text_ratio = Histogram::with_opts(
            HistogramOpts::new("riptide_gate_feature_text_ratio", "Text to HTML size ratio")
                .const_label("service", "riptide-api")
                .buckets(vec![0.0, 0.05, 0.1, 0.15, 0.2, 0.3, 0.4, 0.5, 0.75, 1.0]),
        )?;

        let gate_feature_script_density = Histogram::with_opts(
            HistogramOpts::new(
                "riptide_gate_feature_script_density",
                "JavaScript to HTML size ratio",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![0.0, 0.05, 0.1, 0.2, 0.3, 0.4, 0.5, 0.75, 1.0]),
        )?;

        let gate_feature_spa_markers = IntCounterVec::new(
            Opts::new(
                "riptide_gate_feature_spa_markers_total",
                "SPA marker detection counts",
            )
            .const_label("service", "riptide-api"),
            &["marker_count"],
        )?;

        let gate_decision_duration_ms = Histogram::with_opts(
            HistogramOpts::new(
                "riptide_gate_decision_duration_milliseconds",
                "Gate decision latency in milliseconds",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 25.0, 50.0, 100.0]),
        )?;

        // Extraction Quality Metrics
        let extraction_quality_score = HistogramVec::new(
            HistogramOpts::new(
                "riptide_extraction_quality_score",
                "Extraction quality score by mode (0-100)",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![0.0, 20.0, 40.0, 60.0, 70.0, 80.0, 90.0, 95.0, 100.0]),
            &["mode"],
        )?;

        let extraction_quality_success_rate = GaugeVec::new(
            Opts::new(
                "riptide_extraction_quality_success_rate",
                "Extraction success rate by mode (0.0-1.0)",
            )
            .const_label("service", "riptide-api"),
            &["mode"],
        )?;

        let extraction_content_length = HistogramVec::new(
            HistogramOpts::new(
                "riptide_extraction_content_length_bytes",
                "Extracted content length by mode",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![
                100.0, 500.0, 1000.0, 5000.0, 10000.0, 50000.0, 100000.0, 500000.0,
            ]),
            &["mode"],
        )?;

        let extraction_links_found = HistogramVec::new(
            HistogramOpts::new(
                "riptide_extraction_links_found",
                "Number of links extracted by mode",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![0.0, 1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 200.0, 500.0]),
            &["mode"],
        )?;

        let extraction_images_found = HistogramVec::new(
            HistogramOpts::new(
                "riptide_extraction_images_found",
                "Number of images extracted by mode",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![0.0, 1.0, 5.0, 10.0, 20.0, 50.0, 100.0]),
            &["mode"],
        )?;

        let extraction_has_author = IntCounterVec::new(
            Opts::new(
                "riptide_extraction_has_author_total",
                "Extractions with author metadata by mode",
            )
            .const_label("service", "riptide-api"),
            &["mode", "has_author"],
        )?;

        let extraction_has_date = IntCounterVec::new(
            Opts::new(
                "riptide_extraction_has_date_total",
                "Extractions with publication date by mode",
            )
            .const_label("service", "riptide-api"),
            &["mode", "has_date"],
        )?;

        // Extraction Performance
        let extraction_duration_by_mode = HistogramVec::new(
            HistogramOpts::new(
                "riptide_extraction_duration_by_mode_seconds",
                "Extraction duration by mode",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0]),
            &["mode"],
        )?;

        let extraction_fallback_triggered = IntCounterVec::new(
            Opts::new(
                "riptide_extraction_fallback_triggered_total",
                "Extraction fallback events",
            )
            .const_label("service", "riptide-api"),
            &["from_mode", "to_mode", "reason"],
        )?;

        // Pipeline Phase Timing
        let pipeline_phase_gate_analysis_ms = Histogram::with_opts(
            HistogramOpts::new(
                "riptide_pipeline_phase_gate_analysis_milliseconds",
                "Gate analysis phase duration",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 25.0, 50.0, 100.0]),
        )?;

        let pipeline_phase_extraction_ms = Histogram::with_opts(
            HistogramOpts::new(
                "riptide_pipeline_phase_extraction_milliseconds",
                "Extraction phase duration",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![
                10.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0,
            ]),
        )?;

        // Jemalloc Memory Metrics
        let jemalloc_allocated_bytes = Gauge::with_opts(
            Opts::new(
                "riptide_jemalloc_allocated_bytes",
                "Total bytes allocated by the application via jemalloc",
            )
            .const_label("service", "riptide-api"),
        )?;

        let jemalloc_active_bytes = Gauge::with_opts(
            Opts::new(
                "riptide_jemalloc_active_bytes",
                "Total bytes in active pages allocated by the application",
            )
            .const_label("service", "riptide-api"),
        )?;

        let jemalloc_resident_bytes = Gauge::with_opts(
            Opts::new(
                "riptide_jemalloc_resident_bytes",
                "Maximum bytes in physically resident data pages mapped",
            )
            .const_label("service", "riptide-api"),
        )?;

        let jemalloc_metadata_bytes = Gauge::with_opts(
            Opts::new(
                "riptide_jemalloc_metadata_bytes",
                "Total bytes dedicated to jemalloc metadata",
            )
            .const_label("service", "riptide-api"),
        )?;

        let jemalloc_mapped_bytes = Gauge::with_opts(
            Opts::new(
                "riptide_jemalloc_mapped_bytes",
                "Total bytes in chunks mapped on behalf of the application",
            )
            .const_label("service", "riptide-api"),
        )?;

        let jemalloc_retained_bytes = Gauge::with_opts(
            Opts::new(
                "riptide_jemalloc_retained_bytes",
                "Total bytes retained for future allocations",
            )
            .const_label("service", "riptide-api"),
        )?;

        let jemalloc_fragmentation_ratio = Gauge::with_opts(
            Opts::new(
                "riptide_jemalloc_fragmentation_ratio",
                "Memory fragmentation ratio (active/allocated)",
            )
            .const_label("service", "riptide-api"),
        )?;

        let jemalloc_metadata_ratio = Gauge::with_opts(
            Opts::new(
                "riptide_jemalloc_metadata_ratio",
                "Metadata overhead ratio (metadata/allocated)",
            )
            .const_label("service", "riptide-api"),
        )?;

        // Register all metrics
        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration.clone()))?;
        registry.register(Box::new(active_connections.clone()))?;
        registry.register(Box::new(cache_hit_rate.clone()))?;
        registry.register(Box::new(fetch_phase_duration.clone()))?;
        registry.register(Box::new(gate_phase_duration.clone()))?;
        registry.register(Box::new(wasm_phase_duration.clone()))?;
        registry.register(Box::new(render_phase_duration.clone()))?;
        registry.register(Box::new(gate_decisions_raw.clone()))?;
        registry.register(Box::new(gate_decisions_probes_first.clone()))?;
        registry.register(Box::new(gate_decisions_headless.clone()))?;
        registry.register(Box::new(gate_decisions_cached.clone()))?;
        registry.register(Box::new(errors_total.clone()))?;
        registry.register(Box::new(redis_errors.clone()))?;
        registry.register(Box::new(wasm_errors.clone()))?;
        registry.register(Box::new(http_errors.clone()))?;
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
        registry.register(Box::new(spider_crawls_total.clone()))?;
        registry.register(Box::new(spider_pages_crawled.clone()))?;
        registry.register(Box::new(spider_pages_failed.clone()))?;
        registry.register(Box::new(spider_active_crawls.clone()))?;
        registry.register(Box::new(spider_frontier_size.clone()))?;
        registry.register(Box::new(spider_crawl_duration.clone()))?;
        registry.register(Box::new(spider_pages_per_second.clone()))?;
        registry.register(Box::new(pdf_total_processed.clone()))?;
        registry.register(Box::new(pdf_total_failed.clone()))?;
        registry.register(Box::new(pdf_memory_limit_failures.clone()))?;
        registry.register(Box::new(pdf_processing_time.clone()))?;
        registry.register(Box::new(pdf_peak_memory_mb.clone()))?;
        registry.register(Box::new(pdf_pages_per_pdf.clone()))?;
        registry.register(Box::new(pdf_memory_spikes_handled.clone()))?;
        registry.register(Box::new(pdf_cleanup_operations.clone()))?;
        registry.register(Box::new(wasm_memory_pages.clone()))?;
        registry.register(Box::new(wasm_grow_failed_total.clone()))?;
        registry.register(Box::new(wasm_peak_memory_pages.clone()))?;
        registry.register(Box::new(wasm_cold_start_time_ms.clone()))?;
        registry.register(Box::new(wasm_aot_cache_hits.clone()))?;
        registry.register(Box::new(wasm_aot_cache_misses.clone()))?;
        registry.register(Box::new(worker_pool_size.clone()))?;
        registry.register(Box::new(worker_pool_healthy.clone()))?;
        registry.register(Box::new(worker_jobs_submitted.clone()))?;
        registry.register(Box::new(worker_jobs_completed.clone()))?;
        registry.register(Box::new(worker_jobs_failed.clone()))?;
        registry.register(Box::new(worker_jobs_retried.clone()))?;
        registry.register(Box::new(worker_processing_time.clone()))?;
        registry.register(Box::new(worker_queue_depth.clone()))?;

        // Register WEEK 1 PHASE 1B metrics
        registry.register(Box::new(gate_decision_total.clone()))?;
        registry.register(Box::new(gate_score_histogram.clone()))?;
        registry.register(Box::new(gate_feature_text_ratio.clone()))?;
        registry.register(Box::new(gate_feature_script_density.clone()))?;
        registry.register(Box::new(gate_feature_spa_markers.clone()))?;
        registry.register(Box::new(gate_decision_duration_ms.clone()))?;
        registry.register(Box::new(extraction_quality_score.clone()))?;
        registry.register(Box::new(extraction_quality_success_rate.clone()))?;
        registry.register(Box::new(extraction_content_length.clone()))?;
        registry.register(Box::new(extraction_links_found.clone()))?;
        registry.register(Box::new(extraction_images_found.clone()))?;
        registry.register(Box::new(extraction_has_author.clone()))?;
        registry.register(Box::new(extraction_has_date.clone()))?;
        registry.register(Box::new(extraction_duration_by_mode.clone()))?;
        registry.register(Box::new(extraction_fallback_triggered.clone()))?;
        registry.register(Box::new(pipeline_phase_gate_analysis_ms.clone()))?;
        registry.register(Box::new(pipeline_phase_extraction_ms.clone()))?;

        // Register jemalloc memory metrics
        registry.register(Box::new(jemalloc_allocated_bytes.clone()))?;
        registry.register(Box::new(jemalloc_active_bytes.clone()))?;
        registry.register(Box::new(jemalloc_resident_bytes.clone()))?;
        registry.register(Box::new(jemalloc_metadata_bytes.clone()))?;
        registry.register(Box::new(jemalloc_mapped_bytes.clone()))?;
        registry.register(Box::new(jemalloc_retained_bytes.clone()))?;
        registry.register(Box::new(jemalloc_fragmentation_ratio.clone()))?;
        registry.register(Box::new(jemalloc_metadata_ratio.clone()))?;

        info!("Prometheus metrics registry initialized with spider, PDF, WASM, worker, jemalloc, and comprehensive Phase 1B metrics");

        Ok(Self {
            registry,
            http_requests_total,
            http_request_duration,
            active_connections,
            cache_hit_rate,
            fetch_phase_duration,
            gate_phase_duration,
            wasm_phase_duration,
            render_phase_duration,
            gate_decisions_raw,
            gate_decisions_probes_first,
            gate_decisions_headless,
            gate_decisions_cached,
            errors_total,
            redis_errors,
            wasm_errors,
            http_errors,
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
            spider_crawls_total,
            spider_pages_crawled,
            spider_pages_failed,
            spider_active_crawls,
            spider_frontier_size,
            spider_crawl_duration,
            spider_pages_per_second,
            pdf_total_processed,
            pdf_total_failed,
            pdf_memory_limit_failures,
            pdf_processing_time,
            pdf_peak_memory_mb,
            pdf_pages_per_pdf,
            pdf_memory_spikes_handled,
            pdf_cleanup_operations,
            wasm_memory_pages,
            wasm_grow_failed_total,
            wasm_peak_memory_pages,
            wasm_cold_start_time_ms,
            wasm_aot_cache_hits,
            wasm_aot_cache_misses,
            worker_pool_size,
            worker_pool_healthy,
            worker_jobs_submitted,
            worker_jobs_completed,
            worker_jobs_failed,
            worker_jobs_retried,
            worker_processing_time,
            worker_queue_depth,
            // WEEK 1 PHASE 1B metrics
            gate_decision_total,
            gate_score_histogram,
            gate_feature_text_ratio,
            gate_feature_script_density,
            gate_feature_spa_markers,
            gate_decision_duration_ms,
            extraction_quality_score,
            extraction_quality_success_rate,
            extraction_content_length,
            extraction_links_found,
            extraction_images_found,
            extraction_has_author,
            extraction_has_date,
            extraction_duration_by_mode,
            extraction_fallback_triggered,
            pipeline_phase_gate_analysis_ms,
            pipeline_phase_extraction_ms,
            // Jemalloc memory metrics
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
    ///
    /// Collects current memory stats from jemalloc and updates all related metrics.
    /// This should be called periodically (e.g., every 30 seconds) to track memory usage.
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

    /// Record phase timing
    pub fn record_phase_timing(&self, phase: &str, duration: f64) {
        match phase {
            "fetch" => self.fetch_phase_duration.observe(duration),
            "gate" => self.gate_phase_duration.observe(duration),
            "wasm" => self.wasm_phase_duration.observe(duration),
            "render" => self.render_phase_duration.observe(duration),
            _ => {} // Ignore unknown phases
        }
    }

    /// Record gate decision
    pub fn record_gate_decision(&self, decision: &str) {
        match decision {
            "raw" => self.gate_decisions_raw.inc(),
            "probes_first" => self.gate_decisions_probes_first.inc(),
            "headless" => self.gate_decisions_headless.inc(),
            "cached" => self.gate_decisions_cached.inc(),
            _ => warn!("Unknown gate decision: {}", decision),
        }
    }

    /// Record error
    pub fn record_error(&self, error_type: ErrorType) {
        self.errors_total.inc();
        match error_type {
            ErrorType::Redis => self.redis_errors.inc(),
            ErrorType::Wasm => self.wasm_errors.inc(),
            ErrorType::Http => self.http_errors.inc(),
        }
    }

    /// Update cache hit rate
    #[allow(dead_code)] // Public API - will be wired up in cache metrics collection
    pub fn update_cache_hit_rate(&self, rate: f64) {
        self.cache_hit_rate.set(rate);
    }

    /// Update active connections
    #[allow(dead_code)] // Public API - will be wired up in connection metrics collection
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

        // Note: Counters (messages_sent/dropped) should be incremented via
        // record_streaming_message_sent() and record_streaming_message_dropped()
        // methods below, not set directly from snapshot values.

        self.streaming_error_rate.set(streaming_metrics.error_rate);
        self.streaming_memory_usage_bytes
            .set(streaming_metrics.memory_usage_bytes as f64);

        // For connection duration, we'd typically observe individual durations
        // This would be called elsewhere when connections end
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

    /// Record spider crawl start
    pub fn record_spider_crawl_start(&self) {
        self.spider_crawls_total.inc();
        self.spider_active_crawls.inc();
    }

    /// Record spider crawl completion
    pub fn record_spider_crawl_completion(
        &self,
        pages_crawled: u64,
        pages_failed: u64,
        duration: f64,
    ) {
        self.spider_active_crawls.dec();
        self.spider_pages_crawled.inc_by(pages_crawled as f64);
        self.spider_pages_failed.inc_by(pages_failed as f64);
        self.spider_crawl_duration.observe(duration);

        // Calculate pages per second
        if duration > 0.0 {
            let pages_per_second = pages_crawled as f64 / duration;
            self.spider_pages_per_second.set(pages_per_second);
        }
    }

    /// Update spider frontier size
    pub fn update_spider_frontier_size(&self, size: usize) {
        self.spider_frontier_size.set(size as f64);
    }

    /// Update PDF metrics from PdfMetricsCollector
    #[allow(dead_code)] // Public API - integrates PDF metrics from collector
    pub fn update_pdf_metrics_from_collector(&self, pdf_metrics: &PdfMetricsCollector) {
        let snapshot = pdf_metrics.get_snapshot();

        // Update counters (increment by difference from last update)
        // Note: In production, you'd want to track previous values to get deltas
        self.pdf_total_processed.get(); // Get current value first
                                        // For simplicity, we'll set gauges and observe individual operations elsewhere

        // Update gauges with current snapshot values
        self.pdf_peak_memory_mb
            .set(snapshot.peak_memory_usage as f64 / (1024.0 * 1024.0));
        self.pdf_pages_per_pdf.set(snapshot.avg_pages_per_pdf);
    }

    /// Export PDF metrics as Prometheus format from PdfMetricsCollector
    #[allow(dead_code)] // Public API - exports PDF metrics for Prometheus
    pub fn export_pdf_metrics(&self, pdf_metrics: &PdfMetricsCollector) -> HashMap<String, f64> {
        pdf_metrics.export_for_prometheus()
    }

    /// Record PDF processing success
    pub fn record_pdf_processing_success(&self, duration_seconds: f64, pages: u32, memory_mb: f64) {
        self.pdf_total_processed.inc();
        self.pdf_processing_time.observe(duration_seconds);
        self.pdf_peak_memory_mb.set(memory_mb);
        if pages > 0 {
            self.pdf_pages_per_pdf.set(pages as f64);
        }
    }

    /// Record PDF processing failure
    pub fn record_pdf_processing_failure(&self, is_memory_limit: bool) {
        self.pdf_total_failed.inc();
        if is_memory_limit {
            self.pdf_memory_limit_failures.inc();
        }
    }

    /// Record PDF memory spike handled
    /// Reserved for PDF memory spike handling - will be wired up when feature is implemented
    #[allow(dead_code)]
    pub fn record_pdf_memory_spike(&self) {
        self.pdf_memory_spikes_handled.inc();
    }

    /// Record PDF cleanup operation
    /// Reserved for PDF cleanup tracking - will be wired up when feature is implemented
    #[allow(dead_code)]
    pub fn record_pdf_cleanup(&self) {
        self.pdf_cleanup_operations.inc();
    }

    /// Update WASM memory metrics (only available with wasm-extractor feature)
    #[cfg(feature = "wasm-extractor")]
    pub fn update_wasm_memory_metrics(
        &self,
        current_pages: usize,
        grow_failed: u64,
        peak_pages: usize,
    ) {
        self.wasm_memory_pages.set(current_pages as f64);
        self.wasm_peak_memory_pages.set(peak_pages as f64);

        // For counter, we need to track the difference (simplified approach)
        let current_failures = self.wasm_grow_failed_total.get();
        if grow_failed as f64 > current_failures {
            let diff = (grow_failed as f64 - current_failures) as u64;
            for _ in 0..diff {
                self.wasm_grow_failed_total.inc();
            }
        }
    }

    /// Update WASM cold start time (only available with wasm-extractor feature)
    #[cfg(feature = "wasm-extractor")]
    pub fn update_wasm_cold_start_time(&self, time_ms: f64) {
        self.wasm_cold_start_time_ms.set(time_ms);
    }

    /// Record WASM AOT cache hit
    /// Reserved for WASM AOT caching - will be wired up when caching is implemented
    #[allow(dead_code)]
    pub fn record_wasm_aot_cache_hit(&self) {
        self.wasm_aot_cache_hits.inc();
    }

    /// Record WASM AOT cache miss
    /// Reserved for WASM AOT caching - will be wired up when caching is implemented
    #[allow(dead_code)]
    pub fn record_wasm_aot_cache_miss(&self) {
        self.wasm_aot_cache_misses.inc();
    }

    /// Update WASM metrics from component extractor
    #[allow(dead_code)] // Public API - integrates WASM metrics from extractor
    pub fn update_wasm_metrics_from_extractor(
        &self,
        wasm_metrics: &std::collections::HashMap<String, f64>,
    ) {
        if let Some(&pages) = wasm_metrics.get("riptide_wasm_memory_pages") {
            self.wasm_memory_pages.set(pages);
        }
        if let Some(&peak_pages) = wasm_metrics.get("riptide_wasm_peak_memory_pages") {
            self.wasm_peak_memory_pages.set(peak_pages);
        }
        if let Some(&cold_start) = wasm_metrics.get("riptide_wasm_cold_start_time_ms") {
            self.wasm_cold_start_time_ms.set(cold_start);
        }
        if let Some(&cache_hits) = wasm_metrics.get("riptide_wasm_aot_cache_hits") {
            // Set to current value (in practice, you'd track deltas)
            let current = self.wasm_aot_cache_hits.get();
            let diff = (cache_hits - current).max(0.0);
            for _ in 0..diff as u64 {
                self.wasm_aot_cache_hits.inc();
            }
        }
        if let Some(&cache_misses) = wasm_metrics.get("riptide_wasm_aot_cache_misses") {
            let current = self.wasm_aot_cache_misses.get();
            let diff = (cache_misses - current).max(0.0);
            for _ in 0..diff as u64 {
                self.wasm_aot_cache_misses.inc();
            }
        }
    }

    /// Update worker metrics from WorkerService stats (Phase 4B Feature 5)
    #[cfg(feature = "workers")]
    pub fn update_worker_stats(&self, stats: &riptide_workers::WorkerPoolStats) {
        self.worker_pool_size.set(stats.total_workers as f64);
        self.worker_pool_healthy.set(stats.healthy_workers as f64);

        // Update job counters (track deltas to avoid double counting)
        let current_completed = self.worker_jobs_completed.get();
        let new_completed = stats.total_jobs_processed as f64;
        if new_completed > current_completed {
            let delta = (new_completed - current_completed) as u64;
            for _ in 0..delta {
                self.worker_jobs_completed.inc();
            }
        }

        let current_failed = self.worker_jobs_failed.get();
        let new_failed = stats.total_jobs_failed as f64;
        if new_failed > current_failed {
            let delta = (new_failed - current_failed) as u64;
            for _ in 0..delta {
                self.worker_jobs_failed.inc();
            }
        }
    }

    /// Update worker metrics from comprehensive snapshot (Phase 4B Feature 5)
    #[cfg(feature = "workers")]
    pub fn update_worker_metrics(&self, metrics: &riptide_workers::WorkerMetricsSnapshot) {
        // Update pool stats
        self.worker_pool_size.set(metrics.total_workers as f64);
        self.worker_pool_healthy.set(metrics.healthy_workers as f64);

        // Update job counters with delta tracking
        let current_submitted = self.worker_jobs_submitted.get();
        if metrics.jobs_submitted as f64 > current_submitted {
            let delta = metrics.jobs_submitted as f64 - current_submitted;
            for _ in 0..delta as u64 {
                self.worker_jobs_submitted.inc();
            }
        }

        let current_completed = self.worker_jobs_completed.get();
        if metrics.jobs_completed as f64 > current_completed {
            let delta = metrics.jobs_completed as f64 - current_completed;
            for _ in 0..delta as u64 {
                self.worker_jobs_completed.inc();
            }
        }

        let current_failed = self.worker_jobs_failed.get();
        if metrics.jobs_failed as f64 > current_failed {
            let delta = metrics.jobs_failed as f64 - current_failed;
            for _ in 0..delta as u64 {
                self.worker_jobs_failed.inc();
            }
        }

        let current_retried = self.worker_jobs_retried.get();
        if metrics.jobs_retried as f64 > current_retried {
            let delta = metrics.jobs_retried as f64 - current_retried;
            for _ in 0..delta as u64 {
                self.worker_jobs_retried.inc();
            }
        }

        // Update queue depth
        let total_queue_depth = metrics.total_queue_depth();
        self.worker_queue_depth.set(total_queue_depth as f64);
    }

    /// Record worker job completion with processing time (Phase 4B Feature 5)
    /// Reserved for worker metrics - will be wired up when worker monitoring is implemented
    #[allow(dead_code)]
    pub fn record_worker_job_completion(&self, processing_time_ms: u64) {
        self.worker_jobs_completed.inc();
        self.worker_processing_time
            .observe(processing_time_ms as f64 / 1000.0);
    }

    /// Record worker job failure (Phase 4B Feature 5)
    /// Reserved for worker metrics - will be wired up when worker monitoring is implemented
    #[allow(dead_code)]
    pub fn record_worker_job_failure(&self) {
        self.worker_jobs_failed.inc();
    }

    /// Record worker job submission (Phase 4B Feature 5)
    /// Reserved for worker metrics - will be wired up when worker monitoring is implemented
    #[allow(dead_code)]
    pub fn record_worker_job_submission(&self) {
        self.worker_jobs_submitted.inc();
    }

    // ===== WEEK 1 PHASE 1B: Recording Methods =====

    /// Record enhanced gate decision with features and score
    pub fn record_gate_decision_enhanced(
        &self,
        decision_type: &str,
        score: f32,
        text_ratio: f32,
        script_density: f32,
        spa_markers: u8,
        duration_ms: f64,
    ) {
        // Decision counter
        self.gate_decision_total
            .with_label_values(&[decision_type])
            .inc();

        // Score distribution
        self.gate_score_histogram.observe(score as f64);

        // Feature tracking
        self.gate_feature_text_ratio.observe(text_ratio as f64);
        self.gate_feature_script_density
            .observe(script_density as f64);

        // SPA markers
        self.gate_feature_spa_markers
            .with_label_values(&[&spa_markers.to_string()])
            .inc();

        // Decision latency
        self.gate_decision_duration_ms.observe(duration_ms);
    }

    /// Record extraction result with quality metrics
    #[allow(clippy::too_many_arguments)]
    pub fn record_extraction_result(
        &self,
        mode: &str,
        duration_ms: u64,
        success: bool,
        quality_score: f32,
        content_length: usize,
        links_count: usize,
        images_count: usize,
        has_author: bool,
        has_date: bool,
    ) {
        // Duration
        self.extraction_duration_by_mode
            .with_label_values(&[mode])
            .observe(duration_ms as f64 / 1000.0);

        // Quality metrics if successful
        if success {
            self.extraction_quality_score
                .with_label_values(&[mode])
                .observe(quality_score as f64);

            self.extraction_content_length
                .with_label_values(&[mode])
                .observe(content_length as f64);

            self.extraction_links_found
                .with_label_values(&[mode])
                .observe(links_count as f64);

            self.extraction_images_found
                .with_label_values(&[mode])
                .observe(images_count as f64);

            // Success rate (moving average via gauge)
            let success_val = if quality_score > 60.0 { 1.0 } else { 0.0 };
            self.extraction_quality_success_rate
                .with_label_values(&[mode])
                .set(success_val);

            // Author and date presence
            let author_label = if has_author { "true" } else { "false" };
            self.extraction_has_author
                .with_label_values(&[mode, author_label])
                .inc();

            let date_label = if has_date { "true" } else { "false" };
            self.extraction_has_date
                .with_label_values(&[mode, date_label])
                .inc();
        }
    }

    /// Record extraction fallback event
    pub fn record_extraction_fallback(&self, from_mode: &str, to_mode: &str, reason: &str) {
        self.extraction_fallback_triggered
            .with_label_values(&[from_mode, to_mode, reason])
            .inc();
    }

    /// Record engine selection decision (Phase 10)
    pub fn increment_engine_selection(&self, engine: &str, _confidence: f64) {
        // For now, just increment the gate decision counters
        // This maps to existing metrics until we add dedicated engine selection metrics
        match engine {
            "raw" => self.gate_decisions_raw.inc(),
            "wasm" => self.gate_decisions_probes_first.inc(),
            "headless" => self.gate_decisions_headless.inc(),
            _ => {}
        }
    }

    /// Get engine selection statistics (Phase 10)
    pub fn get_engine_stats(&self) -> EngineStats {
        // For now, return basic stats from gate decision counters
        // In production, this would track engine-specific counters
        let raw_count = self.gate_decisions_raw.get() as u64;
        let wasm_count = self.gate_decisions_probes_first.get() as u64;
        let headless_count = self.gate_decisions_headless.get() as u64;
        let cached_count = self.gate_decisions_cached.get() as u64;

        let total = raw_count + wasm_count + headless_count + cached_count;

        let mut engine_counts = HashMap::new();
        engine_counts.insert("raw".to_string(), raw_count);
        engine_counts.insert("wasm".to_string(), wasm_count);
        engine_counts.insert("headless".to_string(), headless_count);
        engine_counts.insert("cached".to_string(), cached_count);

        let mut engine_percentages = HashMap::new();
        if total > 0 {
            engine_percentages.insert("raw".to_string(), (raw_count as f64 / total as f64) * 100.0);
            engine_percentages.insert(
                "wasm".to_string(),
                (wasm_count as f64 / total as f64) * 100.0,
            );
            engine_percentages.insert(
                "headless".to_string(),
                (headless_count as f64 / total as f64) * 100.0,
            );
            engine_percentages.insert(
                "cached".to_string(),
                (cached_count as f64 / total as f64) * 100.0,
            );
        }

        EngineStats {
            total_requests: total,
            engine_counts,
            engine_percentages,
        }
    }

    /// Record pipeline phase timing
    /// Reserved for pipeline phase tracking - will be wired up when metrics collection is implemented
    #[allow(dead_code)]
    pub fn record_pipeline_phase_ms(&self, phase: &str, duration_ms: f64) {
        match phase {
            "gate_analysis" => self.pipeline_phase_gate_analysis_ms.observe(duration_ms),
            "extraction" => self.pipeline_phase_extraction_ms.observe(duration_ms),
            _ => {}
        }
    }
}

/// Phase types for timing measurements
#[derive(Debug, Clone, Copy)]
pub enum PhaseType {
    Fetch,
    Gate,
    Wasm,
    Render,
}

/// Error types for metrics
#[derive(Debug, Clone, Copy)]
pub enum ErrorType {
    Redis,
    Wasm,
    Http,
}

/// Engine selection statistics (Phase 10)
#[derive(Debug, Clone)]
pub struct EngineStats {
    pub total_requests: u64,
    pub engine_counts: HashMap<String, u64>,
    pub engine_percentages: HashMap<String, f64>,
}

/// Phase timing tracker for structured logging and metrics
#[derive(Debug)]
pub struct PhaseTimer {
    phase: PhaseType,
    start_time: Instant,
    url: String,
}

impl PhaseTimer {
    /// Start timing a phase
    pub fn start(phase: PhaseType, url: String) -> Self {
        info!(
            phase = ?phase,
            url = %url,
            "Phase started"
        );

        Self {
            phase,
            start_time: Instant::now(),
            url,
        }
    }

    /// End timing and log results
    #[allow(deprecated)]
    pub fn end(self, metrics: &RipTideMetrics, success: bool) {
        let duration = self.start_time.elapsed();
        let duration_secs = duration.as_secs_f64();

        info!(
            phase = ?self.phase,
            url = %self.url,
            duration_ms = duration.as_millis(),
            duration_seconds = duration_secs,
            success = success,
            "Phase completed"
        );

        // Convert PhaseType to string for record_phase_timing
        let phase_str = match self.phase {
            PhaseType::Fetch => "fetch",
            PhaseType::Gate => "gate",
            PhaseType::Wasm => "wasm",
            PhaseType::Render => "render",
        };
        metrics.record_phase_timing(phase_str, duration_secs);

        if !success {
            match self.phase {
                PhaseType::Fetch => metrics.record_error(ErrorType::Http),
                PhaseType::Wasm => metrics.record_error(ErrorType::Wasm),
                _ => metrics.record_error(ErrorType::Http),
            }
        }
    }
}

/// Create Prometheus metric layer for Axum
pub fn create_metrics_layer() -> anyhow::Result<(PrometheusMetricLayer<'static>, PrometheusHandle)>
{
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
    info!("Prometheus metrics layer created");
    Ok((prometheus_layer, metric_handle))
}
