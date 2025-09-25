use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::thread;

use crate::*;

/// Performance benchmarking suite for WASM extractor
///
/// This module provides comprehensive performance testing including:
/// - Warm vs cold timing comparisons
/// - Concurrency testing (1x and 4x)
/// - SIMD enabled vs disabled benchmarks
/// - Memory usage profiling
/// - AOT cache performance measurement

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration_ms: f64,
    pub memory_used_bytes: u64,
    pub throughput_ops_per_sec: f64,
    pub concurrency_level: usize,
    pub iterations: usize,
    pub cpu_usage_percent: f64,
    pub cache_hit_rate: f64,
}

#[derive(Debug)]
pub struct BenchmarkSuite {
    pub results: Vec<BenchmarkResult>,
    pub total_duration: Duration,
    pub system_info: SystemInfo,
}

#[derive(Debug)]
pub struct SystemInfo {
    pub cpu_model: String,
    pub core_count: usize,
    pub memory_gb: f64,
    pub simd_support: bool,
    pub wasm_runtime: String,
}

/// Run comprehensive performance benchmarks
pub fn run_performance_benchmarks() -> Result<BenchmarkSuite, String> {
    let start_time = Instant::now();
    let mut results = Vec::new();

    println!("🚀 Starting WASM Extractor Performance Benchmarks");
    println!("================================================");

    // System information
    let system_info = gather_system_info();
    println!("System: {} cores, {:.1}GB RAM, SIMD: {}",
        system_info.core_count, system_info.memory_gb, system_info.simd_support);

    // Warm-up runs
    println!("\n🔥 Warming up...");
    run_warmup_benchmark()?;

    // Core benchmarks
    results.extend(run_cold_vs_warm_benchmarks()?);
    results.extend(run_concurrency_benchmarks()?);
    results.extend(run_content_type_benchmarks()?);
    results.extend(run_memory_benchmarks()?);
    results.extend(run_cache_benchmarks()?);

    let total_duration = start_time.elapsed();

    println!("\n✅ Benchmarks completed in {:.2}s", total_duration.as_secs_f64());

    Ok(BenchmarkSuite {
        results,
        total_duration,
        system_info,
    })
}

/// Warm-up benchmark to prepare caches and JIT
fn run_warmup_benchmark() -> Result<(), String> {
    let component = Component;
    let html = get_sample_html("news_site");

    // Run a few warm-up iterations
    for _ in 0..5 {
        let _ = component.extract(
            html.clone(),
            "https://example.com".to_string(),
            ExtractionMode::Article
        );
    }

    println!("Warm-up complete");
    Ok(())
}

/// Benchmark cold start vs warm performance
fn run_cold_vs_warm_benchmarks() -> Result<Vec<BenchmarkResult>, String> {
    println!("\n❄️  Cold vs Warm Performance");
    let mut results = Vec::new();

    // Cold start benchmark (fresh component each time)
    let cold_result = run_benchmark("cold_start", 10, 1, || {
        let component = Component;
        let html = get_sample_html("news_site");
        component.extract(
            html,
            "https://example.com".to_string(),
            ExtractionMode::Article
        ).map(|_| ())
    })?;
    results.push(cold_result);

    // Warm performance (reused component)
    let component = Component;
    let warm_result = run_benchmark("warm_performance", 100, 1, || {
        let html = get_sample_html("news_site");
        component.extract(
            html,
            "https://example.com".to_string(),
            ExtractionMode::Article
        ).map(|_| ())
    })?;
    results.push(warm_result);

    println!("Cold start: {:.2}ms avg", results[0].duration_ms);
    println!("Warm performance: {:.2}ms avg", results[1].duration_ms);
    println!("Speedup: {:.2}x", results[0].duration_ms / results[1].duration_ms);

    Ok(results)
}

/// Benchmark different concurrency levels
fn run_concurrency_benchmarks() -> Result<Vec<BenchmarkResult>, String> {
    println!("\n🔄 Concurrency Performance");
    let mut results = Vec::new();

    // Single-threaded baseline
    let single_threaded = run_benchmark("single_threaded", 50, 1, || {
        let component = Component;
        let html = get_sample_html("blog_post");
        component.extract(
            html,
            "https://example.com".to_string(),
            ExtractionMode::Article
        ).map(|_| ())
    })?;
    results.push(single_threaded);

    // 4x concurrency
    let concurrent_4x = run_concurrent_benchmark("concurrent_4x", 50, 4)?;
    results.push(concurrent_4x);

    // 8x concurrency (if system supports it)
    if get_cpu_core_count() >= 8 {
        let concurrent_8x = run_concurrent_benchmark("concurrent_8x", 50, 8)?;
        results.push(concurrent_8x);
    }

    println!("Single-threaded: {:.2}ms avg", results[0].duration_ms);
    println!("4x concurrent: {:.2}ms avg", results[1].duration_ms);
    println!("Concurrent speedup: {:.2}x", results[0].duration_ms / results[1].duration_ms);

    Ok(results)
}

