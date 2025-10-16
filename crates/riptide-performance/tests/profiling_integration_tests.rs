//! Integration tests for Phase 4 profiling infrastructure
//!
//! Tests the complete profiling workflow including:
//! - Memory tracking and snapshots
//! - Leak detection and analysis
//! - Allocation pattern analysis
//! - Telemetry export (mocked)
//! - HTTP endpoints (mocked)
//! - Alert triggering

use anyhow::Result;
use riptide_performance::profiling::{
    allocation_analyzer::AllocationAnalyzer, leak_detector::LeakDetector,
    memory_tracker::MemoryTracker, AllocationInfo, LeakAnalysis,
};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

// ============================================================================
// Test Suite 1: End-to-End Profiling Workflow
// ============================================================================

#[tokio::test]
async fn test_complete_profiling_workflow() {
    // Simulate a complete profiling session:
    // Start -> Collect samples -> Analyze -> Stop -> Verify report

    // Phase 1: Initialize all components
    let memory_tracker = MemoryTracker::new().expect("Failed to create memory tracker");
    let mut leak_detector = LeakDetector::new().expect("Failed to create leak detector");
    let mut allocation_analyzer =
        AllocationAnalyzer::new().expect("Failed to create allocation analyzer");

    // Phase 2: Collect baseline snapshot
    let baseline_snapshot = memory_tracker
        .get_current_snapshot()
        .await
        .expect("Failed to get baseline snapshot");

    assert!(baseline_snapshot.rss_bytes > 0, "RSS should be non-zero");
    assert!(
        baseline_snapshot.virtual_bytes >= baseline_snapshot.rss_bytes,
        "Virtual memory should be >= RSS"
    );

    // Phase 3: Simulate workload with allocations
    let mut test_allocations = Vec::new();

    for i in 0..100 {
        let allocation = create_test_allocation(
            format!("component_{}", i % 5),
            format!("operation_{}", i % 3),
            1024 * (i % 10 + 1),
        );

        test_allocations.push(allocation.clone());

        // Record in all analyzers
        leak_detector
            .record_allocation(allocation.clone())
            .await
            .expect("Failed to record allocation in leak detector");

        allocation_analyzer
            .record_allocation(allocation)
            .await
            .expect("Failed to record allocation in analyzer");
    }

    // Phase 4: Collect post-workload snapshot
    let workload_snapshot = memory_tracker
        .get_current_snapshot()
        .await
        .expect("Failed to get workload snapshot");

    // Verify memory increased (or stayed same if allocations were optimized)
    assert!(
        workload_snapshot.rss_bytes >= baseline_snapshot.rss_bytes,
        "RSS should not decrease during allocation"
    );

    // Phase 5: Analyze leaks
    let leak_analysis = leak_detector
        .analyze_leaks()
        .await
        .expect("Failed to analyze leaks");

    assert!(
        !leak_analysis.potential_leaks.is_empty(),
        "Should detect potential leaks from test allocations"
    );

    // Phase 6: Analyze allocation patterns
    let top_allocators = allocation_analyzer
        .get_top_allocators()
        .await
        .expect("Failed to get top allocators");

    assert!(
        !top_allocators.is_empty(),
        "Should have top allocators data"
    );
    assert!(
        top_allocators.len() <= 5,
        "Should have at most 5 components"
    );

    let recommendations = allocation_analyzer
        .analyze_patterns()
        .await
        .expect("Failed to analyze patterns");

    assert!(
        !recommendations.is_empty(),
        "Should generate optimization recommendations"
    );

    // Phase 7: Get memory breakdown
    let breakdown = memory_tracker
        .get_memory_breakdown()
        .await
        .expect("Failed to get memory breakdown");

    assert!(
        breakdown.contains_key("rss_total"),
        "Breakdown should include RSS"
    );
    assert!(
        breakdown.contains_key("virtual_total"),
        "Breakdown should include virtual memory"
    );

    // Phase 8: Verify metrics
    let memory_pressure = leak_detector
        .get_memory_pressure()
        .await
        .expect("Failed to get memory pressure");

    assert!(
        memory_pressure >= 0.0 && memory_pressure <= 1.0,
        "Memory pressure should be between 0 and 1"
    );

    let efficiency_score = allocation_analyzer
        .calculate_efficiency_score()
        .await
        .expect("Failed to calculate efficiency");

    assert!(
        efficiency_score >= 0.0 && efficiency_score <= 1.0,
        "Efficiency score should be between 0 and 1"
    );
}

