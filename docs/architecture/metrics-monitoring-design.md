# Metrics and Monitoring Architecture Design
## RipTide Extraction Enhancement Project

**Document Version:** 1.0
**Date:** 2025-10-13
**Author:** System Architect Agent
**Status:** Draft for Review

---

## Executive Summary

This document defines the comprehensive metrics, monitoring, and observability architecture for the RipTide extraction enhancement project. The system tracks gate decisions, extraction quality, performance metrics, and provides real-time monitoring dashboards with anomaly detection.

### Key Design Goals

1. **Gate Decision Tracking**: Monitor distribution of Raw/ProbesFirst/Headless decisions
2. **Quality Scoring**: Track extraction quality across different modes
3. **Performance Metrics**: Monitor per-mode extraction performance
4. **Real-time Dashboards**: Grafana dashboards for operational visibility
5. **Anomaly Detection**: Automated alerting for degraded performance

---

## 1. Metrics Taxonomy

### 1.1 Gate Decision Metrics

#### Core Counters
```rust
// Gate decision distribution
riptide_gate_decisions_raw_total{service="riptide-api"}
riptide_gate_decisions_probes_first_total{service="riptide-api"}
riptide_gate_decisions_headless_total{service="riptide-api"}
riptide_gate_decisions_cached_total{service="riptide-api"}
```

#### New Enhanced Metrics
```rust
// Gate score distribution (histogram)
riptide_gate_score_distribution{service="riptide-api"}
  .buckets: [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]

// Gate decision time
riptide_gate_decision_duration_ms{service="riptide-api"}
  .buckets: [1, 5, 10, 25, 50, 100, 250, 500]

// Feature-specific metrics
riptide_gate_feature_text_ratio{service="riptide-api"}
riptide_gate_feature_script_density{service="riptide-api"}
riptide_gate_feature_spa_markers{service="riptide-api", marker_count="0|1|2|3+"}
riptide_gate_feature_has_og{service="riptide-api", present="true|false"}
riptide_gate_feature_has_jsonld{service="riptide-api", present="true|false"}

// Domain prior distribution
riptide_gate_domain_prior{service="riptide-api", domain="example.com"}
```

### 1.2 Extraction Quality Metrics

#### Quality Score Distribution
```rust
// Overall quality score by extraction mode
riptide_extraction_quality_score{service="riptide-api", mode="raw|headless|probes"}
  .buckets: [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100]

// Quality score success rate (score >= 60)
riptide_extraction_quality_success_rate{service="riptide-api", mode="raw|headless|probes"}

// Quality score comparison (raw vs headless for same URL)
riptide_extraction_quality_delta{service="riptide-api", comparison="raw_vs_headless"}
```

#### Content Extraction Metrics
```rust
// Content length extracted
riptide_extraction_content_length_bytes{service="riptide-api", mode="raw|headless|probes"}
  .buckets: [0, 1024, 5120, 10240, 51200, 102400, 512000, 1048576]

// Links found
riptide_extraction_links_found{service="riptide-api", mode="raw|headless|probes"}
  .buckets: [0, 10, 25, 50, 100, 250, 500, 1000]

// Media items extracted
riptide_extraction_media_count{service="riptide-api", mode="raw|headless|probes", type="image|video"}

// Markdown generation success
riptide_extraction_markdown_generated{service="riptide-api", mode="raw|headless|probes"}
```

#### Success and Failure Tracking
```rust
// Extraction success rate by mode
riptide_extraction_success_total{service="riptide-api", mode="raw|headless|probes"}
riptide_extraction_failure_total{service="riptide-api", mode="raw|headless|probes", reason="timeout|parse_error|network|other"}

// Fallback trigger rate (ProbesFirst -> Headless)
riptide_extraction_fallback_triggered_total{service="riptide-api", from="raw", to="headless"}
```

### 1.3 Performance Metrics

#### Extraction Duration by Mode
```rust
// Per-mode extraction duration
riptide_extraction_duration_ms{service="riptide-api", mode="raw"}
  .buckets: [10, 50, 100, 250, 500, 1000, 2000, 5000]

riptide_extraction_duration_ms{service="riptide-api", mode="headless"}
  .buckets: [100, 500, 1000, 2000, 3000, 5000, 10000, 30000]

riptide_extraction_duration_ms{service="riptide-api", mode="probes"}
  .buckets: [10, 100, 500, 1000, 2000, 5000, 10000, 30000]
```

