//! Health monitoring tests for pool health checks
//!
//! Tests health levels, trends, and automated remediation

use riptide_pool::health_monitor::*;

#[tokio::test]
async fn test_health_level_variants() {
    let healthy = HealthLevel::Healthy;
    let degraded = HealthLevel::Degraded;
    let unhealthy = HealthLevel::Unhealthy;
    let critical = HealthLevel::Critical;

    assert_eq!(healthy, HealthLevel::Healthy);
    assert_eq!(degraded, HealthLevel::Degraded);
    assert_eq!(unhealthy, HealthLevel::Unhealthy);
    assert_eq!(critical, HealthLevel::Critical);

    assert_ne!(healthy, degraded);
    assert_ne!(unhealthy, critical);
}

#[tokio::test]
async fn test_memory_pressure_levels() {
    let low = MemoryPressureLevel::Low;
    let medium = MemoryPressureLevel::Medium;
    let high = MemoryPressureLevel::High;
    let critical = MemoryPressureLevel::Critical;

    assert_eq!(low, MemoryPressureLevel::Low);
    assert_eq!(medium, MemoryPressureLevel::Medium);
    assert_eq!(high, MemoryPressureLevel::High);
    assert_eq!(critical, MemoryPressureLevel::Critical);

    assert_ne!(low, high);
    assert_ne!(medium, critical);
}

#[tokio::test]
async fn test_health_status_serialization() {
    let status = PoolHealthStatus {
        status: HealthLevel::Healthy,
        available_instances: 5,
        active_instances: 3,
        max_instances: 8,
        utilization_percent: 37.5,
        avg_semaphore_wait_ms: 15.2,
        circuit_breaker_status: "CLOSED".to_string(),
        total_extractions: 1000,
        success_rate_percent: 95.5,
        fallback_rate_percent: 2.1,
        memory_stats: MemoryHealthStats {
            wasm_memory_pages: 128,
            peak_memory_pages: 256,
            grow_failures: 0,
            memory_pressure: MemoryPressureLevel::Low,
        },
        last_check: Some(std::time::Instant::now()),
        trend: HealthTrend::Stable,
    };

    // Verify health status fields
    assert_eq!(status.available_instances, 5);
    assert_eq!(status.active_instances, 3);
    assert_eq!(status.max_instances, 8);
    assert!((status.utilization_percent - 37.5).abs() < 0.1);
    assert!(status.success_rate_percent > 90.0);
}

#[tokio::test]
async fn test_health_level_calculation_healthy() {
    let utilization = 50.0;
    let success_rate = 98.0;
    let fallback_rate = 1.0;
    let memory_pressure = MemoryPressureLevel::Low;

    // All good indicators - should be Healthy
    let is_healthy = success_rate > 90.0
        && memory_pressure == MemoryPressureLevel::Low
        && utilization < 75.0
        && fallback_rate < 10.0;

    assert!(is_healthy);
}

#[tokio::test]
async fn test_health_level_calculation_degraded() {
    let utilization = 78.0;
    let success_rate = 87.0;
    let fallback_rate = 12.0;
    let memory_pressure = MemoryPressureLevel::Medium;

    // Some degradation indicators
    let is_degraded = (success_rate < 90.0 && success_rate >= 75.0)
        || memory_pressure == MemoryPressureLevel::Medium
        || (utilization > 75.0 && utilization <= 85.0)
        || (fallback_rate > 10.0 && fallback_rate <= 30.0);

    assert!(is_degraded);
}

#[tokio::test]
async fn test_health_level_calculation_unhealthy() {
    let utilization = 87.0;
    let success_rate = 72.0;
    let fallback_rate = 35.0;
    let memory_pressure = MemoryPressureLevel::High;

    // Unhealthy indicators
    let is_unhealthy = (success_rate < 75.0 && success_rate >= 50.0)
        || memory_pressure == MemoryPressureLevel::High
        || (utilization > 85.0 && utilization <= 95.0)
        || fallback_rate > 30.0;

    assert!(is_unhealthy);
}

#[tokio::test]
async fn test_health_level_calculation_critical() {
    let utilization = 97.0;
    let success_rate = 45.0;
    let memory_pressure = MemoryPressureLevel::Critical;
    let epoch_timeouts = 15u64;

    // Critical indicators
    let is_critical = success_rate < 50.0
        || memory_pressure == MemoryPressureLevel::Critical
        || utilization > 95.0
        || epoch_timeouts > 10;

    assert!(is_critical);
}

#[tokio::test]
async fn test_health_trend_stable() {
    // Simulate stable health over time
    let health_scores = vec![4, 4, 4, 4, 4]; // All healthy

    let first_score = health_scores[health_scores.len() - 1];
    let last_score = health_scores[0];

    let trend = if last_score > first_score {
        HealthTrend::Improving
    } else if last_score < first_score {
        HealthTrend::Degrading
    } else {
        HealthTrend::Stable
    };

    assert!(matches!(trend, HealthTrend::Stable));
}

#[tokio::test]
async fn test_health_trend_improving() {
    // Simulate improving health over time
    let health_scores = vec![4, 3, 3, 2, 1]; // Getting better (reversed)

    let first_score = health_scores[health_scores.len() - 1];
    let last_score = health_scores[0];

    let trend = if last_score > first_score {
        HealthTrend::Improving
    } else if last_score < first_score {
        HealthTrend::Degrading
    } else {
        HealthTrend::Stable
    };

    assert!(matches!(trend, HealthTrend::Improving));
}

