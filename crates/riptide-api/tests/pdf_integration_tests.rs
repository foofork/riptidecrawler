//! Comprehensive integration tests for the PDF pipeline
//!
//! This module provides end-to-end integration tests covering:
//! - Basic PDF processing through API endpoints
//! - Streaming PDF processing with progress tracking
//! - Worker service integration for PDF jobs
//! - Memory management and limits testing
//! - Error handling for invalid PDFs
//! - Concurrent processing capabilities
//! - Extraction options validation
//! - Timeout handling
//! - Performance benchmarks

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use base64::prelude::*;
use futures_util::StreamExt;
use riptide_pdf::types::ProgressUpdate;
use serde_json::{json, Value};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{mpsc, Mutex},
    time::timeout,
};
use tower::ServiceExt;

// Test utilities and mock data
mod test_utils {
    use super::*;

    /// Generate a valid minimal PDF for testing
    pub fn create_mock_pdf_data() -> Vec<u8> {
        // Minimal valid PDF structure
        let pdf_content = b"%PDF-1.7\n\
1 0 obj\n\
<< /Type /Catalog /Pages 2 0 R >>\n\
endobj\n\
\n\
2 0 obj\n\
<< /Type /Pages /Kids [3 0 R] /Count 1 >>\n\
endobj\n\
\n\
3 0 obj\n\
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\n\
endobj\n\
\n\
4 0 obj\n\
<< /Length 44 >>\n\
stream\n\
BT\n\
/F1 12 Tf\n\
100 700 Td\n\
(Hello, World!) Tj\n\
ET\n\
endstream\n\
endobj\n\
\n\
5 0 obj\n\
<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\n\
endobj\n\
\n\
xref\n\
0 6\n\
0000000000 65535 f \n\
0000000009 00000 n \n\
0000000058 00000 n \n\
0000000115 00000 n \n\
0000000244 00000 n \n\
0000000338 00000 n \n\
trailer\n\
<< /Size 6 /Root 1 0 R >>\n\
startxref\n\
0000000423\n\
%%EOF\n";

        pdf_content.to_vec()
    }

    /// Create a larger mock PDF for memory testing
    pub fn create_large_mock_pdf_data(size_mb: usize) -> Vec<u8> {
        let mut base_pdf = create_mock_pdf_data();
        let target_size = size_mb * 1024 * 1024;

        // Pad with whitespace to reach desired size
        while base_pdf.len() < target_size {
            let remaining = target_size - base_pdf.len();
            let pad_size = remaining.min(4096);
            base_pdf.extend(vec![b' '; pad_size]);
        }

        base_pdf
    }

    /// Create an invalid PDF for error testing
    pub fn create_invalid_pdf_data() -> Vec<u8> {
        b"This is not a PDF file at all".to_vec()
    }

    /// Create a corrupted PDF header
    pub fn create_corrupted_pdf_data() -> Vec<u8> {
        b"%PDF-CORRUPTED\nThis PDF has invalid structure".to_vec()
    }

    /// Create a PDF that will timeout (simulate stuck processing)
    pub fn create_timeout_pdf_data() -> Vec<u8> {
        // Create a PDF with complex structure that might cause processing delays
        let mut pdf = create_mock_pdf_data();

        // Add a large number of objects to simulate complexity
        for i in 6..1000 {
            let obj = format!(
                "{} 0 obj\n<< /Type /Object /Data (Complex data {}) >>\nendobj\n",
                i, i
            );
            pdf.extend(obj.as_bytes());
        }

        pdf
    }

    /// Encode PDF data to base64
    pub fn encode_pdf_base64(data: &[u8]) -> String {
        BASE64_STANDARD.encode(data)
    }

    /// Memory monitoring helper
    pub struct MemoryMonitor {
        initial_memory: u64,
        peak_memory: Arc<AtomicU64>,
        measurements: Arc<Mutex<Vec<u64>>>,
    }