#### Pipeline Phase Breakdown
```rust
// Existing phase metrics (already implemented)
riptide_fetch_phase_duration_seconds{service="riptide-api"}
riptide_gate_phase_duration_seconds{service="riptide-api"}
riptide_wasm_phase_duration_seconds{service="riptide-api"}
riptide_render_phase_duration_seconds{service="riptide-api"}

// New detailed phase metrics
riptide_extraction_phase_breakdown_ms{service="riptide-api", phase="fetch|gate|extract|cache", mode="raw|headless|probes"}
```

#### Throughput Metrics
```rust
// Requests per second by decision type
riptide_extraction_throughput_rps{service="riptide-api", mode="raw|headless|probes"}

// Concurrent extractions
riptide_extraction_concurrent_active{service="riptide-api", mode="raw|headless|probes"}
```

### 1.4 Resource Utilization

#### Memory Metrics
```rust
// Per-mode memory usage
riptide_extraction_memory_bytes{service="riptide-api", mode="raw|headless|probes"}

// Peak memory per extraction
riptide_extraction_peak_memory_bytes{service="riptide-api", mode="raw|headless|probes"}
```

#### WASM-specific Metrics (existing)
```rust
riptide_wasm_memory_pages{service="riptide-api"}
riptide_wasm_peak_memory_pages{service="riptide-api"}
riptide_wasm_cold_start_time_ms{service="riptide-api"}
```

### 1.5 Anomaly Detection Metrics

#### Statistical Indicators
```rust
// Rolling window quality averages
riptide_extraction_quality_rolling_avg{service="riptide-api", mode="raw|headless|probes", window="5m|15m|1h"}

// Outlier detection
riptide_extraction_outlier_detected{service="riptide-api", metric="duration|quality|size", mode="raw|headless|probes"}

// Error rate tracking
riptide_extraction_error_rate{service="riptide-api", mode="raw|headless|probes", window="1m|5m|15m"}
```

---

## 2. Data Flow Architecture

### 2.1 Metrics Collection Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                      Extraction Pipeline                         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Metrics Injection Points                      │
├─────────────────────────────────────────────────────────────────┤
│ 1. Gate Analysis Phase (gate.rs)                                │
│    - Record decision (raw/probes/headless)                       │
│    - Record quality score                                        │
│    - Record gate features                                        │
│    - Record decision duration                                    │
│                                                                   │
│ 2. Extraction Phase (pipeline.rs)                                │
│    - Record extraction start                                     │
│    - Track mode-specific duration                                │
│    - Record success/failure                                      │
│    - Measure content metrics                                     │
│                                                                   │
│ 3. Fallback Detection (reliability.rs)                           │
│    - Record fallback triggers                                    │
│    - Track retry attempts                                        │
│    - Measure quality deltas                                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   RipTideMetrics Struct                          │
│                  (metrics.rs - Enhanced)                         │
├─────────────────────────────────────────────────────────────────┤
│ - Aggregates metrics from all sources                            │
│ - Performs non-blocking async updates                            │
│ - Maintains Prometheus Registry                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Prometheus Exporter                           │
│                  (HTTP /metrics endpoint)                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Prometheus Server                           │
│                   (Scrape & Time-series DB)                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Grafana Dashboards                          │
│                   (Visualization & Alerts)                       │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Non-Blocking Metrics Recording

All metrics recording operations MUST be non-blocking to avoid impacting extraction performance:

```rust
// ✅ CORRECT: Non-blocking metric recording
impl PipelineOrchestrator {
    async fn execute_single(&self, url: &str) -> ApiResult<PipelineResult> {
        // ... gate analysis ...

        // Non-blocking metric update
        tokio::spawn({
            let metrics = self.state.metrics.clone();
            let decision = gate_decision_str.clone();
            let score = quality_score;
            async move {
                metrics.record_gate_decision_enhanced(&decision, score);
            }
        });

        // Continue pipeline without waiting
    }
}
```

---

## 3. Dashboard Design

### 3.1 Overview Dashboard

**Dashboard Name:** RipTide Extraction Overview
**Refresh Rate:** 30s
**Time Range:** Last 24 hours (configurable)

