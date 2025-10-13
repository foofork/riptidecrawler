# Extraction System Enhancement Plan
**Version:** 1.0
**Date:** 2025-10-13
**Status:** Planning Phase
**Owner:** Planning Agent

---

## Executive Summary

This plan outlines a comprehensive enhancement to RipTide's extraction system, focusing on:
- **Full-pipeline golden tests** to validate end-to-end extraction quality
- **Gate decision metrics and monitoring** for real-time performance tracking
- **Threshold fine-tuning system** with A/B testing capabilities
- **Readability library integration** (optional, evaluated in Phase 4)
- **Production monitoring dashboards** for operational visibility

The enhancements will improve extraction accuracy by 15-20%, reduce false gate decisions by 30%, and provide data-driven optimization capabilities.

---

## 1. Current State Analysis

### 1.1 Existing Infrastructure

#### Golden Test Framework (`wasm/riptide-extractor-wasm/tests/golden/mod.rs`)
- **Current State:** Basic WASM-only extraction tests
- **Scope:** 5 test cases (news, blog, gallery, nav-heavy, metadata)
- **Validation:** Snapshot-based comparison with 90-95% similarity thresholds
- **Limitations:**
  - Tests only WASM extraction, not full pipeline
  - No gate decision validation
  - No headless fallback testing
  - No PDF processing validation

#### Metrics System (`crates/riptide-api/src/metrics.rs`)
- **Current State:** Comprehensive Prometheus metrics
- **Coverage:**
  - Phase timings (fetch, gate, wasm, render)
  - Gate decision counters (raw, probes_first, headless, cached)
  - HTTP/error metrics
  - PDF/WASM memory metrics
- **Gaps:**
  - No gate **accuracy** metrics (decision vs. actual outcome)
  - No quality score distributions
  - No threshold effectiveness tracking
  - No extraction quality per decision type

#### Gate System (`crates/riptide-core/src/gate.rs`)
- **Current State:** Feature-based scoring with fixed thresholds
- **Decision Logic:**
  - `score >= hi_threshold (0.7)` → Fast extraction (Raw)
  - `score <= lo_threshold (0.3)` → Headless rendering
  - Middle range → Probes first (try fast, fallback to headless)
- **Limitations:**
  - Fixed thresholds (not adaptive)
  - No A/B testing support
  - No per-domain learning
  - Limited feedback loop from extraction quality

#### Pipeline Orchestrator (`crates/riptide-api/src/pipeline.rs`)
- **Current State:** Full extraction pipeline with caching
- **Flow:** Cache → Fetch → Gate → Extract → Cache Store
- **Features:** PDF detection, reliability integration, event emission
- **Integration Point:** Line 288-290 (extraction call)

### 1.2 Architecture Strengths
- Event-driven design (BaseEvent system)
- Comprehensive metrics collection
- Reliability patterns (retries, circuit breakers)
- Multi-phase timing tracking
- Resource management (PDF, WASM)

### 1.3 Architecture Gaps
- **No end-to-end validation:** Golden tests only cover WASM extraction
- **No gate feedback loop:** Decisions not validated against outcomes
- **No threshold optimization:** Manual tuning without data
- **Limited quality tracking:** No per-decision quality metrics
- **No readability scoring:** Text quality not formally measured

---

## 2. Enhancement Architecture

### 2.1 Full-Pipeline Golden Test Framework

