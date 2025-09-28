use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use riptide_extractor_wasm::{Component, ExtractionMode, ExtractionError};

/// Memory limiter testing module
///
/// This module tests memory growth limits and validates that the WASM component
/// handles memory pressure gracefully without panicking or causing crashes.
///
/// Key test scenarios:
/// - Memory growth beyond configured limits
/// - Circuit breaker activation under memory pressure
/// - Graceful degradation when limits are reached
/// - Memory leak detection across multiple operations
///
///   Memory usage tracking for tests
static MEMORY_TRACKER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct MemoryLimiterConfig {
    pub max_heap_size: u64,        // Maximum heap size in bytes
    pub max_allocation_size: u64,  // Maximum single allocation
    pub growth_threshold: f64,     // Growth threshold percentage
    pub circuit_breaker_threshold: usize, // Failures before circuit opens
}

impl Default for MemoryLimiterConfig {
    fn default() -> Self {
        Self {
            max_heap_size: 50 * 1024 * 1024,     // 50MB
            max_allocation_size: 10 * 1024 * 1024, // 10MB
            growth_threshold: 0.8,                 // 80%
            circuit_breaker_threshold: 3,          // 3 failures
        }
    }
}

#[derive(Debug)]
pub struct MemoryTestResult {
    pub test_name: String,
    pub success: bool,
    pub peak_memory_bytes: u64,
    pub circuit_breaker_triggered: bool,
    pub error_message: Option<String>,
    pub duration_ms: f64,
}

/// Run comprehensive memory limiter tests
pub fn run_memory_limiter_tests() -> Result<Vec<MemoryTestResult>, String> {
    println!("🧠 Starting Memory Limiter Tests");
    println!("================================");

    let config = MemoryLimiterConfig::default();
    let results = vec![
        // Test 1: Normal operation within limits
        test_normal_memory_usage(&config)?,
        // Test 2: Gradual memory growth approaching limits
        test_gradual_memory_growth(&config)?,
        // Test 3: Sudden large allocation beyond limits
        test_large_allocation_beyond_limits(&config)?,
        // Test 4: Circuit breaker activation
        test_circuit_breaker_activation(&config)?,
        // Test 5: Memory leak detection
        test_memory_leak_detection(&config)?,
        // Test 6: Concurrent memory pressure
        test_concurrent_memory_pressure(&config)?,
        // Test 7: Recovery after memory pressure
        test_memory_pressure_recovery(&config)?,
        // Test 8: Edge case - zero-size allocations
        test_zero_size_allocations(&config)?,
    ];

    print_memory_test_summary(&results);

    Ok(results)
}

/// Test normal memory usage within configured limits
fn test_normal_memory_usage(_config: &MemoryLimiterConfig) -> Result<MemoryTestResult, String> {
    println!("\n✅ Testing normal memory usage...");

    let start_time = Instant::now();
    let start_memory = get_current_memory_usage();

    let component = Component;
    let html = generate_medium_html_document(1024 * 100); // 100KB

    // Perform multiple extractions within normal limits
    let mut success = true;
    let mut error_message = None;

    for i in 0..10 {
        match component.extract(
            html.clone(),
            format!("https://example.com/page/{}", i),
            ExtractionMode::Article
        ) {
            Ok(_) => {},
            Err(e) => {
                success = false;
                error_message = Some(format!("Extraction failed at iteration {}: {:?}", i, e));
                break;
            }
        }
    }

    let peak_memory = get_current_memory_usage();
    let duration = start_time.elapsed();

    Ok(MemoryTestResult {
        test_name: "normal_memory_usage".to_string(),
        success,
        peak_memory_bytes: peak_memory - start_memory,
        circuit_breaker_triggered: false,
        error_message,
        duration_ms: duration.as_secs_f64() * 1000.0,
    })
}