// ============================================================================
// Test Suite 2: Telemetry Export (Mocked)
// ============================================================================

#[tokio::test]
async fn test_telemetry_export() {
    // Test that metrics can be exported in OTLP-compatible format

    let memory_tracker = MemoryTracker::new().expect("Failed to create memory tracker");
    let snapshot = memory_tracker
        .get_current_snapshot()
        .await
        .expect("Failed to get snapshot");

    // Verify snapshot structure matches telemetry requirements
    let metrics = convert_snapshot_to_metrics(&snapshot);

    assert!(
        metrics.contains_key("memory.rss"),
        "Should export RSS metric"
    );
    assert!(
        metrics.contains_key("memory.virtual"),
        "Should export virtual memory metric"
    );
    assert!(
        metrics.contains_key("memory.heap"),
        "Should export heap metric"
    );
    assert!(
        metrics.get("memory.rss").unwrap() > &0,
        "RSS metric should be positive"
    );

    // Verify timestamp is recent
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let snapshot_timestamp = snapshot.timestamp.timestamp() as u64;
    assert!(
        now - snapshot_timestamp < 5,
        "Snapshot timestamp should be recent"
    );
}

#[tokio::test]
async fn test_telemetry_batch_export() {
    // Test exporting multiple snapshots in batch

    let memory_tracker = MemoryTracker::new().expect("Failed to create memory tracker");
    let mut batch_metrics = Vec::new();

    // Collect multiple snapshots
    for _ in 0..5 {
        let snapshot = memory_tracker
            .get_current_snapshot()
            .await
            .expect("Failed to get snapshot");

        batch_metrics.push(convert_snapshot_to_metrics(&snapshot));
        sleep(Duration::from_millis(10)).await;
    }

    assert_eq!(batch_metrics.len(), 5, "Should collect 5 snapshots");

    // Verify all batches have required metrics
    for metrics in batch_metrics {
        assert!(metrics.contains_key("memory.rss"));
        assert!(metrics.contains_key("memory.virtual"));
    }
}

// ============================================================================
// Test Suite 3: HTTP Endpoints (Mocked)
// ============================================================================

#[tokio::test]
async fn test_http_endpoints() {
    // Test that profiling data can be exposed via HTTP-compatible format

    let memory_tracker = MemoryTracker::new().expect("Failed to create memory tracker");
    let mut leak_detector = LeakDetector::new().expect("Failed to create leak detector");

    // Simulate some allocations
    for i in 0..10 {
        let allocation = create_test_allocation(
            "test_component".to_string(),
            "test_operation".to_string(),
            1024 * (i + 1),
        );

        leak_detector
            .record_allocation(allocation)
            .await
            .expect("Failed to record allocation");
    }

    // GET /metrics endpoint (Prometheus-style)
    let metrics_response = get_metrics_endpoint(&memory_tracker, &leak_detector)
        .await
        .expect("Failed to get metrics");

    assert!(
        metrics_response.contains("memory_rss_bytes"),
        "Should include RSS metric"
    );
    assert!(
        metrics_response.contains("memory_pressure"),
        "Should include pressure metric"
    );

    // GET /health endpoint
    let health_response = get_health_endpoint(&memory_tracker)
        .await
        .expect("Failed to get health");

    assert_eq!(health_response.status, "healthy");
    assert!(health_response.uptime_seconds > 0);

    // GET /snapshot endpoint (JSON)
    let snapshot_response = get_snapshot_endpoint(&memory_tracker)
        .await
        .expect("Failed to get snapshot");

    assert!(snapshot_response.rss_bytes > 0);
    assert!(snapshot_response.timestamp_unix > 0);

    // GET /leaks endpoint
    let leaks_response = get_leaks_endpoint(&leak_detector)
        .await
        .expect("Failed to get leaks");

    // Since potential_leak_count is usize, it's always >= 0, just verify it exists
    let _ = leaks_response.potential_leak_count; // Use the field to verify it exists
}