#### Panel 1: Gate Decision Distribution (Pie Chart)
```promql
# Query
sum by (decision) (
  rate(riptide_gate_decisions_raw_total[5m]) +
  rate(riptide_gate_decisions_probes_first_total[5m]) +
  rate(riptide_gate_decisions_headless_total[5m])
)

# Visualization
- Type: Pie Chart
- Colors: Raw (Green), ProbesFirst (Yellow), Headless (Orange)
- Show percentages and absolute values
```

#### Panel 2: Extraction Success Rate by Mode (Time Series)
```promql
# Query
sum by (mode) (rate(riptide_extraction_success_total[5m])) /
sum by (mode) (
  rate(riptide_extraction_success_total[5m]) +
  rate(riptide_extraction_failure_total[5m])
) * 100

# Visualization
- Type: Time Series
- Y-axis: Percentage (0-100%)
- Threshold: Warning < 95%, Critical < 90%
```

#### Panel 3: Quality Score Distribution (Heatmap)
```promql
# Query
histogram_quantile(0.5, riptide_extraction_quality_score_bucket{mode="raw"}) as "Raw P50"
histogram_quantile(0.95, riptide_extraction_quality_score_bucket{mode="raw"}) as "Raw P95"
histogram_quantile(0.5, riptide_extraction_quality_score_bucket{mode="headless"}) as "Headless P50"
histogram_quantile(0.95, riptide_extraction_quality_score_bucket{mode="headless"}) as "Headless P95"

# Visualization
- Type: Time Series
- Multiple series for comparison
- Color bands: Red (0-30), Yellow (30-60), Green (60-100)
```

#### Panel 4: Extraction Duration Comparison (Bar Chart)
```promql
# Query - P95 latency by mode
histogram_quantile(0.95, rate(riptide_extraction_duration_ms_bucket[5m])) by (mode)

# Visualization
- Type: Bar Chart
- X-axis: Mode (Raw, ProbesFirst, Headless)
- Y-axis: Duration (ms)
- Target line: 500ms (Raw), 3000ms (Headless)
```

### 3.2 Gate Analysis Dashboard

**Dashboard Name:** RipTide Gate Deep Dive
**Purpose:** Analyze gate decision-making patterns

#### Panel 1: Gate Score Distribution (Histogram)
```promql
# Query
rate(riptide_gate_score_distribution_bucket[5m])

# Visualization
- Type: Histogram
- Buckets: 0.0-1.0 in 0.1 increments
- Overlay decision thresholds (hi=0.7, lo=0.3)
```

#### Panel 2: Feature Impact Analysis (Multi-series)
```promql
# Text Ratio vs Score
avg(riptide_gate_feature_text_ratio) by (decision)

# Script Density vs Score
avg(riptide_gate_feature_script_density) by (decision)

# SPA Markers Impact
sum(riptide_gate_feature_spa_markers) by (marker_count, decision)

# Visualization
- Type: Multi-axis Time Series
- Show correlation between features and decisions
```

#### Panel 3: Domain Prior Performance (Table)
```promql
# Query - Top domains by decision type
topk(20, sum by (domain) (rate(riptide_gate_domain_prior[1h])))

# Visualization
- Type: Table
- Columns: Domain, Raw %, ProbesFirst %, Headless %, Avg Score
- Sortable by any column
```

### 3.3 Performance Dashboard

**Dashboard Name:** RipTide Performance Metrics

#### Panel 1: Extraction Latency Percentiles
```promql
# P50, P90, P95, P99 by mode
histogram_quantile(0.50, rate(riptide_extraction_duration_ms_bucket[5m])) by (mode)
histogram_quantile(0.90, rate(riptide_extraction_duration_ms_bucket[5m])) by (mode)
histogram_quantile(0.95, rate(riptide_extraction_duration_ms_bucket[5m])) by (mode)
histogram_quantile(0.99, rate(riptide_extraction_duration_ms_bucket[5m])) by (mode)
```

#### Panel 2: Pipeline Phase Breakdown
```promql
# Stacked area chart showing time spent in each phase
avg(rate(riptide_fetch_phase_duration_seconds[5m])) as "Fetch"
avg(rate(riptide_gate_phase_duration_seconds[5m])) as "Gate"
avg(rate(riptide_wasm_phase_duration_seconds[5m])) as "WASM"
avg(rate(riptide_render_phase_duration_seconds[5m])) as "Render"
```