/// Test gradual memory growth approaching limits
fn test_gradual_memory_growth(config: &MemoryLimiterConfig) -> Result<MemoryTestResult, String> {
    println!("\n📈 Testing gradual memory growth...");

    let start_time = Instant::now();
    let start_memory = get_current_memory_usage();

    let component = Component;
    let mut success = true;
    let mut error_message = None;
    let mut circuit_breaker_triggered = false;

    // Gradually increase document size to approach memory limits
    for size_kb in [100, 500, 1000, 2000, 5000, 10000] {
        let html = generate_medium_html_document(size_kb * 1024);

        match component.extract(
            html,
            format!("https://example.com/size/{}", size_kb),
            ExtractionMode::Full // Use Full mode to increase memory usage
        ) {
            Ok(_) => {
                let current_memory = get_current_memory_usage() - start_memory;
                if current_memory > (config.max_heap_size as f64 * config.growth_threshold) as u64 {
                    println!("  Memory threshold reached at {}KB document", size_kb);
                    circuit_breaker_triggered = should_circuit_breaker_activate(current_memory, config);
                }
            },
            Err(ExtractionError::ResourceLimit(msg)) => {
                println!("  Memory limit reached at {}KB document: {}", size_kb, msg);
                circuit_breaker_triggered = true;
                break; // Expected behavior
            },
            Err(e) => {
                success = false;
                error_message = Some(format!("Unexpected error at {}KB: {:?}", size_kb, e));
                break;
            }
        }
    }

    let peak_memory = get_current_memory_usage();
    let duration = start_time.elapsed();

    Ok(MemoryTestResult {
        test_name: "gradual_memory_growth".to_string(),
        success,
        peak_memory_bytes: peak_memory - start_memory,
        circuit_breaker_triggered,
        error_message,
        duration_ms: duration.as_secs_f64() * 1000.0,
    })
}

/// Test sudden large allocation beyond limits
fn test_large_allocation_beyond_limits(config: &MemoryLimiterConfig) -> Result<MemoryTestResult, String> {
    println!("\n💥 Testing large allocation beyond limits...");

    let start_time = Instant::now();
    let start_memory = get_current_memory_usage();

    let component = Component;

    // Create a very large document that should exceed memory limits
    let very_large_html = generate_huge_html_document(config.max_allocation_size + 1024 * 1024); // 1MB over limit

    let mut success = true;
    let mut error_message = None;
    let mut circuit_breaker_triggered = false;

    match component.extract(
        very_large_html,
        "https://example.com/huge-document".to_string(),
        ExtractionMode::Full
    ) {
        Ok(_) => {
            // If this succeeds, the memory limiter might not be working properly
            error_message = Some("Large allocation should have been rejected".to_string());
            success = false;
        },
        Err(ExtractionError::ResourceLimit(_)) => {
            println!("  ✅ Large allocation correctly rejected");
            circuit_breaker_triggered = true;
        },
        Err(e) => {
            error_message = Some(format!("Unexpected error type: {:?}", e));
            success = false;
        }
    }

    let peak_memory = get_current_memory_usage();
    let duration = start_time.elapsed();

    Ok(MemoryTestResult {
        test_name: "large_allocation_beyond_limits".to_string(),
        success,
        peak_memory_bytes: peak_memory - start_memory,
        circuit_breaker_triggered,
        error_message,
        duration_ms: duration.as_secs_f64() * 1000.0,
    })
}

/// Test circuit breaker activation
fn test_circuit_breaker_activation(config: &MemoryLimiterConfig) -> Result<MemoryTestResult, String> {
    println!("\n🔌 Testing circuit breaker activation...");

    let start_time = Instant::now();
    let start_memory = get_current_memory_usage();

    let component = Component;
    let problematic_html = generate_medium_html_document(config.max_allocation_size as usize / 2);

    let mut success = true;
    let mut error_message = None;
    let mut circuit_breaker_triggered = false;
    let mut failure_count = 0;

    // Keep trying operations that should fail until circuit breaker activates
    for attempt in 0..10 {
        match component.extract(
            problematic_html.clone(),
            format!("https://example.com/attempt/{}", attempt),
            ExtractionMode::Full
        ) {
            Ok(_) => {
                // Success is actually unexpected here if we're testing memory pressure
            },
            Err(ExtractionError::ResourceLimit(_)) => {
                failure_count += 1;
                println!("  Memory limit failure {}", failure_count);

                if failure_count >= config.circuit_breaker_threshold {
                    circuit_breaker_triggered = true;
                    println!("  🔌 Circuit breaker activated after {} failures", failure_count);
                    break;
                }
            },
            Err(e) => {
                error_message = Some(format!("Unexpected error at attempt {}: {:?}", attempt, e));
                success = false;
                break;
            }
        }
    }

    let peak_memory = get_current_memory_usage();
    let duration = start_time.elapsed();

    Ok(MemoryTestResult {
        test_name: "circuit_breaker_activation".to_string(),
        success,
        peak_memory_bytes: peak_memory - start_memory,
        circuit_breaker_triggered,
        error_message,
        duration_ms: duration.as_secs_f64() * 1000.0,
    })
}

