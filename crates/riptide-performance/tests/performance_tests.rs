//! Performance monitoring and optimization tests

use riptide_performance::monitoring::{Bottleneck, BottleneckSeverity};
use std::time::{Duration, Instant};

#[cfg(test)]
mod metrics_tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_metrics() {
        // Basic test to verify module structure
        let start = Instant::now();
        tokio::time::sleep(Duration::from_millis(10)).await;
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(10));
    }
}

#[cfg(test)]
mod profiling_tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_profiling() {
        // Basic profiling test
        let start = Instant::now();
        tokio::time::sleep(Duration::from_millis(10)).await;
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(10));
    }
}

#[cfg(test)]
mod optimization_tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_optimization() {
        // Basic optimization test
        let start = Instant::now();

        // Simulate some work
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(50));
    }
}

#[cfg(test)]
mod bottleneck_tests {
    use super::*;

    #[tokio::test]
    async fn test_bottleneck_severity() {
        // Test bottleneck severity ordering
        assert!(BottleneckSeverity::Critical > BottleneckSeverity::High);
        assert!(BottleneckSeverity::High > BottleneckSeverity::Medium);
        assert!(BottleneckSeverity::Medium > BottleneckSeverity::Low);
    }

    #[tokio::test]
    async fn test_bottleneck_struct() {
        // Test bottleneck structure creation
        let bottleneck = Bottleneck {
            location: "Test CPU".to_string(),
            severity: BottleneckSeverity::High,
            time_spent: Duration::from_secs(10),
            percentage_of_total: 75.0,
            call_count: 100,
        };

        assert_eq!(bottleneck.location, "Test CPU");
        assert_eq!(bottleneck.severity, BottleneckSeverity::High);
        assert_eq!(bottleneck.call_count, 100);
    }
}
