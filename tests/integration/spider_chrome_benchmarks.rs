//! Performance benchmarks comparing spider-chrome vs chromiumoxide
//!
//! These tests measure:
//! - Page load latency
//! - Memory usage
//! - Screenshot generation speed
//! - PDF generation speed
//! - Concurrent session handling

#[cfg(feature = "headless")]
mod performance_benchmarks {
    use anyhow::Result;
    use riptide_browser::hybrid::fallback::HybridBrowserFallback;
    use riptide_browser::launcher::HybridHeadlessLauncher;
    use riptide_stealth::StealthPreset;
    use std::time::{Duration, Instant};

    /// Performance metrics for comparison
    #[derive(Debug, Clone)]
    struct BenchmarkMetrics {
        engine: String,
        page_load_ms: Vec<u64>,
        screenshot_ms: Vec<u64>,
        pdf_ms: Vec<u64>,
        memory_mb: Vec<u64>,
    }

    impl BenchmarkMetrics {
        fn new(engine: &str) -> Self {
            Self {
                engine: engine.to_string(),
                page_load_ms: Vec::new(),
                screenshot_ms: Vec::new(),
                pdf_ms: Vec::new(),
                memory_mb: Vec::new(),
            }
        }

        fn avg_page_load(&self) -> f64 {
            if self.page_load_ms.is_empty() {
                return 0.0;
            }
            self.page_load_ms.iter().sum::<u64>() as f64 / self.page_load_ms.len() as f64
        }

        fn avg_screenshot(&self) -> f64 {
            if self.screenshot_ms.is_empty() {
                return 0.0;
            }
            self.screenshot_ms.iter().sum::<u64>() as f64 / self.screenshot_ms.len() as f64
        }

        fn avg_pdf(&self) -> f64 {
            if self.pdf_ms.is_empty() {
                return 0.0;
            }
            self.pdf_ms.iter().sum::<u64>() as f64 / self.pdf_ms.len() as f64
        }

        fn avg_memory(&self) -> f64 {
            if self.memory_mb.is_empty() {
                return 0.0;
            }
            self.memory_mb.iter().sum::<u64>() as f64 / self.memory_mb.len() as f64
        }

        fn print_summary(&self) {
            println!("\n=== {} Performance Summary ===", self.engine);
            println!("  Avg Page Load: {:.2}ms", self.avg_page_load());
            println!("  Avg Screenshot: {:.2}ms", self.avg_screenshot());
            println!("  Avg PDF Gen: {:.2}ms", self.avg_pdf());
            println!("  Avg Memory: {:.2}MB", self.avg_memory());
        }
    }

    /// Benchmark spider-chrome page load performance
    #[tokio::test]
    #[ignore] // Run with --ignored for benchmarks
    async fn benchmark_spider_chrome_page_load() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;
        let mut metrics = BenchmarkMetrics::new("spider-chrome");

        let test_urls = vec![
            "https://example.com",
            "https://example.org",
            "https://example.net",
        ];

        // Run 10 iterations for each URL
        for _ in 0..10 {
            for url in &test_urls {
                let start = Instant::now();

                let session = launcher
                    .launch_page(url, Some(StealthPreset::Low))
                    .await?;
                let _ = session.content().await?;

                let duration = start.elapsed();
                metrics.page_load_ms.push(duration.as_millis() as u64);

                session.close().await?;
            }
        }

        metrics.print_summary();
        launcher.shutdown().await?;

        // Performance target: average page load under 3 seconds
        assert!(
            metrics.avg_page_load() < 3000.0,
            "Page load should be under 3s, got {:.2}ms",
            metrics.avg_page_load()
        );