#### Panel 3: Throughput by Mode
```promql
# Requests per second
sum by (mode) (rate(riptide_extraction_success_total[1m]))
```

### 3.4 Quality Dashboard

**Dashboard Name:** RipTide Quality Monitoring

#### Panel 1: Quality Score Trends
```promql
# Rolling averages
avg_over_time(riptide_extraction_quality_rolling_avg{window="5m"}[1h]) by (mode)
avg_over_time(riptide_extraction_quality_rolling_avg{window="15m"}[1h]) by (mode)
avg_over_time(riptide_extraction_quality_rolling_avg{window="1h"}[24h]) by (mode)
```

#### Panel 2: Content Extraction Metrics
```promql
# Average content length by mode
avg(riptide_extraction_content_length_bytes) by (mode)

# Average links found
avg(riptide_extraction_links_found) by (mode)

# Media extraction rate
sum(rate(riptide_extraction_media_count[5m])) by (type, mode)
```

#### Panel 3: Raw vs Headless Quality Comparison
```promql
# For URLs processed by both modes, compare quality
avg(riptide_extraction_quality_delta{comparison="raw_vs_headless"})

# Visualization
- Positive delta: Headless better
- Negative delta: Raw better
- Show distribution and outliers
```

---

## 4. Alerting Rules

### 4.1 Critical Alerts

#### Alert: High Extraction Failure Rate
```yaml
- alert: HighExtractionFailureRate
  expr: |
    (sum(rate(riptide_extraction_failure_total[5m])) by (mode) /
     sum(rate(riptide_extraction_success_total[5m] + riptide_extraction_failure_total[5m])) by (mode)) > 0.10
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "Extraction failure rate above 10% for mode {{ $labels.mode }}"
    description: "{{ $value | humanizePercentage }} failure rate detected"
```

#### Alert: Gate Decision Imbalance
```yaml
- alert: GateDecisionImbalance
  expr: |
    (sum(rate(riptide_gate_decisions_headless_total[15m])) /
     sum(rate(riptide_gate_decisions_raw_total[15m] +
              riptide_gate_decisions_probes_first_total[15m] +
              riptide_gate_decisions_headless_total[15m]))) > 0.50
  for: 15m
  labels:
    severity: warning
  annotations:
    summary: "Over 50% of requests going to headless rendering"
    description: "May indicate gate scoring issues or content changes"
```

### 4.2 Performance Alerts

#### Alert: High P95 Latency
```yaml
- alert: HighP95ExtractionLatency
  expr: |
    histogram_quantile(0.95, rate(riptide_extraction_duration_ms_bucket{mode="raw"}[5m])) > 500
  for: 10m
  labels:
    severity: warning
  annotations:
    summary: "Raw extraction P95 latency above 500ms"
    description: "Current P95: {{ $value }}ms"

- alert: HighHeadlessLatency
  expr: |
    histogram_quantile(0.95, rate(riptide_extraction_duration_ms_bucket{mode="headless"}[5m])) > 5000
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "Headless extraction P95 latency above 5s"
```

### 4.3 Quality Alerts

#### Alert: Quality Score Degradation
```yaml
- alert: QualityScoreDegradation
  expr: |
    avg_over_time(riptide_extraction_quality_rolling_avg{window="15m"}[1h]) < 60
  for: 30m
  labels:
    severity: warning
  annotations:
    summary: "Average quality score below 60 for mode {{ $labels.mode }}"
    description: "Current average: {{ $value }}"
```

#### Alert: Increased Fallback Rate
```yaml
- alert: HighFallbackRate
  expr: |
    rate(riptide_extraction_fallback_triggered_total[10m]) > 0.30
  for: 15m
  labels:
    severity: info
  annotations:
    summary: "ProbesFirst fallback rate above 30%"
    description: "Many raw extractions falling back to headless"
```

---

## 5. Implementation Plan

### 5.1 Phase 1: Core Metrics Extension (Week 1)

**File: `/workspaces/eventmesh/crates/riptide-api/src/metrics.rs`**

#### Changes Required:

1. **Add New Metric Fields to RipTideMetrics Struct**
```rust
pub struct RipTideMetrics {
    // ... existing fields ...

    // Gate decision metrics
    pub gate_score_distribution: Histogram,
    pub gate_decision_duration: Histogram,
    pub gate_feature_text_ratio: Gauge,
    pub gate_feature_script_density: Gauge,
    pub gate_feature_spa_markers: Counter,

    // Extraction quality metrics
    pub extraction_quality_score: Histogram,
    pub extraction_quality_success_rate: Gauge,
    pub extraction_content_length: Histogram,
    pub extraction_links_found: Histogram,
    pub extraction_media_count: Counter,
    pub extraction_success_total: Counter,
    pub extraction_failure_total: Counter,
    pub extraction_fallback_triggered: Counter,

    // Performance metrics by mode
    pub extraction_duration_by_mode: HashMap<String, Histogram>,
    pub extraction_concurrent_active: Gauge,
    pub extraction_throughput_rps: Gauge,

    // Quality tracking
    pub extraction_quality_rolling_avg: Gauge,
    pub extraction_outlier_detected: Counter,
}
```

2. **Add Helper Methods**
```rust
impl RipTideMetrics {
    /// Record enhanced gate decision with features
    pub fn record_gate_decision_enhanced(
        &self,
        decision: &str,
        score: f32,
        features: &GateFeatures,
    ) {
        // Record decision (existing)
        self.record_gate_decision(decision);

        // Record score distribution
        self.gate_score_distribution.observe(score as f64);

        // Record feature values
        let text_ratio = if features.html_bytes > 0 {
            features.visible_text_chars as f64 / features.html_bytes as f64
        } else {
            0.0
        };
        self.gate_feature_text_ratio.set(text_ratio);

        let script_density = if features.html_bytes > 0 {
            features.script_bytes as f64 / features.html_bytes as f64
        } else {
            0.0
        };
        self.gate_feature_script_density.set(script_density);

        // SPA markers counter (with labels)
        self.gate_feature_spa_markers.inc();
    }

    /// Record extraction result with quality metrics
    pub fn record_extraction_result(
        &self,
        mode: &str,
        duration_ms: u64,
        success: bool,
        document: Option<&ExtractedDoc>,
    ) {
        // Record duration
        if let Some(histogram) = self.extraction_duration_by_mode.get(mode) {
            histogram.observe(duration_ms as f64);
        }

        // Record success/failure
        if success {
            self.extraction_success_total.inc();

            // Record quality metrics if document available
            if let Some(doc) = document {
                if let Some(quality) = doc.quality_score {
                    self.extraction_quality_score.observe(quality as f64);

                    // Update success rate if quality >= 60
                    if quality >= 60.0 {
                        self.extraction_quality_success_rate.inc();
                    }
                }

                // Content metrics
                self.extraction_content_length.observe(doc.text.len() as f64);
                self.extraction_links_found.observe(doc.links.len() as f64);

                // Media count
                self.extraction_media_count.inc_by(doc.media.len() as f64);
            }
        } else {
            self.extraction_failure_total.inc();
        }
    }

    /// Record fallback trigger (ProbesFirst -> Headless)
    pub fn record_extraction_fallback(&self, from: &str, to: &str, reason: &str) {
        self.extraction_fallback_triggered.inc();

        info!(
            from = %from,
            to = %to,
            reason = %reason,
            "Extraction fallback triggered"
        );
    }
}
```

### 5.2 Phase 2: Pipeline Integration (Week 1-2)

**File: `/workspaces/eventmesh/crates/riptide-api/src/pipeline.rs`**

#### Injection Points:

1. **Gate Analysis Phase** (Line ~245-265)
```rust
// After gate analysis
let gate_features = self.analyze_content(&html_content, url).await?;
let quality_score = score(&gate_features);
let decision = decide(&gate_features, hi, lo);

// ✅ ADD: Enhanced metrics recording
tokio::spawn({
    let metrics = self.state.metrics.clone();
    let features = gate_features.clone();
    let score = quality_score;
    let decision_str = format!("{:?}", decision);
    async move {
        metrics.record_gate_decision_enhanced(&decision_str, score, &features);
    }
});
```

