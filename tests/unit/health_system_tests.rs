//! Unit tests for health system components
//!
//! This module provides comprehensive testing for health checking, metrics
//! collection, and health calculation components.

#[cfg(test)]
mod health_checker_tests {
    use riptide_api::health::HealthChecker;
    use riptide_api::models::{DependencyStatus, HealthResponse, ServiceHealth, SystemMetrics};
    use riptide_api::state::AppState;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Helper to create a test AppState
    async fn create_test_app_state() -> AppState {
        let config = riptide_api::AppConfig {
            port: 0,
            redis_url: "redis://localhost:6379".to_string(),
            headless_url: Some("http://localhost:3001".to_string()),
            cache_ttl: 300,
            max_concurrency: 10,
            gate_hi_threshold: 0.8,
            gate_lo_threshold: 0.3,
            cors_origins: vec![],
            api_key: None,
            openai_api_key: None,
            spider_config: None,
        };

        AppState {
            config: Arc::new(config),
            http_client: reqwest::Client::new(),
            cache: Arc::new(Mutex::new(riptide_api::cache::Cache::new())),
            metrics: riptide_api::metrics::MetricsCollector::new(),
            resource_manager: Arc::new(riptide_api::resources::ResourceManager::new()),
            spider: None,
        }
    }

    #[tokio::test]
    async fn test_health_checker_initialization() {
        let health_checker = HealthChecker::new();

        // Should initialize with reasonable defaults
        assert!(!health_checker.git_sha.is_empty());
        assert!(!health_checker.build_timestamp.is_empty());
        assert!(!health_checker.component_versions.is_empty());

        // Should have core component versions
        assert!(health_checker.component_versions.contains_key("riptide-api"));
        assert!(health_checker.component_versions.contains_key("axum"));
        assert!(health_checker.component_versions.contains_key("tokio"));
    }

    #[tokio::test]
    async fn test_health_checker_environment_variables() {
        // Set environment variables for testing
        std::env::set_var("GIT_SHA", "test-sha-123");
        std::env::set_var("BUILD_TIMESTAMP", "2024-01-01T00:00:00Z");

        let health_checker = HealthChecker::new();

        assert_eq!(health_checker.git_sha, "test-sha-123");
        assert_eq!(health_checker.build_timestamp, "2024-01-01T00:00:00Z");

        // Cleanup
        std::env::remove_var("GIT_SHA");
        std::env::remove_var("BUILD_TIMESTAMP");
    }

    #[tokio::test]
    async fn test_health_checker_github_sha_fallback() {
        // Remove GIT_SHA and set GITHUB_SHA
        std::env::remove_var("GIT_SHA");
        std::env::set_var("GITHUB_SHA", "github-sha-456");

        let health_checker = HealthChecker::new();

        assert_eq!(health_checker.git_sha, "github-sha-456");

        // Cleanup
        std::env::remove_var("GITHUB_SHA");
    }

    #[tokio::test]
    async fn test_comprehensive_health_check() {
        let health_checker = HealthChecker::new();
        let state = create_test_app_state().await;

        let health_response = health_checker.check_health(&state).await;

        // Should return valid health response
        assert!(!health_response.status.is_empty());
        assert!(!health_response.version.is_empty());
        assert!(!health_response.timestamp.is_empty());
        assert!(health_response.uptime >= 0);

        // Should have dependency status
        assert!(!health_response.dependencies.redis.status.is_empty());
        assert!(!health_response.dependencies.extractor.status.is_empty());
        assert!(!health_response.dependencies.http_client.status.is_empty());

        // Should have headless service status (configured in test state)
        assert!(health_response.dependencies.headless_service.is_some());

        // Should have metrics
        assert!(health_response.metrics.is_some());
    }

    #[tokio::test]
    async fn test_redis_health_check() {
        let health_checker = HealthChecker::new();
        let state = create_test_app_state().await;

        let redis_health = health_checker.check_redis_health(&state).await;

        // Should return service health status
        assert!(!redis_health.status.is_empty());
        assert!(!redis_health.last_check.is_empty());

        // Status should be one of the valid values
        assert!(matches!(redis_health.status.as_str(), "healthy" | "unhealthy"));

        if redis_health.status == "healthy" {
            assert!(redis_health.response_time_ms.is_some());
            assert!(redis_health.response_time_ms.unwrap() >= 0);
        }
    }

