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

        info!("Prometheus metrics registry initialized");

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