2. **Extraction Phase** (Line ~287-295)
```rust
// Before extraction
let extract_start = Instant::now();
let mode_str = match decision {
    Decision::Raw => "raw",
    Decision::ProbesFirst => "probes",
    Decision::Headless => "headless",
};

// After extraction
let document = self.extract_content(&html_content, url, decision).await?;
let extract_duration = extract_start.elapsed();

// ✅ ADD: Record extraction result
tokio::spawn({
    let metrics = self.state.metrics.clone();
    let mode = mode_str.to_string();
    let duration_ms = extract_duration.as_millis() as u64;
    let doc = document.clone();
    async move {
        metrics.record_extraction_result(&mode, duration_ms, true, Some(&doc));
    }
});
```

3. **Error Handling** (Add to error branches)
```rust
.map_err(|e| {
    // ✅ ADD: Record extraction failure
    let metrics = self.state.metrics.clone();
    let mode = mode_str.to_string();
    let duration_ms = extract_start.elapsed().as_millis() as u64;
    tokio::spawn(async move {
        metrics.record_extraction_result(&mode, duration_ms, false, None);
    });

    ApiError::ExtractionError(e.to_string())
})
```

### 5.3 Phase 3: Reliability Integration (Week 2)

**File: `/workspaces/eventmesh/crates/riptide-core/src/reliability.rs`**

#### Fallback Detection (Line ~150-154)

```rust
ExtractionMode::ProbesFirst => {
    let fast_result = self.extract_fast(url, wasm_extractor, request_id).await;

    match fast_result {
        Ok(doc) if self.is_quality_acceptable(&doc) => Ok(doc),
        _ => {
            // ✅ ADD: Record fallback trigger
            if let Some(metrics) = metrics {
                metrics.record_extraction_fallback(
                    "raw",
                    "headless",
                    "quality_threshold_not_met"
                );
            }

            // Fallback to headless
            self.extract_headless(url, headless_url, wasm_extractor, request_id).await
        }
    }
}
```

### 5.4 Phase 4: Dashboard Deployment (Week 2-3)

#### File Structure:
```
/workspaces/eventmesh/
├── monitoring/
│   ├── grafana/
│   │   ├── dashboards/
│   │   │   ├── overview.json
│   │   │   ├── gate-analysis.json
│   │   │   ├── performance.json
│   │   │   └── quality.json
│   │   └── provisioning/
│   │       ├── datasources/prometheus.yaml
│   │       └── dashboards/dashboards.yaml
│   ├── prometheus/
│   │   ├── prometheus.yml
│   │   └── alerts/
│   │       ├── extraction.rules
│   │       ├── quality.rules
│   │       └── performance.rules
│   └── docker-compose.monitoring.yml
```

#### Docker Compose Configuration:
```yaml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - ./prometheus/alerts:/etc/prometheus/alerts
      - prometheus-data:/prometheus
    ports:
      - "9090:9090"
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.enable-lifecycle'

  grafana:
    image: grafana/grafana:latest
    volumes:
      - ./grafana/provisioning:/etc/grafana/provisioning
      - ./grafana/dashboards:/var/lib/grafana/dashboards
      - grafana-data:/var/lib/grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_INSTALL_PLUGINS=grafana-piechart-panel

volumes:
  prometheus-data:
  grafana-data:
```

### 5.5 Phase 5: Anomaly Detection (Week 3-4)

Implement statistical anomaly detection using Z-score and moving averages:

```rust
/// Anomaly detection for metrics
pub struct AnomalyDetector {
    window_size: usize,
    z_threshold: f64,
    metrics_buffer: HashMap<String, VecDeque<f64>>,
}

impl AnomalyDetector {
    /// Detect anomalies in extraction duration
    pub fn check_duration_anomaly(&mut self, mode: &str, duration_ms: f64) -> bool {
        let key = format!("duration_{}", mode);
        let buffer = self.metrics_buffer.entry(key).or_insert_with(VecDeque::new);

        buffer.push_back(duration_ms);
        if buffer.len() > self.window_size {
            buffer.pop_front();
        }

        if buffer.len() < self.window_size {
            return false; // Not enough data
        }

        let mean = buffer.iter().sum::<f64>() / buffer.len() as f64;
        let variance = buffer.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / buffer.len() as f64;
        let std_dev = variance.sqrt();

        let z_score = (duration_ms - mean) / std_dev;
        z_score.abs() > self.z_threshold
    }
}
```

---

## 6. Performance Impact Analysis

### 6.1 Overhead Estimation

