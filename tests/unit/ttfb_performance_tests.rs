//! TTFB (Time To First Byte) Performance Tests
//!
//! These tests validate the critical TTFB < 500ms requirement with warm cache.
//! Tests cover various scenarios that affect TTFB performance including:
//! - Cache hit scenarios
//! - Buffer initialization time
//! - First response serialization time
//! - Network overhead simulation
//! - Concurrent connection impact

use std::time::{Duration, Instant};
use tokio::time::{sleep, timeout};
use futures::stream::StreamExt;
use serde_json::Value;
use bytes::Bytes;
use riptide_utils::http::{HttpClientFactory, HttpConfig};

/// TTFB test framework
struct TTFBTestFramework {
    base_url: String,
    client: reqwest::Client,
}

impl TTFBTestFramework {
    fn new() -> Self {
        let config = HttpConfig {
            timeout_ms: 30000,
            ..HttpConfig::default()
        };
        Self {
            base_url: "http://localhost:8080".to_string(),
            client: HttpClientFactory::create(config).expect("Failed to create HTTP client"),
        }
    }

    /// Make request and measure TTFB precisely
    async fn measure_ttfb(&self, endpoint: &str, body: Value) -> Result<TTFBMeasurement, String> {
        let start_time = Instant::now();
        
        let response = self.client
            .post(&format!("{}/{}", self.base_url, endpoint))
            .header("Accept", "application/x-ndjson")
            .header("Connection", "keep-alive") // Reuse connections
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let ttfb = start_time.elapsed();
        let status = response.status();
        let headers = response.headers().clone();
        
        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status, 
                response.text().await.unwrap_or_default()));
        }

        // Measure time to first data chunk
        let mut stream = response.bytes_stream();
        let first_chunk_time = if let Some(chunk_result) = stream.next().await {
            let first_chunk_time = start_time.elapsed();
            chunk_result.map_err(|e| format!("Stream error: {}", e))?;
            first_chunk_time
        } else {
            ttfb // No chunks received
        };

        Ok(TTFBMeasurement {
            ttfb,
            first_chunk_time,
            status,
            headers,
            start_time,
        })
    }

    /// Warm up cache with initial request
    async fn warm_cache(&self, endpoint: &str, body: Value) -> Result<(), String> {
        let _ = self.client
            .post(&format!("{}/{}", self.base_url, endpoint))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Warmup request failed: {}", e))?;
        
        // Allow time for cache population
        sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    /// Create cache-friendly crawl request
    fn create_cacheable_request(&self, urls: Vec<String>) -> Value {
        serde_json::json!({
            "urls": urls,
            "options": {
                "cache_mode": "write_through", // Enable caching
                "concurrency": 1,
                "stream": true,
                "timeout_ms": 10000,
                "user_agent": "TTFBTest/1.0",
                "respect_robots": false
            }
        })
    }

    /// Create high-performance request optimized for speed
    fn create_optimized_request(&self, urls: Vec<String>) -> Value {
        serde_json::json!({
            "urls": urls,
            "options": {
                "cache_mode": "read_through", // Prioritize cache reads
                "concurrency": 2,
                "stream": true,
                "timeout_ms": 5000,
                "user_agent": "TTFBOptimized/1.0",
                "respect_robots": false,
                "enable_compression": true
            }
        })
    }
}

#[derive(Debug)]
struct TTFBMeasurement {
    ttfb: Duration,
    first_chunk_time: Duration,
    status: reqwest::StatusCode,
    headers: reqwest::header::HeaderMap,
    start_time: Instant,
}

impl TTFBMeasurement {
    fn validate_ttfb_requirement(&self, max_ttfb_ms: u64) -> Result<(), String> {
        if self.ttfb.as_millis() > max_ttfb_ms as u128 {
            return Err(format!(
                "TTFB {}ms exceeds requirement of {}ms",
                self.ttfb.as_millis(),
                max_ttfb_ms
            ));
        }
        Ok(())
    }

    fn validate_streaming_headers(&self) -> Result<(), String> {
        let content_type = self.headers.get("content-type")
            .and_then(|v| v.to_str().ok())
            .ok_or("Missing content-type header")?;

        if content_type != "application/x-ndjson" {
            return Err(format!("Expected NDJSON content-type, got: {}", content_type));
        }

        if self.headers.get("transfer-encoding").is_none() {
            return Err("Missing transfer-encoding header for streaming".to_string());
        }

        Ok(())
    }
}

// ==================== TTFB PERFORMANCE TESTS ====================