// ============================================================================
// Test Suite 4: Alert Triggering
// ============================================================================

#[tokio::test]
async fn test_memory_leak_alert() {
    // Test that alerts are triggered when leak thresholds are exceeded

    let mut leak_detector = LeakDetector::new().expect("Failed to create leak detector");

    // Simulate a memory leak pattern: many allocations without deallocations
    for _i in 0..200 {
        let allocation = create_test_allocation(
            "leaking_component".to_string(),
            "leak_operation".to_string(),
            1024 * 1024, // 1MB each
        );

        leak_detector
            .record_allocation(allocation)
            .await
            .expect("Failed to record allocation");
    }

    // Analyze leaks
    let leak_analysis = leak_detector
        .analyze_leaks()
        .await
        .expect("Failed to analyze leaks");

    // Verify leak detection
    assert!(
        !leak_analysis.potential_leaks.is_empty(),
        "Should detect leaking component"
    );

    let leaking_component = leak_analysis
        .potential_leaks
        .iter()
        .find(|leak| leak.component == "leaking_component");

    assert!(
        leaking_component.is_some(),
        "Should identify leaking component"
    );

    let leak_info = leaking_component.unwrap();

    // Verify leak severity
    assert!(
        leak_info.total_size_bytes > 50 * 1024 * 1024,
        "Should have significant memory usage"
    );
    assert_eq!(
        leak_info.allocation_count, 200,
        "Should track all allocations"
    );

    // Test alert triggering logic
    let alerts = generate_leak_alerts(&leak_analysis);

    assert!(!alerts.is_empty(), "Should generate leak alerts");

    let critical_alerts: Vec<_> = alerts
        .iter()
        .filter(|alert| alert.severity == AlertSeverity::Critical)
        .collect();

    assert!(
        !critical_alerts.is_empty(),
        "Should generate critical alert for large leak"
    );
}

#[tokio::test]
async fn test_memory_pressure_alert() {
    // Test that alerts are triggered when memory pressure is high

    let mut leak_detector = LeakDetector::new().expect("Failed to create leak detector");

    // Simulate high memory pressure with many allocations
    for i in 0..1000 {
        let allocation = create_test_allocation(
            format!("component_{}", i % 10),
            "pressure_operation".to_string(),
            1024 * 100, // 100KB each
        );

        leak_detector
            .record_allocation(allocation)
            .await
            .expect("Failed to record allocation");
    }

    let pressure = leak_detector
        .get_memory_pressure()
        .await
        .expect("Failed to get pressure");

    assert!(
        pressure > 0.0,
        "Memory pressure should be elevated with many allocations"
    );

    // Test pressure-based alerts
    let alert = if pressure > 0.8 {
        Some(Alert {
            severity: AlertSeverity::Critical,
            message: format!("Critical memory pressure: {:.2}%", pressure * 100.0),
            component: "system".to_string(),
            timestamp: chrono::Utc::now(),
        })
    } else if pressure > 0.5 {
        Some(Alert {
            severity: AlertSeverity::Warning,
            message: format!("High memory pressure: {:.2}%", pressure * 100.0),
            component: "system".to_string(),
            timestamp: chrono::Utc::now(),
        })
    } else {
        None
    };

    assert!(alert.is_some(), "Should generate pressure alert");
}

// ============================================================================
// Test Suite 5: Memory Tracker Accuracy
// ============================================================================

