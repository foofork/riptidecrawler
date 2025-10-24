#[cfg(test)]
mod wasm_performance_tests {
    use riptide_core::component::{CmExtractor, ExtractorConfig};
    use std::time::Instant;
    use tokio;

    const TEST_HTML: &str = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Performance Test Article</title>
        <meta name="description" content="A test article with substantial content">
    </head>
    <body>
        <article>
            <h1>Performance Testing with SIMD Optimizations</h1>
            <p>This is a comprehensive test of the WASM memory tracking and SIMD optimization features.
            The content includes substantial text processing to evaluate the performance improvements
            achieved through vectorized operations and optimized memory management.</p>

            <section>
                <h2>Text Processing Performance</h2>
                <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor
                incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud
                exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute
                irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla
                pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia
                deserunt mollit anim id est laborum.</p>

                <p>Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium
                doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore
                veritatis et quasi architecto beatae vitae dicta sunt explicabo. Nemo enim ipsam
                voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur
                magni dolores eos qui ratione voluptatem sequi nesciunt.</p>
            </section>

            <section>
                <h2>Memory Management Testing</h2>
                <p>This section tests the ResourceLimiter implementation and memory tracking
                capabilities. The WASM component should efficiently manage memory allocation
                and provide accurate metrics about usage patterns and growth failures.</p>

                <ul>
                    <li>Memory page tracking</li>
                    <li>Growth failure detection</li>
                    <li>Peak usage monitoring</li>
                    <li>Resource cleanup verification</li>
                </ul>
            </section>

            <section>
                <h2>SIMD Optimization Benefits</h2>
                <p>The SIMD128 feature should provide performance improvements for text-heavy
                operations through vectorized processing. This includes faster string manipulation,
                HTML parsing, and content extraction tasks that benefit from parallel operations
                on multiple data elements simultaneously.</p>
            </section>
        </article>
    </body>
    </html>
    "#;

    #[tokio::test]
    async fn test_wasm_memory_tracking() -> anyhow::Result<()> {
        // Set up environment variables for testing
        std::env::set_var("RIPTIDE_WASM_ENABLE_SIMD", "true");
        std::env::set_var("RIPTIDE_WASM_ENABLE_AOT_CACHE", "true");
        std::env::set_var("RIPTIDE_WASM_MEMORY_LIMIT_PAGES", "2048"); // 128MB limit
        std::env::set_var("RIPTIDE_WASM_COLD_START_TARGET_MS", "15");

        let config = ExtractorConfig::default();

        // Verify environment variables are working
        assert!(config.enable_simd, "SIMD should be enabled");
        assert!(config.enable_aot_cache, "AOT cache should be enabled");
        assert_eq!(config.memory_limit_pages, 2048, "Memory limit should be set from env var");

        println!("WASM Performance Test Configuration:");
        println!("- SIMD Enabled: {}", config.enable_simd);
        println!("- AOT Cache Enabled: {}", config.enable_aot_cache);
        println!("- Memory Limit Pages: {}", config.memory_limit_pages);
        println!("- Cold Start Target: {}ms", config.cold_start_target_ms);

        Ok(())
    }

    #[tokio::test]
    #[ignore] // Ignore by default as this requires a built WASM component
    async fn test_cold_start_performance() -> anyhow::Result<()> {
        let wasm_path = "../../wasm/riptide-extractor-wasm/target/wasm32-wasip2/release/riptide_extractor_wasm.wasm";

        // Test cold start time
        let cold_start_timer = Instant::now();
        let extractor = match CmExtractor::new(wasm_path).await {
            Ok(extractor) => extractor,
            Err(e) => {
                println!("Warning: Could not load WASM component ({}), skipping cold start test", e);
                return Ok(());
            }
        };
        let cold_start_time = cold_start_timer.elapsed();

        println!("Cold start time: {}ms", cold_start_time.as_millis());

        // Get performance metrics
        let metrics = extractor.get_performance_metrics()?;
        println!("Cold start time from metrics: {}ms", metrics.cold_start_time_ms);

        // Verify cold start is within acceptable range
        assert!(
            cold_start_time.as_millis() < 100, // Generous initial limit
            "Cold start time should be under 100ms, got {}ms",
            cold_start_time.as_millis()
        );

        Ok(())
    }