/// Test TTFB with warm cache - primary requirement
#[tokio::test]
async fn test_ttfb_warm_cache_500ms_requirement() {
    let framework = TTFBTestFramework::new();
    
    // Use simple, cacheable URLs
    let urls = vec![
        "https://httpbin.org/json".to_string(),
        "https://httpbin.org/user-agent".to_string(),
    ];
    
    let request = framework.create_cacheable_request(urls.clone());
    
    // First request to warm cache
    framework.warm_cache("crawl/stream", request.clone()).await
        .expect("Cache warming should succeed");
    
    // Second request should hit warm cache
    let measurement = framework.measure_ttfb("crawl/stream", request).await
        .expect("TTFB measurement should succeed");
    
    // Validate TTFB requirement
    measurement.validate_ttfb_requirement(500)
        .expect("Should meet TTFB < 500ms requirement with warm cache");
    
    measurement.validate_streaming_headers()
        .expect("Should have proper streaming headers");
    
    println!("TTFB with warm cache: {}ms (requirement: 500ms)", 
             measurement.ttfb.as_millis());
    
    // Additional performance validations
    assert!(measurement.first_chunk_time.as_millis() <= measurement.ttfb.as_millis() + 50,
            "First chunk should arrive shortly after TTFB");
}

/// Test TTFB with different cache strategies
#[tokio::test]
async fn test_ttfb_different_cache_strategies() {
    let framework = TTFBTestFramework::new();
    let urls = vec!["https://httpbin.org/delay/1".to_string()];
    
    let cache_strategies = vec![
        ("read_through", "Read-through cache"),
        ("write_through", "Write-through cache"),
        ("write_around", "Write-around cache"),
    ];
    
    for (cache_mode, description) in cache_strategies {
        let mut request = framework.create_cacheable_request(urls.clone());
        request["options"]["cache_mode"] = Value::String(cache_mode.to_string());
        
        // Warm cache
        framework.warm_cache("crawl/stream", request.clone()).await
            .expect("Cache warming should succeed");
        
        // Measure TTFB
        let measurement = framework.measure_ttfb("crawl/stream", request).await
            .expect("TTFB measurement should succeed");
        
        println!("{}: TTFB {}ms", description, measurement.ttfb.as_millis());
        
        // Different cache strategies may have different TTFB characteristics
        // But all should be reasonable for cached content
        if cache_mode == "read_through" || cache_mode == "write_through" {
            assert!(measurement.ttfb.as_millis() < 800, 
                    "Cache strategy {} should have reasonable TTFB: {}ms", 
                    cache_mode, measurement.ttfb.as_millis());
        }
    }
}

/// Test TTFB under different concurrency levels
#[tokio::test]
async fn test_ttfb_concurrency_impact() {
    let framework = TTFBTestFramework::new();
    let urls = vec![
        "https://httpbin.org/json".to_string(),
        "https://httpbin.org/headers".to_string(),
    ];
    
    let concurrency_levels = vec![1, 2, 5, 10];
    
    for concurrency in concurrency_levels {
        let mut request = framework.create_cacheable_request(urls.clone());
        request["options"]["concurrency"] = Value::Number(concurrency.into());
        
        // Warm cache
        framework.warm_cache("crawl/stream", request.clone()).await
            .expect("Cache warming should succeed");
        
        // Measure TTFB
        let measurement = framework.measure_ttfb("crawl/stream", request).await
            .expect("TTFB measurement should succeed");
        
        println!("Concurrency {}: TTFB {}ms", concurrency, measurement.ttfb.as_millis());
        
        // TTFB should not significantly degrade with higher concurrency for cached content
        assert!(measurement.ttfb.as_millis() < 1000,
                "Concurrency {} should not significantly impact TTFB: {}ms",
                concurrency, measurement.ttfb.as_millis());
    }
}