    impl MemoryMonitor {
        pub fn new() -> Self {
            let initial = get_current_memory_usage();
            Self {
                initial_memory: initial,
                peak_memory: Arc::new(AtomicU64::new(initial)),
                measurements: Arc::new(Mutex::new(Vec::new())),
            }
        }

        pub async fn start_monitoring(&self, interval: Duration) -> mpsc::UnboundedReceiver<u64> {
            let (tx, rx) = mpsc::unbounded_channel();
            let peak = Arc::clone(&self.peak_memory);
            let measurements = Arc::clone(&self.measurements);

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(interval);
                loop {
                    interval.tick().await;
                    let current = get_current_memory_usage();
                    peak.fetch_max(current, Ordering::Relaxed);
                    measurements.lock().await.push(current);

                    if tx.send(current).is_err() {
                        break;
                    }
                }
            });

            rx
        }

        pub fn get_peak_memory(&self) -> u64 {
            self.peak_memory.load(Ordering::Relaxed)
        }

        pub fn get_memory_spike(&self) -> u64 {
            self.get_peak_memory().saturating_sub(self.initial_memory)
        }

        #[allow(dead_code)]
        pub async fn get_average_memory(&self) -> u64 {
            let measurements = self.measurements.lock().await;
            if measurements.is_empty() {
                self.initial_memory
            } else {
                measurements.iter().sum::<u64>() / measurements.len() as u64
            }
        }
    }

    fn get_current_memory_usage() -> u64 {
        // Simple memory usage estimation using system-specific APIs
        #[cfg(unix)]
        {
            use std::fs;
            if let Ok(status) = fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb * 1024; // Convert to bytes
                            }
                        }
                    }
                }
            }
        }

        // Fallback estimate
        1024 * 1024 * 50 // 50MB baseline
    }

    /// Performance measurement helper
    pub struct PerformanceTimer {
        start: Instant,
        checkpoints: Vec<(String, Instant)>,
    }

    impl PerformanceTimer {
        pub fn new() -> Self {
            Self {
                start: Instant::now(),
                checkpoints: Vec::new(),
            }
        }

        pub fn checkpoint(&mut self, name: &str) {
            self.checkpoints.push((name.to_string(), Instant::now()));
        }

        pub fn get_duration(&self) -> Duration {
            self.start.elapsed()
        }

        #[allow(dead_code)]
        pub fn get_checkpoint_durations(&self) -> Vec<(String, Duration)> {
            let mut results = Vec::new();
            let mut last_time = self.start;

            for (name, time) in &self.checkpoints {
                results.push((name.clone(), time.duration_since(last_time)));
                last_time = *time;
            }

            results
        }
    }
}

// Integration test setup
mod test_setup {
    use axum::Router;
    use riptide_api::{
        health::HealthChecker,
        metrics::RipTideMetrics,
        routes,
        state::{AppConfig, AppState},
    };
    use std::sync::Arc;

    pub async fn create_test_app() -> Router {
        let config = AppConfig::default();
        let metrics = Arc::new(RipTideMetrics::new().expect("Failed to create metrics"));
        let health_checker = Arc::new(HealthChecker::new());

        let app_state = AppState::new(config, metrics, health_checker)
            .await
            .expect("Failed to create app state");

        Router::new()
            .nest("/pdf", routes::pdf::pdf_routes())
            .with_state(app_state)
    }
}

