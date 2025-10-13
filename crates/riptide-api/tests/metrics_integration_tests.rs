//! Integration tests for the comprehensive metrics system (30+ metrics)
//! Tests WEEK 1 PHASE 1B metrics recording and retrieval

use prometheus::Registry;
use riptide_api::metrics::{ErrorType, PhaseType, RipTideMetrics};
use std::time::Instant;

#[test]
fn test_metrics_initialization() {
    let result = RipTideMetrics::new();
    assert!(result.is_ok(), "Metrics initialization should succeed");

    let metrics = result.unwrap();
    assert!(!metrics.registry.gather().is_empty(), "Registry should contain metrics");
}

#[test]
fn test_gate_decision_enhanced_metrics() {
    let metrics = RipTideMetrics::new().unwrap();

    // Record enhanced gate decision
    metrics.record_gate_decision_enhanced(
        "raw",
        0.85,
        0.45,
        0.15,
        3,
        2.5,
    );

    // Verify metrics were recorded
    let metric_families = metrics.registry.gather();

    // Check gate decision counter
    let gate_decisions = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_gate_decision_total");
    assert!(gate_decisions.is_some(), "Gate decision metric should exist");

    // Check gate score histogram
    let gate_score = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_gate_score");
    assert!(gate_score.is_some(), "Gate score metric should exist");

    // Check feature metrics
    let text_ratio = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_gate_feature_text_ratio");
    assert!(text_ratio.is_some(), "Text ratio metric should exist");

    let script_density = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_gate_feature_script_density");
    assert!(script_density.is_some(), "Script density metric should exist");
}

#[test]
fn test_extraction_quality_metrics() {
    let metrics = RipTideMetrics::new().unwrap();

    // Record extraction result
    metrics.record_extraction_result(
        "raw",
        150,
        true,
        85.0,
        5000,
        25,
        10,
        true,
        true,
    );

    let metric_families = metrics.registry.gather();

    // Check quality score
    let quality_score = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_extraction_quality_score");
    assert!(quality_score.is_some(), "Quality score metric should exist");

    // Check content length
    let content_length = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_extraction_content_length_bytes");
    assert!(content_length.is_some(), "Content length metric should exist");

    // Check links found
    let links_found = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_extraction_links_found");
    assert!(links_found.is_some(), "Links found metric should exist");

    // Check images found
    let images_found = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_extraction_images_found");
    assert!(images_found.is_some(), "Images found metric should exist");

    // Check author presence
    let has_author = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_extraction_has_author_total");
    assert!(has_author.is_some(), "Has author metric should exist");

    // Check date presence
    let has_date = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_extraction_has_date_total");
    assert!(has_date.is_some(), "Has date metric should exist");
}

#[test]
fn test_extraction_fallback_metrics() {
    let metrics = RipTideMetrics::new().unwrap();

    // Record fallback event
    metrics.record_extraction_fallback("raw", "headless", "low_quality");

    let metric_families = metrics.registry.gather();

    let fallback = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_extraction_fallback_triggered_total");
    assert!(fallback.is_some(), "Fallback metric should exist");
}

#[test]
fn test_pipeline_phase_timing_metrics() {
    let metrics = RipTideMetrics::new().unwrap();

    // Record pipeline phase timings
    metrics.record_pipeline_phase_ms("gate_analysis", 5.5);
    metrics.record_pipeline_phase_ms("extraction", 125.0);

    let metric_families = metrics.registry.gather();

    let gate_analysis = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_pipeline_phase_gate_analysis_milliseconds");
    assert!(gate_analysis.is_some(), "Gate analysis phase metric should exist");

    let extraction = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_pipeline_phase_extraction_milliseconds");
    assert!(extraction.is_some(), "Extraction phase metric should exist");
}

#[test]
fn test_metrics_non_blocking() {
    // Verify metrics recording doesn't block pipeline
    let metrics = RipTideMetrics::new().unwrap();
    let start = Instant::now();

    // Simulate 100 metric recordings
    for i in 0..100_u32 {
        metrics.record_gate_decision_enhanced(
            if i % 3 == 0 { "raw" } else { "probes_first" },
            (i as f32) / 100.0,
            0.45,
            0.15,
            (i % 5) as u8,
            2.5,
        );

        metrics.record_extraction_result(
            "raw",
            150,
            true,
            85.0,
            5000,
            25,
            10,
            i % 2 == 0,
            i % 3 == 0,
        );
    }

    let duration = start.elapsed();
    assert!(duration.as_millis() < 50, "Metrics should be non-blocking (<50ms for 100 recordings), took: {}ms", duration.as_millis());
}

