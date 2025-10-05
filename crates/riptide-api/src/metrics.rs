use axum_prometheus::{metrics_exporter_prometheus::PrometheusHandle, PrometheusMetricLayer};
use prometheus::{Counter, Gauge, Histogram, HistogramOpts, Opts, Registry};
use riptide_core::pdf::PdfMetricsCollector;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, warn};

/// Metrics collection and management for RipTide API
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
    pub pdf_memory_spikes_handled: Counter,
    pub pdf_cleanup_operations: Counter,

    /// WASM memory metrics
    pub wasm_memory_pages: Gauge,
    pub wasm_grow_failed_total: Counter,
    pub wasm_peak_memory_pages: Gauge,
    pub wasm_cold_start_time_ms: Gauge,
    pub wasm_aot_cache_hits: Counter,
    pub wasm_aot_cache_misses: Counter,

    /// Worker management metrics (Phase 4B Feature 5)
    pub worker_pool_size: Gauge,
    pub worker_pool_healthy: Gauge,
    pub worker_jobs_submitted: Counter,
    pub worker_jobs_completed: Counter,
    pub worker_jobs_failed: Counter,
    pub worker_jobs_retried: Counter,
    pub worker_processing_time: Histogram,
    pub worker_queue_depth: Gauge,
}

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

        info!("Prometheus metrics registry initialized with spider, PDF, WASM, and worker metrics");

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
        })
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
    pub fn record_phase_timing(&self, phase: PhaseType, duration: f64) {
        match phase {
            PhaseType::Fetch => self.fetch_phase_duration.observe(duration),
            PhaseType::Gate => self.gate_phase_duration.observe(duration),
            PhaseType::Wasm => self.wasm_phase_duration.observe(duration),
            PhaseType::Render => self.render_phase_duration.observe(duration),
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
    pub fn update_cache_hit_rate(&self, rate: f64) {
        self.cache_hit_rate.set(rate);
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

        // For counters, we need to track the difference and add it
        // This is a simplified approach - in production you'd want to track previous values
        // For now, we'll just set the gauge to the current value
        let _messages_sent_diff = streaming_metrics.total_messages_sent as f64;
        let _messages_dropped_diff = streaming_metrics.total_messages_dropped as f64;

        // Since counters can't be set directly, we observe individual increments
        // This method should ideally be called with delta values, not absolute values
        // For this integration, we'll track via separate gauges that mirror the counter values

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
    pub fn record_pdf_memory_spike(&self) {
        self.pdf_memory_spikes_handled.inc();
    }

    /// Record PDF cleanup operation
    pub fn record_pdf_cleanup(&self) {
        self.pdf_cleanup_operations.inc();
    }

    /// Update WASM memory metrics
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

    /// Update WASM cold start time
    pub fn update_wasm_cold_start_time(&self, time_ms: f64) {
        self.wasm_cold_start_time_ms.set(time_ms);
    }

    /// Record WASM AOT cache hit
    pub fn record_wasm_aot_cache_hit(&self) {
        self.wasm_aot_cache_hits.inc();
    }

    /// Record WASM AOT cache miss
    pub fn record_wasm_aot_cache_miss(&self) {
        self.wasm_aot_cache_misses.inc();
    }

    /// Update WASM metrics from component extractor
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
    pub fn record_worker_job_completion(&self, processing_time_ms: u64) {
        self.worker_jobs_completed.inc();
        self.worker_processing_time
            .observe(processing_time_ms as f64 / 1000.0);
    }

    /// Record worker job failure (Phase 4B Feature 5)
    pub fn record_worker_job_failure(&self) {
        self.worker_jobs_failed.inc();
    }

    /// Record worker job submission (Phase 4B Feature 5)
    pub fn record_worker_job_submission(&self) {
        self.worker_jobs_submitted.inc();
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

        metrics.record_phase_timing(self.phase, duration_secs);

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