#[tokio::test]
async fn test_memory_tracker_accuracy() {
    // Test that memory tracker accurately reports allocations

    let memory_tracker = MemoryTracker::new().expect("Failed to create memory tracker");

    // Get baseline
    let baseline = memory_tracker
        .get_current_snapshot()
        .await
        .expect("Failed to get baseline");

    // Allocate known amount of memory
    let test_data: Vec<Vec<u8>> = (0..100)
        .map(|_| vec![0u8; 1024 * 1024]) // 1MB each = 100MB total
        .collect();

    // Get snapshot after allocation
    let after_allocation = memory_tracker
        .get_current_snapshot()
        .await
        .expect("Failed to get snapshot");

    // Calculate difference
    let rss_increase = after_allocation
        .rss_bytes
        .saturating_sub(baseline.rss_bytes);

    // Verify increase is reasonable (should be close to 100MB, but allow for overhead)
    assert!(
        rss_increase >= 50 * 1024 * 1024,
        "Should track at least 50MB increase (actual: {}MB)",
        rss_increase / 1024 / 1024
    );

    // Keep test_data alive to prevent premature deallocation
    assert_eq!(test_data.len(), 100);

    // Test memory breakdown accuracy
    let breakdown = memory_tracker
        .get_memory_breakdown()
        .await
        .expect("Failed to get breakdown");

    let rss_total = breakdown.get("rss_total").unwrap_or(&0);
    assert!(*rss_total > 0, "RSS total should be positive in breakdown");
}

#[tokio::test]
async fn test_memory_stats_over_time() {
    // Test that memory statistics are collected over time

    let memory_tracker = MemoryTracker::new().expect("Failed to create memory tracker");

    // Collect stats over a short duration
    let stats = memory_tracker
        .get_memory_stats(Duration::from_millis(100))
        .await
        .expect("Failed to get memory stats");

    assert!(stats.peak_rss > 0, "Peak RSS should be positive");
    assert!(stats.average_rss > 0, "Average RSS should be positive");
    assert!(stats.min_rss > 0, "Min RSS should be positive");
    assert!(stats.samples > 0, "Should have samples");
    assert!(
        stats.peak_rss >= stats.average_rss,
        "Peak should be >= average"
    );
    assert!(
        stats.average_rss >= stats.min_rss,
        "Average should be >= min"
    );
}

// ============================================================================
// Test Suite 6: Leak Detection Patterns
// ============================================================================

#[tokio::test]
async fn test_leak_detection_patterns() {
    // Test various leak patterns are correctly detected

    let mut leak_detector = LeakDetector::new().expect("Failed to create leak detector");

    // Pattern 1: Exponential growth
    for i in 0..10 {
        let size = 1024 * (2_usize.pow(i as u32)); // Exponential growth
        let allocation = create_test_allocation(
            "exponential_component".to_string(),
            "grow_operation".to_string(),
            size,
        );

        leak_detector
            .record_allocation(allocation)
            .await
            .expect("Failed to record allocation");
    }

    // Pattern 2: Many small allocations
    for _i in 0..2000 {
        let allocation = create_test_allocation(
            "small_alloc_component".to_string(),
            "small_operation".to_string(),
            128, // Small allocations
        );

        leak_detector
            .record_allocation(allocation)
            .await
            .expect("Failed to record allocation");
    }

    // Pattern 3: Regular large allocations
    for _i in 0..10 {
        let allocation = create_test_allocation(
            "large_alloc_component".to_string(),
            "large_operation".to_string(),
            2 * 1024 * 1024, // 2MB each
        );

        leak_detector
            .record_allocation(allocation)
            .await
            .expect("Failed to record allocation");
    }

    // Analyze all patterns
    let analysis = leak_detector
        .analyze_leaks()
        .await
        .expect("Failed to analyze leaks");

    // Verify pattern detection
    assert!(
        analysis.potential_leaks.len() >= 2,
        "Should detect multiple leak patterns"
    );
    assert!(
        !analysis.suspicious_patterns.is_empty(),
        "Should detect suspicious patterns"
    );

    // Verify specific patterns
    let has_exponential = analysis
        .suspicious_patterns
        .iter()
        .any(|p| p.contains("Exponential"));

    let has_frequent_large = analysis
        .suspicious_patterns
        .iter()
        .any(|p| p.contains("Frequent large"));

    assert!(
        has_exponential || has_frequent_large,
        "Should detect known patterns"
    );
}