#[test]
fn test_gate_decision_multiple_types() {
    let metrics = RipTideMetrics::new().unwrap();

    // Record different decision types
    metrics.record_gate_decision_enhanced("raw", 0.85, 0.45, 0.15, 3, 2.5);
    metrics.record_gate_decision_enhanced("probes_first", 0.65, 0.35, 0.25, 5, 3.0);
    metrics.record_gate_decision_enhanced("headless", 0.45, 0.25, 0.45, 8, 4.5);

    let metric_families = metrics.registry.gather();
    let gate_decisions = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_gate_decision_total")
        .expect("Gate decision metric should exist");

    // Verify multiple decision types are tracked
    assert!(
        gate_decisions.get_metric().len() >= 3,
        "Should track multiple decision types"
    );
}

#[test]
fn test_extraction_duration_by_mode() {
    let metrics = RipTideMetrics::new().unwrap();

    // Record extractions with different modes
    metrics.record_extraction_result("raw", 100, true, 80.0, 4000, 20, 8, true, true);
    metrics.record_extraction_result("probes_first", 200, true, 85.0, 5000, 25, 10, true, true);
    metrics.record_extraction_result("headless", 500, true, 90.0, 6000, 30, 15, true, true);

    let metric_families = metrics.registry.gather();
    let duration = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_extraction_duration_by_mode_seconds")
        .expect("Duration by mode metric should exist");

    assert!(
        duration.get_metric().len() >= 3,
        "Should track durations for multiple modes"
    );
}

#[test]
fn test_extraction_success_vs_failure() {
    let metrics = RipTideMetrics::new().unwrap();

    // Record successful extraction
    metrics.record_extraction_result("raw", 150, true, 85.0, 5000, 25, 10, true, true);

    // Record failed extraction (success = false)
    metrics.record_extraction_result("raw", 150, false, 0.0, 0, 0, 0, false, false);

    let metric_families = metrics.registry.gather();

    // Successful extractions should record quality metrics
    let quality_score = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_extraction_quality_score");
    assert!(quality_score.is_some(), "Quality score should be recorded for successes");
}

#[test]
fn test_spa_markers_tracking() {
    let metrics = RipTideMetrics::new().unwrap();

    // Record different SPA marker counts
    for marker_count in 0..10_u8 {
        metrics.record_gate_decision_enhanced(
            "raw",
            0.85,
            0.45,
            0.15,
            marker_count,
            2.5,
        );
    }

    let metric_families = metrics.registry.gather();
    let spa_markers = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_gate_feature_spa_markers_total");
    assert!(spa_markers.is_some(), "SPA markers metric should exist");
}

#[test]
fn test_extraction_metadata_presence() {
    let metrics = RipTideMetrics::new().unwrap();

    // With author and date
    metrics.record_extraction_result("raw", 150, true, 85.0, 5000, 25, 10, true, true);

    // Without author, with date
    metrics.record_extraction_result("raw", 150, true, 80.0, 4800, 20, 8, false, true);

    // With author, without date
    metrics.record_extraction_result("raw", 150, true, 82.0, 4900, 22, 9, true, false);

    // Neither
    metrics.record_extraction_result("raw", 150, true, 75.0, 4500, 18, 7, false, false);

    let metric_families = metrics.registry.gather();

    let has_author = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_extraction_has_author_total");
    assert!(has_author.is_some(), "Author presence metric should exist");

    let has_date = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_extraction_has_date_total");
    assert!(has_date.is_some(), "Date presence metric should exist");
}

#[test]
fn test_gate_score_distribution() {
    let metrics = RipTideMetrics::new().unwrap();

    // Record various scores across the distribution
    let scores = vec![0.0, 0.2, 0.4, 0.6, 0.7, 0.8, 0.9, 1.0];
    for score in scores {
        metrics.record_gate_decision_enhanced(
            "raw",
            score,
            0.45,
            0.15,
            3,
            2.5,
        );
    }

    let metric_families = metrics.registry.gather();
    let gate_score = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_gate_score")
        .expect("Gate score histogram should exist");

    // Verify histogram buckets are populated
    let histogram = gate_score.get_metric()[0].get_histogram();
    assert!(
        histogram.get_sample_count() == 8,
        "Should record all score samples"
    );
}