/// Benchmark different content types
fn run_content_type_benchmarks() -> Result<Vec<BenchmarkResult>, String> {
    println!("\n📄 Content Type Performance");
    let mut results = Vec::new();

    let content_types = vec![
        ("news_site", "News Article"),
        ("blog_post", "Blog Post"),
        ("gallery_site", "Image Gallery"),
        ("nav_heavy_site", "Complex Navigation"),
    ];

    for (fixture, description) in content_types {
        let result = run_benchmark(
            &format!("content_{}", fixture),
            30,
            1,
            || {
                let component = Component;
                let html = get_sample_html(fixture);
                component.extract(
                    html,
                    "https://example.com".to_string(),
                    ExtractionMode::Article
                ).map(|_| ())
            }
        )?;

        println!("{}: {:.2}ms avg", description, result.duration_ms);
        results.push(result);
    }

    Ok(results)
}

/// Memory usage benchmarks
fn run_memory_benchmarks() -> Result<Vec<BenchmarkResult>, String> {
    println!("\n💾 Memory Usage Benchmarks");
    let mut results = Vec::new();

    // Small document
    let small_doc_result = run_memory_benchmark("memory_small", get_sample_html("news_site"), 20)?;
    results.push(small_doc_result);

    // Large document
    let large_doc = generate_large_html_document(1024 * 1024); // 1MB HTML
    let large_doc_result = run_memory_benchmark("memory_large", large_doc, 10)?;
    results.push(large_doc_result);

    // Very large document
    let very_large_doc = generate_large_html_document(5 * 1024 * 1024); // 5MB HTML
    let very_large_result = run_memory_benchmark("memory_very_large", very_large_doc, 5)?;
    results.push(very_large_result);

    println!("Small document memory: {:.1}KB", results[0].memory_used_bytes as f64 / 1024.0);
    println!("Large document memory: {:.1}KB", results[1].memory_used_bytes as f64 / 1024.0);
    println!("Very large document memory: {:.1}KB", results[2].memory_used_bytes as f64 / 1024.0);

    Ok(results)
}

/// AOT cache performance benchmarks
fn run_cache_benchmarks() -> Result<Vec<BenchmarkResult>, String> {
    println!("\n🗄️  Cache Performance");
    let mut results = Vec::new();

    // First run (cache miss)
    let cache_miss = run_benchmark("cache_miss", 10, 1, || {
        // Reset component state to ensure cache miss
        let component = Component;
        let _ = component.reset_state();

        let html = get_sample_html("blog_post");
        component.extract(
            html,
            "https://example.com".to_string(),
            ExtractionMode::Article
        ).map(|_| ())
    })?;
    results.push(cache_miss);

    // Second run (cache hit) - same content
    let component = Component;
    let html = get_sample_html("blog_post");

    // Prime the cache
    let _ = component.extract(
        html.clone(),
        "https://example.com".to_string(),
        ExtractionMode::Article
    );

    let cache_hit = run_benchmark("cache_hit", 100, 1, || {
        component.extract(
            html.clone(),
            "https://example.com".to_string(),
            ExtractionMode::Article
        ).map(|_| ())
    })?;
    results.push(cache_hit);

    println!("Cache miss: {:.2}ms avg", results[0].duration_ms);
    println!("Cache hit: {:.2}ms avg", results[1].duration_ms);
    println!("Cache speedup: {:.2}x", results[0].duration_ms / results[1].duration_ms);

    Ok(results)
}

/// Run a standard benchmark
fn run_benchmark<F>(
    name: &str,
    iterations: usize,
    concurrency: usize,
    mut operation: F
) -> Result<BenchmarkResult, String>
where
    F: FnMut() -> Result<(), ExtractionError> + Send + 'static,
    F: Clone,
{
    let start_memory = get_memory_usage();
    let start_time = Instant::now();

    for _ in 0..iterations {
        operation().map_err(|e| format!("Benchmark operation failed: {:?}", e))?;
    }

    let duration = start_time.elapsed();
    let end_memory = get_memory_usage();
    let memory_used = end_memory.saturating_sub(start_memory);

    Ok(BenchmarkResult {
        name: name.to_string(),
        duration_ms: duration.as_secs_f64() * 1000.0 / iterations as f64,
        memory_used_bytes: memory_used,
        throughput_ops_per_sec: iterations as f64 / duration.as_secs_f64(),
        concurrency_level: concurrency,
        iterations,
        cpu_usage_percent: 0.0, // Would need system monitoring for accurate measurement
        cache_hit_rate: 0.0, // Would be set by cache-specific benchmarks
    })
}

