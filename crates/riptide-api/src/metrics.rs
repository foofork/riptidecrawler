use axum_prometheus::{metrics_exporter_prometheus::PrometheusHandle, PrometheusMetricLayer};
use prometheus::{Counter, Gauge, Histogram, HistogramOpts, Opts, Registry};
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
            Opts::new("riptide_streaming_active_connections", "Active streaming connections")
                .const_label("service", "riptide-api"),
        )?;

        let streaming_total_connections = Gauge::with_opts(
            Opts::new("riptide_streaming_total_connections", "Total streaming connections created")
                .const_label("service", "riptide-api"),
        )?;

        let streaming_messages_sent = Counter::with_opts(
            Opts::new("riptide_streaming_messages_sent_total", "Total streaming messages sent")
                .const_label("service", "riptide-api"),
        )?;

        let streaming_messages_dropped = Counter::with_opts(
            Opts::new("riptide_streaming_messages_dropped_total", "Total streaming messages dropped")
                .const_label("service", "riptide-api"),
        )?;

        let streaming_error_rate = Gauge::with_opts(
            Opts::new("riptide_streaming_error_rate", "Streaming error rate (0.0 to 1.0)")
                .const_label("service", "riptide-api"),
        )?;

        let streaming_memory_usage_bytes = Gauge::with_opts(
            Opts::new("riptide_streaming_memory_usage_bytes", "Streaming memory usage in bytes")
                .const_label("service", "riptide-api"),
        )?;

        let streaming_connection_duration = Histogram::with_opts(
            HistogramOpts::new(
                "riptide_streaming_connection_duration_seconds",
                "Streaming connection duration",
            )
            .const_label("service", "riptide-api")
            .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 300.0]),
        )?;

        // Spider crawling metrics
        let spider_crawls_total = Counter::with_opts(
            Opts::new("riptide_spider_crawls_total", "Total spider crawl operations")
                .const_label("service", "riptide-api"),
        )?;

        let spider_pages_crawled = Counter::with_opts(
            Opts::new("riptide_spider_pages_crawled_total", "Total pages crawled by spider")
                .const_label("service", "riptide-api"),
        )?;

        let spider_pages_failed = Counter::with_opts(
            Opts::new("riptide_spider_pages_failed_total", "Total pages failed by spider")
                .const_label("service", "riptide-api"),
        )?;

        let spider_active_crawls = Gauge::with_opts(
            Opts::new("riptide_spider_active_crawls", "Number of active spider crawls")
                .const_label("service", "riptide-api"),
        )?;

        let spider_frontier_size = Gauge::with_opts(
            Opts::new("riptide_spider_frontier_size", "Current spider frontier queue size")
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
            Opts::new("riptide_spider_pages_per_second", "Spider crawl rate in pages per second")
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

        info!("Prometheus metrics registry initialized with spider metrics");

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
        })
    }

    /// Record HTTP request
    pub fn record_http_request(&self, method: &str, path: &str, status: u16, duration: f64) {
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
    pub fn update_streaming_metrics(&self, streaming_metrics: &crate::streaming::GlobalStreamingMetrics) {
        self.streaming_active_connections.set(streaming_metrics.active_connections as f64);
        self.streaming_total_connections.set(streaming_metrics.total_connections as f64);

        // For counters, we need to track the difference and add it
        // This is a simplified approach - in production you'd want to track previous values
        // For now, we'll just set the gauge to the current value
        let messages_sent_diff = streaming_metrics.total_messages_sent as f64;
        let messages_dropped_diff = streaming_metrics.total_messages_dropped as f64;

        // Since counters can't be set directly, we observe individual increments
        // This method should ideally be called with delta values, not absolute values
        // For this integration, we'll track via separate gauges that mirror the counter values

        self.streaming_error_rate.set(streaming_metrics.error_rate);
        self.streaming_memory_usage_bytes.set(streaming_metrics.memory_usage_bytes as f64);

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
    pub fn record_spider_crawl_completion(&self, pages_crawled: u64, pages_failed: u64, duration: f64) {
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
