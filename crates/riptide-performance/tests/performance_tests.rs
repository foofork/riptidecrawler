//! Performance monitoring and optimization tests

use riptide_performance::*;
use std::time::{Duration, Instant};

#[cfg(test)]
mod metrics_tests {
    use super::*;
    use riptide_performance::metrics::{PerformanceMetrics, MetricsCollector};

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();

        // Record some metrics
        collector.record_request(Duration::from_millis(50)).await;
        collector.record_request(Duration::from_millis(100)).await;
        collector.record_request(Duration::from_millis(75)).await;

        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.request_count, 3);
        assert_eq!(metrics.average_latency_ms, 75);
        assert_eq!(metrics.p50_latency_ms, 75);
        assert_eq!(metrics.p99_latency_ms, 100);
    }

    #[tokio::test]
    async fn test_throughput_calculation() {
        let collector = MetricsCollector::new();

        let start = Instant::now();
        for _ in 0..100 {
            collector.record_request(Duration::from_millis(10)).await;
        }
        let elapsed = start.elapsed();

        let throughput = collector.calculate_throughput().await;
        let expected = 100.0 / elapsed.as_secs_f64();

        // Should be close to expected (within 10%)
        assert!((throughput - expected).abs() / expected < 0.1);
    }

    #[tokio::test]
    async fn test_memory_tracking() {
        let tracker = MemoryTracker::new();

        tracker.record_allocation(1024 * 1024).await; // 1MB
        tracker.record_allocation(512 * 1024).await; // 512KB

        let usage = tracker.get_memory_usage().await;
        assert_eq!(usage.allocated_mb, 1.5);

        tracker.record_deallocation(512 * 1024).await;
        let usage = tracker.get_memory_usage().await;
        assert_eq!(usage.allocated_mb, 1.0);
    }
}

#[cfg(test)]
mod profiling_tests {
    use super::*;
    use riptide_performance::profiler::{Profiler, ProfileScope};

    #[test]
    fn test_profile_scope() {
        let profiler = Profiler::new();

        {
            let _scope = ProfileScope::new(&profiler, "test_operation");
            std::thread::sleep(Duration::from_millis(50));
        }

        let profile = profiler.get_profile("test_operation");
        assert!(profile.is_some());
        assert!(profile.unwrap().duration_ms >= 50);
    }

    #[tokio::test]
    async fn test_async_profiling() {
        let profiler = Profiler::new();

        let _scope = ProfileScope::new(&profiler, "async_operation");
        tokio::time::sleep(Duration::from_millis(100)).await;
        drop(_scope);

        let profile = profiler.get_profile("async_operation");
        assert!(profile.unwrap().duration_ms >= 100);
    }

    #[test]
    fn test_flame_graph_generation() {
        let profiler = Profiler::new();

        // Simulate nested operations
        {
            let _outer = ProfileScope::new(&profiler, "outer");
            {
                let _inner1 = ProfileScope::new(&profiler, "inner1");
                std::thread::sleep(Duration::from_millis(10));
            }
            {
                let _inner2 = ProfileScope::new(&profiler, "inner2");
                std::thread::sleep(Duration::from_millis(20));
            }
        }

        let flame_graph = profiler.generate_flame_graph();
        assert!(flame_graph.contains("outer"));
        assert!(flame_graph.contains("inner1"));
        assert!(flame_graph.contains("inner2"));
    }
}

#[cfg(test)]
mod optimization_tests {
    use super::*;
    use riptide_performance::optimizer::{PerformanceOptimizer, OptimizationConfig};

    #[tokio::test]
    async fn test_auto_tuning() {
        let config = OptimizationConfig {
            target_latency_ms: 100,
            max_concurrent_requests: 50,
            enable_auto_tuning: true,
        };

        let optimizer = PerformanceOptimizer::new(config);

        // Simulate varying load
        for i in 0..100 {
            let latency = if i < 50 { 50 } else { 150 };
            optimizer.record_latency(Duration::from_millis(latency)).await;
        }

        // Should adjust concurrency based on latency
        let settings = optimizer.get_optimized_settings().await;
        assert!(settings.concurrent_limit < 50); // Should reduce due to high latency
    }

    #[tokio::test]
    async fn test_resource_pooling() {
        let pool = ResourcePool::new(PoolConfig {
            min_size: 2,
            max_size: 10,
            idle_timeout: Duration::from_secs(60),
        });

        // Acquire resources
        let r1 = pool.acquire().await.unwrap();
        let r2 = pool.acquire().await.unwrap();

        assert_eq!(pool.active_count().await, 2);

        // Return resources
        pool.release(r1).await;
        pool.release(r2).await;

        assert_eq!(pool.idle_count().await, 2);
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let limiter = AdaptiveRateLimiter::new(10.0); // 10 req/sec

        let start = Instant::now();
        for _ in 0..5 {
            limiter.wait_if_needed().await;
        }
        let elapsed = start.elapsed();

        // Should take at least 400ms (5 requests at 10/sec)
        assert!(elapsed >= Duration::from_millis(400));
    }
}

#[cfg(test)]
mod bottleneck_tests {
    use super::*;
    use riptide_performance::bottleneck::{BottleneckDetector, SystemMetrics};

    #[tokio::test]
    async fn test_bottleneck_detection() {
        let detector = BottleneckDetector::new();

        let metrics = SystemMetrics {
            cpu_usage: 95.0,
            memory_usage: 45.0,
            disk_io: 30.0,
            network_io: 20.0,
        };

        let bottlenecks = detector.analyze(metrics).await;
        assert!(bottlenecks.contains(&Bottleneck::CPU));
        assert!(!bottlenecks.contains(&Bottleneck::Memory));
    }

    #[tokio::test]
    async fn test_performance_recommendations() {
        let detector = BottleneckDetector::new();

        let metrics = SystemMetrics {
            cpu_usage: 50.0,
            memory_usage: 90.0,
            disk_io: 80.0,
            network_io: 30.0,
        };

        let recommendations = detector.get_recommendations(metrics).await;
        assert!(recommendations.iter().any(|r| r.contains("memory")));
        assert!(recommendations.iter().any(|r| r.contains("disk")));
    }
}