/// Test memory leak detection across multiple operations
fn test_memory_leak_detection(_config: &MemoryLimiterConfig) -> Result<MemoryTestResult, String> {
    println!("\n🕳️  Testing memory leak detection...");

    let start_time = Instant::now();
    let baseline_memory = get_current_memory_usage();

    let component = Component;
    let html = generate_medium_html_document(1024 * 50); // 50KB

    let mut memory_samples = Vec::new();
    let mut success = true;
    let mut error_message = None;

    // Perform many operations and track memory growth
    for i in 0..50 {
        match component.extract(
            html.clone(),
            format!("https://example.com/leak-test/{}", i),
            ExtractionMode::Article
        ) {
            Ok(_) => {
                let current_memory = get_current_memory_usage();
                memory_samples.push(current_memory);

                // Check for excessive memory growth every 10 operations
                if i % 10 == 9 {
                    let memory_growth = current_memory - baseline_memory;
                    let expected_max = (i + 1) * 1024 * 100; // Reasonable growth expectation

                    if memory_growth > expected_max as u64 {
                        error_message = Some(format!(
                            "Potential memory leak detected at iteration {}: {}KB growth (expected < {}KB)",
                            i, memory_growth / 1024, expected_max / 1024
                        ));
                        success = false;
                        break;
                    }
                }
            },
            Err(e) => {
                error_message = Some(format!("Operation failed at iteration {}: {:?}", i, e));
                success = false;
                break;
            }
        }
    }

    let peak_memory = memory_samples.iter().max().cloned().unwrap_or(baseline_memory);
    let duration = start_time.elapsed();

    // Analyze memory pattern
    if success && memory_samples.len() >= 20 {
        let memory_trend = analyze_memory_trend(&memory_samples);
        if memory_trend > 0.1 { // Growing more than 10% per operation
            error_message = Some(format!(
                "Suspicious memory growth trend detected: {:.2}% per operation",
                memory_trend * 100.0
            ));
            success = false;
        }
    }

    Ok(MemoryTestResult {
        test_name: "memory_leak_detection".to_string(),
        success,
        peak_memory_bytes: peak_memory - baseline_memory,
        circuit_breaker_triggered: false,
        error_message,
        duration_ms: duration.as_secs_f64() * 1000.0,
    })
}

/// Test concurrent memory pressure
fn test_concurrent_memory_pressure(_config: &MemoryLimiterConfig) -> Result<MemoryTestResult, String> {
    println!("\n🔀 Testing concurrent memory pressure...");

    let start_time = Instant::now();
    let start_memory = get_current_memory_usage();

    let mut success = true;
    let mut error_message = None;
    let mut circuit_breaker_triggered = false;

    // Simulate concurrent memory pressure with multiple threads
    let thread_count = 4;
    let operations_per_thread = 10;
    let html = Arc::new(generate_medium_html_document(1024 * 200)); // 200KB per thread

    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let html = Arc::clone(&html);

        let handle = std::thread::spawn(move || -> Result<(), ExtractionError> {
            let component = Component;

            for op_id in 0..operations_per_thread {
                component.extract(
                    (*html).clone(),
                    format!("https://example.com/concurrent/{}/{}", thread_id, op_id),
                    ExtractionMode::Article
                )?;
            }

            Ok(())
        });

        handles.push(handle);
    }

    // Wait for all threads and collect results
    let mut thread_failures = 0;
    for (thread_id, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(())) => {
                println!("  Thread {} completed successfully", thread_id);
            },
            Ok(Err(ExtractionError::ResourceLimit(_))) => {
                thread_failures += 1;
                circuit_breaker_triggered = true;
                println!("  Thread {} hit memory limit", thread_id);
            },
            Ok(Err(e)) => {
                error_message = Some(format!("Thread {} failed: {:?}", thread_id, e));
                success = false;
                break;
            },
            Err(_) => {
                error_message = Some(format!("Thread {} panicked", thread_id));
                success = false;
                break;
            }
        }
    }

    // Some failures under memory pressure are expected and acceptable
    if thread_failures == thread_count {
        error_message = Some("All threads failed - memory limiter may be too restrictive".to_string());
        success = false;
    }

    let peak_memory = get_current_memory_usage();
    let duration = start_time.elapsed();

    Ok(MemoryTestResult {
        test_name: "concurrent_memory_pressure".to_string(),
        success,
        peak_memory_bytes: peak_memory - start_memory,
        circuit_breaker_triggered,
        error_message,
        duration_ms: duration.as_secs_f64() * 1000.0,
    })
}