#[tokio::test]
async fn test_deallocation_tracking() {
    // Test that deallocations are correctly tracked

    let mut leak_detector = LeakDetector::new().expect("Failed to create leak detector");

    // Record allocation
    let allocation = create_test_allocation(
        "test_component".to_string(),
        "test_operation".to_string(),
        1024 * 1024,
    );

    leak_detector
        .record_allocation(allocation)
        .await
        .expect("Failed to record allocation");

    // Record deallocation
    leak_detector
        .record_deallocation("test_component", 1024 * 1024)
        .await
        .expect("Failed to record deallocation");

    // Analyze - should not detect leak since memory was freed
    let analysis = leak_detector
        .analyze_leaks()
        .await
        .expect("Failed to analyze leaks");

    // Component should have low or zero total size
    if let Some(leak_info) = analysis
        .potential_leaks
        .iter()
        .find(|l| l.component == "test_component")
    {
        assert!(
            leak_info.total_size_bytes == 0,
            "Component should have zero size after deallocation"
        );
    }
}

// ============================================================================
// Test Suite 7: Allocation Analysis
// ============================================================================

#[tokio::test]
async fn test_allocation_analysis() {
    // Test allocation pattern analysis and recommendations

    let mut analyzer = AllocationAnalyzer::new().expect("Failed to create analyzer");

    // Create diverse allocation patterns
    let patterns = vec![
        ("pool_candidate", "allocate", 64, 1000),      // Many small
        ("large_component", "process", 10_000_000, 5), // Few large
        ("medium_component", "compute", 100_000, 50),  // Medium
    ];

    for (component, operation, size, count) in patterns {
        for _ in 0..count {
            let allocation =
                create_test_allocation(component.to_string(), operation.to_string(), size);

            analyzer
                .record_allocation(allocation)
                .await
                .expect("Failed to record allocation");
        }
    }

    // Test top allocators
    let top_allocators = analyzer
        .get_top_allocators()
        .await
        .expect("Failed to get top allocators");

    assert_eq!(
        top_allocators.len(),
        3,
        "Should have 3 components as top allocators"
    );

    // Verify sorting (largest first)
    assert!(
        top_allocators[0].1 >= top_allocators[1].1,
        "Top allocators should be sorted by size"
    );

    // Test top operations
    let top_operations = analyzer
        .get_top_operations()
        .await
        .expect("Failed to get top operations");

    assert!(!top_operations.is_empty(), "Should have top operations");

    // Test size distribution
    let distribution = analyzer
        .get_size_distribution()
        .await
        .expect("Failed to get distribution");

    assert!(
        distribution.get("tiny (<1KB)").unwrap_or(&0) > &0
            || distribution.get("small (1KB-64KB)").unwrap_or(&0) > &0,
        "Should have small allocations"
    );

    // Test pattern analysis and recommendations
    let recommendations = analyzer
        .analyze_patterns()
        .await
        .expect("Failed to analyze patterns");

    assert!(
        !recommendations.is_empty(),
        "Should generate recommendations"
    );

    // Verify specific recommendations for our test patterns
    let has_pool_recommendation = recommendations
        .iter()
        .any(|r| r.contains("pool") || r.contains("small allocations"));

    assert!(
        has_pool_recommendation,
        "Should recommend pooling for small allocations"
    );

    // Test fragmentation analysis
    let fragmentation = analyzer
        .analyze_fragmentation()
        .await
        .expect("Failed to analyze fragmentation");

    assert!(
        !fragmentation.is_empty(),
        "Should have fragmentation metrics"
    );

    // Test efficiency score
    let efficiency = analyzer
        .calculate_efficiency_score()
        .await
        .expect("Failed to calculate efficiency");

    assert!(
        efficiency >= 0.0 && efficiency <= 1.0,
        "Efficiency should be between 0 and 1"
    );
}