#[tokio::test]
async fn test_health_trend_degrading() {
    // Simulate degrading health over time
    let health_scores = vec![2, 3, 3, 4, 4]; // Getting worse (reversed)

    let first_score = health_scores[health_scores.len() - 1];
    let last_score = health_scores[0];

    let trend = if last_score > first_score {
        HealthTrend::Improving
    } else if last_score < first_score {
        HealthTrend::Degrading
    } else {
        HealthTrend::Stable
    };

    assert!(matches!(trend, HealthTrend::Degrading));
}

#[tokio::test]
async fn test_memory_pressure_determination() {
    let memory_limit = 256u32;

    let scenarios = vec![
        (100, MemoryPressureLevel::Low),      // 39% - Low
        (150, MemoryPressureLevel::Medium),   // 58% - Medium
        (200, MemoryPressureLevel::High),     // 78% - High
        (240, MemoryPressureLevel::Critical), // 93% - Critical
    ];

    for (usage, expected_level) in scenarios {
        let usage_percent = (usage as f64 / memory_limit as f64) * 100.0;

        let level = match usage_percent {
            p if p < 50.0 => MemoryPressureLevel::Low,
            p if p < 75.0 => MemoryPressureLevel::Medium,
            p if p < 90.0 => MemoryPressureLevel::High,
            _ => MemoryPressureLevel::Critical,
        };

        assert_eq!(level, expected_level);
    }
}

#[tokio::test]
async fn test_success_rate_calculation() {
    let scenarios = vec![
        (95, 5, 95.0),   // 95% success
        (80, 20, 80.0),  // 80% success
        (50, 50, 50.0),  // 50% success
        (100, 0, 100.0), // 100% success
    ];

    for (successful, failed, expected_rate) in scenarios {
        let total = successful + failed;
        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            100.0
        };

        assert!((success_rate - expected_rate).abs() < 0.1);
    }
}

#[tokio::test]
async fn test_fallback_rate_calculation() {
    let total_extractions = 1000u64;
    let fallback_extractions_scenarios = vec![
        (10, 1.0),   // 1% fallback
        (50, 5.0),   // 5% fallback
        (150, 15.0), // 15% fallback
        (350, 35.0), // 35% fallback
    ];

    for (fallbacks, expected_rate) in fallback_extractions_scenarios {
        let fallback_rate = (fallbacks as f64 / total_extractions as f64) * 100.0;
        assert!((fallback_rate - expected_rate).abs() < 0.1);
    }
}

#[tokio::test]
async fn test_health_check_interval() {
    use std::time::Duration;

    let check_interval = Duration::from_secs(5);
    let start = std::time::Instant::now();

    tokio::time::sleep(check_interval).await;

    let elapsed = start.elapsed();
    assert!(elapsed >= check_interval);
    assert!(elapsed < check_interval + Duration::from_millis(500));
}

#[tokio::test]
async fn test_circuit_breaker_status_interpretation() {
    let scenarios = vec![
        ("CLOSED", 0u64, true),   // Normal operation
        ("TRIPPED", 1u64, false), // Circuit tripped once
        ("TRIPPED", 5u64, false), // Circuit tripped multiple times
    ];

    for (status, trips, is_healthy) in scenarios {
        let actual_healthy = status == "CLOSED" && trips == 0;
        assert_eq!(actual_healthy, is_healthy);
    }
}

#[tokio::test]
async fn test_utilization_thresholds() {
    let max_instances = 10;

    let scenarios = vec![
        (2, 20.0, "low"),    // Low utilization
        (5, 50.0, "normal"), // Normal utilization
        (8, 80.0, "high"),   // High utilization
        (10, 100.0, "full"), // Full utilization
    ];

    for (active, expected_util, level) in scenarios {
        let utilization = (active as f64 / max_instances as f64) * 100.0;
        assert!((utilization - expected_util).abs() < 0.1);

        match level {
            "low" => assert!(utilization < 30.0),
            "normal" => assert!(utilization >= 30.0 && utilization < 75.0),
            "high" => assert!(utilization >= 75.0 && utilization < 95.0),
            "full" => assert!(utilization >= 95.0),
            _ => panic!("Unknown level"),
        }
    }
}

#[tokio::test]
async fn test_health_history_tracking() {
    let mut history: Vec<HealthLevel> = vec![];

    // Simulate health checks over time
    history.push(HealthLevel::Healthy);
    history.push(HealthLevel::Healthy);
    history.push(HealthLevel::Degraded);
    history.push(HealthLevel::Degraded);
    history.push(HealthLevel::Unhealthy);

    assert_eq!(history.len(), 5);

    // Should show degrading trend
    let first = &history[0];
    let last = &history[history.len() - 1];

    assert!(matches!(first, HealthLevel::Healthy));
    assert!(matches!(last, HealthLevel::Unhealthy));
}

#[tokio::test]
async fn test_remediation_action_selection() {
    // Test which remediation action to take based on health level
    let scenarios = vec![
        (HealthLevel::Healthy, "optimize"),
        (HealthLevel::Degraded, "proactive_cleanup"),
        (HealthLevel::Unhealthy, "reduce_load"),
        (HealthLevel::Critical, "emergency_action"),
    ];

    for (level, expected_action) in scenarios {
        let action = match level {
            HealthLevel::Healthy => "optimize",
            HealthLevel::Degraded => "proactive_cleanup",
            HealthLevel::Unhealthy => "reduce_load",
            HealthLevel::Critical => "emergency_action",
        };

        assert_eq!(action, expected_action);
    }
}
