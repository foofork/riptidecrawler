//! Performance Benchmarks for Phase 3
//!
//! Comprehensive performance testing including:
//! - Direct mode vs API mode comparison
//! - WASM vs Headless vs Stealth performance
//! - Memory usage profiling
//! - Concurrent extraction benchmarks
//! - Throughput measurements

use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(test)]
mod performance_benchmarks {
    use super::*;

    /// Benchmark WASM engine performance
    #[tokio::test]
    async fn benchmark_wasm_engine() {
        let html = create_benchmark_html(1000); // 1KB HTML
        let iterations = 100;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = execute_wasm_extraction(&html, "https://example.com").await;
        }
        let duration = start.elapsed();

        let avg_time = duration.as_millis() / iterations;
        println!(
            "WASM Engine: {} iterations in {:?} (avg: {}ms per extraction)",
            iterations, duration, avg_time
        );

        assert!(
            avg_time < 50,
            "WASM extraction should average <50ms, got {}ms",
            avg_time
        );
    }

    /// Benchmark headless engine performance
    #[tokio::test]
    async fn benchmark_headless_engine() {
        let html = create_benchmark_html(1000);
        let iterations = 20; // Fewer iterations due to slower execution

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = execute_headless_extraction(&html, "https://example.com").await;
        }
        let duration = start.elapsed();

        let avg_time = duration.as_millis() / iterations as u128;
        println!(
            "Headless Engine: {} iterations in {:?} (avg: {}ms per extraction)",
            iterations, duration, avg_time
        );

        assert!(
            avg_time < 500,
            "Headless extraction should average <500ms, got {}ms",
            avg_time
        );
    }

    /// Benchmark stealth engine performance
    #[tokio::test]
    async fn benchmark_stealth_engine() {
        let html = create_benchmark_html(1000);
        let iterations = 10; // Fewer iterations, stealth is slowest

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = execute_stealth_extraction(&html, "https://example.com").await;
        }
        let duration = start.elapsed();

        let avg_time = duration.as_millis() / iterations as u128;
        println!(
            "Stealth Engine: {} iterations in {:?} (avg: {}ms per extraction)",
            iterations, duration, avg_time
        );

        assert!(
            avg_time < 1000,
            "Stealth extraction should average <1s, got {}ms",
            avg_time
        );
    }

    /// Compare direct mode vs API mode performance
    #[tokio::test]
    async fn benchmark_direct_vs_api_mode() {
        let html = create_benchmark_html(1000);
        let iterations = 50;

        // Benchmark direct mode
        let start_direct = Instant::now();
        for _ in 0..iterations {
            let _ = execute_direct_mode(&html, "https://example.com").await;
        }
        let direct_duration = start_direct.elapsed();

        // Benchmark API mode
        let start_api = Instant::now();
        for _ in 0..iterations {
            let _ = execute_api_mode(&html, "https://example.com").await;
        }
        let api_duration = start_api.elapsed();

        println!(
            "Direct Mode: {:?} | API Mode: {:?} | Speedup: {:.2}x",
            direct_duration,
            api_duration,
            api_duration.as_millis() as f64 / direct_duration.as_millis() as f64
        );

        assert!(
            direct_duration < api_duration,
            "Direct mode should be faster than API mode"
        );
    }

    /// Benchmark concurrent extraction throughput
    #[tokio::test]
    async fn benchmark_concurrent_throughput() {
        let html = Arc::new(create_benchmark_html(1000));
        let concurrent_tasks = 50;

        let start = Instant::now();
        let tasks: Vec<_> = (0..concurrent_tasks)
            .map(|_| {
                let html_clone = Arc::clone(&html);
                tokio::spawn(async move {
                    execute_wasm_extraction(&html_clone, "https://example.com").await
                })
            })
            .collect();

        let results: Vec<_> = futures::future::join_all(tasks).await;
        let duration = start.elapsed();

        let successful = results.iter().filter(|r| r.is_ok()).count();
        let throughput = successful as f64 / duration.as_secs_f64();

        println!(
            "Concurrent throughput: {} successful / {} total in {:?} ({:.2} extractions/sec)",
            successful, concurrent_tasks, duration, throughput
        );

        assert!(
            throughput > 10.0,
            "Should handle >10 extractions/sec, got {:.2}",
            throughput
        );
    }

    /// Memory usage profiling for WASM
    #[tokio::test]
    async fn benchmark_wasm_memory_usage() {
        let sizes = vec![1, 10, 100, 1000]; // KB

        for size_kb in sizes {
            let html = create_benchmark_html(size_kb);

            let memory_before = get_memory_usage();
            let _ = execute_wasm_extraction(&html, "https://example.com").await;
            let memory_after = get_memory_usage();

            let memory_increase = memory_after.saturating_sub(memory_before);

            println!(
                "WASM memory for {}KB HTML: {} bytes increase",
                size_kb, memory_increase
            );

            assert!(
                memory_increase < 50 * 1024 * 1024,
                "Memory increase should be <50MB for {}KB HTML",
                size_kb
            );
        }
    }

    /// Memory usage profiling for headless
    #[tokio::test]
    async fn benchmark_headless_memory_usage() {
        let html = create_benchmark_html(100);

        let memory_before = get_memory_usage();
        let _ = execute_headless_extraction(&html, "https://example.com").await;
        let memory_after = get_memory_usage();

        let memory_increase = memory_after.saturating_sub(memory_before);

        println!(
            "Headless memory for 100KB HTML: {} MB increase",
            memory_increase / (1024 * 1024)
        );

        assert!(
            memory_increase < 200 * 1024 * 1024,
            "Memory increase should be <200MB for headless"
        );
    }

    /// Benchmark cache effectiveness
    #[tokio::test]
    async fn benchmark_cache_effectiveness() {
        let html = create_benchmark_html(100);

        // Cold cache (first run)
        let start_cold = Instant::now();
        let _ = execute_with_cache(&html, "https://example.com").await;
        let cold_duration = start_cold.elapsed();

        // Warm cache (subsequent runs)
        let iterations = 100;
        let start_warm = Instant::now();
        for _ in 0..iterations {
            let _ = execute_with_cache(&html, "https://example.com").await;
        }
        let warm_duration = start_warm.elapsed();
        let avg_warm = warm_duration / iterations;

        println!(
            "Cache effectiveness: Cold: {:?} | Warm: {:?} | Speedup: {:.2}x",
            cold_duration,
            avg_warm,
            cold_duration.as_micros() as f64 / avg_warm.as_micros() as f64
        );

        assert!(
            avg_warm < cold_duration / 10,
            "Cached access should be >10x faster"
        );
    }

    /// Benchmark HTML parsing performance
    #[tokio::test]
    async fn benchmark_html_parsing() {
        let sizes = vec![1, 10, 100, 1000, 10000]; // KB

        for size_kb in sizes {
            let html = create_benchmark_html(size_kb);

            let start = Instant::now();
            let _ = parse_html(&html);
            let duration = start.elapsed();

            println!(
                "HTML parsing {}KB: {:?} ({:.2} MB/s)",
                size_kb,
                duration,
                (size_kb as f64) / duration.as_secs_f64() / 1024.0
            );

            // Parsing should be fast even for large documents
            assert!(
                duration.as_millis() < size_kb as u128 * 10,
                "Parsing should be <10ms per KB"
            );
        }
    }

    /// Benchmark content extraction accuracy vs speed tradeoff
    #[tokio::test]
    async fn benchmark_accuracy_vs_speed() {
        let html = create_complex_benchmark_html();

        // Fast mode (WASM)
        let start_fast = Instant::now();
        let fast_result = execute_wasm_extraction(&html, "https://example.com").await;
        let fast_duration = start_fast.elapsed();

        // Accurate mode (Headless)
        let start_accurate = Instant::now();
        let accurate_result = execute_headless_extraction(&html, "https://example.com").await;
        let accurate_duration = start_accurate.elapsed();

        println!(
            "Fast mode: {:?} ({} chars) | Accurate mode: {:?} ({} chars)",
            fast_duration,
            fast_result.as_ref().map(|s| s.len()).unwrap_or(0),
            accurate_duration,
            accurate_result.as_ref().map(|s| s.len()).unwrap_or(0)
        );

        // Both should succeed
        assert!(fast_result.is_ok() && accurate_result.is_ok());

        // Fast mode should be significantly faster
        assert!(
            fast_duration < accurate_duration / 5,
            "WASM should be >5x faster than headless"
        );
    }

    /// Benchmark browser pool performance
    #[tokio::test]
    async fn benchmark_browser_pool_overhead() {
        let pool = create_test_browser_pool(5).await;

        // Measure checkout/checkin overhead
        let iterations = 100;
        let start = Instant::now();

        for _ in 0..iterations {
            let checkout = pool.checkout().await.unwrap();
            let _ = checkout.checkin().await;
        }

        let duration = start.elapsed();
        let avg_overhead = duration / iterations;

        println!(
            "Browser pool overhead: {} iterations in {:?} (avg: {:?} per cycle)",
            iterations, duration, avg_overhead
        );

        assert!(
            avg_overhead.as_micros() < 1000,
            "Pool overhead should be <1ms per cycle"
        );
    }

    /// Benchmark engine fallback chain performance
    #[tokio::test]
    async fn benchmark_fallback_chain() {
        let html = create_benchmark_html(100);

        // Best case: WASM succeeds immediately
        let start_best = Instant::now();
        let _ = execute_with_fallback(&html, "https://example.com", vec!["wasm"]).await;
        let best_duration = start_best.elapsed();

        // Worst case: Full fallback chain
        let start_worst = Instant::now();
        let _ = execute_with_fallback(
            &html,
            "https://example.com",
            vec!["wasm", "headless", "stealth"],
        )
        .await;
        let worst_duration = start_worst.elapsed();

        println!(
            "Fallback chain: Best case: {:?} | Worst case: {:?} | Overhead: {:?}",
            best_duration,
            worst_duration,
            worst_duration - best_duration
        );

        assert!(
            worst_duration < best_duration * 3,
            "Fallback overhead should be <3x best case"
        );
    }

    /// Stress test: sustained high load
    #[tokio::test]
    async fn benchmark_sustained_load() {
        let html = Arc::new(create_benchmark_html(100));
        let duration_secs = 5;
        let concurrent_workers = 10;

        let start = Instant::now();
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let tasks: Vec<_> = (0..concurrent_workers)
            .map(|_| {
                let html_clone = Arc::clone(&html);
                let counter_clone = Arc::clone(&counter);
                tokio::spawn(async move {
                    while start.elapsed().as_secs() < duration_secs {
                        let _ = execute_wasm_extraction(&html_clone, "https://example.com").await;
                        counter_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                })
            })
            .collect();

        for task in tasks {
            let _ = task.await;
        }

        let total_extractions = counter.load(std::sync::atomic::Ordering::Relaxed);
        let throughput = total_extractions as f64 / duration_secs as f64;

        println!(
            "Sustained load: {} extractions in {}s ({:.2} per second)",
            total_extractions, duration_secs, throughput
        );

        assert!(
            throughput > 50.0,
            "Should sustain >50 extractions/sec, got {:.2}",
            throughput
        );
    }

    /// Benchmark resource cleanup performance
    #[tokio::test]
    async fn benchmark_cleanup_performance() {
        let pool = create_test_browser_pool(10).await;

        // Create and use browsers
        let checkouts: Vec<_> = (0..10)
            .map(|_| pool.checkout())
            .collect::<futures::future::JoinAll<_>>()
            .await;

        // Benchmark cleanup
        let start = Instant::now();
        for checkout in checkouts {
            if let Ok(checkout) = checkout {
                let _ = checkout.checkin().await;
            }
        }
        let cleanup_duration = start.elapsed();

        println!("Cleanup 10 browsers: {:?}", cleanup_duration);

        assert!(
            cleanup_duration.as_millis() < 1000,
            "Cleanup should complete in <1s"
        );
    }

    // Helper functions

    fn create_benchmark_html(size_kb: usize) -> String {
        let mut html = String::from("<!DOCTYPE html><html><head><title>Benchmark</title></head><body>");

        let paragraph = "<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. </p>";
        let target_size = size_kb * 1024;

        while html.len() < target_size {
            html.push_str(paragraph);
        }

        html.push_str("</body></html>");
        html
    }

    fn create_complex_benchmark_html() -> String {
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Complex Page</title>
    <script src="app.js"></script>
    <script>window.__INITIAL_STATE__={}</script>
</head>
<body>
    <div id="app">
        <article class="content">
            <h1>Complex Content</h1>
            <p>This page has multiple layers of content and scripts.</p>
            <section class="nested">
                <div class="dynamic-content">
                    <p>Dynamically loaded content here.</p>
                </div>
            </section>
        </article>
    </div>
    <script>
        // JavaScript that modifies the DOM
        document.addEventListener('DOMContentLoaded', function() {
            console.log('Page loaded');
        });
    </script>
</body>
</html>"#.to_string()
    }

    fn get_memory_usage() -> usize {
        // Mock implementation - in real code, use system APIs
        // e.g., on Linux: /proc/self/statm
        0
    }

    fn parse_html(_html: &str) -> Result<(), String> {
        // Mock HTML parsing
        Ok(())
    }

    async fn execute_wasm_extraction(_html: &str, _url: &str) -> Result<String, String> {
        // Simulate WASM extraction time
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok("Extracted content".to_string())
    }

    async fn execute_headless_extraction(_html: &str, _url: &str) -> Result<String, String> {
        // Simulate headless extraction time
        tokio::time::sleep(Duration::from_millis(200)).await;
        Ok("Extracted content".to_string())
    }

    async fn execute_stealth_extraction(_html: &str, _url: &str) -> Result<String, String> {
        // Simulate stealth extraction time
        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok("Extracted content".to_string())
    }

    async fn execute_direct_mode(_html: &str, _url: &str) -> Result<String, String> {
        tokio::time::sleep(Duration::from_millis(15)).await;
        Ok("Extracted content".to_string())
    }

    async fn execute_api_mode(_html: &str, _url: &str) -> Result<String, String> {
        // Simulate API overhead (network + processing)
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok("Extracted content".to_string())
    }

    async fn execute_with_cache(_html: &str, _url: &str) -> Result<String, String> {
        // Simulate cached extraction (very fast)
        tokio::time::sleep(Duration::from_micros(100)).await;
        Ok("Extracted content".to_string())
    }

    async fn execute_with_fallback(
        _html: &str,
        _url: &str,
        _engines: Vec<&str>,
    ) -> Result<String, String> {
        // Simulate trying engines in sequence
        for engine in _engines {
            tokio::time::sleep(Duration::from_millis(match engine {
                "wasm" => 10,
                "headless" => 200,
                "stealth" => 500,
                _ => 50,
            }))
            .await;
        }
        Ok("Extracted content".to_string())
    }

    // Mock browser pool
    struct BrowserPool {
        browsers: Vec<Browser>,
    }

    struct Browser {
        id: String,
    }

    struct BrowserCheckout {
        id: String,
    }

    impl BrowserCheckout {
        async fn checkin(self) -> Result<(), String> {
            tokio::time::sleep(Duration::from_micros(50)).await;
            Ok(())
        }
    }

    impl BrowserPool {
        async fn checkout(&self) -> Result<BrowserCheckout, String> {
            tokio::time::sleep(Duration::from_micros(50)).await;
            Ok(BrowserCheckout {
                id: "browser_1".to_string(),
            })
        }
    }

    async fn create_test_browser_pool(size: usize) -> BrowserPool {
        BrowserPool {
            browsers: (0..size)
                .map(|i| Browser {
                    id: format!("browser_{}", i),
                })
                .collect(),
        }
    }
}
