//! Test fixtures and mock data for health endpoint tests

use serde_json::json;

/// Mock health response for healthy system
pub fn mock_healthy_response() -> serde_json::Value {
    json!({
        "status": "healthy",
        "version": "0.1.0",
        "timestamp": "2025-10-17T07:00:00Z",
        "uptime": 3600,
        "dependencies": {
            "redis": {
                "status": "healthy",
                "message": "Redis operations successful",
                "response_time_ms": 15,
                "last_check": "2025-10-17T07:00:00Z"
            },
            "extractor": {
                "status": "healthy",
                "message": "WASM extractor initialized successfully",
                "response_time_ms": null,
                "last_check": "2025-10-17T07:00:00Z"
            },
            "http_client": {
                "status": "healthy",
                "message": "HTTP client tests: 2/2 successful",
                "response_time_ms": 250,
                "last_check": "2025-10-17T07:00:00Z"
            }
        },
        "metrics": {
            "memory_usage_bytes": 104857600,
            "active_connections": 5,
            "total_requests": 1000,
            "requests_per_second": 10.5,
            "avg_response_time_ms": 125.0
        }
    })
}

/// Mock health response for degraded system
pub fn mock_degraded_response() -> serde_json::Value {
    json!({
        "status": "degraded",
        "version": "0.1.0",
        "timestamp": "2025-10-17T07:00:00Z",
        "uptime": 3600,
        "dependencies": {
            "redis": {
                "status": "healthy",
                "message": "Redis operations successful",
                "response_time_ms": 15,
                "last_check": "2025-10-17T07:00:00Z"
            },
            "extractor": {
                "status": "healthy",
                "message": "WASM extractor initialized successfully",
                "response_time_ms": null,
                "last_check": "2025-10-17T07:00:00Z"
            },
            "http_client": {
                "status": "degraded",
                "message": "HTTP client tests: 1/2 successful",
                "response_time_ms": 800,
                "last_check": "2025-10-17T07:00:00Z"
            }
        },
        "metrics": {
            "memory_usage_bytes": 2147483648,
            "active_connections": 8,
            "total_requests": 1000,
            "requests_per_second": 10.5,
            "avg_response_time_ms": 850.0
        }
    })
}

/// Mock health response for unhealthy system
pub fn mock_unhealthy_response() -> serde_json::Value {
    json!({
        "status": "unhealthy",
        "version": "0.1.0",
        "timestamp": "2025-10-17T07:00:00Z",
        "uptime": 3600,
        "dependencies": {
            "redis": {
                "status": "unhealthy",
                "message": "Redis error: Connection refused",
                "response_time_ms": null,
                "last_check": "2025-10-17T07:00:00Z"
            },
            "extractor": {
                "status": "healthy",
                "message": "WASM extractor initialized successfully",
                "response_time_ms": null,
                "last_check": "2025-10-17T07:00:00Z"
            },
            "http_client": {
                "status": "unhealthy",
                "message": "All HTTP tests failed",
                "response_time_ms": 2000,
                "last_check": "2025-10-17T07:00:00Z"
            }
        },
        "metrics": {
            "memory_usage_bytes": 4294967296,
            "active_connections": 0,
            "total_requests": 1000,
            "requests_per_second": 0.5,
            "avg_response_time_ms": 5000.0
        }
    })
}

/// Mock service health - healthy
pub fn mock_service_healthy() -> serde_json::Value {
    json!({
        "status": "healthy",
        "message": "Service operational",
        "response_time_ms": 50,
        "last_check": "2025-10-17T07:00:00Z"
    })
}

/// Mock service health - unhealthy
pub fn mock_service_unhealthy() -> serde_json::Value {
    json!({
        "status": "unhealthy",
        "message": "Service error: Connection timeout",
        "response_time_ms": null,
        "last_check": "2025-10-17T07:00:00Z"
    })
}

/// Mock metrics data
pub fn mock_metrics() -> serde_json::Value {
    json!({
        "memory_usage_bytes": 104857600,
        "active_connections": 5,
        "total_requests": 1000,
        "requests_per_second": 10.5,
        "avg_response_time_ms": 125.0,
        "cpu_usage_percent": 45.5,
        "disk_usage_bytes": 1073741824,
        "file_descriptor_count": 128,
        "thread_count": 8,
        "load_average": [1.2, 1.5, 1.8]
    })
}

/// Test data generator for load testing
pub struct HealthTestDataGenerator;

impl HealthTestDataGenerator {
    /// Generate realistic health responses with varying status
    pub fn generate_responses(count: usize) -> Vec<serde_json::Value> {
        (0..count)
            .map(|i| {
                let status = match i % 10 {
                    0 => "unhealthy",
                    1..=2 => "degraded",
                    _ => "healthy",
                };

                json!({
                    "status": status,
                    "version": "0.1.0",
                    "timestamp": format!("2025-10-17T07:00:{:02}Z", i % 60),
                    "uptime": 3600 + i * 10,
                    "dependencies": Self::generate_dependencies(status),
                    "metrics": Self::generate_metrics(status)
                })
            })
            .collect()
    }

    fn generate_dependencies(status: &str) -> serde_json::Value {
        let redis_status = if status == "unhealthy" { "unhealthy" } else { "healthy" };
        let http_status = if status == "degraded" { "degraded" } else { "healthy" };

        json!({
            "redis": {
                "status": redis_status,
                "message": format!("Redis {}", redis_status),
                "response_time_ms": if redis_status == "healthy" { Some(15) } else { None },
                "last_check": "2025-10-17T07:00:00Z"
            },
            "extractor": {
                "status": "healthy",
                "message": "WASM extractor OK",
                "response_time_ms": null,
                "last_check": "2025-10-17T07:00:00Z"
            },
            "http_client": {
                "status": http_status,
                "message": format!("HTTP client {}", http_status),
                "response_time_ms": Some(if http_status == "healthy" { 200 } else { 800 }),
                "last_check": "2025-10-17T07:00:00Z"
            }
        })
    }

    fn generate_metrics(status: &str) -> serde_json::Value {
        let (memory, response_time) = match status {
            "unhealthy" => (4 * 1024 * 1024 * 1024u64, 5000.0),
            "degraded" => (2 * 1024 * 1024 * 1024u64, 850.0),
            _ => (100 * 1024 * 1024u64, 125.0),
        };

        json!({
            "memory_usage_bytes": memory,
            "active_connections": if status == "unhealthy" { 0 } else { 5 },
            "total_requests": 1000,
            "requests_per_second": if status == "unhealthy" { 0.5 } else { 10.5 },
            "avg_response_time_ms": response_time
        })
    }
}

#[cfg(test)]
mod fixture_tests {
    use super::*;

    #[test]
    fn test_mock_healthy_response_valid() {
        let response = mock_healthy_response();
        assert_eq!(response["status"], "healthy");
        assert!(response["dependencies"].is_object());
        assert!(response["metrics"].is_object());
    }

    #[test]
    fn test_mock_degraded_response_valid() {
        let response = mock_degraded_response();
        assert_eq!(response["status"], "degraded");
    }

    #[test]
    fn test_mock_unhealthy_response_valid() {
        let response = mock_unhealthy_response();
        assert_eq!(response["status"], "unhealthy");
    }

    #[test]
    fn test_data_generator() {
        let responses = HealthTestDataGenerator::generate_responses(20);
        assert_eq!(responses.len(), 20);

        // Verify status distribution
        let healthy_count = responses
            .iter()
            .filter(|r| r["status"] == "healthy")
            .count();

        assert!(healthy_count >= 10, "Should have multiple healthy responses");
    }
}