#### Metrics Collection Overhead
- **Memory**: ~200 bytes per histogram bucket × 50 metrics = ~10KB
- **CPU**: < 1% overhead for metric recording (non-blocking)
- **Network**: Prometheus scrapes add ~5KB per scrape (15s interval)

#### Mitigation Strategies
1. **Non-blocking Recording**: All metrics updates use `tokio::spawn`
2. **Aggregation**: Use histograms instead of raw values
3. **Sampling**: For high-volume metrics, consider sampling (1 in N)
4. **Caching**: Cache metric lookups in hot paths

### 6.2 Storage Requirements

#### Prometheus TSDB
- **Time Series**: ~500 unique series (metrics × labels)
- **Data Point Size**: ~16 bytes per sample
- **Retention**: 30 days
- **Estimated Storage**: 500 series × 2880 samples/day × 16 bytes × 30 days = ~700 MB

#### Optimization
- Use recording rules for frequently queried aggregations
- Downsample older data (e.g., 1-minute resolution after 7 days)

---

## 7. Testing Strategy

### 7.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_decision_metrics() {
        let metrics = RipTideMetrics::new().unwrap();
        let features = GateFeatures {
            html_bytes: 10000,
            visible_text_chars: 5000,
            // ... other fields
        };

        metrics.record_gate_decision_enhanced("raw", 0.85, &features);

        // Verify counters incremented
        assert_eq!(metrics.gate_decisions_raw.get(), 1.0);
    }

    #[tokio::test]
    async fn test_extraction_metrics() {
        let metrics = RipTideMetrics::new().unwrap();
        let doc = ExtractedDoc {
            quality_score: Some(75.0),
            text: "Sample content".repeat(100),
            links: vec![],
            // ... other fields
        };

        metrics.record_extraction_result("raw", 250, true, Some(&doc));

        assert_eq!(metrics.extraction_success_total.get(), 1.0);
    }
}
```

### 7.2 Integration Tests

Test end-to-end metric collection:

```rust
#[tokio::test]
async fn test_metrics_pipeline_integration() {
    let state = setup_test_state().await;
    let orchestrator = PipelineOrchestrator::new(state.clone(), CrawlOptions::default());

    // Execute pipeline
    let result = orchestrator.execute_single("https://example.com").await.unwrap();

    // Verify metrics recorded
    assert!(state.metrics.gate_decisions_raw.get() > 0.0 ||
            state.metrics.gate_decisions_probes_first.get() > 0.0 ||
            state.metrics.gate_decisions_headless.get() > 0.0);

    assert_eq!(state.metrics.extraction_success_total.get(), 1.0);
}
```

### 7.3 Load Testing

Use `k6` or `locust` to validate metrics under load:

```javascript
// k6 load test
import http from 'k6/http';
import { check } from 'k6';

export const options = {
  stages: [
    { duration: '2m', target: 100 },
    { duration: '5m', target: 100 },
    { duration: '2m', target: 0 },
  ],
};