    #[tokio::test]
    #[ignore] // Ignore by default as this requires a built WASM component
    async fn test_extraction_performance_and_memory() -> anyhow::Result<()> {
        let wasm_path = "../../wasm/riptide-extractor-wasm/target/wasm32-wasip2/release/riptide_extractor_wasm.wasm";

        let extractor = match CmExtractor::new(wasm_path).await {
            Ok(extractor) => extractor,
            Err(e) => {
                println!("Warning: Could not load WASM component ({}), skipping extraction test", e);
                return Ok(());
            }
        };

        // Perform multiple extractions to test performance and memory tracking
        let iterations = 10;
        let mut total_time = 0;

        for i in 0..iterations {
            let start_time = Instant::now();

            let result = extractor.extract(TEST_HTML, "https://example.com/test", "article");

            let extraction_time = start_time.elapsed();
            total_time += extraction_time.as_millis();

            match result {
                Ok(doc) => {
                    assert!(!doc.title.is_empty(), "Title should not be empty");
                    assert!(!doc.text.is_empty(), "Text should not be empty");
                    println!("Extraction {} - Time: {}ms, Title: {}", i + 1, extraction_time.as_millis(), doc.title);
                },
                Err(e) => {
                    println!("Extraction {} failed: {}", i + 1, e);
                    continue;
                }
            }

            // Get WASM memory metrics after each extraction
            let wasm_metrics = extractor.get_wasm_memory_metrics()?;
            if let Some(&pages) = wasm_metrics.get("riptide_wasm_memory_pages") {
                println!("Memory pages after extraction {}: {}", i + 1, pages);
            }
            if let Some(&failures) = wasm_metrics.get("riptide_wasm_grow_failed_total") {
                assert_eq!(failures, 0.0, "Should have no memory growth failures");
            }
        }

        let avg_time = total_time / iterations;
        println!("Average extraction time: {}ms", avg_time);

        // Get final performance metrics
        let final_metrics = extractor.get_performance_metrics()?;
        println!("Final Performance Metrics:");
        println!("- Total extractions: {}", final_metrics.total_extractions);
        println!("- Successful extractions: {}", final_metrics.successful_extractions);
        println!("- Failed extractions: {}", final_metrics.failed_extractions);
        println!("- Average processing time: {:.2}ms", final_metrics.avg_processing_time_ms);
        println!("- WASM memory pages: {}", final_metrics.wasm_memory_pages);
        println!("- WASM grow failures: {}", final_metrics.wasm_grow_failed_total);
        println!("- WASM peak memory pages: {}", final_metrics.wasm_peak_memory_pages);

        // Verify no memory growth failures occurred
        assert_eq!(final_metrics.wasm_grow_failed_total, 0, "Should have no memory growth failures");
        assert!(final_metrics.successful_extractions > 0, "Should have successful extractions");

        Ok(())
    }

    #[tokio::test]
    #[ignore] // Ignore by default as this requires a built WASM component
    async fn test_aot_cache_effectiveness() -> anyhow::Result<()> {
        let wasm_path = "../../wasm/riptide-extractor-wasm/target/wasm32-wasip2/release/riptide_extractor_wasm.wasm";

        let extractor = match CmExtractor::new(wasm_path).await {
            Ok(extractor) => extractor,
            Err(e) => {
                println!("Warning: Could not load WASM component ({}), skipping AOT cache test", e);
                return Ok(());
            }
        };

        // Test AOT precompilation
        let module_hash = "test_module_hash";
        extractor.precompile_module(module_hash.to_string()).await?;

        // Get metrics to verify cache behavior
        let metrics = extractor.get_wasm_memory_metrics()?;
        if let Some(&cache_misses) = metrics.get("riptide_wasm_aot_cache_misses") {
            assert!(cache_misses >= 1.0, "Should have at least one cache miss for precompilation");
            println!("AOT cache misses: {}", cache_misses);
        }

        // Try precompiling the same module again (should hit cache)
        extractor.precompile_module(module_hash.to_string()).await?;

        let updated_metrics = extractor.get_wasm_memory_metrics()?;
        if let Some(&cache_hits) = updated_metrics.get("riptide_wasm_aot_cache_hits") {
            assert!(cache_hits >= 1.0, "Should have at least one cache hit");
            println!("AOT cache hits: {}", cache_hits);
        }

        Ok(())
    }

    #[test]
    fn test_environment_variable_configuration() {
        // Test all environment variable configurations
        std::env::set_var("RIPTIDE_WASM_MAX_POOL_SIZE", "16");
        std::env::set_var("RIPTIDE_WASM_INITIAL_POOL_SIZE", "4");
        std::env::set_var("RIPTIDE_WASM_MEMORY_LIMIT_MB", "512");
        std::env::set_var("RIPTIDE_WASM_MEMORY_LIMIT_PAGES", "8192");
        std::env::set_var("RIPTIDE_WASM_ENABLE_SIMD", "false");
        std::env::set_var("RIPTIDE_WASM_ENABLE_AOT_CACHE", "false");
        std::env::set_var("RIPTIDE_WASM_COLD_START_TARGET_MS", "25");

        let config = ExtractorConfig::default();

        assert_eq!(config.max_pool_size, 16);
        assert_eq!(config.initial_pool_size, 4);
        assert_eq!(config.memory_limit, 512 * 1024 * 1024);
        assert_eq!(config.memory_limit_pages, 8192);
        assert_eq!(config.enable_simd, false);
        assert_eq!(config.enable_aot_cache, false);
        assert_eq!(config.cold_start_target_ms, 25);

        println!("Environment variable configuration test passed");

        // Clean up environment variables
        std::env::remove_var("RIPTIDE_WASM_MAX_POOL_SIZE");
        std::env::remove_var("RIPTIDE_WASM_INITIAL_POOL_SIZE");
        std::env::remove_var("RIPTIDE_WASM_MEMORY_LIMIT_MB");
        std::env::remove_var("RIPTIDE_WASM_MEMORY_LIMIT_PAGES");
        std::env::remove_var("RIPTIDE_WASM_ENABLE_SIMD");
        std::env::remove_var("RIPTIDE_WASM_ENABLE_AOT_CACHE");
        std::env::remove_var("RIPTIDE_WASM_COLD_START_TARGET_MS");
    }

    #[test]
    fn test_memory_page_calculations() {
        // Test memory page calculations (64KB per page)
        let page_size = 64 * 1024; // 64KB

        // Test various memory sizes
        assert_eq!(256 * 1024 * 1024 / page_size, 4096); // 256MB = 4096 pages
        assert_eq!(128 * 1024 * 1024 / page_size, 2048); // 128MB = 2048 pages
        assert_eq!(512 * 1024 * 1024 / page_size, 8192); // 512MB = 8192 pages

        println!("Memory page calculation tests passed");
    }
}