    #[tokio::test]
    async fn test_http_client_health_check() {
        let health_checker = HealthChecker::new();
        let state = create_test_app_state().await;

        let http_health = health_checker.check_http_client_health(&state).await;

        // Should return service health status
        assert!(!http_health.status.is_empty());
        assert!(!http_health.last_check.is_empty());

        // Status should be valid
        assert!(matches!(http_health.status.as_str(), "healthy" | "unhealthy" | "degraded"));

        // Should have response time
        assert!(http_health.response_time_ms.is_some());

        // Should have meaningful message
        if let Some(message) = &http_health.message {
            assert!(!message.is_empty());
        }
    }

    #[tokio::test]
    async fn test_extractor_health_check() {
        let health_checker = HealthChecker::new();
        let state = create_test_app_state().await;

        let extractor_health = health_checker.check_extractor_health(&state).await;

        // WASM extractor should be healthy by default (initialized at startup)
        assert_eq!(extractor_health.status, "healthy");
        assert!(!extractor_health.last_check.is_empty());

        if let Some(message) = &extractor_health.message {
            assert!(message.contains("WASM extractor"));
        }
    }

    #[tokio::test]
    async fn test_headless_health_check() {
        let health_checker = HealthChecker::new();
        let state = create_test_app_state().await; // Has headless URL configured

        let headless_health = health_checker.check_headless_health(&state).await;

        // Should return service health status
        assert!(!headless_health.status.is_empty());
        assert!(!headless_health.last_check.is_empty());

        // Status should be valid
        assert!(matches!(
            headless_health.status.as_str(),
            "healthy" | "unhealthy" | "not_configured"
        ));

        if let Some(message) = &headless_health.message {
            assert!(!message.is_empty());
        }
    }

    #[tokio::test]
    async fn test_headless_health_check_not_configured() {
        let health_checker = HealthChecker::new();
        let mut state = create_test_app_state().await;

        // Remove headless URL configuration
        let mut config = (*state.config).clone();
        config.headless_url = None;
        state.config = Arc::new(config);

        let headless_health = health_checker.check_headless_health(&state).await;

        // Should indicate not configured
        assert_eq!(headless_health.status, "not_configured");

        if let Some(message) = &headless_health.message {
            assert!(message.contains("not configured"));
        }
    }

    #[tokio::test]
    async fn test_system_metrics_collection() {
        let health_checker = HealthChecker::new();
        let state = create_test_app_state().await;

        let metrics = health_checker.collect_system_metrics(&state).await;

        // Should have valid metrics
        assert!(metrics.memory_usage_bytes > 0);
        assert!(metrics.active_connections >= 0);
        assert!(metrics.total_requests >= 0);
        assert!(metrics.requests_per_second >= 0.0);
        assert!(metrics.avg_response_time_ms >= 0.0);
    }

    #[tokio::test]
    async fn test_memory_usage_calculation() {
        let memory_usage = HealthChecker::get_memory_usage();

        // Should return reasonable memory usage
        assert!(memory_usage > 0);
        assert!(memory_usage < 100 * 1024 * 1024 * 1024); // Less than 100GB (sanity check)

        // Multiple calls should be consistent
        let memory_usage2 = HealthChecker::get_memory_usage();
        let difference = if memory_usage > memory_usage2 {
            memory_usage - memory_usage2
        } else {
            memory_usage2 - memory_usage
        };

        // Memory usage shouldn't vary drastically between calls
        assert!(difference < 100 * 1024 * 1024); // Less than 100MB difference
    }

    #[tokio::test]
    async fn test_health_status_determination() {
        let health_checker = HealthChecker::new();
        let state = create_test_app_state().await;

        let health_response = health_checker.check_health(&state).await;

        // Overall status should be based on dependency health
        let redis_healthy = health_response.dependencies.redis.status == "healthy";
        let extractor_healthy = health_response.dependencies.extractor.status == "healthy";
        let http_healthy = health_response.dependencies.http_client.status == "healthy";

        let all_healthy = redis_healthy && extractor_healthy && http_healthy;

        if all_healthy {
            assert_eq!(health_response.status, "healthy");
        } else {
            assert_eq!(health_response.status, "degraded");
        }
    }

