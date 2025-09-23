//! Health scoring and recommendation system

use crate::monitoring::metrics::{HealthThresholds, PerformanceMetrics};

/// Health calculator for system health scoring
pub struct HealthCalculator {
    thresholds: HealthThresholds,
}

impl HealthCalculator {
    /// Create a new health calculator with given thresholds
    pub fn new(thresholds: HealthThresholds) -> Self {
        Self { thresholds }
    }

    /// Calculate overall health score (0-100)
    pub fn calculate_health(&self, metrics: &PerformanceMetrics) -> f32 {
        let mut score = 100.0;

        // Deduct points for high error rates
        if metrics.error_rate > self.thresholds.error_rate_warning {
            let excess = metrics.error_rate - self.thresholds.error_rate_warning;
            score -= excess * 2.0;

            if metrics.error_rate > self.thresholds.error_rate_critical {
                score -= 10.0; // Additional penalty for critical level
            }
        }

        // Deduct points for high CPU usage
        if metrics.cpu_usage_percent > self.thresholds.cpu_usage_warning {
            let excess = metrics.cpu_usage_percent - self.thresholds.cpu_usage_warning;
            score -= excess as f64 * 0.5;

            if metrics.cpu_usage_percent > self.thresholds.cpu_usage_critical {
                score -= 10.0;
            }
        }

        // Deduct points for high memory usage
        if metrics.memory_usage_bytes > self.thresholds.memory_usage_warning {
            score -= 5.0;

            if metrics.memory_usage_bytes > self.thresholds.memory_usage_critical {
                score -= 15.0;
            }
        }

        // Deduct points for slow extraction times
        if metrics.p99_extraction_time_ms > self.thresholds.extraction_time_warning_ms {
            score -= 5.0;

            if metrics.p99_extraction_time_ms > self.thresholds.extraction_time_critical_ms {
                score -= 10.0;
            }
        }

        // Deduct points for circuit breaker trips
        if metrics.circuit_breaker_trips > 0 {
            score -= (metrics.circuit_breaker_trips as f64).min(20.0);
        }

        // Deduct points for high timeout rate
        if metrics.timeout_rate > 1.0 {
            score -= (metrics.timeout_rate - 1.0) * 10.0;
        }

        // Bonus points for good cache hit ratio
        if metrics.cache_hit_ratio > 0.7 {
            score += 5.0;
        }

        score.max(0.0).min(100.0) as f32
    }

    /// Generate health summary text
    pub fn generate_health_summary(&self, metrics: &PerformanceMetrics) -> String {
        let score = metrics.health_score;

        if score >= 95.0 {
            "Excellent - System performing optimally".to_string()
        } else if score >= 85.0 {
            "Good - Minor performance issues detected".to_string()
        } else if score >= 70.0 {
            "Fair - Performance degradation observed".to_string()
        } else if score >= 50.0 {
            "Poor - Significant performance issues".to_string()
        } else {
            "Critical - System requires immediate attention".to_string()
        }
    }

    /// Generate recommendations based on current metrics
    pub fn generate_recommendations(&self, metrics: &PerformanceMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Error rate recommendations
        if metrics.error_rate > self.thresholds.error_rate_critical {
            recommendations.push(
                "CRITICAL: Very high error rate detected - investigate error patterns immediately and consider enabling circuit breakers".to_string()
            );
        } else if metrics.error_rate > self.thresholds.error_rate_warning {
            recommendations.push(
                "High error rate detected - review error logs and implement retry logic"
                    .to_string(),
            );
        }

        // CPU usage recommendations
        if metrics.cpu_usage_percent > self.thresholds.cpu_usage_critical {
            recommendations.push(
                "CRITICAL: CPU usage at critical levels - scale out immediately or optimize hot code paths".to_string()
            );
        } else if metrics.cpu_usage_percent > self.thresholds.cpu_usage_warning {
            recommendations
                .push("High CPU usage - consider scaling out or optimizing algorithms".to_string());
        }

        // Memory usage recommendations
        if metrics.memory_usage_bytes > self.thresholds.memory_usage_critical {
            recommendations.push(
                "CRITICAL: Memory usage at critical levels - investigate memory leaks and increase heap size".to_string()
            );
        } else if metrics.memory_usage_bytes > self.thresholds.memory_usage_warning {
            recommendations.push(
                "High memory usage - review memory allocations and consider garbage collection tuning".to_string()
            );
        }

        // Extraction time recommendations
        if metrics.avg_extraction_time_ms > self.thresholds.extraction_time_critical_ms {
            recommendations.push(
                "CRITICAL: Very high extraction latency - optimize parsing algorithms or increase timeout thresholds".to_string()
            );
        } else if metrics.avg_extraction_time_ms > self.thresholds.extraction_time_warning_ms {
            recommendations.push(
                "High extraction latency - optimize parsing algorithms or increase instance pool"
                    .to_string(),
            );
        }

        // Cache recommendations
        if metrics.cache_hit_ratio < 0.3 {
            recommendations.push(
                "Very low cache hit ratio - review caching strategy and increase cache size"
                    .to_string(),
            );
        } else if metrics.cache_hit_ratio < 0.5 {
            recommendations
                .push("Low cache hit ratio - review caching strategy and TTL settings".to_string());
        }

        // Pool recommendations
        if metrics.pool_size > 0 && metrics.active_instances >= metrics.pool_size {
            recommendations.push(
                "Instance pool exhausted - increase pool size or implement queue management"
                    .to_string(),
            );
        }

        // Circuit breaker recommendations
        if metrics.circuit_breaker_trips > 10 {
            recommendations.push(
                "Multiple circuit breaker trips detected - investigate root cause and adjust trip thresholds".to_string()
            );
        }

        // Timeout recommendations
        if metrics.timeout_rate > 5.0 {
            recommendations.push(
                "High timeout rate - increase timeout duration or optimize slow operations"
                    .to_string(),
            );
        }

        if recommendations.is_empty() {
            recommendations.push("System is performing well - continue monitoring".to_string());
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_scoring() {
        let calculator = HealthCalculator::new(HealthThresholds::default());

        // Test healthy metrics
        let healthy_metrics = PerformanceMetrics {
            error_rate: 1.0,
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 1024 * 1024 * 512, // 512MB
            p99_extraction_time_ms: 1000.0,
            cache_hit_ratio: 0.8,
            ..Default::default()
        };

        let score = calculator.calculate_health(&healthy_metrics);
        assert!(score > 90.0);

        // Test unhealthy metrics
        let unhealthy_metrics = PerformanceMetrics {
            error_rate: 15.0,
            cpu_usage_percent: 95.0,
            memory_usage_bytes: 1024 * 1024 * 1024 * 5, // 5GB
            p99_extraction_time_ms: 15000.0,
            circuit_breaker_trips: 5,
            timeout_rate: 3.0,
            cache_hit_ratio: 0.2,
            ..Default::default()
        };

        let score = calculator.calculate_health(&unhealthy_metrics);
        assert!(score < 50.0);
    }

    #[test]
    fn test_recommendations() {
        let calculator = HealthCalculator::new(HealthThresholds::default());

        let problematic_metrics = PerformanceMetrics {
            error_rate: 12.0,
            cpu_usage_percent: 92.0,
            cache_hit_ratio: 0.2,
            pool_size: 10,
            active_instances: 10,
            ..Default::default()
        };

        let recommendations = calculator.generate_recommendations(&problematic_metrics);

        // Should generate multiple recommendations
        assert!(recommendations.len() >= 4);

        // Should include critical recommendations
        assert!(recommendations.iter().any(|r| r.contains("CRITICAL")));
    }
}