// Test 1: Basic PDF Processing Through API Endpoint
#[tokio::test]
#[ignore = "Requires WASM extractor and full AppState dependencies - run with --ignored"]
async fn test_basic_pdf_processing_api_endpoint() {
    let app = test_setup::create_test_app().await;
    let pdf_data = test_utils::create_mock_pdf_data();
    let encoded_data = test_utils::encode_pdf_base64(&pdf_data);

    let request_body = json!({
        "pdf_data": encoded_data,
        "filename": "test.pdf",
        "url": "test://document"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/pdf/process")
        .header("content-type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(response_json["success"].as_bool().unwrap());
    assert!(response_json["document"].is_object());
    assert!(response_json["stats"]["processing_time_ms"].is_number());
    assert!(response_json["stats"]["file_size"].as_u64().unwrap() > 0);

    println!("✅ Basic PDF processing test passed");
}

// Test 2: PDF Streaming with Progress Tracking
#[tokio::test]
#[ignore = "Requires WASM extractor and full AppState dependencies - run with --ignored"]
async fn test_pdf_streaming_with_progress() {
    let app = test_setup::create_test_app().await;
    let pdf_data = test_utils::create_mock_pdf_data();
    let encoded_data = test_utils::encode_pdf_base64(&pdf_data);

    let request_body = json!({
        "pdf_data": encoded_data,
        "filename": "test_stream.pdf",
        "stream_progress": true
    });

    let request = Request::builder()
        .method("POST")
        .uri("/pdf/process-stream")
        .header("content-type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check streaming headers
    let headers = response.headers();
    assert_eq!(headers["content-type"], "application/x-ndjson");

    let mut body_stream = response.into_body().into_data_stream();
    let mut progress_updates = Vec::new();
    let mut has_started = false;
    let mut has_completed = false;

    // Collect progress updates with timeout
    while let Ok(Some(chunk)) = timeout(Duration::from_secs(10), body_stream.next()).await {
        if let Ok(chunk) = chunk {
            let text = String::from_utf8_lossy(&chunk);

            for line in text.lines() {
                if line.trim().is_empty() {
                    continue;
                }

                if let Ok(update) = serde_json::from_str::<ProgressUpdate>(line) {
                    progress_updates.push(update.clone());

                    match update {
                        ProgressUpdate::Started { .. } => has_started = true,
                        ProgressUpdate::Completed { .. } => {
                            has_completed = true;
                            break;
                        }
                        ProgressUpdate::Failed { .. } => {
                            panic!("Processing failed during streaming test");
                        }
                        _ => {}
                    }
                }
            }
        }

        if has_completed {
            break;
        }
    }

    assert!(has_started, "Should have received started event");
    assert!(has_completed, "Should have received completed event");
    assert!(
        !progress_updates.is_empty(),
        "Should have received progress updates"
    );

    println!(
        "✅ PDF streaming with progress test passed - {} updates received",
        progress_updates.len()
    );
}

// Test 3: PDF Processing Through Worker Service
#[tokio::test]
#[ignore = "Requires WASM extractor and full AppState dependencies - run with --ignored"]
async fn test_pdf_processing_through_worker_service() {
    let app = test_setup::create_test_app().await;
    let pdf_data = test_utils::create_mock_pdf_data();
    let encoded_data = test_utils::encode_pdf_base64(&pdf_data);

    // Submit PDF job to worker queue
    let job_request = json!({
        "job_type": {
            "type": "custom",
            "job_name": "pdf_processing",
            "payload": {
                "pdf_data": encoded_data,
                "filename": "worker_test.pdf",
                "extract_text": true,
                "extract_images": false,
                "extract_metadata": true
            }
        },
        "priority": "High",
        "timeout_secs": 60,
        "metadata": {
            "source": "integration_test",
            "test_type": "worker_service"
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/workers/jobs")
        .header("content-type", "application/json")
        .body(Body::from(job_request.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    let job_id = response_json["job_id"].as_str().unwrap();
    assert!(!job_id.is_empty());
    assert_eq!(response_json["status"], "submitted");

    println!(
        "✅ PDF worker service integration test passed - Job ID: {}",
        job_id
    );
}

// Test 4: Memory Management with Large PDFs
#[tokio::test]
#[ignore = "Requires WASM extractor and full AppState dependencies - run with --ignored"]
async fn test_memory_management_large_pdfs() {
    let monitor = test_utils::MemoryMonitor::new();
    let _memory_stream = monitor.start_monitoring(Duration::from_millis(100)).await;

    // Test with progressively larger PDFs
    let test_sizes = vec![1, 5, 10]; // MB sizes

    for size_mb in test_sizes {
        let app = test_setup::create_test_app().await;
        let pdf_data = test_utils::create_large_mock_pdf_data(size_mb);
        let encoded_data = test_utils::encode_pdf_base64(&pdf_data);

        let request_body = json!({
            "pdf_data": encoded_data,
            "filename": format!("large_test_{}mb.pdf", size_mb)
        });

        let timer = Instant::now();
        let request = Request::builder()
            .method("POST")
            .uri("/pdf/process")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let _response = app.oneshot(request).await.unwrap();
        let processing_time = timer.elapsed();

        // Memory spike should not exceed 200MB (ROADMAP requirement)
        let memory_spike_mb = monitor.get_memory_spike() / (1024 * 1024);
        assert!(
            memory_spike_mb <= 200,
            "Memory spike {} MB exceeds 200MB limit for {} MB PDF",
            memory_spike_mb,
            size_mb
        );

        // Processing should complete within reasonable time
        assert!(
            processing_time < Duration::from_secs(30),
            "Processing took {} seconds for {} MB PDF",
            processing_time.as_secs(),
            size_mb
        );

        println!(
            "✅ Memory test passed for {} MB PDF - Spike: {} MB, Time: {:?}",
            size_mb, memory_spike_mb, processing_time
        );

        // Allow memory to settle between tests
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

// Test 5: Error Handling for Invalid PDFs
#[tokio::test]
#[ignore = "Requires WASM extractor and full AppState dependencies - run with --ignored"]
async fn test_error_handling_invalid_pdfs() {
    let app = test_setup::create_test_app().await;

    // Test 5.1: Completely invalid data
    let invalid_data = test_utils::create_invalid_pdf_data();
    let encoded_invalid = test_utils::encode_pdf_base64(&invalid_data);

    let request_body = json!({
        "pdf_data": encoded_invalid,
        "filename": "invalid.pdf"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/pdf/process")
        .header("content-type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response_json.get("message").is_some());

    // Test 5.2: Corrupted PDF header
    let app2 = test_setup::create_test_app().await;
    let corrupted_data = test_utils::create_corrupted_pdf_data();
    let encoded_corrupted = test_utils::encode_pdf_base64(&corrupted_data);

    let request_body2 = json!({
        "pdf_data": encoded_corrupted,
        "filename": "corrupted.pdf"
    });

    let request2 = Request::builder()
        .method("POST")
        .uri("/pdf/process")
        .header("content-type", "application/json")
        .body(Body::from(request_body2.to_string()))
        .unwrap();

    let response2 = app2.oneshot(request2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::BAD_REQUEST);

    // Test 5.3: Missing PDF data
    let app3 = test_setup::create_test_app().await;
    let empty_request = json!({
        "filename": "missing.pdf"
    });

    let request3 = Request::builder()
        .method("POST")
        .uri("/pdf/process")
        .header("content-type", "application/json")
        .body(Body::from(empty_request.to_string()))
        .unwrap();

    let response3 = app3.oneshot(request3).await.unwrap();
    assert_eq!(response3.status(), StatusCode::BAD_REQUEST);

    // Test 5.4: Invalid base64 encoding
    let app4 = test_setup::create_test_app().await;
    let invalid_base64_request = json!({
        "pdf_data": "invalid_base64_!@#$%",
        "filename": "invalid_encoding.pdf"
    });

    let request4 = Request::builder()
        .method("POST")
        .uri("/pdf/process")
        .header("content-type", "application/json")
        .body(Body::from(invalid_base64_request.to_string()))
        .unwrap();

    let response4 = app4.oneshot(request4).await.unwrap();
    assert_eq!(response4.status(), StatusCode::BAD_REQUEST);

    println!("✅ Error handling tests passed for all invalid PDF scenarios");
}

// Test 6: Concurrent PDF Processing
#[tokio::test]
#[ignore = "Requires WASM extractor and full AppState dependencies - run with --ignored"]
async fn test_concurrent_pdf_processing() {
    const CONCURRENT_REQUESTS: usize = 5;
    let pdf_data = test_utils::create_mock_pdf_data();
    let encoded_data = test_utils::encode_pdf_base64(&pdf_data);

    // Create multiple concurrent requests
    let mut handles = Vec::new();
    let start_time = Instant::now();

    for i in 0..CONCURRENT_REQUESTS {
        let app = test_setup::create_test_app().await;
        let data = encoded_data.clone();

        let handle = tokio::spawn(async move {
            let request_body = json!({
                "pdf_data": data,
                "filename": format!("concurrent_test_{}.pdf", i)
            });

            let request = Request::builder()
                .method("POST")
                .uri("/pdf/process")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap();

            let response = app.oneshot(request).await.unwrap();
            let status = response.status();
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();

            (i, status, body)
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    let results = futures_util::future::try_join_all(handles).await.unwrap();
    let total_time = start_time.elapsed();

    // Verify all requests succeeded
    let mut success_count = 0;
    for (request_id, status, body) in results {
        if status == StatusCode::OK {
            let response_json: Value = serde_json::from_slice(&body).unwrap();
            if response_json["success"].as_bool().unwrap_or(false) {
                success_count += 1;
            }
        }

        println!("Request {} completed with status: {}", request_id, status);
    }

    assert_eq!(
        success_count, CONCURRENT_REQUESTS,
        "Expected {} successful requests, got {}",
        CONCURRENT_REQUESTS, success_count
    );

    // Verify reasonable performance under concurrency
    let avg_time_per_request = total_time.as_secs_f64() / CONCURRENT_REQUESTS as f64;
    assert!(
        avg_time_per_request < 10.0,
        "Average time per request {} seconds is too high under concurrency",
        avg_time_per_request
    );

    println!(
        "✅ Concurrent processing test passed - {} requests in {:?} (avg: {:.2}s per request)",
        CONCURRENT_REQUESTS, total_time, avg_time_per_request
    );
}

// Test 7: PDF Extraction Options (Text, Images, Metadata)
#[tokio::test]
#[ignore = "Requires WASM extractor and full AppState dependencies - run with --ignored"]
async fn test_pdf_extraction_options() {
    let pdf_data = test_utils::create_mock_pdf_data();
    let encoded_data = test_utils::encode_pdf_base64(&pdf_data);

    // Test different extraction configurations
    let test_configs = vec![
        (
            "text_only",
            json!({"extract_text": true, "extract_images": false, "extract_metadata": false}),
        ),
        (
            "images_only",
            json!({"extract_text": false, "extract_images": true, "extract_metadata": false}),
        ),
        (
            "metadata_only",
            json!({"extract_text": false, "extract_images": false, "extract_metadata": true}),
        ),
        (
            "text_and_metadata",
            json!({"extract_text": true, "extract_images": false, "extract_metadata": true}),
        ),
        (
            "all_options",
            json!({"extract_text": true, "extract_images": true, "extract_metadata": true}),
        ),
    ];

    for (config_name, extraction_options) in test_configs {
        let app = test_setup::create_test_app().await;

        let mut request_body = json!({
            "pdf_data": encoded_data,
            "filename": format!("extraction_test_{}.pdf", config_name)
        });

        // Merge extraction options into request
        if let Some(request_obj) = request_body.as_object_mut() {
            if let Some(options_obj) = extraction_options.as_object() {
                for (key, value) in options_obj {
                    request_obj.insert(key.clone(), value.clone());
                }
            }
        }

        let request = Request::builder()
            .method("POST")
            .uri("/pdf/process")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_json: Value = serde_json::from_slice(&body).unwrap();

        assert!(response_json["success"].as_bool().unwrap());

        // Verify extraction options were respected
        let document = &response_json["document"];
        match config_name {
            "text_only" => {
                assert!(document["text"].is_string() || document["markdown"].is_string());
            }
            "metadata_only" => {
                // Metadata should be present in the document structure
                assert!(document.is_object());
            }
            "all_options" => {
                assert!(document["text"].is_string() || document["markdown"].is_string());
                assert!(document.is_object());
            }
            _ => {
                // Other configurations should also succeed
                assert!(document.is_object());
            }
        }

        println!(
            "✅ Extraction options test passed for configuration: {}",
            config_name
        );
    }
}

// Test 8: Timeout Handling for Stuck PDFs
#[tokio::test]
#[ignore = "Requires WASM extractor and full AppState dependencies - run with --ignored"]
async fn test_timeout_handling_stuck_pdfs() {
    let app = test_setup::create_test_app().await;

    // Create a PDF that might cause processing delays
    let timeout_pdf_data = test_utils::create_timeout_pdf_data();
    let encoded_data = test_utils::encode_pdf_base64(&timeout_pdf_data);

    let request_body = json!({
        "pdf_data": encoded_data,
        "filename": "timeout_test.pdf",
        "timeout_seconds": 5 // Short timeout for testing
    });

    let request = Request::builder()
        .method("POST")
        .uri("/pdf/process")
        .header("content-type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let start_time = Instant::now();

    // Use timeout to ensure the test doesn't hang
    let response_result = timeout(
        Duration::from_secs(10), // Test timeout longer than processing timeout
        app.oneshot(request),
    )
    .await;

    let processing_time = start_time.elapsed();

    match response_result {
        Ok(Ok(response)) => {
            // Processing completed (might be success or controlled failure)
            let status = response.status();
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();

            if status == StatusCode::OK {
                let response_json: Value = serde_json::from_slice(&body).unwrap();
                if !response_json["success"].as_bool().unwrap_or(false) {
                    // Expected timeout error
                    assert!(response_json["error"].is_string());
                    println!("✅ Timeout handling test passed - Processing timed out as expected");
                } else {
                    // Processing succeeded quickly
                    println!(
                        "✅ Timeout handling test passed - Complex PDF processed successfully"
                    );
                }
            } else {
                // HTTP error response (expected for timeout)
                println!("✅ Timeout handling test passed - HTTP error response for timeout");
            }
        }
        Ok(Err(_)) => {
            println!("✅ Timeout handling test passed - Request failed appropriately");
        }
        Err(_) => {
            // Test timeout - this means processing is stuck
            panic!(
                "Timeout handling failed - processing appears to be stuck after {:?}",
                processing_time
            );
        }
    }

    // Verify processing time was reasonable
    assert!(
        processing_time < Duration::from_secs(12),
        "Processing took too long: {:?}",
        processing_time
    );

    println!(
        "✅ Timeout handling test completed in {:?}",
        processing_time
    );
}

// Performance Benchmarks
#[tokio::test]
#[ignore = "Requires WASM extractor and full AppState dependencies - run with --ignored"]
async fn test_pdf_processing_performance_benchmarks() {
    const BENCHMARK_ITERATIONS: usize = 10;
    let pdf_data = test_utils::create_mock_pdf_data();
    let encoded_data = test_utils::encode_pdf_base64(&pdf_data);

    let mut processing_times = Vec::new();
    let mut memory_usages = Vec::new();
    let mut throughput_rates = Vec::new();

    println!("🔬 Running PDF processing performance benchmarks...");

    for iteration in 0..BENCHMARK_ITERATIONS {
        let app = test_setup::create_test_app().await;
        let monitor = test_utils::MemoryMonitor::new();
        let mut timer = test_utils::PerformanceTimer::new();

        let request_body = json!({
            "pdf_data": encoded_data,
            "filename": format!("benchmark_test_{}.pdf", iteration)
        });

        let request = Request::builder()
            .method("POST")
            .uri("/pdf/process")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        timer.checkpoint("request_start");
        let response = app.oneshot(request).await.unwrap();
        timer.checkpoint("response_received");

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        timer.checkpoint("body_parsed");

        let response_json: Value = serde_json::from_slice(&body).unwrap();
        assert!(response_json["success"].as_bool().unwrap());

        let processing_time = timer.get_duration();
        let memory_spike = monitor.get_memory_spike();
        let file_size = pdf_data.len() as u64;
        let throughput = (file_size as f64) / processing_time.as_secs_f64(); // bytes per second

        processing_times.push(processing_time);
        memory_usages.push(memory_spike);
        throughput_rates.push(throughput);

        println!(
            "Iteration {}: {:?}, Memory: {} KB, Throughput: {:.2} KB/s",
            iteration + 1,
            processing_time,
            memory_spike / 1024,
            throughput / 1024.0
        );

        // Small delay between iterations
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Calculate statistics
    let avg_processing_time = Duration::from_nanos(
        processing_times.iter().map(|d| d.as_nanos()).sum::<u128>() as u64
            / BENCHMARK_ITERATIONS as u64,
    );
    let max_processing_time = processing_times.iter().max().unwrap();
    let min_processing_time = processing_times.iter().min().unwrap();

    let avg_memory_usage = memory_usages.iter().sum::<u64>() / BENCHMARK_ITERATIONS as u64;
    let max_memory_usage = *memory_usages.iter().max().unwrap();

    let avg_throughput = throughput_rates.iter().sum::<f64>() / BENCHMARK_ITERATIONS as f64;
    let max_throughput = throughput_rates.iter().fold(0.0_f64, |a, &b| a.max(b));

    // Performance assertions
    assert!(
        avg_processing_time < Duration::from_secs(5),
        "Average processing time {:?} exceeds 5 seconds",
        avg_processing_time
    );

    assert!(
        max_memory_usage < 100 * 1024 * 1024, // 100MB
        "Maximum memory usage {} MB exceeds 100MB",
        max_memory_usage / (1024 * 1024)
    );

    assert!(
        avg_throughput > 1024.0, // 1 KB/s minimum
        "Average throughput {:.2} B/s is too low",
        avg_throughput
    );

    println!("\n📊 Performance Benchmark Results:");
    println!(
        "📈 Processing Time - Avg: {:?}, Min: {:?}, Max: {:?}",
        avg_processing_time, min_processing_time, max_processing_time
    );
    println!(
        "🧠 Memory Usage - Avg: {} MB, Max: {} MB",
        avg_memory_usage / (1024 * 1024),
        max_memory_usage / (1024 * 1024)
    );
    println!(
        "⚡ Throughput - Avg: {:.2} KB/s, Max: {:.2} KB/s",
        avg_throughput / 1024.0,
        max_throughput / 1024.0
    );

    // Performance regression detection
    let performance_score =
        calculate_performance_score(avg_processing_time, avg_memory_usage, avg_throughput);

    assert!(
        performance_score > 70.0,
        "Performance score {:.1} is below acceptable threshold of 70",
        performance_score
    );

    println!("🏆 Overall Performance Score: {:.1}/100", performance_score);
    println!("✅ All performance benchmarks passed!");
}

// Progress Callback Overhead Test
#[tokio::test]
#[ignore = "Requires WASM extractor and full AppState dependencies - run with --ignored"]
async fn test_progress_callback_overhead() {
    let app = test_setup::create_test_app().await;
    let pdf_data = test_utils::create_mock_pdf_data();
    let encoded_data = test_utils::encode_pdf_base64(&pdf_data);

    let request_body = json!({
        "pdf_data": encoded_data,
        "filename": "progress_overhead_test.pdf",
        "stream_progress": true
    });

    let request = Request::builder()
        .method("POST")
        .uri("/pdf/process-stream")
        .header("content-type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let start_time = Instant::now();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let mut body_stream = response.into_body().into_data_stream();
    let mut progress_callback_overheads = Vec::new();

    while let Ok(Some(chunk)) = timeout(Duration::from_secs(10), body_stream.next()).await {
        if let Ok(chunk) = chunk {
            let text = String::from_utf8_lossy(&chunk);

            for line in text.lines() {
                if line.trim().is_empty() {
                    continue;
                }

                if let Ok(update) = serde_json::from_str::<Value>(line) {
                    if let Some(overhead) = update["average_progress_overhead_us"].as_u64() {
                        progress_callback_overheads.push(overhead);
                    }

                    // Check for completion
                    if update
                        .get("update")
                        .and_then(|u| u.get("Completed"))
                        .is_some()
                    {
                        break;
                    }
                }
            }
        }
    }

    let total_time = start_time.elapsed();

    if !progress_callback_overheads.is_empty() {
        let avg_overhead_us = progress_callback_overheads.iter().sum::<u64>()
            / progress_callback_overheads.len() as u64;
        let max_overhead_us = *progress_callback_overheads.iter().max().unwrap();

        // Progress callback overhead should be minimal
        assert!(
            avg_overhead_us < 1000, // Less than 1ms average
            "Progress callback overhead {} μs is too high",
            avg_overhead_us
        );

        assert!(
            max_overhead_us < 5000, // Less than 5ms max
            "Maximum progress callback overhead {} μs is too high",
            max_overhead_us
        );

        println!(
            "✅ Progress callback overhead test passed - Avg: {} μs, Max: {} μs, Total time: {:?}",
            avg_overhead_us, max_overhead_us, total_time
        );
    } else {
        println!(
            "⚠️  Progress callback overhead test completed but no overhead data was collected"
        );
    }
}

// Helper function to calculate performance score
fn calculate_performance_score(
    avg_processing_time: Duration,
    avg_memory_usage: u64,
    avg_throughput: f64,
) -> f64 {
    let time_score = (5.0 - avg_processing_time.as_secs_f64().min(5.0)) / 5.0 * 40.0;
    let memory_score = ((100.0 * 1024.0 * 1024.0) - avg_memory_usage as f64).max(0.0)
        / (100.0 * 1024.0 * 1024.0)
        * 30.0;
    let throughput_score = (avg_throughput / 1024.0).min(100.0) / 100.0 * 30.0;

    time_score + memory_score + throughput_score
}

// Integration test for PDF health check endpoint
#[tokio::test]
#[ignore = "Requires WASM extractor and full AppState dependencies - run with --ignored"]
async fn test_pdf_health_check_endpoint() {
    let app = test_setup::create_test_app().await;

    let request = Request::builder()
        .method("GET")
        .uri("/pdf/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let health_json: Value = serde_json::from_slice(&body).unwrap();

    // Verify health check response structure
    assert!(health_json["status"].is_string());
    assert!(health_json["pdf_processing_available"].is_boolean());
    assert!(health_json["capabilities"].is_object());
    assert!(health_json["features"].is_object());

    let capabilities = &health_json["capabilities"];
    assert!(capabilities["text_extraction"].is_boolean());
    assert!(capabilities["image_extraction"].is_boolean());
    assert!(capabilities["metadata_extraction"].is_boolean());

    let features = &health_json["features"];
    assert!(features["progress_streaming"].is_boolean());
    assert!(features["concurrent_processing"].is_boolean());

    println!("✅ PDF health check endpoint test passed");
}

// Zero panics/unwraps production safety test
#[tokio::test]
#[ignore = "Requires WASM extractor and full AppState dependencies - run with --ignored"]
async fn test_zero_panics_unwraps_production_safety() {
    // This test verifies that the PDF pipeline gracefully handles errors
    // without panicking or using unwrap() in production code paths

    let test_scenarios = vec![
        ("empty_data", "".to_string()),
        ("malformed_base64", "not-base64!@#".to_string()),
        (
            "truncated_pdf",
            test_utils::encode_pdf_base64(&b"%PDF-1.7\ntruncated"[..]),
        ),
        (
            "binary_garbage",
            test_utils::encode_pdf_base64(&vec![0xFF; 1000]),
        ),
    ];

    for (scenario_name, pdf_data) in test_scenarios {
        let app = test_setup::create_test_app().await;
        let request_body = json!({
            "pdf_data": pdf_data,
            "filename": format!("{}_test.pdf", scenario_name)
        });

        let request = Request::builder()
            .method("POST")
            .uri("/pdf/process")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        // This should not panic regardless of input
        let response = app.oneshot(request).await.unwrap();

        // Should get either success or a controlled error response
        assert!(
            response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST,
            "Scenario '{}' returned unexpected status: {}",
            scenario_name,
            response.status()
        );

        println!(
            "✅ Production safety test passed for scenario: {}",
            scenario_name
        );
    }

    println!("✅ All production safety tests passed - No panics detected");
}