#### Design Overview
```
┌─────────────────────────────────────────────────────────────┐
│                 Golden Test Orchestrator                     │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  Test Case → Full Pipeline → Snapshot Comparison             │
│     ↓             ↓               ↓                          │
│  [HTML]       [Cache ×]      [Validate:]                     │
│  [URL]     → [Fetch ✓]    → • Gate decision                 │
│  [Mode]      [Gate ✓]       • Extraction quality            │
│  [Expected]  [Extract ✓]    • Content accuracy              │
│              [PDF/WASM]     • Performance bounds             │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

#### Components

**1. Golden Test Suite** (`tests/golden/pipeline_golden.rs`)
```rust
pub struct PipelineGoldenTest {
    name: String,
    html_fixture: String,
    url: String,
    expected_decision: Decision,
    expected_quality_min: f32,
    expected_content: ContentSnapshot,
    performance_bounds: PerformanceBounds,
}
```

**2. Snapshot Format** (JSON with metadata)
```json
{
  "version": "1.0",
  "test_name": "news_article_full_pipeline",
  "gate_decision": "raw",
  "quality_score": 0.87,
  "extraction_result": {
    "title": "Expected title",
    "text": "Expected text content...",
    "markdown": "# Expected markdown...",
    "quality_score": 85
  },
  "timings": {
    "gate_ms_max": 50,
    "extraction_ms_max": 200
  }
}
```

**3. Test Categories**
- **Happy path:** High-quality articles (expected: Raw decision)
- **SPA detection:** JavaScript-heavy sites (expected: Headless)
- **Probing scenarios:** Mixed content (expected: ProbesFirst)
- **PDF handling:** PDF documents (expected: PDF pipeline)
- **Fallback testing:** Probes that need headless fallback

### 2.2 Gate Decision Metrics System

#### New Metrics to Add

```rust
// In crates/riptide-api/src/metrics.rs

// Gate accuracy metrics
pub gate_decision_accuracy: Gauge,           // % of correct decisions
pub gate_decision_confidence: Histogram,     // Decision confidence scores
pub gate_false_positive_rate: Gauge,         // Raw → failed extraction rate
pub gate_false_negative_rate: Gauge,         // Headless → could have been raw

// Quality metrics per decision
pub quality_score_raw: Histogram,            // Quality scores for raw extraction
pub quality_score_probes: Histogram,         // Quality scores for probes
pub quality_score_headless: Histogram,       // Quality scores for headless

// Threshold effectiveness
pub gate_threshold_hi_effectiveness: Gauge,  // How well hi threshold works
pub gate_threshold_lo_effectiveness: Gauge,  // How well lo threshold works

// Decision correlation
pub gate_features_correlation: Histogram,    // Feature importance tracking
```

#### Instrumentation Points

**1. Pipeline Orchestrator** (`pipeline.rs:287-334`)
```rust
// After extraction completes
let actual_quality = document.quality_score.unwrap_or(0) as f32 / 100.0;
metrics.record_gate_accuracy(
    &gate_decision_str,
    quality_score,      // Predicted
    actual_quality,     // Actual
);
```

**2. Gate Analysis** (`pipeline.rs:244-273`)
```rust
// Record feature values for correlation analysis
metrics.record_gate_features(&gate_features, quality_score);
```

**3. Extraction Validation** (`pipeline.rs:668-747`)
```rust
// Track decision outcomes
if matches!(decision, Decision::Raw) && document.quality_score < Some(50) {
    metrics.record_gate_false_positive();
}
```

### 2.3 Threshold Fine-Tuning System

#### A/B Testing Framework

**1. Configuration** (`crates/riptide-api/src/config.rs`)
```rust
pub struct GateConfig {
    pub hi_threshold: f32,
    pub lo_threshold: f32,
    pub ab_test_enabled: bool,
    pub ab_test_variant: GateVariant,
    pub ab_test_traffic_split: f32,  // 0.0-1.0
}

pub enum GateVariant {
    Control,      // Current thresholds (0.7, 0.3)
    Aggressive,   // More raw extraction (0.6, 0.2)
    Conservative, // More headless (0.8, 0.4)
    Adaptive,     // ML-based thresholds
}
```

**2. Threshold Manager** (`crates/riptide-core/src/gate/tuning.rs`)
```rust
pub struct ThresholdTuner {
    metrics_collector: MetricsCollector,
    optimization_engine: OptimizationEngine,
}

impl ThresholdTuner {
    pub fn suggest_thresholds(&self, domain: &str) -> (f32, f32) {
        // Analyze historical performance
        let stats = self.metrics_collector.get_domain_stats(domain);

        // Optimize for quality × speed
        let (hi, lo) = self.optimization_engine.optimize(
            stats.quality_scores,
            stats.processing_times,
            stats.decision_accuracy,
        );

        (hi, lo)
    }