    #[test]
    fn test_service_health_from_dependency_health() {
        use riptide_api::state::DependencyHealth;

        // Test conversion from DependencyHealth to ServiceHealth
        let healthy = ServiceHealth::from(DependencyHealth::Healthy);
        assert_eq!(healthy.status, "healthy");
        assert!(healthy.message.is_none());

        let unhealthy = ServiceHealth::from(DependencyHealth::Unhealthy("Test error".to_string()));
        assert_eq!(unhealthy.status, "unhealthy");
        assert_eq!(unhealthy.message, Some("Test error".to_string()));

        let unknown = ServiceHealth::from(DependencyHealth::Unknown);
        assert_eq!(unknown.status, "unknown");
        assert!(unknown.message.is_none());
    }

    #[test]
    fn test_health_response_structure() {
        let dependencies = DependencyStatus {
            redis: ServiceHealth {
                status: "healthy".to_string(),
                message: Some("Redis OK".to_string()),
                response_time_ms: Some(15),
                last_check: chrono::Utc::now().to_rfc3339(),
            },
            extractor: ServiceHealth {
                status: "healthy".to_string(),
                message: Some("WASM OK".to_string()),
                response_time_ms: None,
                last_check: chrono::Utc::now().to_rfc3339(),
            },
            http_client: ServiceHealth {
                status: "degraded".to_string(),
                message: Some("Some endpoints slow".to_string()),
                response_time_ms: Some(500),
                last_check: chrono::Utc::now().to_rfc3339(),
            },
            headless_service: None,
            spider_engine: None,
        };

        let metrics = SystemMetrics {
            memory_usage_bytes: 100 * 1024 * 1024, // 100MB
            active_connections: 5,
            total_requests: 1000,
            requests_per_second: 10.5,
            avg_response_time_ms: 125.0,
        };

        let health_response = HealthResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            uptime: 3600, // 1 hour
            dependencies,
            metrics: Some(metrics),
        };

        // Test serialization
        let json = serde_json::to_string(&health_response).expect("Should serialize");
        assert!(json.contains("healthy"));
        assert!(json.contains("1.0.0"));
        assert!(json.contains("Redis OK"));

        // Test deserialization
        let deserialized: HealthResponse = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.status, "healthy");
        assert_eq!(deserialized.version, "1.0.0");
        assert_eq!(deserialized.dependencies.redis.status, "healthy");
    }
}

#[cfg(test)]
mod health_calculator_tests {
    use riptide_core::monitoring::health::HealthCalculator;
    use riptide_core::monitoring::metrics::{HealthThresholds, PerformanceMetrics};

    fn create_test_thresholds() -> HealthThresholds {
        HealthThresholds {
            error_rate_warning: 5.0,
            error_rate_critical: 10.0,
            cpu_usage_warning: 70.0,
            cpu_usage_critical: 85.0,
            memory_usage_warning: 2 * 1024 * 1024 * 1024, // 2GB
            memory_usage_critical: 4 * 1024 * 1024 * 1024, // 4GB
            extraction_time_warning_ms: 5000.0,
            extraction_time_critical_ms: 10000.0,
        }
    }

    fn create_healthy_metrics() -> PerformanceMetrics {
        PerformanceMetrics {
            error_rate: 1.0,
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 1024 * 1024 * 1024, // 1GB
            avg_extraction_time_ms: 2000.0,
            p99_extraction_time_ms: 3000.0,
            cache_hit_ratio: 0.8,
            circuit_breaker_trips: 0,
            timeout_rate: 0.5,
            pool_size: 10,
            active_instances: 3,
            health_score: 0.0, // Will be calculated
        }
    }

    #[test]
    fn test_health_calculation_healthy_system() {
        let calculator = HealthCalculator::new(create_test_thresholds());
        let metrics = create_healthy_metrics();

        let score = calculator.calculate_health(&metrics);

        // Healthy system should score very high
        assert!(score >= 95.0, "Healthy system should score at least 95, got {}", score);
        assert!(score <= 100.0, "Score should not exceed 100, got {}", score);
    }

    #[test]
    fn test_health_calculation_high_error_rate() {
        let calculator = HealthCalculator::new(create_test_thresholds());
        let mut metrics = create_healthy_metrics();

        // Test warning level error rate
        metrics.error_rate = 7.0; // Between warning (5) and critical (10)
        let score = calculator.calculate_health(&metrics);
        assert!(score < 95.0, "High error rate should reduce score, got {}", score);
        assert!(score > 80.0, "Score shouldn't be too low for warning level, got {}", score);

        // Test critical level error rate
        metrics.error_rate = 12.0; // Above critical (10)
        let score = calculator.calculate_health(&metrics);
        assert!(score < 80.0, "Critical error rate should significantly reduce score, got {}", score);
    }

