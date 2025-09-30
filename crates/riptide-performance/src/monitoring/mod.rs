//! Real-time performance monitoring module
//!
//! This module provides continuous monitoring of system performance metrics,
//! including latency, throughput, memory usage, and resource utilization.

// Module declarations
mod alerts;
mod bottlenecks;
mod config;
mod metrics;
mod monitor;
mod reports;

// Public re-exports
pub use alerts::{AlertSeverity, PerformanceAlert};
pub use bottlenecks::{Bottleneck, BottleneckAnalysis, BottleneckSeverity};
pub use config::{AlertMultipliers, MonitoringConfig};
pub use metrics::{ApplicationMetrics, SystemMetrics};
pub use monitor::PerformanceMonitor;
pub use reports::{ApplicationSummary, MonitoringReport, SystemSummary};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PerformanceTargets;
    use std::collections::VecDeque;
    use std::time::Duration;

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let targets = PerformanceTargets::default();
        let monitor = PerformanceMonitor::new(targets).unwrap();
        assert!(!monitor.session_id.is_nil());
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let targets = PerformanceTargets::default();
        let monitor = PerformanceMonitor::new(targets).unwrap();

        let system_metrics = monitor.collect_system_metrics().await.unwrap();
        assert!(system_metrics.cpu_usage_percent >= 0.0);
        assert!(system_metrics.memory_rss_mb > 0.0);

        let app_metrics = monitor.collect_application_metrics().await.unwrap();
        assert!(app_metrics.total_requests > 0);
    }

    #[tokio::test]
    async fn test_current_metrics() {
        let targets = PerformanceTargets::default();
        let monitor = PerformanceMonitor::new(targets).unwrap();

        let metrics = monitor.get_current_metrics().await.unwrap();
        assert!(metrics.memory_rss_mb > 0.0);
        assert!(metrics.cpu_usage_percent >= 0.0);
    }

    #[tokio::test]
    async fn test_bottleneck_analysis_insufficient_data() {
        let targets = PerformanceTargets::default();
        let monitor = PerformanceMonitor::new(targets).unwrap();

        // Should fail with insufficient data
        let result = monitor.analyze_bottlenecks().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Insufficient metrics"));
    }

    #[tokio::test]
    async fn test_bottleneck_analysis_with_data() {
        let targets = PerformanceTargets::default();
        let mut monitor = PerformanceMonitor::new(targets).unwrap();

        // Start monitoring to collect some data
        monitor.start().await.unwrap();

        // Wait a bit for data collection
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Manually add some test metrics
        {
            let mut system_metrics = monitor.system_metrics.write().await;
            for i in 0..5 {
                system_metrics.push_back(SystemMetrics {
                    timestamp: chrono::Utc::now(),
                    cpu_usage_percent: 85.0 + (i as f64 * 2.0), // High CPU
                    memory_rss_mb: 300.0 + (i as f64 * 5.0),
                    memory_heap_mb: 200.0,
                    memory_virtual_mb: 500.0,
                    disk_read_mbps: 50.0,
                    disk_write_mbps: 30.0,
                    network_in_mbps: 100.0,
                    network_out_mbps: 80.0,
                    open_file_descriptors: 150,
                    thread_count: 50,
                });
            }

            let mut app_metrics = monitor.application_metrics.write().await;
            for i in 0..5 {
                app_metrics.push_back(ApplicationMetrics {
                    timestamp: chrono::Utc::now(),
                    active_requests: 10,
                    total_requests: 1000 + (i * 100),
                    successful_requests: 980,
                    failed_requests: 20,
                    avg_response_time_ms: 1200.0,
                    p95_response_time_ms: 2100.0, // High latency
                    cache_hit_rate: 0.65, // Low cache hit rate
                    cache_size_mb: 50.0,
                    ai_processing_queue_size: 5,
                    ai_avg_processing_time_ms: 400.0,
                });
            }
        }

        // Now analyze bottlenecks
        let analysis = monitor.analyze_bottlenecks().await.unwrap();

        // Should detect high CPU
        assert!(!analysis.bottlenecks.is_empty());
        assert!(analysis.analysis_time > Duration::from_nanos(0));
        assert!(!analysis.recommendations.is_empty());

        // Verify CPU bottleneck was detected
        let has_cpu_bottleneck = analysis.bottlenecks.iter()
            .any(|b| b.location.contains("CPU"));
        assert!(has_cpu_bottleneck, "Should detect CPU bottleneck");

        // Verify high latency was detected
        let has_latency_bottleneck = analysis.bottlenecks.iter()
            .any(|b| b.location.contains("Latency") || b.location.contains("Request Processing"));
        assert!(has_latency_bottleneck, "Should detect latency bottleneck");

        // Verify cache inefficiency was detected
        let has_cache_bottleneck = analysis.bottlenecks.iter()
            .any(|b| b.location.contains("Cache"));
        assert!(has_cache_bottleneck, "Should detect cache inefficiency");
    }

    #[tokio::test]
    async fn test_cpu_bottleneck_detection() {
        let targets = PerformanceTargets::default();
        let monitor = PerformanceMonitor::new(targets).unwrap();

        let mut metrics = VecDeque::new();

        // Add metrics with high CPU usage
        for _ in 0..10 {
            metrics.push_back(SystemMetrics {
                timestamp: chrono::Utc::now(),
                cpu_usage_percent: 85.0,
                memory_rss_mb: 300.0,
                memory_heap_mb: 200.0,
                memory_virtual_mb: 500.0,
                disk_read_mbps: 10.0,
                disk_write_mbps: 10.0,
                network_in_mbps: 50.0,
                network_out_mbps: 50.0,
                open_file_descriptors: 100,
                thread_count: 30,
            });
        }

        let bottlenecks = monitor.analyze_cpu_bottlenecks(&metrics).await.unwrap();
        assert!(!bottlenecks.is_empty());
        assert_eq!(bottlenecks[0].severity, BottleneckSeverity::High);
    }

    #[tokio::test]
    async fn test_memory_bottleneck_detection() {
        let targets = PerformanceTargets::default();
        let monitor = PerformanceMonitor::new(targets).unwrap();

        let mut metrics = VecDeque::new();
        let base_time = chrono::Utc::now();

        // Add metrics with memory growth (potential leak)
        for i in 0..10 {
            metrics.push_back(SystemMetrics {
                timestamp: base_time + chrono::Duration::seconds(i * 10),
                cpu_usage_percent: 50.0,
                memory_rss_mb: 300.0 + (i as f64 * 15.0), // Growing memory
                memory_heap_mb: 200.0,
                memory_virtual_mb: 500.0,
                disk_read_mbps: 10.0,
                disk_write_mbps: 10.0,
                network_in_mbps: 50.0,
                network_out_mbps: 50.0,
                open_file_descriptors: 100,
                thread_count: 30,
            });
        }

        let bottlenecks = monitor.analyze_memory_bottlenecks(&metrics).await.unwrap();
        assert!(!bottlenecks.is_empty());

        // Should detect memory growth
        let has_growth = bottlenecks.iter().any(|b| b.location.contains("Growth"));
        assert!(has_growth, "Should detect memory growth");
    }

    #[tokio::test]
    async fn test_io_bottleneck_detection() {
        let targets = PerformanceTargets::default();
        let monitor = PerformanceMonitor::new(targets).unwrap();

        let mut metrics = VecDeque::new();

        // Add metrics with high I/O
        for _ in 0..10 {
            metrics.push_back(SystemMetrics {
                timestamp: chrono::Utc::now(),
                cpu_usage_percent: 50.0,
                memory_rss_mb: 300.0,
                memory_heap_mb: 200.0,
                memory_virtual_mb: 500.0,
                disk_read_mbps: 120.0, // High disk read
                disk_write_mbps: 80.0,  // High disk write
                network_in_mbps: 200.0, // High network
                network_out_mbps: 150.0,
                open_file_descriptors: 100,
                thread_count: 30,
            });
        }

        let bottlenecks = monitor.analyze_io_bottlenecks(&metrics).await.unwrap();
        assert!(!bottlenecks.is_empty());

        // Should detect both disk and network bottlenecks
        let has_disk = bottlenecks.iter().any(|b| b.location.contains("Disk"));
        let has_network = bottlenecks.iter().any(|b| b.location.contains("Network"));
        assert!(has_disk, "Should detect disk I/O bottleneck");
        assert!(has_network, "Should detect network I/O bottleneck");
    }

    #[tokio::test]
    async fn test_application_bottleneck_detection() {
        let targets = PerformanceTargets::default();
        let monitor = PerformanceMonitor::new(targets).unwrap();

        let mut metrics = VecDeque::new();

        // Add metrics with high latency and low cache hit rate
        for i in 0..10 {
            metrics.push_back(ApplicationMetrics {
                timestamp: chrono::Utc::now(),
                active_requests: 10,
                total_requests: 1000 + (i * 100),
                successful_requests: 980,
                failed_requests: 20,
                avg_response_time_ms: 1500.0,
                p95_response_time_ms: 2500.0, // Very high latency
                cache_hit_rate: 0.60, // Low cache hit rate
                cache_size_mb: 50.0,
                ai_processing_queue_size: 5,
                ai_avg_processing_time_ms: 600.0, // High AI time
            });
        }

        let bottlenecks = monitor.analyze_application_bottlenecks(&metrics).await.unwrap();
        assert!(!bottlenecks.is_empty());

        // Should detect latency, AI, and cache bottlenecks
        let has_latency = bottlenecks.iter().any(|b| b.location.contains("Latency"));
        let has_ai = bottlenecks.iter().any(|b| b.location.contains("AI"));
        let has_cache = bottlenecks.iter().any(|b| b.location.contains("Cache"));

        assert!(has_latency, "Should detect latency bottleneck");
        assert!(has_ai, "Should detect AI processing bottleneck");
        assert!(has_cache, "Should detect cache inefficiency");
    }

    #[tokio::test]
    async fn test_bottleneck_severity_ordering() {
        // Verify severity enum ordering
        assert!(BottleneckSeverity::Critical > BottleneckSeverity::High);
        assert!(BottleneckSeverity::High > BottleneckSeverity::Medium);
        assert!(BottleneckSeverity::Medium > BottleneckSeverity::Low);
    }

    #[tokio::test]
    async fn test_bottleneck_recommendations() {
        let targets = PerformanceTargets::default();
        let monitor = PerformanceMonitor::new(targets).unwrap();

        let bottlenecks = vec![
            Bottleneck {
                location: "System CPU".to_string(),
                severity: BottleneckSeverity::Critical,
                time_spent: Duration::from_secs(100),
                percentage_of_total: 85.0,
                call_count: 100,
            },
            Bottleneck {
                location: "Memory Growth/Potential Leak".to_string(),
                severity: BottleneckSeverity::Critical,
                time_spent: Duration::from_secs(200),
                percentage_of_total: 15.0,
                call_count: 50,
            },
            Bottleneck {
                location: "AI Processing".to_string(),
                severity: BottleneckSeverity::High,
                time_spent: Duration::from_millis(500),
                percentage_of_total: 35.0,
                call_count: 1000,
            },
        ];

        let recommendations = monitor.generate_bottleneck_recommendations(&bottlenecks).await.unwrap();

        assert!(!recommendations.is_empty());

        // Should have CPU recommendation
        let has_cpu_rec = recommendations.iter().any(|r| r.contains("CPU"));
        assert!(has_cpu_rec, "Should have CPU recommendation");

        // Should have memory leak recommendation
        let has_leak_rec = recommendations.iter().any(|r| r.contains("leak"));
        assert!(has_leak_rec, "Should have memory leak recommendation");

        // Should have AI recommendation
        let has_ai_rec = recommendations.iter().any(|r| r.contains("AI"));
        assert!(has_ai_rec, "Should have AI recommendation");
    }

    #[tokio::test]
    async fn test_no_bottlenecks_detected() {
        let targets = PerformanceTargets::default();
        let monitor = PerformanceMonitor::new(targets).unwrap();

        let bottlenecks = Vec::new();
        let recommendations = monitor.generate_bottleneck_recommendations(&bottlenecks).await.unwrap();

        assert_eq!(recommendations.len(), 1);
        assert!(recommendations[0].contains("No significant bottlenecks"));
    }
}