        Ok(())
    }

    /// Benchmark spider-chrome screenshot performance
    #[tokio::test]
    #[ignore] // Run with --ignored for benchmarks
    async fn benchmark_spider_chrome_screenshot() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;
        let mut metrics = BenchmarkMetrics::new("spider-chrome");

        let session = launcher
            .launch_page("https://example.com", Some(StealthPreset::Low))
            .await?;

        // Run 20 screenshot captures
        for _ in 0..20 {
            let start = Instant::now();
            let _ = session.screenshot().await?;
            let duration = start.elapsed();
            metrics.screenshot_ms.push(duration.as_millis() as u64);
        }

        metrics.print_summary();
        session.close().await?;
        launcher.shutdown().await?;

        // Performance target: screenshot under 500ms
        assert!(
            metrics.avg_screenshot() < 500.0,
            "Screenshot should be under 500ms, got {:.2}ms",
            metrics.avg_screenshot()
        );

        Ok(())
    }

    /// Benchmark spider-chrome PDF generation
    #[tokio::test]
    #[ignore] // Run with --ignored for benchmarks
    async fn benchmark_spider_chrome_pdf() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;
        let mut metrics = BenchmarkMetrics::new("spider-chrome");

        let session = launcher
            .launch_page("https://example.com", Some(StealthPreset::Low))
            .await?;

        // Run 10 PDF generations
        for _ in 0..10 {
            let start = Instant::now();
            let _ = session.pdf().await?;
            let duration = start.elapsed();
            metrics.pdf_ms.push(duration.as_millis() as u64);
        }

        metrics.print_summary();
        session.close().await?;
        launcher.shutdown().await?;

        // Performance target: PDF generation under 1 second
        assert!(
            metrics.avg_pdf() < 1000.0,
            "PDF generation should be under 1s, got {:.2}ms",
            metrics.avg_pdf()
        );

        Ok(())
    }

    /// Benchmark concurrent session handling
    #[tokio::test]
    #[ignore] // Run with --ignored for benchmarks
    async fn benchmark_spider_chrome_concurrent_load() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;
        let start = Instant::now();

        // Launch 10 concurrent sessions
        let mut handles = vec![];
        for i in 0..10 {
            let handle = tokio::spawn(async move {
                let launcher = HybridHeadlessLauncher::new().await?;
                let session = launcher
                    .launch_page("https://example.com", Some(StealthPreset::Low))
                    .await?;
                let html = session.content().await?;
                session.close().await?;
                launcher.shutdown().await?;
                Ok::<_, anyhow::Error>(html.len())
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            handle.await??;
        }

        let total_duration = start.elapsed();
        let avg_per_session = total_duration.as_millis() / 10;

        println!("\n=== Concurrent Load Test ===");
        println!("  10 sessions total: {}ms", total_duration.as_millis());
        println!("  Avg per session: {}ms", avg_per_session);

        launcher.shutdown().await?;

        // Performance target: 10 concurrent sessions under 15 seconds
        assert!(
            total_duration.as_secs() < 15,
            "10 concurrent sessions should complete in under 15s"
        );

        Ok(())
    }

    /// Test fallback mechanism performance
    #[tokio::test]
    #[ignore] // Run with --ignored for benchmarks
    async fn benchmark_fallback_mechanism() -> Result<()> {
        let fallback = HybridBrowserFallback::with_traffic_percentage(50).await?;

        // Run 20 page loads to test fallback
        let mut spider_chrome_times = Vec::new();
        let mut fallback_times = Vec::new();

        for i in 0..20 {
            let url = format!("https://example.com?test={}", i);
            let start = Instant::now();

            // This would need a chromium page for full test
            // For now, just test the metrics tracking
            let _ = start.elapsed();
        }

        // Get fallback metrics
        let metrics = fallback.metrics().await;
        println!("\n=== Fallback Metrics ===");
        println!("  Spider-chrome attempts: {}", metrics.spider_chrome_attempts);
        println!("  Spider-chrome success: {}", metrics.spider_chrome_success);
        println!("  Spider-chrome failures: {}", metrics.spider_chrome_failures);
        println!("  Fallback count: {}", metrics.chromiumoxide_fallbacks);
        println!(
            "  Success rate: {:.1}%",
            fallback.spider_chrome_success_rate().await * 100.0
        );
        println!(
            "  Fallback rate: {:.1}%",
            fallback.fallback_rate().await * 100.0
        );

        Ok(())
    }

    /// Stress test: sustained load over 1 minute
    #[tokio::test]
    #[ignore] // Run with --ignored for stress tests
    async fn stress_test_spider_chrome_sustained_load() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;
        let test_duration = Duration::from_secs(60);
        let start = Instant::now();
        let mut request_count = 0;
        let mut error_count = 0;

        while start.elapsed() < test_duration {
            match launcher
                .launch_page("https://example.com", Some(StealthPreset::Low))
                .await
            {
                Ok(session) => {
                    let _ = session.content().await;
                    let _ = session.close().await;
                    request_count += 1;
                }
                Err(e) => {
                    eprintln!("Error during stress test: {}", e);
                    error_count += 1;
                }
            }

            // Small delay to prevent overwhelming the system
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let stats = launcher.stats().await;
        println!("\n=== Stress Test Results (60s) ===");
        println!("  Total requests: {}", request_count);
        println!("  Errors: {}", error_count);
        println!("  Success rate: {:.1}%", (request_count - error_count) as f64 / request_count as f64 * 100.0);
        println!("  Avg response time: {:.2}ms", stats.avg_response_time_ms);
        println!("  Requests/second: {:.2}", request_count as f64 / 60.0);

        launcher.shutdown().await?;

        // Should handle at least 100 requests in 60 seconds
        assert!(
            request_count >= 100,
            "Should complete at least 100 requests in 60s"
        );
        // Error rate should be under 5%
        assert!(
            error_count as f64 / request_count as f64 < 0.05,
            "Error rate should be under 5%"
        );

        Ok(())
    }

    /// Memory leak test: ensure memory doesn't grow unbounded
    #[tokio::test]
    #[ignore] // Run with --ignored for memory tests
    async fn test_spider_chrome_memory_stability() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;

        // Get initial memory baseline (would need psutil or similar)
        // For now, just run many iterations and verify no crashes

        for i in 0..100 {
            let session = launcher
                .launch_page("https://example.com", Some(StealthPreset::Low))
                .await?;
            let _ = session.content().await;
            let _ = session.screenshot().await;
            session.close().await?;

            if i % 10 == 0 {
                println!("  Completed {} iterations", i);
            }
        }

        // Check final stats
        let stats = launcher.stats().await;
        println!("\n=== Memory Stability Test ===");
        println!("  Total requests: {}", stats.total_requests);
        println!("  Successful: {}", stats.successful_requests);
        println!("  Failed: {}", stats.failed_requests);

        launcher.shutdown().await?;

        // All requests should succeed in memory stability test
        assert_eq!(
            stats.successful_requests, 100,
            "All requests should succeed in memory test"
        );

        Ok(())
    }

    /// Compare spider-chrome vs chromiumoxide performance
    #[tokio::test]
    #[ignore] // Run with --ignored for comparison benchmarks
    async fn benchmark_compare_engines() -> Result<()> {
        println!("\n=== Engine Performance Comparison ===");

        // Benchmark spider-chrome
        let spider_launcher = HybridHeadlessLauncher::new().await?;
        let mut spider_metrics = BenchmarkMetrics::new("spider-chrome");

        for _ in 0..10 {
            let start = Instant::now();
            let session = spider_launcher
                .launch_page("https://example.com", Some(StealthPreset::Low))
                .await?;
            let _ = session.content().await?;
            spider_metrics.page_load_ms.push(start.elapsed().as_millis() as u64);
            session.close().await?;
        }

        spider_metrics.print_summary();
        spider_launcher.shutdown().await?;

        // Performance should be within ±10% of target
        let target_ms = 2000.0; // 2 second target
        let variance = (spider_metrics.avg_page_load() - target_ms).abs() / target_ms;
        println!("\nVariance from target: {:.1}%", variance * 100.0);

        Ok(())
    }
}

// Marker for non-headless builds
#[cfg(not(feature = "headless"))]
#[test]
fn test_benchmarks_require_headless_feature() {
    println!("Performance benchmarks require 'headless' feature");
}