export default function () {
  const res = http.post('http://localhost:8080/api/v1/extract', JSON.stringify({
    url: 'https://example.com',
  }));

  check(res, {
    'status is 200': (r) => r.status === 200,
    'has metrics': (r) => r.json('processing_time_ms') !== undefined,
  });
}
```

---

## 8. Maintenance and Evolution

### 8.1 Metric Lifecycle

1. **Addition**: New metrics require RFC and review
2. **Deprecation**: Mark deprecated metrics with `_deprecated` suffix for 2 versions
3. **Removal**: Delete after 2 version deprecation period

### 8.2 Dashboard Versioning

- Store dashboards in git with semantic versioning
- Use Grafana provisioning for automated deployment
- Maintain changelog for dashboard updates

### 8.3 Alert Tuning

- Review alert thresholds monthly
- Track false positive/negative rates
- Document threshold changes in ADRs

---

## 9. Architecture Decision Records

### ADR-001: Non-Blocking Metrics Collection

**Status:** Accepted
**Date:** 2025-10-13

**Context:** Metrics collection must not impact extraction performance.

**Decision:** Use `tokio::spawn` for all metric recording to ensure non-blocking execution.

**Consequences:**
- ✅ Zero performance impact on hot path
- ✅ Metrics may lag slightly (acceptable)
- ❌ Metrics may be lost on crash (acceptable for monitoring)

### ADR-002: Histogram Buckets for Duration Metrics

**Status:** Accepted
**Date:** 2025-10-13

**Context:** Need to track duration distributions efficiently.

**Decision:** Use Prometheus histograms with carefully chosen buckets:
- Raw mode: 10ms to 5s
- Headless mode: 100ms to 30s

**Consequences:**
- ✅ Efficient percentile calculations
- ✅ Low storage overhead
- ❌ Cannot recalculate percentiles post-hoc

### ADR-003: Separate Metrics for Each Extraction Mode

**Status:** Accepted
**Date:** 2025-10-13

**Context:** Need to compare performance across modes.

**Decision:** Use `mode` label on extraction metrics instead of separate metric names.

**Consequences:**
- ✅ Easier to compare modes in queries
- ✅ Fewer total metrics
- ❌ Slightly more complex PromQL queries

---

## 10. Deployment Checklist

### Pre-Deployment

- [ ] Metrics code review completed
- [ ] Unit tests passing (>80% coverage)
- [ ] Integration tests passing
- [ ] Load test results acceptable (< 1% overhead)
- [ ] Dashboard JSON validated
- [ ] Alert rules tested in staging

### Deployment

- [ ] Deploy Prometheus with updated scrape config
- [ ] Deploy Grafana with dashboard provisioning
- [ ] Configure alerting (Slack/PagerDuty)
- [ ] Enable metrics endpoint in RipTide API
- [ ] Verify metrics appearing in Prometheus
- [ ] Verify dashboards rendering correctly

### Post-Deployment

- [ ] Monitor for metric gaps
- [ ] Validate alert firing (test scenarios)
- [ ] Baseline performance metrics
- [ ] Update runbooks with new metrics
- [ ] Train team on new dashboards

---

## 11. Future Enhancements

### Machine Learning Integration
- Train models on historical metrics to predict optimal extraction mode
- Automated threshold tuning using ML
- Anomaly detection using autoencoders

### Distributed Tracing
- Integrate OpenTelemetry for end-to-end trace correlation
- Link metrics to traces for deeper debugging

### Cost Tracking
- Calculate cost per extraction mode (CPU, memory, time)
- Optimize gate thresholds for cost efficiency

---

## Appendix A: Metric Naming Conventions

### Standards
- Prefix: `riptide_`
- Component: `gate_`, `extraction_`, `wasm_`, etc.
- Metric type: `_total` (counter), `_bytes` (gauge), `_duration_ms` (histogram)
- Labels: `{service="riptide-api", mode="raw|headless|probes"}`

### Examples
```
✅ riptide_gate_decisions_raw_total{service="riptide-api"}
✅ riptide_extraction_duration_ms{service="riptide-api", mode="raw"}
✅ riptide_extraction_quality_score{service="riptide-api", mode="headless"}

❌ gate_raw_count (missing prefix)
❌ riptide_extraction_time (ambiguous unit)
❌ riptide_gate_decision{decision="raw"} (use separate metrics for decisions)
```

---

## Appendix B: PromQL Query Cookbook

### Gate Decision Distribution (Last 5 minutes)
```promql
sum by (decision) (
  rate(riptide_gate_decisions_raw_total[5m]) or
  rate(riptide_gate_decisions_probes_first_total[5m]) or
  rate(riptide_gate_decisions_headless_total[5m])
)
```

### Extraction Success Rate by Mode
```promql
sum by (mode) (rate(riptide_extraction_success_total[5m])) /
sum by (mode) (
  rate(riptide_extraction_success_total[5m]) +
  rate(riptide_extraction_failure_total[5m])
)
```

### P95 Latency Comparison
```promql
histogram_quantile(0.95,
  rate(riptide_extraction_duration_ms_bucket[5m])
) by (mode)
```

### Quality Score Moving Average
```promql
avg_over_time(
  avg(riptide_extraction_quality_score)[5m:1m]
) by (mode)
```

---

## Document Change Log

| Version | Date       | Author             | Changes                                    |
|---------|------------|--------------------|--------------------------------------------|
| 1.0     | 2025-10-13 | System Architect   | Initial architecture design                |

---

**Document Status:** ✅ Ready for Review
**Next Steps:** Implementation Phase 1 - Core Metrics Extension
**Review Required By:** Tech Lead, DevOps Lead