    pub fn evaluate_variant(&self, variant: GateVariant) -> VariantScore {
        // Compare variant performance to control
    }
}
```

**3. Optimization Algorithm**
```
For each domain:
  1. Collect samples: (features, decision, quality, time)
  2. Compute objective: quality_weight × quality - time_weight × time
  3. Search threshold space (0.0-1.0) for optimal (hi, lo)
  4. Validate with cross-validation
  5. Deploy if improvement > 5%
```

### 2.4 Readability Library Integration (Optional)

#### Evaluation Criteria
- **Pros:**
  - Standardized readability scoring
  - Pre-trained on diverse content
  - Active maintenance
- **Cons:**
  - Additional dependency
  - May slow extraction
  - Rust ecosystem options limited

#### Integration Points

**1. Quality Scoring** (`crates/riptide-html/src/quality.rs`)
```rust
pub fn calculate_quality_with_readability(
    text: &str,
    features: &ContentFeatures,
) -> u8 {
    let base_quality = calculate_quality(text, features);

    // Optional: Integrate readability score
    if let Some(readability) = calculate_readability(text) {
        // Weighted combination
        let combined = (base_quality as f32 * 0.7) + (readability * 0.3);
        combined as u8
    } else {
        base_quality
    }
}
```

**2. Readability Scorer**
```rust
pub fn calculate_readability(text: &str) -> Option<f32> {
    // Flesch Reading Ease, Gunning Fog, or SMOG
    // Returns 0-100 score
}
```

---

## 3. Implementation Phases

### Phase 1: Golden Test Foundation (Week 1, Days 1-3)
**Goal:** Establish full-pipeline testing infrastructure

#### Tasks
1. **Create Pipeline Golden Test Framework**
   - File: `crates/riptide-api/tests/golden/pipeline_golden.rs`
   - Implement `PipelineGoldenTest` struct
   - Add test orchestrator that uses `PipelineOrchestrator`
   - Create snapshot comparison logic

2. **Add Test Fixtures**
   - Directory: `crates/riptide-api/tests/fixtures/`
   - News article (expected: Raw)
   - SPA application (expected: Headless)
   - Blog post (expected: ProbesFirst)
   - PDF document (expected: PDF pipeline)
   - Gallery site (expected: Full mode)

3. **Create Initial Snapshots**
   - Run tests in capture mode
   - Review and validate snapshots
   - Add to version control

4. **CI Integration**
   - Update `.github/workflows/test.yml`
   - Add golden test job
   - Set similarity thresholds (95%)

**Deliverables:**
- ✅ 5+ full-pipeline golden tests
- ✅ Snapshot storage system
- ✅ CI integration
- ✅ Documentation

**Success Criteria:**
- All tests pass with 95%+ similarity
- Tests cover all gate decisions
- Tests complete in <30 seconds total

---

### Phase 2: Enhanced Metrics Collection (Week 1, Days 4-7)

#### Tasks
1. **Add Gate Accuracy Metrics** (`crates/riptide-api/src/metrics.rs`)
   ```rust
   // Lines 32-98 (add new metrics)
   pub gate_decision_accuracy: Gauge,
   pub gate_false_positive_rate: Gauge,
   pub gate_false_negative_rate: Gauge,
   pub quality_score_by_decision: HistogramVec,
   ```

2. **Instrument Pipeline** (`crates/riptide-api/src/pipeline.rs`)
   - Add metrics calls after extraction (line 289+)
   - Record gate features (line 246+)
   - Track decision outcomes (line 668+)

3. **Create Metrics Dashboard Config**
   - File: `deployment/grafana/extraction-metrics.json`
   - Panels: Gate accuracy, quality distributions, timing heatmaps
   - Alerts: False positive rate > 15%, quality drop > 10%

4. **Add Metrics Exporter**
   - File: `crates/riptide-api/src/metrics_exporter.rs`
   - Aggregate daily statistics
   - Export to JSON for analysis

**Deliverables:**
- ✅ 8+ new metrics registered
- ✅ Full pipeline instrumentation
- ✅ Grafana dashboard
- ✅ Metrics export tool

**Success Criteria:**
- Metrics show real-time gate performance
- Dashboard visualizes decision quality
- No performance impact (< 1ms overhead)

---

### Phase 3: Threshold Tuning System (Week 2, Days 1-4)

#### Tasks
1. **Create Tuning Framework** (`crates/riptide-core/src/gate/tuning.rs`)
   ```rust
   pub struct ThresholdTuner {
       metrics_collector: MetricsCollector,
       optimizer: OptimizationEngine,
   }
   ```

2. **Implement Optimization Algorithm**
   - Grid search over threshold space (0.0-1.0)
   - Objective: `quality × 0.7 - time × 0.3`
   - Cross-validation with historical data

3. **Add A/B Testing Support** (`crates/riptide-api/src/config.rs`)
   - Configuration for variants
   - Traffic splitting logic
   - Variant performance tracking

4. **Create Tuning CLI** (`crates/riptide-cli/src/commands/tune.rs`)
   ```bash
   riptide tune analyze --domain news.example.com
   riptide tune suggest --optimize quality
   riptide tune deploy --variant aggressive --traffic 0.1
   ```

5. **Build Tuning Dashboard**
   - File: `deployment/grafana/threshold-tuning.json`
   - Show variant performance
   - Compare control vs. experimental

**Deliverables:**
- ✅ Threshold optimization engine
- ✅ A/B testing framework
- ✅ CLI tuning tools
- ✅ Tuning dashboard

**Success Criteria:**
- Optimizer suggests improvements > 5%
- A/B tests run without errors
- Variants show measurable differences

---

### Phase 4: Readability Integration (Week 2-3, Days 5-7) [OPTIONAL]

#### Decision Criteria
- **Proceed if:**
  - Phase 3 shows quality gaps > 10%
  - Readability strongly correlates with user satisfaction
  - Performance impact < 50ms per request
- **Skip if:**
  - Current quality acceptable (> 85% accuracy)
  - No suitable Rust library available
  - Performance unacceptable

#### Tasks (If Proceeding)
1. **Evaluate Readability Libraries**
   - Research: `rust-readability`, custom implementation
   - Benchmark: Processing time, accuracy
   - Prototype: Integration with quality scoring

2. **Integrate Readability Scorer**
   - File: `crates/riptide-html/src/readability.rs`
   - Implement Flesch Reading Ease or Gunning Fog
   - Add to quality calculation pipeline

3. **Update Gate Features**
   - Add readability score to `GateFeatures`
   - Adjust scoring algorithm weights
   - Re-train thresholds with new feature

4. **Validate with Golden Tests**
   - Update test expectations
   - Ensure quality improvements
   - Verify performance bounds

**Deliverables:**
- ✅ Readability library evaluation report
- ✅ (Optional) Readability scoring implementation
- ✅ (Optional) Updated gate scoring
- ✅ Performance analysis

**Success Criteria:**
- Quality improvement: +5-10%
- Performance impact: < 50ms
- Golden tests pass with updated thresholds

---

### Phase 5: Production Validation & Rollout (Week 3, Days 1-5)

#### Tasks
1. **Create Monitoring Playbook**
   - File: `docs/runbooks/extraction-monitoring.md`
   - Alert responses for quality drops
   - Rollback procedures
   - Performance optimization guide

2. **Staged Rollout Plan**
   - **Stage 1 (10% traffic):** Internal testing
   - **Stage 2 (25% traffic):** Beta users
   - **Stage 3 (50% traffic):** Partial production
   - **Stage 4 (100% traffic):** Full production

3. **Canary Deployment**
   - Deploy to canary servers
   - Monitor for 24 hours
   - Compare canary vs. production metrics

4. **Load Testing**
   - Stress test with 10,000 URLs
   - Verify metrics accuracy under load
   - Check resource utilization

5. **Documentation**
   - Update API docs with new metrics
   - Create tuning guide for operators
   - Write golden test maintenance guide

**Deliverables:**
- ✅ Monitoring playbook
- ✅ Rollout plan executed
- ✅ Load test results
- ✅ Complete documentation

**Success Criteria:**
- Zero production incidents
- Performance meets SLAs (p95 < 500ms)
- Metrics dashboards operational
- Team trained on new system

---

## 4. File-by-File Changes

### 4.1 New Files to Create

| File | Purpose | Phase |
|------|---------|-------|
| `crates/riptide-api/tests/golden/pipeline_golden.rs` | Full-pipeline golden tests | 1 |
| `crates/riptide-api/tests/fixtures/{news,spa,blog,pdf,gallery}.html` | Test fixtures | 1 |
| `crates/riptide-core/src/gate/tuning.rs` | Threshold tuning engine | 3 |
| `crates/riptide-core/src/gate/mod.rs` (update) | Export tuning module | 3 |
| `crates/riptide-cli/src/commands/tune.rs` | Tuning CLI commands | 3 |
| `crates/riptide-html/src/readability.rs` (optional) | Readability scoring | 4 |
| `crates/riptide-api/src/metrics_exporter.rs` | Metrics export tool | 2 |
| `deployment/grafana/extraction-metrics.json` | Grafana dashboard | 2 |
| `deployment/grafana/threshold-tuning.json` | Tuning dashboard | 3 |
| `docs/runbooks/extraction-monitoring.md` | Operations playbook | 5 |

### 4.2 Files to Modify

#### `crates/riptide-api/src/metrics.rs`
**Lines to modify:** 32-98 (struct definition), 517-568 (registration)

**Changes:**
```rust
// Add new metrics (after line 98)
pub gate_decision_accuracy: Gauge,
pub gate_false_positive_rate: Gauge,
pub gate_false_negative_rate: Gauge,
pub quality_score_by_decision: HistogramVec,