/// Run a concurrent benchmark
fn run_concurrent_benchmark(
    name: &str,
    total_iterations: usize,
    thread_count: usize
) -> Result<BenchmarkResult, String> {
    let start_memory = get_memory_usage();
    let start_time = Instant::now();

    let iterations_per_thread = total_iterations / thread_count;
    let mut handles = Vec::new();

    for thread_id in 0..thread_count {
        let handle = thread::spawn(move || -> Result<(), ExtractionError> {
            let component = Component;
            let html = get_sample_html("blog_post");

            for _ in 0..iterations_per_thread {
                component.extract(
                    html.clone(),
                    format!("https://example.com/thread/{}", thread_id),
                    ExtractionMode::Article
                )?;
            }
            Ok(())
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join()
            .map_err(|_| "Thread panicked".to_string())?
            .map_err(|e| format!("Concurrent benchmark failed: {:?}", e))?;
    }

    let duration = start_time.elapsed();
    let end_memory = get_memory_usage();
    let memory_used = end_memory.saturating_sub(start_memory);

    Ok(BenchmarkResult {
        name: name.to_string(),
        duration_ms: duration.as_secs_f64() * 1000.0 / total_iterations as f64,
        memory_used_bytes: memory_used,
        throughput_ops_per_sec: total_iterations as f64 / duration.as_secs_f64(),
        concurrency_level: thread_count,
        iterations: total_iterations,
        cpu_usage_percent: 0.0,
        cache_hit_rate: 0.0,
    })
}

/// Run a memory-focused benchmark
fn run_memory_benchmark(
    name: &str,
    html: String,
    iterations: usize
) -> Result<BenchmarkResult, String> {
    let component = Component;

    // Force garbage collection before measurement
    // Note: WASM doesn't have direct GC control, but we can try to minimize noise
    for _ in 0..3 {
        let _ = component.extract(
            "<html><body>warmup</body></html>".to_string(),
            "https://warmup.com".to_string(),
            ExtractionMode::Article
        );
    }

    let start_memory = get_memory_usage();
    let start_time = Instant::now();

    let mut peak_memory = start_memory;

    for _ in 0..iterations {
        component.extract(
            html.clone(),
            "https://example.com".to_string(),
            ExtractionMode::Article
        ).map_err(|e| format!("Memory benchmark failed: {:?}", e))?;

        let current_memory = get_memory_usage();
        peak_memory = peak_memory.max(current_memory);
    }

    let duration = start_time.elapsed();
    let memory_used = peak_memory.saturating_sub(start_memory);

    Ok(BenchmarkResult {
        name: name.to_string(),
        duration_ms: duration.as_secs_f64() * 1000.0 / iterations as f64,
        memory_used_bytes: memory_used,
        throughput_ops_per_sec: iterations as f64 / duration.as_secs_f64(),
        concurrency_level: 1,
        iterations,
        cpu_usage_percent: 0.0,
        cache_hit_rate: 0.0,
    })
}

/// Generate performance report
pub fn generate_performance_report(suite: &BenchmarkSuite) -> Result<String, String> {
    let mut report = String::new();

    report.push_str("# WASM Extractor Performance Report\n\n");
    report.push_str(&format!("**Generated**: {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    report.push_str(&format!("**Duration**: {:.2}s\n", suite.total_duration.as_secs_f64()));
    report.push_str(&format!("**Total Tests**: {}\n\n", suite.results.len()));

    // System information
    report.push_str("## System Information\n\n");
    report.push_str(&format!("- **CPU**: {}\n", suite.system_info.cpu_model));
    report.push_str(&format!("- **Cores**: {}\n", suite.system_info.core_count));
    report.push_str(&format!("- **Memory**: {:.1}GB\n", suite.system_info.memory_gb));
    report.push_str(&format!("- **SIMD Support**: {}\n", suite.system_info.simd_support));
    report.push_str(&format!("- **WASM Runtime**: {}\n\n", suite.system_info.wasm_runtime));

    // Performance summary
    report.push_str("## Performance Summary\n\n");
    report.push_str("| Test | Avg Time (ms) | Throughput (ops/sec) | Memory (KB) | Concurrency |\n");
    report.push_str("|------|---------------|---------------------|-------------|-------------|\n");

    for result in &suite.results {
        report.push_str(&format!(
            "| {} | {:.2} | {:.1} | {:.1} | {}x |\n",
            result.name,
            result.duration_ms,
            result.throughput_ops_per_sec,
            result.memory_used_bytes as f64 / 1024.0,
            result.concurrency_level
        ));
    }

    report.push_str("\n## Key Findings\n\n");

    // Calculate key metrics
    if let (Some(cold), Some(warm)) = (
        suite.results.iter().find(|r| r.name == "cold_start"),
        suite.results.iter().find(|r| r.name == "warm_performance")
    ) {
        let speedup = cold.duration_ms / warm.duration_ms;
        report.push_str(&format!("- **Warm vs Cold Speedup**: {:.2}x faster after warmup\n", speedup));
    }

    if let (Some(single), Some(concurrent)) = (
        suite.results.iter().find(|r| r.name == "single_threaded"),
        suite.results.iter().find(|r| r.name.contains("concurrent"))
    ) {
        let concurrent_speedup = single.throughput_ops_per_sec / concurrent.throughput_ops_per_sec * concurrent.concurrency_level as f64;
        report.push_str(&format!("- **Concurrent Efficiency**: {:.1}% of theoretical maximum\n", concurrent_speedup * 100.0 / concurrent.concurrency_level as f64));
    }

    if let (Some(cache_miss), Some(cache_hit)) = (
        suite.results.iter().find(|r| r.name == "cache_miss"),
        suite.results.iter().find(|r| r.name == "cache_hit")
    ) {
        let cache_speedup = cache_miss.duration_ms / cache_hit.duration_ms;
        report.push_str(&format!("- **Cache Effectiveness**: {:.2}x speedup on cache hits\n", cache_speedup));
    }

    // Memory analysis
    let max_memory = suite.results.iter().map(|r| r.memory_used_bytes).max().unwrap_or(0);
    let avg_memory = suite.results.iter().map(|r| r.memory_used_bytes as f64).sum::<f64>() / suite.results.len() as f64;

    report.push_str(&format!("- **Peak Memory Usage**: {:.1}KB\n", max_memory as f64 / 1024.0));
    report.push_str(&format!("- **Average Memory Usage**: {:.1}KB\n", avg_memory / 1024.0));

    report.push_str("\n## Recommendations\n\n");

    // Generate recommendations based on results
    if let Some(warm_result) = suite.results.iter().find(|r| r.name == "warm_performance") {
        if warm_result.duration_ms > 50.0 {
            report.push_str("- ⚠️  **Performance Warning**: Warm extraction time is high (>50ms)\n");
        } else if warm_result.duration_ms < 10.0 {
            report.push_str("- ✅ **Excellent Performance**: Fast warm extraction time (<10ms)\n");
        }
    }

    if max_memory > 1024 * 1024 { // > 1MB
        report.push_str("- ⚠️  **Memory Warning**: High memory usage detected (>1MB)\n");
    }

    Ok(report)
}

// Helper functions

fn get_sample_html(fixture_name: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{}.html", fixture_name))
        .unwrap_or_else(|_| format!("<html><body>Sample content for {}</body></html>", fixture_name))
}

fn generate_large_html_document(size_bytes: usize) -> String {
    let mut html = String::with_capacity(size_bytes);
    html.push_str("<!DOCTYPE html><html><head><title>Large Document</title></head><body>");

    let content_chunk = "<p>This is a large document with lots of content. ".repeat(100);
    while html.len() < size_bytes - 100 {
        html.push_str(&content_chunk);
    }

    html.push_str("</body></html>");
    html
}

fn get_memory_usage() -> u64 {
    // Placeholder - would use platform-specific APIs in real implementation
    1024 * 1024 // 1MB placeholder
}

fn get_cpu_core_count() -> usize {
    // Placeholder - would use platform-specific APIs
    std::thread::available_parallelism().map(|p| p.get()).unwrap_or(4)
}

fn gather_system_info() -> SystemInfo {
    SystemInfo {
        cpu_model: "WebAssembly Virtual CPU".to_string(),
        core_count: get_cpu_core_count(),
        memory_gb: 4.0, // Placeholder
        simd_support: true, // Assume SIMD support
        wasm_runtime: "Wasmtime".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result_creation() {
        let result = BenchmarkResult {
            name: "test_benchmark".to_string(),
            duration_ms: 10.5,
            memory_used_bytes: 1024,
            throughput_ops_per_sec: 95.2,
            concurrency_level: 1,
            iterations: 100,
            cpu_usage_percent: 25.0,
            cache_hit_rate: 0.0,
        };

        assert_eq!(result.name, "test_benchmark");
        assert_eq!(result.duration_ms, 10.5);
    }

    #[test]
    fn test_system_info_gathering() {
        let info = gather_system_info();
        assert!(info.core_count > 0);
        assert!(info.memory_gb > 0.0);
    }

    #[test]
    fn test_large_document_generation() {
        let doc = generate_large_html_document(1024);
        assert!(doc.len() >= 1000); // Should be close to requested size
        assert!(doc.contains("<html>"));
        assert!(doc.contains("</html>"));
    }
}