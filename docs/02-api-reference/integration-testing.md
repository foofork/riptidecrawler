# Integration Testing Guide

## Overview

This guide provides comprehensive integration testing strategies for the riptide system, covering all components from the browser pool and WASM modules to streaming endpoints and API services.

## Table of Contents

1. [Integration Testing Strategy](#integration-testing-strategy)
2. [Test Harness Setup](#test-harness-setup)
3. [Component Integration Tests](#component-integration-tests)
4. [End-to-End Test Scenarios](#end-to-end-test-scenarios)
5. [Performance Testing](#performance-testing)
6. [Load Testing with Streaming](#load-testing-with-streaming)
7. [Browser Pool Testing](#browser-pool-testing)
8. [WASM Component Testing](#wasm-component-testing)
9. [CI/CD Integration](#cicd-integration)
10. [Test Data Management](#test-data-management)

## Integration Testing Strategy

### 1. Testing Pyramid for riptide

```
         /\
        /E2E\      <- Full pipeline tests (5%)
       /------\
      /Service \ <- Component integration (25%)
     /----------\
    /   Unit     \ <- Unit tests (70%)
   /--------------\
```

### 2. Component Architecture Testing

```rust
// Test configuration for component integration
#[cfg(test)]
pub struct IntegrationTestConfig {
    pub redis_url: String,
    pub browser_pool_size: usize,
    pub wasm_module_path: String,
    pub test_db_path: String,
    pub api_port: u16,
    pub streaming_port: u16,
}

impl Default for IntegrationTestConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379/15".to_string(), // Test DB
            browser_pool_size: 2,
            wasm_module_path: "./target/wasm32-unknown-unknown/debug/riptide_extractor_wasm.wasm".to_string(),
            test_db_path: "./test_data.db".to_string(),
            api_port: 8081,
            streaming_port: 8082,
        }
    }
}
```

### 3. Integration Test Categories

- **Component Integration**: Testing interactions between core modules
- **Service Integration**: Testing API endpoints with backend services
- **Data Flow Integration**: End-to-end data processing pipeline
- **External Integration**: Third-party services and dependencies
- **Performance Integration**: System behavior under load

## Test Harness Setup

### 1. Test Environment Configuration

```rust
// tests/common/mod.rs
use std::sync::Once;
use tokio::runtime::Runtime;
use riptide_core::{memory_manager::MemoryManager, cache::Cache};
use riptide_api::config::Config;
use riptide_headless::pool::BrowserPool;

static INIT: Once = Once::new();

pub struct TestHarness {
    pub config: IntegrationTestConfig,
    pub runtime: Runtime,
    pub memory_manager: MemoryManager,
    pub browser_pool: BrowserPool,
    pub cache: Cache,
}

impl TestHarness {
    pub async fn new() -> Self {
        INIT.call_once(|| {
            env_logger::init();
        });

        let config = IntegrationTestConfig::default();
        let runtime = Runtime::new().expect("Failed to create runtime");

        let memory_manager = MemoryManager::new().await
            .expect("Failed to initialize memory manager");

        let browser_pool = BrowserPool::new(config.browser_pool_size).await
            .expect("Failed to create browser pool");

        let cache = Cache::new(&config.redis_url).await
            .expect("Failed to connect to Redis");

        Self {
            config,
            runtime,
            memory_manager,
            browser_pool,
            cache,
        }
    }

    pub async fn cleanup(&mut self) {
        // Clean up test resources
        self.cache.flush_all().await.ok();
        self.browser_pool.shutdown().await.ok();
        self.memory_manager.clear().await.ok();

        // Remove test files
        std::fs::remove_file(&self.config.test_db_path).ok();
    }
}
```

### 2. Test Fixtures and Mocks

```rust
// tests/fixtures/mod.rs
use serde_json::json;
use reqwest::Response;
use mockito::{Matcher, Mock};

pub struct TestFixtures;

impl TestFixtures {
    pub fn sample_urls() -> Vec<String> {
        vec![
            "https://example.com".to_string(),
            "https://httpbin.org/json".to_string(),
            "https://jsonplaceholder.typicode.com/posts/1".to_string(),
        ]
    }

    pub fn sample_extraction_config() -> serde_json::Value {
        json!({
            "strategy": "comprehensive",
            "selectors": {
                "title": "title",
                "content": "p",
                "links": "a[href]"
            },
            "extract_images": true,
            "extract_metadata": true
        })
    }

    pub fn mock_external_api() -> Mock {
        mockito::mock("GET", "/api/data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"status": "success", "data": "test"}"#)
            .create()
    }

    pub fn sample_batch_request() -> serde_json::Value {
        json!({
            "urls": Self::sample_urls(),
            "config": Self::sample_extraction_config(),
            "options": {
                "concurrent_limit": 2,
                "timeout_ms": 30000,
                "retry_attempts": 3
            }
        })
    }
}
```

### 3. Test Database Setup

```rust
// tests/database/mod.rs
use sqlx::{Pool, Sqlite, SqlitePool};
use std::path::Path;

pub struct TestDatabase {
    pool: SqlitePool,
    db_path: String,
}

impl TestDatabase {
    pub async fn new(db_path: &str) -> Result<Self, sqlx::Error> {
        if Path::new(db_path).exists() {
            std::fs::remove_file(db_path).ok();
        }

        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path)).await?;

        // Initialize test schema
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS extraction_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                content TEXT,
                metadata TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        "#)
        .execute(&pool)
        .await?;

        Ok(Self {
            pool,
            db_path: db_path.to_string(),
        })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn cleanup(&self) -> Result<(), sqlx::Error> {
        self.pool.close().await;
        std::fs::remove_file(&self.db_path).ok();
        Ok(())
    }
}
```

## Component Integration Tests

### 1. Core Module Integration

```rust
// tests/integration/core_integration_test.rs
use tokio_test;
use riptide_core::{extract::Extractor, fetch::Fetcher, cache::Cache};
use crate::common::TestHarness;

#[tokio::test]
async fn test_fetch_extract_cache_pipeline() {
    let mut harness = TestHarness::new().await;

    let url = "https://example.com";
    let fetcher = Fetcher::new();
    let extractor = Extractor::new();

    // Test fetch -> extract -> cache pipeline
    let fetch_result = fetcher.fetch(url).await
        .expect("Failed to fetch URL");

    assert!(!fetch_result.content.is_empty());
    assert_eq!(fetch_result.status_code, 200);

    let extract_result = extractor.extract(&fetch_result.content).await
        .expect("Failed to extract content");

    assert!(!extract_result.title.is_empty());

    // Cache the result
    let cache_key = format!("test:{}", url);
    harness.cache.set(&cache_key, &extract_result, 3600).await
        .expect("Failed to cache result");

    // Verify cache retrieval
    let cached_result: ExtractionResult = harness.cache.get(&cache_key).await
        .expect("Failed to get cached result")
        .expect("Cache result not found");

    assert_eq!(cached_result.title, extract_result.title);

    harness.cleanup().await;
}
```

### 2. API Service Integration

```rust
// tests/integration/api_integration_test.rs
use reqwest::Client;
use tokio::task;
use riptide_api::handlers::ApiServer;
use crate::common::TestHarness;
use crate::fixtures::TestFixtures;

#[tokio::test]
async fn test_api_service_integration() {
    let harness = TestHarness::new().await;
    let client = Client::new();

    // Start API server
    let server_handle = task::spawn(async move {
        let server = ApiServer::new(harness.config.api_port).await
            .expect("Failed to create API server");
        server.run().await
    });

    // Wait for server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    let base_url = format!("http://localhost:{}", harness.config.api_port);

    // Test health endpoint
    let health_response = client
        .get(&format!("{}/health", base_url))
        .send()
        .await
        .expect("Health check failed");

    assert_eq!(health_response.status(), 200);

    // Test extraction endpoint
    let extract_request = TestFixtures::sample_batch_request();
    let extract_response = client
        .post(&format!("{}/extract", base_url))
        .json(&extract_request)
        .send()
        .await
        .expect("Extract request failed");

    assert_eq!(extract_response.status(), 200);

    let extract_result: serde_json::Value = extract_response
        .json()
        .await
        .expect("Failed to parse extract response");

    assert!(extract_result["results"].is_array());
    assert!(!extract_result["results"].as_array().unwrap().is_empty());

    server_handle.abort();
}
```

### 3. Browser Pool Integration

```rust
// tests/integration/browser_pool_integration_test.rs
use riptide_headless::pool::{BrowserPool, BrowserTask};
use crate::common::TestHarness;

#[tokio::test]
async fn test_browser_pool_task_execution() {
    let harness = TestHarness::new().await;

    // Create multiple browser tasks
    let tasks = vec![
        BrowserTask::new("https://example.com", None),
        BrowserTask::new("https://httpbin.org/json", None),
        BrowserTask::new("https://jsonplaceholder.typicode.com/posts/1", None),
    ];

    // Execute tasks concurrently
    let results = harness.browser_pool
        .execute_batch(tasks)
        .await
        .expect("Failed to execute browser tasks");

    assert_eq!(results.len(), 3);

    for result in results {
        assert!(result.is_ok());
        let page_result = result.unwrap();
        assert!(!page_result.html.is_empty());
        assert!(page_result.success);
    }
}
```

## End-to-End Test Scenarios

### 1. Complete Extraction Pipeline

```rust
// tests/e2e/extraction_pipeline_test.rs
use reqwest::Client;
use serde_json::json;
use crate::common::TestHarness;

#[tokio::test]
async fn test_complete_extraction_pipeline() {
    let harness = TestHarness::new().await;
    let client = Client::new();

    // Start full system
    let _server = start_full_system(&harness).await;

    let pipeline_request = json!({
        "pipeline": "comprehensive",
        "urls": [
            "https://example.com",
            "https://httpbin.org/json"
        ],
        "options": {
            "use_browser": true,
            "extract_content": true,
            "extract_metadata": true,
            "store_results": true
        }
    });

    // Submit pipeline request
    let response = client
        .post("http://localhost:8081/pipeline/run")
        .json(&pipeline_request)
        .send()
        .await
        .expect("Pipeline request failed");

    assert_eq!(response.status(), 202); // Accepted

    let pipeline_response: serde_json::Value = response.json().await
        .expect("Failed to parse pipeline response");

    let pipeline_id = pipeline_response["pipeline_id"].as_str()
        .expect("Missing pipeline_id");

    // Poll for completion
    let mut attempts = 0;
    let max_attempts = 30;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let status_response = client
            .get(&format!("http://localhost:8081/pipeline/status/{}", pipeline_id))
            .send()
            .await
            .expect("Status request failed");

        let status: serde_json::Value = status_response.json().await
            .expect("Failed to parse status");

        match status["status"].as_str().unwrap() {
            "completed" => {
                // Verify results
                let results = status["results"].as_array().expect("Missing results");
                assert_eq!(results.len(), 2);

                for result in results {
                    assert!(result["success"].as_bool().unwrap());
                    assert!(!result["content"].as_str().unwrap().is_empty());
                }
                break;
            }
            "failed" => {
                panic!("Pipeline failed: {:?}", status["error"]);
            }
            _ => {
                attempts += 1;
                if attempts >= max_attempts {
                    panic!("Pipeline timeout after {} attempts", max_attempts);
                }
            }
        }
    }
}
```

### 2. Streaming Pipeline Test

```rust
// tests/e2e/streaming_pipeline_test.rs
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

#[tokio::test]
async fn test_streaming_extraction_pipeline() {
    let harness = TestHarness::new().await;
    let _server = start_streaming_server(&harness).await;

    let ws_url = "ws://localhost:8082/stream/extract";
    let (ws_stream, _) = connect_async(ws_url).await
        .expect("Failed to connect to WebSocket");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Send streaming extraction request
    let stream_request = json!({
        "type": "extract",
        "urls": [
            "https://example.com",
            "https://httpbin.org/json",
            "https://jsonplaceholder.typicode.com/posts/1"
        ],
        "config": {
            "stream_results": true,
            "batch_size": 1
        }
    });

    ws_sender.send(Message::Text(stream_request.to_string())).await
        .expect("Failed to send request");

    let mut results_received = 0;
    let expected_results = 3;

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let result: serde_json::Value = serde_json::from_str(&text)
                    .expect("Failed to parse message");

                match result["type"].as_str().unwrap() {
                    "result" => {
                        results_received += 1;
                        assert!(result["success"].as_bool().unwrap());
                        assert!(!result["data"]["content"].as_str().unwrap().is_empty());
                    }
                    "complete" => {
                        assert_eq!(results_received, expected_results);
                        break;
                    }
                    "error" => {
                        panic!("Streaming error: {:?}", result["error"]);
                    }
                    _ => {}
                }
            }
            Ok(Message::Close(_)) => break,
            Err(e) => panic!("WebSocket error: {}", e),
            _ => {}
        }
    }

    assert_eq!(results_received, expected_results);
}
```

## Performance Testing

### 1. Load Testing Framework

```rust
// tests/performance/load_test.rs
use std::time::{Duration, Instant};
use tokio::task::JoinSet;
use reqwest::Client;
use serde_json::json;

pub struct LoadTestConfig {
    pub concurrent_users: usize,
    pub requests_per_user: usize,
    pub ramp_up_duration: Duration,
    pub test_duration: Duration,
}

pub struct LoadTestResults {
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub avg_response_time: Duration,
    pub p95_response_time: Duration,
    pub p99_response_time: Duration,
    pub requests_per_second: f64,
}

#[tokio::test]
async fn test_api_load_performance() {
    let config = LoadTestConfig {
        concurrent_users: 50,
        requests_per_user: 20,
        ramp_up_duration: Duration::from_secs(10),
        test_duration: Duration::from_secs(60),
    };

    let harness = TestHarness::new().await;
    let _server = start_api_server(&harness).await;

    let results = run_load_test(config).await;

    // Performance assertions
    assert!(results.requests_per_second >= 100.0);
    assert!(results.avg_response_time <= Duration::from_millis(500));
    assert!(results.p95_response_time <= Duration::from_secs(2));
    assert!(results.successful_requests as f64 / results.total_requests as f64 >= 0.95);

    println!("Load test results: {:#?}", results);
}

async fn run_load_test(config: LoadTestConfig) -> LoadTestResults {
    let client = Client::new();
    let mut join_set = JoinSet::new();
    let mut response_times = Vec::new();
    let test_start = Instant::now();

    // Spawn concurrent users
    for user_id in 0..config.concurrent_users {
        let client = client.clone();
        let user_requests = config.requests_per_user;

        join_set.spawn(async move {
            let mut user_response_times = Vec::new();

            for request_id in 0..user_requests {
                let start = Instant::now();

                let result = client
                    .post("http://localhost:8081/extract")
                    .json(&json!({
                        "url": format!("https://example.com?user={}&req={}", user_id, request_id),
                        "config": {"strategy": "basic"}
                    }))
                    .send()
                    .await;

                let duration = start.elapsed();

                match result {
                    Ok(response) => {
                        if response.status().is_success() {
                            user_response_times.push((duration, true));
                        } else {
                            user_response_times.push((duration, false));
                        }
                    }
                    Err(_) => {
                        user_response_times.push((duration, false));
                    }
                }

                // Simulate user think time
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            user_response_times
        });

        // Ramp up delay
        if user_id > 0 {
            tokio::time::sleep(config.ramp_up_duration / config.concurrent_users as u32).await;
        }
    }

    // Collect results
    let mut total_requests = 0;
    let mut successful_requests = 0;
    let mut failed_requests = 0;

    while let Some(user_results) = join_set.join_next().await {
        if let Ok(results) = user_results {
            for (duration, success) in results {
                response_times.push(duration);
                total_requests += 1;
                if success {
                    successful_requests += 1;
                } else {
                    failed_requests += 1;
                }
            }
        }
    }

    // Calculate statistics
    response_times.sort();
    let avg_response_time = response_times.iter().sum::<Duration>() / response_times.len() as u32;
    let p95_index = (response_times.len() as f64 * 0.95) as usize;
    let p99_index = (response_times.len() as f64 * 0.99) as usize;
    let p95_response_time = response_times[p95_index.min(response_times.len() - 1)];
    let p99_response_time = response_times[p99_index.min(response_times.len() - 1)];

    let test_duration = test_start.elapsed();
    let requests_per_second = total_requests as f64 / test_duration.as_secs_f64();

    LoadTestResults {
        total_requests,
        successful_requests,
        failed_requests,
        avg_response_time,
        p95_response_time,
        p99_response_time,
        requests_per_second,
    }
}
```

### 2. Memory and Resource Testing

```rust
// tests/performance/resource_test.rs
use sysinfo::{System, SystemExt, ProcessExt};
use std::process;

#[tokio::test]
async fn test_memory_usage_under_load() {
    let mut system = System::new_all();
    let pid = process::id();

    let harness = TestHarness::new().await;
    let _server = start_full_system(&harness).await;

    // Baseline memory measurement
    system.refresh_all();
    let process = system.process(pid as i32).expect("Process not found");
    let baseline_memory = process.memory();

    // Execute memory-intensive operations
    let load_config = LoadTestConfig {
        concurrent_users: 100,
        requests_per_user: 50,
        ramp_up_duration: Duration::from_secs(5),
        test_duration: Duration::from_secs(120),
    };

    let _results = run_load_test(load_config).await;

    // Measure memory after load test
    system.refresh_all();
    let process = system.process(pid as i32).expect("Process not found");
    let peak_memory = process.memory();

    // Allow for garbage collection
    tokio::time::sleep(Duration::from_secs(10)).await;
    system.refresh_all();
    let process = system.process(pid as i32).expect("Process not found");
    let final_memory = process.memory();

    // Memory usage assertions
    let memory_increase = peak_memory - baseline_memory;
    let memory_retained = final_memory - baseline_memory;

    println!("Baseline memory: {} MB", baseline_memory / (1024 * 1024));
    println!("Peak memory: {} MB", peak_memory / (1024 * 1024));
    println!("Final memory: {} MB", final_memory / (1024 * 1024));
    println!("Memory increase: {} MB", memory_increase / (1024 * 1024));
    println!("Memory retained: {} MB", memory_retained / (1024 * 1024));

    // Assert reasonable memory usage
    assert!(memory_increase < 500 * 1024 * 1024); // Less than 500MB increase
    assert!(memory_retained < 100 * 1024 * 1024); // Less than 100MB retained
}
```

## Load Testing with Streaming

### 1. WebSocket Streaming Load Test

```rust
// tests/performance/streaming_load_test.rs
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn test_streaming_endpoint_load() {
    let harness = TestHarness::new().await;
    let _server = start_streaming_server(&harness).await;

    let concurrent_connections = 50;
    let messages_per_connection = 20;
    let total_messages_expected = concurrent_connections * messages_per_connection * 3; // 3 URLs per message

    let messages_received = Arc::new(AtomicUsize::new(0));
    let connections_completed = Arc::new(AtomicUsize::new(0));

    let mut join_set = JoinSet::new();

    // Create concurrent WebSocket connections
    for connection_id in 0..concurrent_connections {
        let messages_received = Arc::clone(&messages_received);
        let connections_completed = Arc::clone(&connections_completed);

        join_set.spawn(async move {
            let ws_url = "ws://localhost:8082/stream/extract";
            let (ws_stream, _) = connect_async(ws_url).await
                .expect("Failed to connect to WebSocket");

            let (mut ws_sender, mut ws_receiver) = ws_stream.split();

            // Send multiple streaming requests
            for message_id in 0..messages_per_connection {
                let stream_request = json!({
                    "type": "extract",
                    "connection_id": connection_id,
                    "message_id": message_id,
                    "urls": [
                        format!("https://example.com?conn={}&msg={}&url=1", connection_id, message_id),
                        format!("https://httpbin.org/json?conn={}&msg={}&url=2", connection_id, message_id),
                        format!("https://jsonplaceholder.typicode.com/posts/{}?conn={}&msg={}",
                               (message_id % 10) + 1, connection_id, message_id)
                    ],
                    "config": {
                        "stream_results": true,
                        "batch_size": 1
                    }
                });

                ws_sender.send(Message::Text(stream_request.to_string())).await
                    .expect("Failed to send request");
            }

            // Receive responses
            let mut local_messages_received = 0;

            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        let result: serde_json::Value = serde_json::from_str(&text)
                            .expect("Failed to parse message");

                        match result["type"].as_str().unwrap() {
                            "result" => {
                                local_messages_received += 1;
                                messages_received.fetch_add(1, Ordering::Relaxed);
                            }
                            "complete" => {
                                if local_messages_received >= messages_per_connection * 3 {
                                    break;
                                }
                            }
                            "error" => {
                                eprintln!("Streaming error: {:?}", result["error"]);
                            }
                            _ => {}
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Err(e) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }

            connections_completed.fetch_add(1, Ordering::Relaxed);
            local_messages_received
        });
    }

    // Wait for all connections to complete
    let mut total_received = 0;
    while let Some(result) = join_set.join_next().await {
        if let Ok(count) = result {
            total_received += count;
        }
    }

    let final_messages_received = messages_received.load(Ordering::Relaxed);
    let final_connections_completed = connections_completed.load(Ordering::Relaxed);

    println!("Total messages expected: {}", total_messages_expected);
    println!("Total messages received: {}", final_messages_received);
    println!("Connections completed: {}", final_connections_completed);
    println!("Messages per connection avg: {}", total_received / concurrent_connections);

    // Performance assertions
    assert_eq!(final_connections_completed, concurrent_connections);
    assert!(final_messages_received >= total_messages_expected * 80 / 100); // At least 80% success rate
}
```

### 2. Streaming Throughput Test

```rust
// tests/performance/streaming_throughput_test.rs
#[tokio::test]
async fn test_streaming_throughput() {
    let harness = TestHarness::new().await;
    let _server = start_streaming_server(&harness).await;

    let test_duration = Duration::from_secs(60);
    let start_time = Instant::now();

    let throughput_counter = Arc::new(AtomicUsize::new(0));
    let error_counter = Arc::new(AtomicUsize::new(0));

    let mut join_set = JoinSet::new();

    // Spawn continuous streaming tasks
    for _worker_id in 0..10 {
        let throughput_counter = Arc::clone(&throughput_counter);
        let error_counter = Arc::clone(&error_counter);
        let test_end_time = start_time + test_duration;

        join_set.spawn(async move {
            while Instant::now() < test_end_time {
                match perform_streaming_request().await {
                    Ok(count) => {
                        throughput_counter.fetch_add(count, Ordering::Relaxed);
                    }
                    Err(_) => {
                        error_counter.fetch_add(1, Ordering::Relaxed);
                    }
                }

                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
    }

    // Wait for test duration
    tokio::time::sleep(test_duration).await;

    // Stop all workers
    join_set.abort_all();

    let total_throughput = throughput_counter.load(Ordering::Relaxed);
    let total_errors = error_counter.load(Ordering::Relaxed);
    let elapsed = start_time.elapsed();

    let throughput_per_second = total_throughput as f64 / elapsed.as_secs_f64();
    let error_rate = total_errors as f64 / (total_throughput + total_errors) as f64;

    println!("Streaming throughput: {:.2} results/second", throughput_per_second);
    println!("Error rate: {:.2}%", error_rate * 100.0);

    // Throughput assertions
    assert!(throughput_per_second >= 50.0); // At least 50 results per second
    assert!(error_rate <= 0.05); // Less than 5% error rate
}

async fn perform_streaming_request() -> Result<usize, Box<dyn std::error::Error>> {
    let ws_url = "ws://localhost:8082/stream/extract";
    let (ws_stream, _) = connect_async(ws_url).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let request = json!({
        "type": "extract",
        "urls": ["https://httpbin.org/json"],
        "config": {"stream_results": true}
    });

    ws_sender.send(Message::Text(request.to_string())).await?;

    let mut count = 0;
    while let Some(msg) = ws_receiver.next().await {
        match msg? {
            Message::Text(text) => {
                let result: serde_json::Value = serde_json::from_str(&text)?;
                match result["type"].as_str().unwrap() {
                    "result" => count += 1,
                    "complete" => break,
                    "error" => return Err("Streaming error".into()),
                    _ => {}
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    Ok(count)
}
```

## Browser Pool Testing

### 1. Browser Pool Stress Test

```rust
// tests/integration/browser_pool_stress_test.rs
use riptide_headless::pool::{BrowserPool, BrowserTask, PoolMetrics};

#[tokio::test]
async fn test_browser_pool_stress_concurrent_tasks() {
    let pool_size = 5;
    let total_tasks = 100;
    let concurrent_batches = 10;

    let browser_pool = BrowserPool::new(pool_size).await
        .expect("Failed to create browser pool");

    let start_time = Instant::now();
    let mut join_set = JoinSet::new();

    // Create concurrent task batches
    for batch_id in 0..concurrent_batches {
        let pool = browser_pool.clone();

        join_set.spawn(async move {
            let mut tasks = Vec::new();

            for task_id in 0..(total_tasks / concurrent_batches) {
                let url = format!("https://httpbin.org/delay/1?batch={}&task={}", batch_id, task_id);
                tasks.push(BrowserTask::new(&url, None));
            }

            let results = pool.execute_batch(tasks).await
                .expect("Failed to execute batch");

            let mut successful = 0;
            for result in results {
                if result.is_ok() {
                    successful += 1;
                }
            }

            (batch_id, successful, total_tasks / concurrent_batches)
        });
    }

    // Collect results
    let mut total_successful = 0;
    let mut total_attempted = 0;

    while let Some(result) = join_set.join_next().await {
        if let Ok((batch_id, successful, attempted)) = result {
            println!("Batch {}: {}/{} successful", batch_id, successful, attempted);
            total_successful += successful;
            total_attempted += attempted;
        }
    }

    let execution_time = start_time.elapsed();
    let success_rate = total_successful as f64 / total_attempted as f64;

    println!("Browser pool stress test results:");
    println!("Total tasks: {}", total_attempted);
    println!("Successful tasks: {}", total_successful);
    println!("Success rate: {:.2}%", success_rate * 100.0);
    println!("Execution time: {:?}", execution_time);
    println!("Tasks per second: {:.2}", total_attempted as f64 / execution_time.as_secs_f64());

    // Get pool metrics
    let metrics = browser_pool.get_metrics().await;
    println!("Pool metrics: {:?}", metrics);

    // Performance assertions
    assert!(success_rate >= 0.95); // At least 95% success rate
    assert!(execution_time <= Duration::from_secs(60)); // Complete within 60 seconds

    browser_pool.shutdown().await.expect("Failed to shutdown browser pool");
}
```

### 2. Browser Pool Resource Management Test

```rust
// tests/integration/browser_pool_resources_test.rs
#[tokio::test]
async fn test_browser_pool_resource_management() {
    let pool_size = 3;
    let browser_pool = BrowserPool::new(pool_size).await
        .expect("Failed to create browser pool");

    // Test pool scaling under load
    let initial_metrics = browser_pool.get_metrics().await;
    assert_eq!(initial_metrics.active_browsers, 0);
    assert_eq!(initial_metrics.total_capacity, pool_size);

    // Create more tasks than pool capacity
    let excessive_tasks = pool_size * 3;
    let mut tasks = Vec::new();

    for i in 0..excessive_tasks {
        let url = format!("https://httpbin.org/delay/2?task={}", i);
        tasks.push(BrowserTask::new(&url, None));
    }

    let start_time = Instant::now();

    // Execute tasks (should queue some due to pool size limit)
    let results = browser_pool.execute_batch(tasks).await
        .expect("Failed to execute excessive tasks");

    let execution_time = start_time.elapsed();

    // Verify all tasks completed successfully
    let successful_tasks = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(successful_tasks, excessive_tasks);

    // Check that execution time reflects queuing (should be longer than single batch)
    assert!(execution_time >= Duration::from_secs(4)); // At least 2 rounds of 2-second delays

    // Verify pool metrics after execution
    let final_metrics = browser_pool.get_metrics().await;
    println!("Final pool metrics: {:?}", final_metrics);

    // Test pool cleanup
    browser_pool.shutdown().await.expect("Failed to shutdown browser pool");

    // Verify browsers are cleaned up (this would require pool implementation to expose cleanup status)
}
```

## WASM Component Testing

### 1. WASM Module Integration Test

```rust
// tests/integration/wasm_integration_test.rs
use wasmtime::{Engine, Module, Store, Instance, Linker};
use wasmtime_wasi::WasiCtxBuilder;

#[tokio::test]
async fn test_wasm_extractor_integration() {
    let harness = TestHarness::new().await;

    // Initialize WASM runtime
    let engine = Engine::default();
    let module = Module::from_file(&engine, &harness.config.wasm_module_path)
        .expect("Failed to load WASM module");

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)
        .expect("Failed to add WASI to linker");

    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .build();

    let mut store = Store::new(&engine, wasi);
    let instance = linker.instantiate(&mut store, &module)
        .expect("Failed to instantiate WASM module");

    // Test WASM extraction function
    let extract_func = instance.get_typed_func::<(i32, i32), i32>(&mut store, "extract")
        .expect("Failed to get extract function");

    // Prepare test HTML content
    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Test Page</title></head>
        <body>
            <h1>Main Heading</h1>
            <p>This is test content for WASM extraction.</p>
            <a href="https://example.com">Example Link</a>
        </body>
        </html>
    "#;

    // Allocate memory for HTML content in WASM
    let memory = instance.get_memory(&mut store, "memory")
        .expect("Failed to get WASM memory");

    let html_bytes = html_content.as_bytes();
    let html_ptr = allocate_in_wasm(&mut store, &instance, html_bytes.len() as i32)
        .expect("Failed to allocate memory in WASM");

    memory.write(&mut store, html_ptr as usize, html_bytes)
        .expect("Failed to write HTML to WASM memory");

    // Call WASM extraction function
    let result_ptr = extract_func.call(&mut store, (html_ptr, html_bytes.len() as i32))
        .expect("WASM extraction failed");

    // Read extraction result from WASM memory
    let result_json = read_string_from_wasm(&mut store, &memory, result_ptr)
        .expect("Failed to read result from WASM");

    let extraction_result: serde_json::Value = serde_json::from_str(&result_json)
        .expect("Failed to parse extraction result");

    // Verify extraction results
    assert_eq!(extraction_result["title"].as_str().unwrap(), "Test Page");
    assert!(extraction_result["content"].as_str().unwrap().contains("test content"));
    assert!(extraction_result["links"].as_array().unwrap().len() > 0);

    println!("WASM extraction result: {}", serde_json::to_string_pretty(&extraction_result).unwrap());
}

fn allocate_in_wasm(store: &mut Store<WasiCtx>, instance: &Instance, size: i32) -> Result<i32, Box<dyn std::error::Error>> {
    let alloc_func = instance.get_typed_func::<i32, i32>(store, "alloc")?;
    let ptr = alloc_func.call(store, size)?;
    Ok(ptr)
}

fn read_string_from_wasm(store: &mut Store<WasiCtx>, memory: &wasmtime::Memory, ptr: i32) -> Result<String, Box<dyn std::error::Error>> {
    // Read length first (assuming first 4 bytes contain string length)
    let mut len_bytes = [0u8; 4];
    memory.read(store, ptr as usize, &mut len_bytes)?;
    let len = u32::from_le_bytes(len_bytes) as usize;

    // Read string content
    let mut string_bytes = vec![0u8; len];
    memory.read(store, (ptr + 4) as usize, &mut string_bytes)?;

    Ok(String::from_utf8(string_bytes)?)
}
```

### 2. WASM Performance Test

```rust
// tests/performance/wasm_performance_test.rs
#[tokio::test]
async fn test_wasm_extraction_performance() {
    let harness = TestHarness::new().await;

    // Load test HTML files of various sizes
    let test_files = vec![
        ("small.html", generate_html_content(1_000)),     // 1KB
        ("medium.html", generate_html_content(10_000)),   // 10KB
        ("large.html", generate_html_content(100_000)),   // 100KB
        ("huge.html", generate_html_content(1_000_000)),  // 1MB
    ];

    let mut performance_results = Vec::new();

    for (file_name, html_content) in test_files {
        let start_time = Instant::now();
        let iterations = 100;

        // Initialize WASM runtime
        let engine = Engine::default();
        let module = Module::from_file(&engine, &harness.config.wasm_module_path)
            .expect("Failed to load WASM module");

        let mut total_extraction_time = Duration::new(0, 0);

        for _iteration in 0..iterations {
            let mut linker = Linker::new(&engine);
            wasmtime_wasi::add_to_linker(&mut linker, |s| s)
                .expect("Failed to add WASI to linker");

            let wasi = WasiCtxBuilder::new().build();
            let mut store = Store::new(&engine, wasi);
            let instance = linker.instantiate(&mut store, &module)
                .expect("Failed to instantiate WASM module");

            let extract_func = instance.get_typed_func::<(i32, i32), i32>(&mut store, "extract")
                .expect("Failed to get extract function");

            let memory = instance.get_memory(&mut store, "memory")
                .expect("Failed to get WASM memory");

            // Measure extraction time only
            let extraction_start = Instant::now();

            let html_bytes = html_content.as_bytes();
            let html_ptr = allocate_in_wasm(&mut store, &instance, html_bytes.len() as i32)
                .expect("Failed to allocate memory in WASM");

            memory.write(&mut store, html_ptr as usize, html_bytes)
                .expect("Failed to write HTML to WASM memory");

            let _result_ptr = extract_func.call(&mut store, (html_ptr, html_bytes.len() as i32))
                .expect("WASM extraction failed");

            total_extraction_time += extraction_start.elapsed();
        }

        let total_time = start_time.elapsed();
        let avg_extraction_time = total_extraction_time / iterations;
        let extractions_per_second = iterations as f64 / total_time.as_secs_f64();

        let result = WasmPerformanceResult {
            file_name: file_name.to_string(),
            content_size: html_content.len(),
            iterations,
            total_time,
            avg_extraction_time,
            extractions_per_second,
        };

        performance_results.push(result);
    }

    // Print and verify performance results
    println!("WASM Extraction Performance Results:");
    println!("{:<15} {:>10} {:>12} {:>15} {:>15}", "File", "Size (B)", "Avg Time (ms)", "Extractions/s", "Throughput MB/s");
    println!("{:-<75}", "");

    for result in &performance_results {
        let throughput_mb_per_s = (result.content_size as f64 * result.extractions_per_second) / (1024.0 * 1024.0);

        println!("{:<15} {:>10} {:>12.2} {:>15.2} {:>15.2}",
            result.file_name,
            result.content_size,
            result.avg_extraction_time.as_millis(),
            result.extractions_per_second,
            throughput_mb_per_s
        );

        // Performance assertions based on content size
        match result.content_size {
            size if size <= 10_000 => {
                assert!(result.avg_extraction_time <= Duration::from_millis(10));
                assert!(result.extractions_per_second >= 50.0);
            }
            size if size <= 100_000 => {
                assert!(result.avg_extraction_time <= Duration::from_millis(50));
                assert!(result.extractions_per_second >= 10.0);
            }
            _ => {
                assert!(result.avg_extraction_time <= Duration::from_millis(200));
                assert!(result.extractions_per_second >= 2.0);
            }
        }
    }
}

#[derive(Debug)]
struct WasmPerformanceResult {
    file_name: String,
    content_size: usize,
    iterations: u32,
    total_time: Duration,
    avg_extraction_time: Duration,
    extractions_per_second: f64,
}

fn generate_html_content(target_size: usize) -> String {
    let base_content = r#"<!DOCTYPE html><html><head><title>Performance Test Page</title></head><body><h1>Test Content</h1><p>This is test content for performance evaluation. "#;
    let repeated_content = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ";
    let closing_content = "</p></body></html>";

    let mut html = String::new();
    html.push_str(base_content);

    let remaining_size = target_size.saturating_sub(base_content.len() + closing_content.len());
    let repeats = remaining_size / repeated_content.len();

    for _ in 0..repeats {
        html.push_str(repeated_content);
    }

    html.push_str(closing_content);
    html
}
```

## CI/CD Integration

### 1. GitHub Actions Workflow

```yaml
# .github/workflows/integration-tests.yml
name: Integration Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  integration-tests:
    runs-on: ubuntu-latest

    services:
      redis:
        image: redis:7
        ports:
          - 6379:6379
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
        target: wasm32-unknown-unknown

    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target/
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
        restore-keys: |
          ${{ runner.os }}-cargo-

    - name: Install Chrome
      run: |
        wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | sudo apt-key add -
        sudo sh -c 'echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" >> /etc/apt/sources.list.d/google-chrome.list'
        sudo apt-get update
        sudo apt-get install -y google-chrome-stable

    - name: Build project
      run: cargo build --verbose

    - name: Build WASM modules
      run: |
        cargo build --target wasm32-unknown-unknown --release -p riptide-extractor-wasm

    - name: Run unit tests
      run: cargo test --verbose --lib

    - name: Run integration tests
      run: cargo test --verbose --test '*integration*'
      env:
        REDIS_URL: redis://localhost:6379/15
        BROWSER_EXECUTABLE: /usr/bin/google-chrome

    - name: Run performance tests
      run: cargo test --verbose --test '*performance*' --release
      env:
        REDIS_URL: redis://localhost:6379/15
        BROWSER_EXECUTABLE: /usr/bin/google-chrome

    - name: Run WASM tests
      run: cargo test --verbose --test '*wasm*'

    - name: Generate test report
      if: always()
      run: |
        cargo test --verbose -- --format json > test-results.json || true

    - name: Upload test results
      uses: actions/upload-artifact@v3
      if: always()
      with:
        name: test-results
        path: test-results.json

    - name: Performance regression check
      run: |
        cargo bench --bench performance_benches -- --output-format json > bench-results.json
        # Add performance regression detection script here

  load-tests:
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'

    services:
      redis:
        image: redis:7
        ports:
          - 6379:6379

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: wasm32-unknown-unknown

    - name: Build release
      run: cargo build --release

    - name: Run load tests
      run: cargo test --release --test '*load*' -- --nocapture
      env:
        REDIS_URL: redis://localhost:6379/15
        LOAD_TEST_DURATION: 300
        LOAD_TEST_USERS: 100

    - name: Archive load test results
      uses: actions/upload-artifact@v3
      with:
        name: load-test-results
        path: load-test-*.json
```

### 2. Test Configuration for Different Environments

```rust
// tests/config/mod.rs
use std::env;

#[derive(Debug, Clone)]
pub struct TestEnvironmentConfig {
    pub environment: TestEnvironment,
    pub redis_url: String,
    pub browser_executable: Option<String>,
    pub api_base_url: String,
    pub load_test_duration: u64,
    pub load_test_concurrency: usize,
    pub performance_thresholds: PerformanceThresholds,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestEnvironment {
    Unit,
    Integration,
    Staging,
    Performance,
    Load,
}

#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub max_response_time_ms: u64,
    pub min_requests_per_second: f64,
    pub max_error_rate_percent: f64,
    pub max_memory_usage_mb: u64,
}

impl TestEnvironmentConfig {
    pub fn from_env() -> Self {
        let environment = match env::var("TEST_ENVIRONMENT").as_deref() {
            Ok("unit") => TestEnvironment::Unit,
            Ok("integration") => TestEnvironment::Integration,
            Ok("staging") => TestEnvironment::Staging,
            Ok("performance") => TestEnvironment::Performance,
            Ok("load") => TestEnvironment::Load,
            _ => TestEnvironment::Integration,
        };

        let performance_thresholds = match environment {
            TestEnvironment::Unit => PerformanceThresholds {
                max_response_time_ms: 100,
                min_requests_per_second: 1000.0,
                max_error_rate_percent: 1.0,
                max_memory_usage_mb: 100,
            },
            TestEnvironment::Integration => PerformanceThresholds {
                max_response_time_ms: 500,
                min_requests_per_second: 100.0,
                max_error_rate_percent: 2.0,
                max_memory_usage_mb: 500,
            },
            TestEnvironment::Performance => PerformanceThresholds {
                max_response_time_ms: 1000,
                min_requests_per_second: 50.0,
                max_error_rate_percent: 5.0,
                max_memory_usage_mb: 1000,
            },
            TestEnvironment::Load => PerformanceThresholds {
                max_response_time_ms: 2000,
                min_requests_per_second: 25.0,
                max_error_rate_percent: 10.0,
                max_memory_usage_mb: 2000,
            },
            TestEnvironment::Staging => PerformanceThresholds {
                max_response_time_ms: 800,
                min_requests_per_second: 75.0,
                max_error_rate_percent: 3.0,
                max_memory_usage_mb: 750,
            },
        };

        Self {
            environment,
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379/15".to_string()),
            browser_executable: env::var("BROWSER_EXECUTABLE").ok(),
            api_base_url: env::var("API_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            load_test_duration: env::var("LOAD_TEST_DURATION")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            load_test_concurrency: env::var("LOAD_TEST_CONCURRENCY")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            performance_thresholds,
        }
    }
}
```

## Test Data Management

### 1. Test Data Generation

```rust
// tests/data/mod.rs
use serde_json::{json, Value};
use uuid::Uuid;
use std::collections::HashMap;

pub struct TestDataGenerator;

impl TestDataGenerator {
    pub fn generate_extraction_batch(size: usize) -> Value {
        let mut urls = Vec::new();

        for i in 0..size {
            urls.push(format!("https://example.com/page-{}", i));
        }

        json!({
            "batch_id": Uuid::new_v4().to_string(),
            "urls": urls,
            "config": {
                "strategy": "comprehensive",
                "timeout_ms": 30000,
                "retry_attempts": 3,
                "extract_content": true,
                "extract_metadata": true,
                "extract_images": true
            }
        })
    }

    pub fn generate_streaming_config() -> Value {
        json!({
            "stream_id": Uuid::new_v4().to_string(),
            "batch_size": 5,
            "max_concurrent": 10,
            "buffer_size": 100,
            "heartbeat_interval_ms": 5000,
            "timeout_ms": 60000
        })
    }

    pub fn generate_browser_pool_config() -> Value {
        json!({
            "pool_size": 3,
            "max_idle_time_ms": 300000,
            "launch_options": {
                "headless": true,
                "disable_images": false,
                "disable_javascript": false,
                "user_agent": "riptide-Test-Agent/1.0"
            },
            "retry_config": {
                "max_attempts": 3,
                "backoff_ms": 1000
            }
        })
    }

    pub fn generate_test_html_pages(count: usize) -> HashMap<String, String> {
        let mut pages = HashMap::new();

        for i in 0..count {
            let html = format!(r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Test Page {}</title>
                    <meta name="description" content="Test page {} for extraction testing">
                    <meta name="keywords" content="test, extraction, page{}">
                </head>
                <body>
                    <header>
                        <h1>Test Page {} - Main Heading</h1>
                        <nav>
                            <a href="/page/{}/section1">Section 1</a>
                            <a href="/page/{}/section2">Section 2</a>
                        </nav>
                    </header>
                    <main>
                        <article>
                            <h2>Article Title {}</h2>
                            <p>This is the main content for test page {}. It contains various elements for extraction testing.</p>
                            <p>Second paragraph with <strong>bold text</strong> and <em>italic text</em>.</p>
                            <ul>
                                <li>List item 1 for page {}</li>
                                <li>List item 2 for page {}</li>
                                <li>List item 3 for page {}</li>
                            </ul>
                        </article>
                        <aside>
                            <h3>Sidebar Content</h3>
                            <p>Additional information in the sidebar.</p>
                            <img src="/images/test-image-{}.jpg" alt="Test image {}">
                        </aside>
                    </main>
                    <footer>
                        <p>&copy; 2024 riptide Test Suite</p>
                        <div class="social-links">
                            <a href="https://twitter.com/test">Twitter</a>
                            <a href="https://facebook.com/test">Facebook</a>
                        </div>
                    </footer>
                </body>
                </html>
            "#, i, i, i, i, i, i, i, i, i, i, i, i);

            pages.insert(format!("page-{}.html", i), html);
        }

        pages
    }

    pub fn generate_performance_test_data() -> Value {
        json!({
            "scenarios": [
                {
                    "name": "light_load",
                    "concurrent_users": 10,
                    "requests_per_user": 50,
                    "ramp_up_duration_s": 30,
                    "test_duration_s": 300
                },
                {
                    "name": "moderate_load",
                    "concurrent_users": 50,
                    "requests_per_user": 100,
                    "ramp_up_duration_s": 60,
                    "test_duration_s": 600
                },
                {
                    "name": "heavy_load",
                    "concurrent_users": 100,
                    "requests_per_user": 200,
                    "ramp_up_duration_s": 120,
                    "test_duration_s": 1200
                }
            ],
            "urls": [
                "https://httpbin.org/json",
                "https://httpbin.org/xml",
                "https://httpbin.org/html",
                "https://jsonplaceholder.typicode.com/posts/1",
                "https://example.com"
            ]
        })
    }
}
```

### 2. Test Data Cleanup

```rust
// tests/cleanup/mod.rs
use std::fs;
use std::path::Path;
use tokio::time::{timeout, Duration};

pub struct TestDataCleanup;

impl TestDataCleanup {
    pub async fn cleanup_test_environment() -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting test environment cleanup...");

        // Cleanup test files
        Self::cleanup_test_files().await?;

        // Cleanup test databases
        Self::cleanup_test_databases().await?;

        // Cleanup temporary directories
        Self::cleanup_temp_directories().await?;

        // Cleanup background processes
        Self::cleanup_background_processes().await?;

        println!("Test environment cleanup completed.");
        Ok(())
    }

    async fn cleanup_test_files() -> Result<(), Box<dyn std::error::Error>> {
        let test_file_patterns = vec![
            "test_*.db",
            "test_*.log",
            "test_*.json",
            "*_test_results.xml",
            "coverage_*.xml",
        ];

        for pattern in test_file_patterns {
            // Use glob crate or manual cleanup
            if let Ok(entries) = glob::glob(pattern) {
                for entry in entries {
                    if let Ok(path) = entry {
                        if path.is_file() {
                            fs::remove_file(&path).ok();
                            println!("Cleaned up test file: {:?}", path);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn cleanup_test_databases() -> Result<(), Box<dyn std::error::Error>> {
        // Redis test database cleanup
        let redis_client = redis::Client::open("redis://localhost:6379/15")?;
        let mut con = redis_client.get_connection()?;
        redis::cmd("FLUSHDB").execute(&mut con);
        println!("Cleaned up Redis test database");

        // SQLite test database cleanup
        let test_db_files = vec![
            "./test_data.db",
            "./integration_test.db",
            "./performance_test.db",
        ];

        for db_file in test_db_files {
            if Path::new(db_file).exists() {
                fs::remove_file(db_file).ok();
                println!("Cleaned up SQLite database: {}", db_file);
            }
        }

        Ok(())
    }

    async fn cleanup_temp_directories() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dirs = vec![
            "./tmp/test_browser_profiles",
            "./tmp/test_downloads",
            "./tmp/test_cache",
            "./tmp/wasm_test_modules",
        ];

        for dir in temp_dirs {
            if Path::new(dir).exists() {
                fs::remove_dir_all(dir).ok();
                println!("Cleaned up temporary directory: {}", dir);
            }
        }

        Ok(())
    }

    async fn cleanup_background_processes() -> Result<(), Box<dyn std::error::Error>> {
        // Attempt graceful shutdown of any background services
        let shutdown_timeout = Duration::from_secs(10);

        // Example: Shutdown browser processes
        if let Ok(_) = timeout(shutdown_timeout, Self::shutdown_browser_processes()).await {
            println!("Browser processes shut down gracefully");
        } else {
            println!("Warning: Browser processes shutdown timed out");
            // Force kill if necessary
            Self::force_kill_browser_processes().await.ok();
        }

        // Example: Shutdown API servers
        if let Ok(_) = timeout(shutdown_timeout, Self::shutdown_api_servers()).await {
            println!("API servers shut down gracefully");
        } else {
            println!("Warning: API servers shutdown timed out");
        }

        Ok(())
    }

    async fn shutdown_browser_processes() -> Result<(), Box<dyn std::error::Error>> {
        // Send SIGTERM to browser processes
        use std::process::Command;

        let output = Command::new("pkill")
            .args(&["-f", "chrome.*--remote-debugging-port"])
            .output()?;

        if output.status.success() {
            println!("Sent shutdown signal to browser processes");
        }

        Ok(())
    }

    async fn force_kill_browser_processes() -> Result<(), Box<dyn std::error::Error>> {
        use std::process::Command;

        let output = Command::new("pkill")
            .args(&["-9", "-f", "chrome.*--remote-debugging-port"])
            .output()?;

        if output.status.success() {
            println!("Force killed browser processes");
        }

        Ok(())
    }

    async fn shutdown_api_servers() -> Result<(), Box<dyn std::error::Error>> {
        // Attempt to shutdown any running test API servers
        use reqwest::Client;

        let client = Client::new();
        let shutdown_endpoints = vec![
            "http://localhost:8081/shutdown",
            "http://localhost:8082/shutdown",
        ];

        for endpoint in shutdown_endpoints {
            if let Ok(_) = client.post(endpoint).send().await {
                println!("Sent shutdown request to: {}", endpoint);
            }
        }

        Ok(())
    }
}

// Cleanup helper for individual tests
pub async fn cleanup_after_test(test_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Cleaning up after test: {}", test_name);

    // Test-specific cleanup logic
    match test_name {
        name if name.contains("browser") => {
            TestDataCleanup::shutdown_browser_processes().await.ok();
        }
        name if name.contains("streaming") => {
            // Close WebSocket connections, cleanup streaming resources
        }
        name if name.contains("load") => {
            // Cleanup load test artifacts
            TestDataCleanup::cleanup_test_files().await.ok();
        }
        _ => {
            // General cleanup
        }
    }

    Ok(())
}
```

## Running Integration Tests

### 1. Test Execution Commands

```bash
# Run all integration tests
cargo test --test '*integration*' --release

# Run specific test category
cargo test --test browser_pool_integration_test --release
cargo test --test wasm_integration_test --release
cargo test --test api_integration_test --release

# Run performance tests
cargo test --test '*performance*' --release -- --nocapture

# Run load tests (longer duration)
TEST_ENVIRONMENT=load LOAD_TEST_DURATION=300 cargo test --test '*load*' --release -- --nocapture

# Run with specific Redis instance
REDIS_URL=redis://localhost:6379/15 cargo test --test '*integration*'

# Run with browser executable specified
BROWSER_EXECUTABLE=/usr/bin/chromium-browser cargo test --test browser_pool_integration_test
```

### 2. Test Organization and Best Practices

- **Parallel Execution**: Tests are designed to run in parallel with isolated resources
- **Environment-Specific**: Different thresholds and configurations for various environments
- **Resource Management**: Proper cleanup and resource allocation
- **Realistic Data**: Test data generation that mirrors production scenarios
- **Performance Monitoring**: Built-in performance assertions and monitoring
- **Error Handling**: Comprehensive error scenarios and recovery testing

This comprehensive integration testing guide provides the foundation for ensuring riptide system reliability, performance, and correctness across all components and deployment scenarios.