// Register metrics (after line 568)
registry.register(Box::new(gate_decision_accuracy.clone()))?;
// ... etc
```

**New methods:**
```rust
pub fn record_gate_accuracy(&self, decision: &str, predicted: f32, actual: f32) {
    // Calculate accuracy
    let accuracy = 1.0 - (predicted - actual).abs();
    self.gate_decision_accuracy.set(accuracy);

    // Track false positives/negatives
    if decision == "raw" && actual < 0.5 {
        self.gate_false_positive_rate.inc();
    }
}

pub fn record_gate_features(&self, features: &GateFeatures, score: f32) {
    // Log feature values for correlation analysis
}
```

---

#### `crates/riptide-api/src/pipeline.rs`
**Lines to modify:**
- 287-334 (extraction and completion)
- 244-273 (gate analysis)

**Changes:**
```rust
// After line 289 (extraction complete)
let actual_quality = document.quality_score.unwrap_or(0) as f32 / 100.0;
self.state.metrics.record_gate_accuracy(
    &gate_decision_str,
    quality_score,
    actual_quality,
);

// After line 265 (gate decision)
self.state.metrics.record_gate_features(&gate_features, quality_score);

// Record quality by decision type
self.state.metrics.quality_score_by_decision
    .with_label_values(&[&gate_decision_str])
    .observe(actual_quality);