    #[test]
    fn test_health_calculation_high_cpu_usage() {
        let calculator = HealthCalculator::new(create_test_thresholds());
        let mut metrics = create_healthy_metrics();

        // Test warning level CPU usage
        metrics.cpu_usage_percent = 75.0; // Between warning (70) and critical (85)
        let score = calculator.calculate_health(&metrics);
        assert!(score < 95.0, "High CPU usage should reduce score, got {}", score);

        // Test critical level CPU usage
        metrics.cpu_usage_percent = 90.0; // Above critical (85)
        let score = calculator.calculate_health(&metrics);
        assert!(score < 80.0, "Critical CPU usage should significantly reduce score, got {}", score);
    }

    #[test]
    fn test_health_calculation_high_memory_usage() {
        let calculator = HealthCalculator::new(create_test_thresholds());
        let mut metrics = create_healthy_metrics();

        // Test warning level memory usage
        metrics.memory_usage_bytes = 3 * 1024 * 1024 * 1024; // 3GB (between 2GB warning and 4GB critical)
        let score = calculator.calculate_health(&metrics);
        assert!(score < 95.0, "High memory usage should reduce score, got {}", score);

        // Test critical level memory usage
        metrics.memory_usage_bytes = 5 * 1024 * 1024 * 1024; // 5GB (above 4GB critical)
        let score = calculator.calculate_health(&metrics);
        assert!(score < 85.0, "Critical memory usage should significantly reduce score, got {}", score);
    }

    #[test]
    fn test_health_calculation_slow_extraction() {
        let calculator = HealthCalculator::new(create_test_thresholds());
        let mut metrics = create_healthy_metrics();

        // Test warning level extraction time
        metrics.p99_extraction_time_ms = 7000.0; // Between warning (5000) and critical (10000)
        let score = calculator.calculate_health(&metrics);
        assert!(score < 95.0, "Slow extraction should reduce score, got {}", score);

        // Test critical level extraction time
        metrics.p99_extraction_time_ms = 12000.0; // Above critical (10000)
        let score = calculator.calculate_health(&metrics);
        assert!(score < 90.0, "Very slow extraction should significantly reduce score, got {}", score);
    }

    #[test]
    fn test_health_calculation_circuit_breaker_trips() {
        let calculator = HealthCalculator::new(create_test_thresholds());
        let mut metrics = create_healthy_metrics();

        metrics.circuit_breaker_trips = 5;
        let score = calculator.calculate_health(&metrics);
        assert!(score < 95.0, "Circuit breaker trips should reduce score, got {}", score);

        metrics.circuit_breaker_trips = 20; // Should be capped at max 20 point deduction
        let score = calculator.calculate_health(&metrics);
        assert!(score >= 80.0, "Circuit breaker penalty should be capped, got {}", score);
    }

    #[test]
    fn test_health_calculation_cache_bonus() {
        let calculator = HealthCalculator::new(create_test_thresholds());
        let mut metrics = create_healthy_metrics();

        // Good cache hit ratio should provide bonus
        metrics.cache_hit_ratio = 0.85; // Above 0.7 threshold
        let score_with_bonus = calculator.calculate_health(&metrics);

        metrics.cache_hit_ratio = 0.5; // Below 0.7 threshold
        let score_without_bonus = calculator.calculate_health(&metrics);

        assert!(score_with_bonus > score_without_bonus,
               "Good cache ratio should provide bonus: {} vs {}",
               score_with_bonus, score_without_bonus);
    }

    #[test]
    fn test_health_calculation_bounds() {
        let calculator = HealthCalculator::new(create_test_thresholds());

        // Test extremely bad metrics
        let bad_metrics = PerformanceMetrics {
            error_rate: 50.0,
            cpu_usage_percent: 100.0,
            memory_usage_bytes: 10 * 1024 * 1024 * 1024, // 10GB
            avg_extraction_time_ms: 30000.0,
            p99_extraction_time_ms: 50000.0,
            cache_hit_ratio: 0.1,
            circuit_breaker_trips: 50,
            timeout_rate: 10.0,
            pool_size: 10,
            active_instances: 3,
            health_score: 0.0,
        };

        let score = calculator.calculate_health(&bad_metrics);
        assert!(score >= 0.0, "Score should not go below 0, got {}", score);
        assert!(score <= 100.0, "Score should not exceed 100, got {}", score);
    }