#[test]
fn test_feature_ratio_histograms() {
    let metrics = RipTideMetrics::new().unwrap();

    // Record various feature ratios
    let ratios = vec![
        (0.1, 0.05),
        (0.3, 0.15),
        (0.5, 0.25),
        (0.7, 0.35),
        (0.9, 0.45),
    ];

    for (text_ratio, script_density) in ratios {
        metrics.record_gate_decision_enhanced(
            "raw",
            0.75,
            text_ratio,
            script_density,
            3,
            2.5,
        );
    }

    let metric_families = metrics.registry.gather();

    let text_ratio = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_gate_feature_text_ratio");
    assert!(text_ratio.is_some(), "Text ratio histogram should exist");

    let script_density = metric_families
        .iter()
        .find(|m| m.get_name() == "riptide_gate_feature_script_density");
    assert!(script_density.is_some(), "Script density histogram should exist");
}

#[test]
fn test_metrics_registry_isolation() {
    // Create two separate metrics instances
    let metrics1 = RipTideMetrics::new().unwrap();
    let metrics2 = RipTideMetrics::new().unwrap();

    // Record to first instance
    metrics1.record_gate_decision_enhanced("raw", 0.85, 0.45, 0.15, 3, 2.5);

    // Record to second instance
    metrics2.record_gate_decision_enhanced("probes_first", 0.65, 0.35, 0.25, 5, 3.0);

    // Each should have independent registries
    let families1 = metrics1.registry.gather();
    let families2 = metrics2.registry.gather();

    assert!(!families1.is_empty(), "First registry should have metrics");
    assert!(!families2.is_empty(), "Second registry should have metrics");
}

#[test]
fn test_all_30_plus_metrics_exist() {
    let metrics = RipTideMetrics::new().unwrap();

    // Record comprehensive metrics
    metrics.record_gate_decision_enhanced("raw", 0.85, 0.45, 0.15, 3, 2.5);
    metrics.record_extraction_result("raw", 150, true, 85.0, 5000, 25, 10, true, true);
    metrics.record_extraction_fallback("raw", "headless", "low_quality");
    metrics.record_pipeline_phase_ms("gate_analysis", 5.5);
    metrics.record_pipeline_phase_ms("extraction", 125.0);

    let metric_families = metrics.registry.gather();

    // Phase 1B metrics that should exist
    let expected_metrics = vec![
        "riptide_gate_decision_total",
        "riptide_gate_score",
        "riptide_gate_feature_text_ratio",
        "riptide_gate_feature_script_density",
        "riptide_gate_feature_spa_markers_total",
        "riptide_gate_decision_duration_milliseconds",
        "riptide_extraction_quality_score",
        "riptide_extraction_quality_success_rate",
        "riptide_extraction_content_length_bytes",
        "riptide_extraction_links_found",
        "riptide_extraction_images_found",
        "riptide_extraction_has_author_total",
        "riptide_extraction_has_date_total",
        "riptide_extraction_duration_by_mode_seconds",
        "riptide_extraction_fallback_triggered_total",
        "riptide_pipeline_phase_gate_analysis_milliseconds",
        "riptide_pipeline_phase_extraction_milliseconds",
    ];

    let metric_names: Vec<String> = metric_families
        .iter()
        .map(|m| m.get_name().to_string())
        .collect();

    for expected in &expected_metrics {
        assert!(
            metric_names.contains(&expected.to_string()),
            "Metric '{}' should exist in registry",
            expected
        );
    }

    // Verify we have at least 30 unique metrics (including existing ones)
    assert!(
        metric_names.len() >= 30,
        "Should have at least 30 metrics, found: {}",
        metric_names.len()
    );
}

#[test]
fn test_metrics_performance_overhead() {
    let metrics = RipTideMetrics::new().unwrap();

    // Measure overhead of metrics recording
    let iterations = 1000_u32;
    let start = Instant::now();

    for i in 0..iterations {
        metrics.record_gate_decision_enhanced(
            "raw",
            (i as f32) / (iterations as f32),
            0.45,
            0.15,
            (i % 10) as u8,
            2.5,
        );
    }

    let duration = start.elapsed();
    let avg_per_call = duration.as_nanos() / iterations as u128;

    // Verify overhead is less than 1% of typical operation time
    // Assuming typical operation takes ~100ms, 1% = 1ms = 1,000,000ns
    assert!(
        avg_per_call < 10_000, // 10 microseconds per call
        "Metrics overhead too high: {}ns per call",
        avg_per_call
    );
}