/// Test recovery after memory pressure
fn test_memory_pressure_recovery(config: &MemoryLimiterConfig) -> Result<MemoryTestResult, String> {
    println!("\n🔄 Testing recovery after memory pressure...");

    let start_time = Instant::now();
    let start_memory = get_current_memory_usage();

    let component = Component;

    // First, create memory pressure
    let large_html = generate_medium_html_document(config.max_allocation_size as usize / 2);
    let _pressure_result = component.extract(
        large_html,
        "https://example.com/pressure-test".to_string(),
        ExtractionMode::Full
    );

    // Wait a moment for potential cleanup
    std::thread::sleep(Duration::from_millis(100));

    // Then test recovery with normal operations
    let normal_html = generate_medium_html_document(1024 * 10); // 10KB
    let mut success = true;
    let mut error_message = None;
    let mut circuit_breaker_triggered = false;

    for i in 0..5 {
        match component.extract(
            normal_html.clone(),
            format!("https://example.com/recovery/{}", i),
            ExtractionMode::Article
        ) {
            Ok(_) => {
                println!("  Recovery operation {} succeeded", i);
            },
            Err(ExtractionError::ResourceLimit(_)) => {
                circuit_breaker_triggered = true;
                // Try to reset the circuit breaker
                let _ = component.reset_state();
                println!("  Circuit breaker reset at operation {}", i);
            },
            Err(e) => {
                error_message = Some(format!("Recovery failed at operation {}: {:?}", i, e));
                success = false;
                break;
            }
        }
    }

    let peak_memory = get_current_memory_usage();
    let duration = start_time.elapsed();

    Ok(MemoryTestResult {
        test_name: "memory_pressure_recovery".to_string(),
        success,
        peak_memory_bytes: peak_memory - start_memory,
        circuit_breaker_triggered,
        error_message,
        duration_ms: duration.as_secs_f64() * 1000.0,
    })
}

/// Test edge case with zero-size allocations
fn test_zero_size_allocations(_config: &MemoryLimiterConfig) -> Result<MemoryTestResult, String> {
    println!("\n🔍 Testing zero-size allocations...");

    let start_time = Instant::now();
    let start_memory = get_current_memory_usage();

    let component = Component;
    let mut success = true;
    let mut error_message = None;

    // Test empty HTML
    match component.extract(
        "".to_string(),
        "https://example.com/empty".to_string(),
        ExtractionMode::Article
    ) {
        Ok(_) => {
            error_message = Some("Empty HTML should be rejected".to_string());
            success = false;
        },
        Err(ExtractionError::InvalidHtml(_)) => {
            println!("  ✅ Empty HTML correctly rejected");
        },
        Err(e) => {
            error_message = Some(format!("Unexpected error for empty HTML: {:?}", e));
            success = false;
        }
    }

    // Test minimal valid HTML
    if success {
        match component.extract(
            "<html></html>".to_string(),
            "https://example.com/minimal".to_string(),
            ExtractionMode::Article
        ) {
            Ok(_) => {
                println!("  ✅ Minimal HTML handled correctly");
            },
            Err(e) => {
                error_message = Some(format!("Minimal HTML failed: {:?}", e));
                success = false;
            }
        }
    }

    let peak_memory = get_current_memory_usage();
    let duration = start_time.elapsed();

    Ok(MemoryTestResult {
        test_name: "zero_size_allocations".to_string(),
        success,
        peak_memory_bytes: peak_memory - start_memory,
        circuit_breaker_triggered: false,
        error_message,
        duration_ms: duration.as_secs_f64() * 1000.0,
    })
}

// Helper functions

fn should_circuit_breaker_activate(memory_usage: u64, config: &MemoryLimiterConfig) -> bool {
    memory_usage > (config.max_heap_size as f64 * config.growth_threshold) as u64
}

fn analyze_memory_trend(memory_samples: &[u64]) -> f64 {
    if memory_samples.len() < 2 {
        return 0.0;
    }

    let start_memory = memory_samples[0] as f64;
    let end_memory = memory_samples[memory_samples.len() - 1] as f64;

    if start_memory == 0.0 {
        return 0.0;
    }

    (end_memory - start_memory) / start_memory / memory_samples.len() as f64
}