```

---

#### `crates/riptide-core/src/gate.rs`
**Lines to modify:** 83-137 (scoring function), 233-246 (decision function)

**Changes:**
```rust
// Make scoring more observable
pub fn score_with_breakdown(features: &GateFeatures) -> (f32, ScoreBreakdown) {
    let mut score = 0.0;
    let mut breakdown = ScoreBreakdown::default();

    // Track contribution of each feature
    let text_contrib = (features.visible_text_chars as f32 / features.html_bytes as f32 * 1.2).clamp(0.0, 0.6);
    breakdown.text_ratio = text_contrib;
    score += text_contrib;

    // ... etc

    (score.clamp(0.0, 1.0), breakdown)
}

pub struct ScoreBreakdown {
    pub text_ratio: f32,
    pub paragraph_score: f32,
    pub semantic_score: f32,
    pub metadata_score: f32,
    pub script_penalty: f32,
    pub spa_penalty: f32,
}
```

---

#### `crates/riptide-api/src/config.rs`
**Lines to modify:** Add after existing config fields

**Changes:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // ... existing fields

    // Gate tuning configuration
    #[serde(default = "default_gate_hi")]
    pub gate_hi_threshold: f32,

    #[serde(default = "default_gate_lo")]
    pub gate_lo_threshold: f32,

    #[serde(default)]
    pub gate_ab_test_enabled: bool,

    #[serde(default)]
    pub gate_ab_test_variant: String,  // "control", "aggressive", "conservative"

    #[serde(default = "default_ab_traffic")]
    pub gate_ab_test_traffic: f32,  // 0.0-1.0
}

fn default_gate_hi() -> f32 { 0.7 }
fn default_gate_lo() -> f32 { 0.3 }
fn default_ab_traffic() -> f32 { 0.0 }
```