#[tokio::test]
async fn test_allocation_timeline() {
    // Test that allocation timeline is tracked correctly

    let mut analyzer = AllocationAnalyzer::new().expect("Failed to create analyzer");

    // Record allocations with delays
    for i in 0..5 {
        let allocation = create_test_allocation(
            "timeline_component".to_string(),
            "timeline_operation".to_string(),
            1024 * (i + 1),
        );

        analyzer
            .record_allocation(allocation)
            .await
            .expect("Failed to record allocation");

        sleep(Duration::from_millis(10)).await;
    }

    // Get timeline
    let timeline = analyzer
        .get_allocation_timeline("timeline_component")
        .await
        .expect("Failed to get timeline");

    assert_eq!(timeline.len(), 5, "Should have 5 timeline entries");

    // Verify timeline is ordered
    for i in 1..timeline.len() {
        assert!(
            timeline[i].0 >= timeline[i - 1].0,
            "Timeline should be chronologically ordered"
        );
    }
}

// ============================================================================
// Test Suite 8: Concurrent Profiling
// ============================================================================

#[tokio::test]
async fn test_concurrent_profiling() {
    // Test that profiling works correctly with concurrent operations

    let memory_tracker = MemoryTracker::new().expect("Failed to create memory tracker");
    let leak_detector_handle = std::sync::Arc::new(tokio::sync::Mutex::new(
        LeakDetector::new().expect("Failed to create leak detector"),
    ));

    // Spawn multiple concurrent tasks that allocate memory
    let mut handles = Vec::new();

    for task_id in 0..10 {
        let detector = leak_detector_handle.clone();

        let handle = tokio::spawn(async move {
            for i in 0..50 {
                let allocation = create_test_allocation(
                    format!("task_{}", task_id),
                    format!("operation_{}", i % 3),
                    1024 * (i + 1),
                );

                let mut detector_lock = detector.lock().await;
                detector_lock
                    .record_allocation(allocation)
                    .await
                    .expect("Failed to record allocation");
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task panicked");
    }

    // Verify all allocations were tracked
    let detector = leak_detector_handle.lock().await;
    let analysis = detector.analyze_leaks().await.expect("Failed to analyze");

    assert!(
        !analysis.potential_leaks.is_empty(),
        "Should detect leaks from concurrent tasks"
    );

    // Verify memory tracking still works
    let snapshot = memory_tracker
        .get_current_snapshot()
        .await
        .expect("Failed to get snapshot");

    assert!(snapshot.rss_bytes > 0, "Should have valid memory snapshot");
}

// ============================================================================
// Test Suite 9: Performance Benchmarks
// ============================================================================

#[tokio::test]
async fn test_profiling_performance() {
    // Test that profiling operations are performant

    let mut analyzer = AllocationAnalyzer::new().expect("Failed to create analyzer");

    let start = std::time::Instant::now();

    // Record 10,000 allocations
    for i in 0..10_000 {
        let allocation = create_test_allocation(
            format!("component_{}", i % 100),
            format!("operation_{}", i % 10),
            1024 * (i % 100 + 1),
        );

        analyzer
            .record_allocation(allocation)
            .await
            .expect("Failed to record allocation");
    }

    let record_duration = start.elapsed();

    assert!(
        record_duration.as_millis() < 1000,
        "Recording 10k allocations should take <1s (took {}ms)",
        record_duration.as_millis()
    );

    // Test analysis performance
    let start = std::time::Instant::now();

    let _top_allocators = analyzer
        .get_top_allocators()
        .await
        .expect("Failed to get top allocators");

    let analysis_duration = start.elapsed();

    assert!(
        analysis_duration.as_millis() < 100,
        "Analysis should take <100ms (took {}ms)",
        analysis_duration.as_millis()
    );
}

#[tokio::test]
async fn test_memory_snapshot_performance() {
    // Test that memory snapshots are fast

    let memory_tracker = MemoryTracker::new().expect("Failed to create memory tracker");

    let start = std::time::Instant::now();

    // Collect 100 snapshots
    for _ in 0..100 {
        let _snapshot = memory_tracker
            .get_current_snapshot()
            .await
            .expect("Failed to get snapshot");
    }

    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 500,
        "100 snapshots should take <500ms (took {}ms)",
        duration.as_millis()
    );
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_allocation(component: String, operation: String, size: usize) -> AllocationInfo {
    AllocationInfo {
        timestamp: chrono::Utc::now(),
        size,
        alignment: 8,
        stack_trace: vec![
            format!("{}::main", component),
            format!("{}::{}", component, operation),
        ],
        component,
        operation,
    }
}

fn convert_snapshot_to_metrics(
    snapshot: &riptide_performance::profiling::MemorySnapshot,
) -> HashMap<String, u64> {
    let mut metrics = HashMap::new();

    metrics.insert("memory.rss".to_string(), snapshot.rss_bytes);
    metrics.insert("memory.virtual".to_string(), snapshot.virtual_bytes);
    metrics.insert("memory.heap".to_string(), snapshot.heap_bytes);
    metrics.insert("memory.resident".to_string(), snapshot.resident_bytes);

    metrics
}

async fn get_metrics_endpoint(tracker: &MemoryTracker, detector: &LeakDetector) -> Result<String> {
    let snapshot = tracker.get_current_snapshot().await?;
    let pressure = detector.get_memory_pressure().await?;

    // Prometheus-style format
    let mut response = String::new();
    response.push_str(&format!(
        "# HELP memory_rss_bytes Resident set size in bytes\n"
    ));
    response.push_str(&format!("# TYPE memory_rss_bytes gauge\n"));
    response.push_str(&format!("memory_rss_bytes {}\n", snapshot.rss_bytes));

    response.push_str(&format!("# HELP memory_pressure Memory pressure (0-1)\n"));
    response.push_str(&format!("# TYPE memory_pressure gauge\n"));
    response.push_str(&format!("memory_pressure {:.2}\n", pressure));

    Ok(response)
}

#[derive(Debug)]
struct HealthResponse {
    status: String,
    uptime_seconds: u64,
}

async fn get_health_endpoint(_tracker: &MemoryTracker) -> Result<HealthResponse> {
    Ok(HealthResponse {
        status: "healthy".to_string(),
        uptime_seconds: 1, // Simplified for test
    })
}

#[derive(Debug)]
struct SnapshotResponse {
    rss_bytes: u64,
    virtual_bytes: u64,
    timestamp_unix: i64,
}

async fn get_snapshot_endpoint(tracker: &MemoryTracker) -> Result<SnapshotResponse> {
    let snapshot = tracker.get_current_snapshot().await?;

    Ok(SnapshotResponse {
        rss_bytes: snapshot.rss_bytes,
        virtual_bytes: snapshot.virtual_bytes,
        timestamp_unix: snapshot.timestamp.timestamp(),
    })
}

#[derive(Debug)]
struct LeaksResponse {
    potential_leak_count: usize,
    growth_rate_mb_per_hour: f64,
}

async fn get_leaks_endpoint(detector: &LeakDetector) -> Result<LeaksResponse> {
    let analysis = detector.analyze_leaks().await?;

    Ok(LeaksResponse {
        potential_leak_count: analysis.potential_leaks.len(),
        growth_rate_mb_per_hour: analysis.growth_rate_mb_per_hour,
    })
}

#[derive(Debug, Clone, PartialEq)]
enum AlertSeverity {
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
struct Alert {
    severity: AlertSeverity,
    message: String,
    component: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

fn generate_leak_alerts(analysis: &LeakAnalysis) -> Vec<Alert> {
    let mut alerts = Vec::new();

    for leak in &analysis.potential_leaks {
        let severity = if leak.total_size_bytes > 100 * 1024 * 1024 {
            AlertSeverity::Critical
        } else if leak.total_size_bytes > 50 * 1024 * 1024 {
            AlertSeverity::Warning
        } else {
            continue;
        };

        alerts.push(Alert {
            severity,
            message: format!(
                "Memory leak detected: {} ({} MB, {} allocations)",
                leak.component,
                leak.total_size_bytes / 1024 / 1024,
                leak.allocation_count
            ),
            component: leak.component.clone(),
            timestamp: chrono::Utc::now(),
        });
    }

    alerts
}