    #[test]
    fn test_health_summary_generation() {
        let calculator = HealthCalculator::new(create_test_thresholds());

        let test_cases = vec![
            (97.0, "Excellent"),
            (88.0, "Good"),
            (75.0, "Fair"),
            (55.0, "Poor"),
            (30.0, "Critical"),
        ];

        for (score, expected_category) in test_cases {
            let mut metrics = create_healthy_metrics();
            metrics.health_score = score;

            let summary = calculator.generate_health_summary(&metrics);
            assert!(summary.contains(expected_category),
                   "Score {} should generate summary containing '{}', got: {}",
                   score, expected_category, summary);
        }
    }

    #[test]
    fn test_recommendations_generation() {
        let calculator = HealthCalculator::new(create_test_thresholds());

        // Test various problematic scenarios
        let test_scenarios = vec![
            (
                PerformanceMetrics {
                    error_rate: 12.0, // Critical
                    ..create_healthy_metrics()
                },
                "CRITICAL",
                "error rate"
            ),
            (
                PerformanceMetrics {
                    cpu_usage_percent: 90.0, // Critical
                    ..create_healthy_metrics()
                },
                "CRITICAL",
                "CPU usage"
            ),
            (
                PerformanceMetrics {
                    memory_usage_bytes: 5 * 1024 * 1024 * 1024, // Critical
                    ..create_healthy_metrics()
                },
                "CRITICAL",
                "Memory usage"
            ),
            (
                PerformanceMetrics {
                    avg_extraction_time_ms: 12000.0, // Critical
                    ..create_healthy_metrics()
                },
                "CRITICAL",
                "extraction latency"
            ),
            (
                PerformanceMetrics {
                    cache_hit_ratio: 0.2, // Very low
                    ..create_healthy_metrics()
                },
                "cache",
                "cache hit ratio"
            ),
        ];

        for (metrics, expected_content, description) in test_scenarios {
            let recommendations = calculator.generate_recommendations(&metrics);
            assert!(!recommendations.is_empty(),
                   "Should generate recommendations for {}", description);

            let recommendations_text = recommendations.join(" ");
            assert!(recommendations_text.to_lowercase().contains(&expected_content.to_lowercase()),
                   "Recommendations for {} should mention '{}', got: {:?}",
                   description, expected_content, recommendations);
        }
    }

    #[test]
    fn test_healthy_system_recommendations() {
        let calculator = HealthCalculator::new(create_test_thresholds());
        let metrics = create_healthy_metrics();

        let recommendations = calculator.generate_recommendations(&metrics);

        // Healthy system should get positive recommendation
        let recommendations_text = recommendations.join(" ");
        assert!(recommendations_text.to_lowercase().contains("performing well") ||
                recommendations_text.to_lowercase().contains("continue monitoring"),
               "Healthy system should get positive recommendation, got: {:?}",
               recommendations);
    }

    #[test]
    fn test_pool_exhaustion_recommendations() {
        let calculator = HealthCalculator::new(create_test_thresholds());
        let mut metrics = create_healthy_metrics();

        // Pool exhaustion scenario
        metrics.pool_size = 10;
        metrics.active_instances = 10; // All instances active

        let recommendations = calculator.generate_recommendations(&metrics);
        let recommendations_text = recommendations.join(" ");

        assert!(recommendations_text.to_lowercase().contains("pool"),
               "Pool exhaustion should generate pool-related recommendation, got: {:?}",
               recommendations);
    }

    #[test]
    fn test_multiple_issue_recommendations() {
        let calculator = HealthCalculator::new(create_test_thresholds());
        let problematic_metrics = PerformanceMetrics {
            error_rate: 8.0,     // Warning level
            cpu_usage_percent: 80.0, // Warning level
            cache_hit_ratio: 0.3,    // Poor
            circuit_breaker_trips: 15, // Multiple trips
            ..create_healthy_metrics()
        };

        let recommendations = calculator.generate_recommendations(&problematic_metrics);

        // Should generate multiple recommendations
        assert!(recommendations.len() >= 3,
               "Multiple issues should generate multiple recommendations, got {} recommendations: {:?}",
               recommendations.len(), recommendations);

        let recommendations_text = recommendations.join(" ");
        assert!(recommendations_text.to_lowercase().contains("error"));
        assert!(recommendations_text.to_lowercase().contains("cpu"));
        assert!(recommendations_text.to_lowercase().contains("cache"));
    }
}