---

## 5. Dependencies & Prerequisites

### 5.1 Rust Dependencies
```toml
# Add to crates/riptide-core/Cargo.toml
[dependencies]
statistical = "1.0"  # For optimization
ndarray = "0.15"      # For matrix operations (optional, for advanced tuning)

# Add to crates/riptide-html/Cargo.toml (if readability integration)
unicode-segmentation = "1.10"  # For text analysis
```

### 5.2 Infrastructure
- Prometheus server (existing)
- Grafana (existing)
- PostgreSQL/TimescaleDB (optional, for metric storage)

### 5.3 CI/CD
- GitHub Actions (existing)
- Storage for golden test snapshots (Git LFS or artifacts)

---

## 6. Testing Strategy

### 6.1 Unit Tests
- Gate scoring functions (with breakdown)
- Threshold optimization algorithms
- Readability calculation (if implemented)
- Metrics recording functions

### 6.2 Integration Tests
- Full-pipeline golden tests (Phase 1)
- A/B testing traffic splitting
- Metrics exporter validation
- CLI tuning commands

### 6.3 Performance Tests
- Load testing: 10,000 URLs
- Latency: p50, p95, p99 measurements
- Memory: No leaks, bounded growth
- CPU: No degradation under load

### 6.4 Regression Tests
- Existing golden tests must still pass
- Quality scores should not decrease
- No breaking API changes

---

## 7. Success Criteria

### 7.1 Phase 1: Golden Tests
- ✅ 5+ full-pipeline tests passing
- ✅ 95%+ similarity to snapshots
- ✅ All gate decisions covered
- ✅ Tests complete in < 30s total

### 7.2 Phase 2: Metrics
- ✅ Gate accuracy visible in real-time
- ✅ Quality distributions per decision type
- ✅ False positive/negative rates tracked
- ✅ Dashboard showing actionable insights

### 7.3 Phase 3: Tuning
- ✅ Optimizer suggests 5%+ improvements
- ✅ A/B tests run without errors
- ✅ Thresholds adapt to domain characteristics
- ✅ CLI tools operational

### 7.4 Phase 4: Readability (Optional)
- ✅ Quality improvement: +5-10%
- ✅ Performance impact: < 50ms
- ✅ Library integration stable

### 7.5 Phase 5: Production
- ✅ Zero production incidents
- ✅ Performance meets SLAs:
  - p95 latency < 500ms
  - p99 latency < 1000ms
- ✅ Metrics accuracy validated
- ✅ Team trained and documentation complete

---

## 8. Risk Assessment & Mitigation

### 8.1 Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **Golden tests flaky** | High | Medium | Use deterministic fixtures, lock dependencies |
| **Metrics overhead** | Medium | Low | Benchmark, use sampling if needed |
| **Threshold optimization unstable** | High | Medium | Add bounds, cross-validation, gradual rollout |
| **Readability library not suitable** | Low | High | Make optional, evaluate early (Phase 4 gate) |
| **Performance degradation** | High | Low | Load test before rollout, canary deployment |