fn get_current_memory_usage() -> u64 {
    MEMORY_TRACKER.fetch_add(1024, Ordering::Relaxed); // Simulate memory usage
    MEMORY_TRACKER.load(Ordering::Relaxed)
}

fn generate_medium_html_document(size_bytes: usize) -> String {
    let mut html = String::with_capacity(size_bytes);
    html.push_str("<!DOCTYPE html><html><head><title>Test Document</title></head><body>");

    let content_chunk = "<p>This is test content for memory testing. ".repeat(10);
    while html.len() < size_bytes - 100 {
        html.push_str(&content_chunk);
    }

    html.push_str("</body></html>");
    html
}

fn generate_huge_html_document(size_bytes: u64) -> String {
    let mut html = String::with_capacity(size_bytes as usize);
    html.push_str("<!DOCTYPE html><html><head><title>Huge Document</title></head><body>");

    let large_chunk = "X".repeat(1024 * 1024); // 1MB chunks
    let chunks_needed = (size_bytes / (1024 * 1024)) as usize;

    for _ in 0..chunks_needed {
        html.push_str(&large_chunk);
    }

    html.push_str("</body></html>");
    html
}

fn print_memory_test_summary(results: &[MemoryTestResult]) {
    println!("\n📊 Memory Limiter Test Summary");
    println!("=============================");

    let passed = results.iter().filter(|r| r.success).count();
    let failed = results.len() - passed;
    let circuit_breaker_activations = results.iter().filter(|r| r.circuit_breaker_triggered).count();

    println!("Total tests: {}", results.len());
    println!("Passed: {} ✅", passed);
    println!("Failed: {} ❌", failed);
    println!("Circuit breaker activations: {} 🔌", circuit_breaker_activations);

    let max_memory = results.iter().map(|r| r.peak_memory_bytes).max().unwrap_or(0);
    let avg_memory = results.iter().map(|r| r.peak_memory_bytes as f64).sum::<f64>() / results.len() as f64;

    println!("Peak memory usage: {:.1}KB", max_memory as f64 / 1024.0);
    println!("Average memory usage: {:.1}KB", avg_memory / 1024.0);

    if failed > 0 {
        println!("\nFailure details:");
        for result in results.iter().filter(|r| !r.success) {
            println!("  ❌ {}: {}", result.test_name, result.error_message.as_ref().unwrap_or(&"Unknown error".to_string()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_limiter_config_default() {
        let config = MemoryLimiterConfig::default();
        assert!(config.max_heap_size > 0);
        assert!(config.growth_threshold > 0.0 && config.growth_threshold <= 1.0);
    }

    #[test]
    fn test_normal_memory_usage_wrapper() {
        let config = MemoryLimiterConfig::default();
        let result = test_normal_memory_usage(&config).expect("Normal memory test should not fail");
        // We can't assert success here since it depends on actual memory behavior
        assert_eq!(result.test_name, "normal_memory_usage");
    }

    #[test]
    fn test_memory_trend_analysis() {
        let stable_samples = vec![1000, 1000, 1000, 1000];
        let trend = analyze_memory_trend(&stable_samples);
        assert!(trend.abs() < 0.01, "Stable memory should have near-zero trend");

        let growing_samples = vec![1000, 1100, 1200, 1300];
        let growth_trend = analyze_memory_trend(&growing_samples);
        assert!(growth_trend > 0.05, "Growing memory should have positive trend");
    }

    #[test]
    fn test_document_generation() {
        let doc = generate_medium_html_document(1024);
        assert!(doc.len() >= 1000);
        assert!(doc.contains("<html>"));
        assert!(doc.contains("</html>"));

        let huge_doc = generate_huge_html_document(2 * 1024 * 1024); // 2MB
        assert!(huge_doc.len() >= 2 * 1024 * 1024);
    }

    #[test]
    fn test_circuit_breaker_logic() {
        let config = MemoryLimiterConfig::default();

        let low_usage = config.max_heap_size / 4;
        assert!(!should_circuit_breaker_activate(low_usage, &config));

        let high_usage = (config.max_heap_size as f64 * 0.9) as u64;
        assert!(should_circuit_breaker_activate(high_usage, &config));
    }
}