/// Test TTFB with multiple concurrent connections
#[tokio::test]
async fn test_ttfb_concurrent_connections() {
    let framework = TTFBTestFramework::new();
    let urls = vec!["https://httpbin.org/json".to_string()];
    
    let request = framework.create_optimized_request(urls.clone());
    
    // Warm cache
    framework.warm_cache("crawl/stream", request.clone()).await
        .expect("Cache warming should succeed");
    
    // Start multiple concurrent requests
    let concurrent_requests = 5;
    let mut handles = Vec::new();
    
    for i in 0..concurrent_requests {
        let framework_clone = TTFBTestFramework::new();
        let request_clone = request.clone();
        
        let handle = tokio::spawn(async move {
            let measurement = framework_clone.measure_ttfb("crawl/stream", request_clone).await
                .expect("Concurrent TTFB measurement should succeed");
            (i, measurement)
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let results = futures::future::join_all(handles).await;
    
    let mut ttfb_measurements = Vec::new();
    for result in results {
        let (index, measurement) = result.expect("Concurrent task should succeed");
        ttfb_measurements.push(measurement.ttfb);
        
        println!("Connection {}: TTFB {}ms", index, measurement.ttfb.as_millis());
        
        // Each connection should meet reasonable TTFB requirements
        assert!(measurement.ttfb.as_millis() < 1000,
                "Concurrent connection {} should have reasonable TTFB: {}ms",
                index, measurement.ttfb.as_millis());
    }
    
    // Calculate statistics
    let avg_ttfb = ttfb_measurements.iter().sum::<Duration>().as_millis() as f64 / concurrent_requests as f64;
    let min_ttfb = ttfb_measurements.iter().min().unwrap().as_millis();
    let max_ttfb = ttfb_measurements.iter().max().unwrap().as_millis();
    
    println!("Concurrent TTFB stats: avg={:.2}ms, min={}ms, max={}ms",
             avg_ttfb, min_ttfb, max_ttfb);
    
    // Even under concurrent load, cached responses should be fast
    assert!(avg_ttfb < 800.0, "Average TTFB under concurrent load should be reasonable");
    assert!(max_ttfb < 1200, "Maximum TTFB under concurrent load should be acceptable");
}

/// Test TTFB with different payload sizes
#[tokio::test]
async fn test_ttfb_payload_size_impact() {
    let framework = TTFBTestFramework::new();
    
    let payload_configs = vec![
        (vec!["https://httpbin.org/json"], "Small payload"),
        (vec![
            "https://httpbin.org/json",
            "https://httpbin.org/headers",
            "https://httpbin.org/user-agent",
        ], "Medium payload"),
        ((0..10).map(|i| format!("https://httpbin.org/status/200?id={}", i)).collect(), "Large payload"),
    ];
    
    for (urls, description) in payload_configs {
        let request = framework.create_optimized_request(urls.clone());
        
        // Warm cache
        framework.warm_cache("crawl/stream", request.clone()).await
            .expect("Cache warming should succeed");
        
        // Measure TTFB
        let measurement = framework.measure_ttfb("crawl/stream", request).await
            .expect("TTFB measurement should succeed");
        
        println!("{} ({} URLs): TTFB {}ms", 
                 description, urls.len(), measurement.ttfb.as_millis());
        
        // TTFB should not significantly increase with payload size for cached content
        let max_expected_ttfb = match urls.len() {
            1..=3 => 500,    // Small payloads should meet strict requirement
            4..=10 => 800,   // Medium payloads should still be fast
            _ => 1200,       // Large payloads get more lenient requirement
        };
        
        assert!(measurement.ttfb.as_millis() < max_expected_ttfb as u128,
                "{} should have TTFB < {}ms, got {}ms",
                description, max_expected_ttfb, measurement.ttfb.as_millis());
    }
}

/// Test TTFB cold vs warm cache performance
#[tokio::test]
async fn test_ttfb_cold_vs_warm_cache() {
    let framework = TTFBTestFramework::new();
    let urls = vec![
        "https://httpbin.org/json".to_string(),
        "https://httpbin.org/headers".to_string(),
    ];
    
    let request = framework.create_cacheable_request(urls);
    
    // Measure cold cache TTFB
    let cold_measurement = framework.measure_ttfb("crawl/stream", request.clone()).await
        .expect("Cold cache TTFB measurement should succeed");
    
    // Allow cache to warm
    sleep(Duration::from_millis(500)).await;
    
    // Measure warm cache TTFB
    let warm_measurement = framework.measure_ttfb("crawl/stream", request).await
        .expect("Warm cache TTFB measurement should succeed");
    
    println!("Cold cache TTFB: {}ms", cold_measurement.ttfb.as_millis());
    println!("Warm cache TTFB: {}ms", warm_measurement.ttfb.as_millis());
    
    // Warm cache should be significantly faster
    let improvement_factor = cold_measurement.ttfb.as_millis() as f64 / 
                           warm_measurement.ttfb.as_millis().max(1) as f64;
    
    println!("Cache improvement factor: {:.2}x", improvement_factor);
    
    // Warm cache should meet the strict requirement
    warm_measurement.validate_ttfb_requirement(500)
        .expect("Warm cache should meet TTFB requirement");
    
    // Warm cache should show measurable improvement
    assert!(improvement_factor > 1.2, 
            "Warm cache should be at least 20% faster than cold cache");
}

/// Test TTFB with timeout scenarios
#[tokio::test]
async fn test_ttfb_timeout_handling() {
    let framework = TTFBTestFramework::new();
    let urls = vec!["https://httpbin.org/json".to_string()];
    
    let request = framework.create_optimized_request(urls);
    
    // Test with very strict timeout
    let timeout_result = timeout(
        Duration::from_millis(100), // Very strict timeout
        framework.measure_ttfb("crawl/stream", request.clone())
    ).await;
    
    match timeout_result {
        Ok(measurement) => {
            // If it succeeded within 100ms, that's excellent
            println!("Excellent TTFB performance: {}ms (within 100ms timeout)", 
                     measurement.ttfb.as_millis());
            assert!(measurement.ttfb.as_millis() < 100);
        }
        Err(_) => {
            // Timeout is acceptable for first request, test with longer timeout
            let relaxed_result = timeout(
                Duration::from_secs(5),
                framework.measure_ttfb("crawl/stream", request)
            ).await;
            
            match relaxed_result {
                Ok(measurement) => {
                    println!("TTFB within relaxed timeout: {}ms", measurement.ttfb.as_millis());
                    assert!(measurement.ttfb.as_millis() < 5000);
                }
                Err(_) => {
                    panic!("TTFB should complete within reasonable timeout");
                }
            }
        }
    }
}

/// Test TTFB measurement precision and consistency
#[tokio::test]
async fn test_ttfb_measurement_precision() {
    let framework = TTFBTestFramework::new();
    let urls = vec!["https://httpbin.org/json".to_string()];
    
    let request = framework.create_optimized_request(urls.clone());
    
    // Warm cache
    framework.warm_cache("crawl/stream", request.clone()).await
        .expect("Cache warming should succeed");
    
    // Take multiple measurements
    let measurements = 5;
    let mut ttfb_values = Vec::new();
    
    for i in 0..measurements {
        let measurement = framework.measure_ttfb("crawl/stream", request.clone()).await
            .expect("TTFB measurement should succeed");
        
        ttfb_values.push(measurement.ttfb.as_millis());
        println!("Measurement {}: TTFB {}ms", i + 1, measurement.ttfb.as_millis());
        
        // Small delay between measurements
        sleep(Duration::from_millis(50)).await;
    }
    
    // Calculate statistics
    let avg = ttfb_values.iter().sum::<u128>() as f64 / measurements as f64;
    let min = *ttfb_values.iter().min().unwrap();
    let max = *ttfb_values.iter().max().unwrap();
    let variance = ttfb_values.iter()
        .map(|&x| (x as f64 - avg).powi(2))
        .sum::<f64>() / measurements as f64;
    let std_dev = variance.sqrt();
    
    println!("TTFB Statistics: avg={:.2}ms, min={}ms, max={}ms, stddev={:.2}ms",
             avg, min, max, std_dev);
    
    // Measurements should be consistent (low variance)
    assert!(std_dev < avg * 0.3, "TTFB measurements should be consistent (stddev < 30% of mean)");
    
    // All measurements should meet requirement
    assert!(max < 600, "All TTFB measurements should be reasonable");
}

/// Benchmark TTFB performance under various conditions
#[tokio::test]
async fn test_ttfb_performance_benchmark() {
    let framework = TTFBTestFramework::new();
    
    let benchmark_scenarios = vec![
        (vec!["https://httpbin.org/json"], "single_url", 500),
        (vec![
            "https://httpbin.org/json",
            "https://httpbin.org/headers",
        ], "dual_url", 600),
        (vec![
            "https://httpbin.org/json",
            "https://httpbin.org/headers",
            "https://httpbin.org/user-agent",
            "https://httpbin.org/ip",
        ], "quad_url", 800),
    ];
    
    println!("\nTTFB Performance Benchmark Results:");
    println!("{:<15} {:<10} {:<10} {:<10} {:<10}", "Scenario", "Cold (ms)", "Warm (ms)", "Target", "Status");
    println!("{:-<60}", "");
    
    for (urls, scenario_name, target_ms) in benchmark_scenarios {
        let request = framework.create_optimized_request(urls);
        
        // Measure cold cache
        let cold_measurement = framework.measure_ttfb("crawl/stream", request.clone()).await
            .expect("Cold measurement should succeed");
        
        // Warm cache
        framework.warm_cache("crawl/stream", request.clone()).await
            .expect("Cache warming should succeed");
        
        // Measure warm cache
        let warm_measurement = framework.measure_ttfb("crawl/stream", request).await
            .expect("Warm measurement should succeed");
        
        let cold_ms = cold_measurement.ttfb.as_millis();
        let warm_ms = warm_measurement.ttfb.as_millis();
        let status = if warm_ms <= target_ms as u128 { "PASS" } else { "FAIL" };
        
        println!("{:<15} {:<10} {:<10} {:<10} {:<10}",
                 scenario_name, cold_ms, warm_ms, target_ms, status);
        
        // Warm cache should meet target
        assert!(warm_ms <= target_ms as u128,
                "Scenario {} should meet target {}ms, got {}ms",
                scenario_name, target_ms, warm_ms);
    }
}