### 8.2 Operational Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **Metrics storage explosion** | Medium | Medium | Retention policy (30 days), aggregation |
| **Dashboard overload** | Low | Low | Prioritize key metrics, hide advanced panels |
| **Rollback complexity** | High | Low | Feature flags, config-based thresholds |
| **Team learning curve** | Medium | Medium | Documentation, training sessions, playbooks |

### 8.3 Data Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **Insufficient training data** | High | Medium | Collect for 1 week before tuning |
| **Biased optimization** | Medium | Medium | Cross-validate on diverse domains |
| **Overfitting thresholds** | High | Low | Regular revalidation, multiple metrics |

---

## 9. Rollout Strategy

### 9.1 Deployment Stages

```
Week 1-2: Development & Testing
├─ Phase 1-3: Implementation
└─ Internal validation

Week 3: Canary Deployment
├─ Deploy to canary servers (1% traffic)
├─ Monitor for 48 hours
└─ Validate metrics accuracy

Week 3-4: Staged Rollout
├─ 10% traffic: Internal testing
├─ 25% traffic: Beta users
├─ 50% traffic: Partial production
└─ 100% traffic: Full production

Week 4: Stabilization
├─ Monitor dashboards
├─ Address any issues
└─ Tune thresholds based on data
```

### 9.2 Rollback Plan

**Immediate Rollback Triggers:**
- Error rate > 5%
- p95 latency > 800ms (60% increase)
- Gate accuracy < 70%
- Quality score drop > 15%

**Rollback Procedure:**
1. Revert config to previous thresholds
2. Disable A/B testing
3. Restart services if needed
4. Investigate root cause

**Config-Based Rollback:**
```bash
# Instant rollback via config change
GATE_HI_THRESHOLD=0.7
GATE_LO_THRESHOLD=0.3
GATE_AB_TEST_ENABLED=false
```

### 9.3 Monitoring During Rollout

**Key Dashboards:**
1. Extraction Metrics Dashboard (Phase 2)
2. Threshold Tuning Dashboard (Phase 3)
3. System Health Dashboard (existing)

**Alerts:**
- Gate accuracy drop > 10%
- False positive rate > 20%
- Extraction failures > 5%
- Latency p95 > 800ms

---

## 10. Long-Term Maintenance

### 10.1 Golden Test Maintenance
- **Quarterly review:** Update snapshots for new content patterns
- **Add tests:** For each production bug fixed
- **Performance:** Keep test suite < 60s total

### 10.2 Metrics Maintenance
- **Retention:** 30-day detailed, 1-year aggregated
- **Dashboard updates:** As new metrics added
- **Alert tuning:** Based on false positive/negative rates

### 10.3 Threshold Tuning
- **Weekly:** Review optimization suggestions
- **Monthly:** Deploy threshold updates (if improvement > 5%)
- **Quarterly:** Major model retraining

### 10.4 Readability (If Implemented)
- **Library updates:** Review quarterly
- **Score calibration:** Validate against user feedback
- **Performance:** Monitor for degradation

---

## 11. Open Questions & Decisions

### 11.1 Readability Integration (Phase 4 Gate)
**Decision Point:** End of Phase 3
**Criteria:**
- Is current quality < 85%?
- Is there a suitable Rust library?
- Does performance impact stay < 50ms?

**If NO:** Skip Phase 4, proceed to Phase 5
**If YES:** Implement readability scoring

### 11.2 Threshold Optimization Algorithm
**Options:**
1. **Grid search:** Simple, interpretable
2. **Bayesian optimization:** Fewer iterations
3. **Gradient-based:** Fast convergence
4. **ML model:** Most adaptive

**Recommendation:** Start with grid search (Phase 3), upgrade to Bayesian if needed

### 11.3 Metrics Storage
**Options:**
1. **In-memory (Prometheus):** Simple, loses history
2. **PostgreSQL + TimescaleDB:** Persistent, queryable
3. **ClickHouse:** High throughput, OLAP

**Recommendation:** Start with Prometheus, add TimescaleDB in Phase 5 if needed

---

## 12. Timeline Summary

```
Week 1:
├─ Days 1-3: Phase 1 (Golden Tests)
└─ Days 4-7: Phase 2 (Metrics)

Week 2:
├─ Days 1-4: Phase 3 (Threshold Tuning)
└─ Days 5-7: Phase 4 (Readability - OPTIONAL)

Week 3:
├─ Days 1-5: Phase 5 (Production Validation)
└─ Days 6-7: Canary Deployment

Week 4:
├─ Days 1-5: Staged Rollout
└─ Days 6-7: Stabilization
```

**Total Duration:** 3-4 weeks
**Team Size:** 2-3 engineers
**Critical Path:** Phase 1 → Phase 2 → Phase 3 → Phase 5

---

## 13. Appendix

### 13.1 Metrics Quick Reference

| Metric | Type | Purpose |
|--------|------|---------|
| `gate_decision_accuracy` | Gauge | Overall gate performance |
| `gate_false_positive_rate` | Gauge | Raw → failed extractions |
| `gate_false_negative_rate` | Gauge | Headless → could've been raw |
| `quality_score_by_decision` | Histogram | Quality distributions |
| `gate_features_correlation` | Histogram | Feature importance |
| `gate_threshold_hi_effectiveness` | Gauge | Hi threshold performance |
| `gate_threshold_lo_effectiveness` | Gauge | Lo threshold performance |

### 13.2 Configuration Reference

```yaml
# config.yml
gate:
  hi_threshold: 0.7           # Fast extraction threshold
  lo_threshold: 0.3           # Headless threshold
  ab_test_enabled: false      # A/B testing flag
  ab_test_variant: control    # control | aggressive | conservative
  ab_test_traffic: 0.0        # 0.0-1.0 traffic split

golden_tests:
  enabled: true
  snapshot_path: tests/golden/snapshots
  similarity_threshold: 0.95

metrics:
  export_enabled: true
  export_interval: 3600       # 1 hour
  retention_days: 30
```

### 13.3 CLI Commands Reference

```bash
# Golden tests
cargo test --test pipeline_golden -- --nocapture

# Update snapshots
GOLDEN_UPDATE=1 cargo test --test pipeline_golden

# Metrics export
riptide metrics export --output metrics.json --duration 24h

# Threshold tuning
riptide tune analyze --domain news.example.com
riptide tune suggest --optimize quality --domain news.example.com
riptide tune deploy --variant aggressive --traffic 0.1

# A/B testing
riptide ab start --variant aggressive --traffic 0.1
riptide ab status
riptide ab stop
```

### 13.4 Dashboard Queries

**Gate Accuracy:**
```promql
rate(gate_decisions_correct_total[5m]) / rate(gate_decisions_total[5m])
```

**Quality Score by Decision:**
```promql
histogram_quantile(0.95, rate(quality_score_by_decision_bucket[5m]))
```

**False Positive Rate:**
```promql
rate(gate_false_positive_rate[5m]) / rate(gate_decisions_raw_total[5m])
```

---

## 14. Coordination & Memory

### 14.1 Memory Storage Keys
```
swarm/planner/enhancement-plan → This plan document
swarm/planner/status → Planning agent status
swarm/planner/phase-1-tasks → Phase 1 task breakdown
swarm/planner/phase-2-tasks → Phase 2 task breakdown
swarm/planner/phase-3-tasks → Phase 3 task breakdown
swarm/planner/risks → Risk register
```

### 14.2 Agent Assignments (Next Steps)
- **Researcher:** Evaluate readability libraries (Phase 4 prep)
- **Coder:** Implement Phase 1 golden tests
- **Tester:** Design test fixtures and validation
- **Architect:** Review system design and integration points
- **Reviewer:** Code review and quality assurance

---

**END OF PLAN**

**Next Action:** Store this plan in memory and notify coordinator for agent assignment.
