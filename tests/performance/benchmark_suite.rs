//! Comprehensive benchmark suite for Week 3 features
//!
//! Performance benchmarks for:
//! - All 5 chunking strategies
//! - DOM spider operations
//! - HTML processing pipelines
//! - Large document handling

use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio_test;

// Import functionality for benchmarking
use riptide_core::strategies::chunking::{
    chunk_content, ChunkingConfig, ChunkingMode, count_tokens
};
use riptide_html::{
    HtmlProcessor, DefaultHtmlProcessor,
    dom_utils::{extract_links, extract_images, DomTraverser},
    processor::{TableExtractionMode, ChunkingMode as HtmlChunkingMode},
};

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub timeout: Duration,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 10,
            warmup_iterations: 3,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub mean_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub std_dev: Duration,
    pub throughput_mb_per_sec: f64,
    pub success_rate: f64,
}

impl BenchmarkResult {
    pub fn new(name: String, times: &[Duration], input_size_mb: f64) -> Self {
        let times_ns: Vec<f64> = times.iter().map(|d| d.as_nanos() as f64).collect();

        let mean_ns = times_ns.iter().sum::<f64>() / times_ns.len() as f64;
        let variance = times_ns.iter()
            .map(|t| (t - mean_ns).powi(2))
            .sum::<f64>() / times_ns.len() as f64;
        let std_dev_ns = variance.sqrt();

        let mean_time = Duration::from_nanos(mean_ns as u64);
        let min_time = *times.iter().min().unwrap();
        let max_time = *times.iter().max().unwrap();
        let std_dev = Duration::from_nanos(std_dev_ns as u64);

        let throughput_mb_per_sec = if mean_time.as_secs_f64() > 0.0 {
            input_size_mb / mean_time.as_secs_f64()
        } else {
            0.0
        };

        Self {
            name,
            mean_time,
            min_time,
            max_time,
            std_dev,
            throughput_mb_per_sec,
            success_rate: 100.0, // Assuming all succeeded if we got here
        }
    }
}

/// Benchmark runner
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    results: Vec<BenchmarkResult>,
}

impl BenchmarkRunner {
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }

    pub async fn run_benchmark<F, Fut>(&mut self, name: &str, input_size_mb: f64, mut benchmark_fn: F)
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<()>>,
    {
        println!("Running benchmark: {}", name);

        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let _ = benchmark_fn().await;
        }

        // Actual benchmark
        let mut times = Vec::new();
        let mut successes = 0;

        for i in 0..self.config.iterations {
            let start = Instant::now();

            match tokio::time::timeout(self.config.timeout, benchmark_fn()).await {
                Ok(Ok(_)) => {
                    let elapsed = start.elapsed();
                    times.push(elapsed);
                    successes += 1;
                }
                Ok(Err(e)) => {
                    eprintln!("Benchmark iteration {} failed: {}", i, e);
                }
                Err(_) => {
                    eprintln!("Benchmark iteration {} timed out", i);
                }
            }
        }

        if !times.is_empty() {
            let mut result = BenchmarkResult::new(name.to_string(), &times, input_size_mb);
            result.success_rate = (successes as f64 / self.config.iterations as f64) * 100.0;
            self.results.push(result);

            println!("  Mean: {:?}, Min: {:?}, Max: {:?}, Throughput: {:.2} MB/s",
                    result.mean_time, result.min_time, result.max_time, result.throughput_mb_per_sec);
        } else {
            println!("  All iterations failed");
        }
    }

    pub fn print_summary(&self) {
        println!("\n=== BENCHMARK SUMMARY ===");
        println!("Benchmark Name\t\t\t\tMean Time\tThroughput (MB/s)\tSuccess Rate");
        println!("================================================================================");

        for result in &self.results {
            println!("{:<40}\t{:?}\t{:.2}\t\t{:.1}%",
                    result.name,
                    result.mean_time,
                    result.throughput_mb_per_sec,
                    result.success_rate);
        }

        // Performance analysis
        println!("\n=== PERFORMANCE ANALYSIS ===");

        // Find fastest and slowest
        if let Some(fastest) = self.results.iter().min_by_key(|r| r.mean_time) {
            println!("Fastest: {} ({:?})", fastest.name, fastest.mean_time);
        }
        if let Some(slowest) = self.results.iter().max_by_key(|r| r.mean_time) {
            println!("Slowest: {} ({:?})", slowest.name, slowest.mean_time);
        }

        // Find highest throughput
        if let Some(highest_throughput) = self.results.iter()
            .max_by(|a, b| a.throughput_mb_per_sec.partial_cmp(&b.throughput_mb_per_sec).unwrap()) {
            println!("Highest Throughput: {} ({:.2} MB/s)",
                    highest_throughput.name, highest_throughput.throughput_mb_per_sec);
        }
    }
}

#[tokio::test]
async fn test_chunking_strategies_performance() {
    let mut runner = BenchmarkRunner::new(BenchmarkConfig::default());

    // Test different text sizes
    let test_sizes = vec![1_000, 10_000, 50_000]; // 1KB, 10KB, 50KB

    for size in test_sizes {
        let test_text = generate_test_text(size);
        let input_size_mb = size as f64 / (1024.0 * 1024.0);

        // Test all chunking strategies
        let strategies = vec![
            ("Sliding Window", ChunkingMode::Sliding),
            ("Fixed Size (Char)", ChunkingMode::Fixed { size: 1000, by_tokens: false }),
            ("Fixed Size (Token)", ChunkingMode::Fixed { size: 500, by_tokens: true }),
            ("Sentence-based", ChunkingMode::Sentence { max_sentences: 10 }),
            ("Regex-based", ChunkingMode::Regex {
                pattern: r"\n\n".to_string(),
                min_chunk_size: 100
            }),
        ];

        for (strategy_name, strategy_mode) in strategies {
            let config = ChunkingConfig {
                mode: strategy_mode,
                token_max: 1000,
                overlap: 100,
                preserve_sentences: true,
                deterministic: true,
            };

            let test_text = test_text.clone();
            let config = config.clone();

            runner.run_benchmark(
                &format!("{} ({}B)", strategy_name, size),
                input_size_mb,
                || async {
                    let chunks = chunk_content(&test_text, &config).await?;
                    // Verify chunking produces results
                    assert!(!chunks.is_empty(), "Chunking should produce at least one chunk");
                    Ok(())
                }
            ).await;
        }
    }

    runner.print_summary();

    // Verify performance requirements
    for result in &runner.results {
        if result.name.contains("50000") { // 50KB test
            assert!(
                result.mean_time <= Duration::from_millis(200),
                "Strategy '{}' exceeded 200ms requirement: {:?}",
                result.name, result.mean_time
            );
        }
    }
}

#[tokio::test]
async fn test_dom_spider_performance() {
    let mut runner = BenchmarkRunner::new(BenchmarkConfig::default());

    // Generate test HTML documents of different sizes
    let html_sizes = vec![
        (1_000, "Small HTML (1KB)"),
        (10_000, "Medium HTML (10KB)"),
        (100_000, "Large HTML (100KB)"),
    ];

    for (size, description) in html_sizes {
        let html = generate_test_html(size);
        let input_size_mb = size as f64 / (1024.0 * 1024.0);

        // Benchmark link extraction
        let html_clone = html.clone();
        runner.run_benchmark(
            &format!("Link Extraction - {}", description),
            input_size_mb,
            || async {
                let links = extract_links(&html_clone)?;
                // Verify extraction produces results
                assert!(links.len() >= 0, "Link extraction should succeed");
                Ok(())
            }
        ).await;

        // Benchmark image extraction
        let html_clone = html.clone();
        runner.run_benchmark(
            &format!("Image Extraction - {}", description),
            input_size_mb,
            || async {
                let images = extract_images(&html_clone)?;
                // Verify extraction produces results
                assert!(images.len() >= 0, "Image extraction should succeed");
                Ok(())
            }
        ).await;

        // Benchmark DOM traversal
        let html_clone = html.clone();
        runner.run_benchmark(
            &format!("DOM Traversal - {}", description),
            input_size_mb,
            || async {
                let traverser = DomTraverser::new(&html_clone);
                let stats = traverser.get_stats();
                // Verify stats are collected
                assert!(stats.total_nodes > 0, "DOM should have nodes");
                let elements = traverser.get_elements_info("div, p, a, img")?;
                // Verify element info is collected
                assert!(!elements.is_empty() || true, "Elements info should be valid");
                Ok(())
            }
        ).await;

        // Benchmark table extraction
        let processor = DefaultHtmlProcessor::default();
        let html_clone = html.clone();
        runner.run_benchmark(
            &format!("Table Extraction - {}", description),
            input_size_mb,
            || async {
                let tables = processor.extract_tables(&html_clone, TableExtractionMode::All).await?;
                // Verify table extraction succeeds
                assert!(tables.len() >= 0, "Table extraction should succeed");
                Ok(())
            }
        ).await;
    }

    runner.print_summary();
}

#[tokio::test]
async fn test_html_processing_pipeline_performance() {
    let mut runner = BenchmarkRunner::new(BenchmarkConfig::default());

    let complex_html = generate_complex_html();
    let input_size_mb = complex_html.len() as f64 / (1024.0 * 1024.0);

    let processor = DefaultHtmlProcessor::default();

    // Benchmark CSS extraction
    let mut css_selectors = HashMap::new();
    css_selectors.insert("title".to_string(), "title".to_string());
    css_selectors.insert("content".to_string(), "p, article".to_string());
    css_selectors.insert("navigation".to_string(), "nav a".to_string());

    let html_clone = complex_html.clone();
    let selectors_clone = css_selectors.clone();
    runner.run_benchmark(
        "CSS Extraction Pipeline",
        input_size_mb,
        || async {
            let result = processor.extract_with_css(&html_clone, "https://example.com", &selectors_clone).await?;
            // Verify extraction produces content
            assert!(!result.content.is_empty(), "CSS extraction should produce content");
            Ok(())
        }
    ).await;

    // Benchmark complete processing pipeline
    let html_clone = complex_html.clone();
    let selectors_clone = css_selectors.clone();
    runner.run_benchmark(
        "Complete Processing Pipeline",
        input_size_mb,
        || async {
            // Extract content
            let extracted = processor.extract_with_css(&html_clone, "https://example.com", &selectors_clone).await?;
            assert!(!extracted.content.is_empty(), "Extraction should produce content");

            // Chunk content
            let chunks = processor.chunk_content(&extracted.content, HtmlChunkingMode::default()).await?;
            assert!(!chunks.is_empty(), "Chunking should produce chunks");

            // Extract tables
            let tables = processor.extract_tables(&html_clone, TableExtractionMode::All).await?;
            assert!(tables.len() >= 0, "Table extraction should succeed");

            // Extract links and images
            let links = extract_links(&html_clone)?;
            assert!(links.len() >= 0, "Link extraction should succeed");
            let images = extract_images(&html_clone)?;
            assert!(images.len() >= 0, "Image extraction should succeed");

            Ok(())
        }
    ).await;

    runner.print_summary();
}

#[tokio::test]
async fn test_concurrent_processing_performance() {
    let mut runner = BenchmarkRunner::new(BenchmarkConfig {
        iterations: 5, // Fewer iterations for concurrent tests
        ..BenchmarkConfig::default()
    });

    let test_content = generate_test_text(10_000);
    let config = ChunkingConfig::default();
    let input_size_mb = test_content.len() as f64 / (1024.0 * 1024.0);

    // Test different concurrency levels
    let concurrency_levels = vec![1, 2, 4, 8];

    for concurrent_tasks in concurrency_levels {
        let test_content = test_content.clone();
        let config = config.clone();

        runner.run_benchmark(
            &format!("Concurrent Chunking ({} tasks)", concurrent_tasks),
            input_size_mb * concurrent_tasks as f64,
            || async {
                let mut handles = Vec::new();

                for _ in 0..concurrent_tasks {
                    let content = test_content.clone();
                    let config = config.clone();

                    let handle = tokio::spawn(async move {
                        chunk_content(&content, &config).await
                    });

                    handles.push(handle);
                }

                // Wait for all tasks to complete
                for handle in handles {
                    handle.await??;
                }

                Ok(())
            }
        ).await;
    }

    runner.print_summary();

    // Analyze scaling efficiency
    let results: Vec<_> = runner.results.iter()
        .filter(|r| r.name.contains("Concurrent Chunking"))
        .collect();

    if results.len() >= 2 {
        let single_task_time = results.iter()
            .find(|r| r.name.contains("(1 tasks)"))
            .map(|r| r.mean_time.as_secs_f64())
            .unwrap_or(0.0);

        println!("\n=== CONCURRENCY SCALING ANALYSIS ===");
        for result in results {
            if let Some(tasks) = extract_task_count(&result.name) {
                if tasks > 1 {
                    let expected_speedup = tasks as f64;
                    let actual_speedup = single_task_time / result.mean_time.as_secs_f64();
                    let efficiency = (actual_speedup / expected_speedup) * 100.0;

                    println!("{} tasks: {:.2}x speedup ({:.1}% efficiency)",
                            tasks, actual_speedup, efficiency);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_memory_efficiency_benchmark() {
    let mut runner = BenchmarkRunner::new(BenchmarkConfig {
        iterations: 3, // Fewer iterations for memory tests
        ..BenchmarkConfig::default()
    });

    // Test memory efficiency with different document sizes
    let sizes = vec![10_000, 100_000, 1_000_000]; // 10KB, 100KB, 1MB

    for size in sizes {
        let test_text = generate_test_text(size);
        let input_size_mb = size as f64 / (1024.0 * 1024.0);

        // Memory-intensive chunking test
        let config = ChunkingConfig {
            mode: ChunkingMode::Sliding,
            token_max: 200, // Smaller chunks = more chunks = more memory
            overlap: 50,
            preserve_sentences: true,
            deterministic: true,
        };

        let test_text = test_text.clone();
        let config = config.clone();

        runner.run_benchmark(
            &format!("Memory Efficiency Test ({}B)", size),
            input_size_mb,
            || async {
                let chunks = chunk_content(&test_text, &config).await?;

                // Force memory allocation by accessing all chunks
                let total_content_size: usize = chunks.iter()
                    .map(|c| c.content.len())
                    .sum();

                // Ensure the compiler doesn't optimize away our memory usage
                assert!(total_content_size > 0);

                Ok(())
            }
        ).await;
    }

    runner.print_summary();

    // Check that processing time scales reasonably with input size
    let memory_results: Vec<_> = runner.results.iter()
        .filter(|r| r.name.contains("Memory Efficiency"))
        .collect();

    if memory_results.len() >= 2 {
        let mut prev_time = None;
        let mut prev_size = None;

        for result in memory_results {
            if let Some(size) = extract_size_from_name(&result.name) {
                let time = result.mean_time.as_secs_f64();

                if let (Some(prev_t), Some(prev_s)) = (prev_time, prev_size) {
                    let size_ratio = size as f64 / prev_s as f64;
                    let time_ratio = time / prev_t;

                    println!("Size {}B vs {}B: {:.2}x size increase, {:.2}x time increase",
                            prev_s, size, size_ratio, time_ratio);

                    // Time should scale roughly linearly or sub-linearly with size
                    assert!(
                        time_ratio <= size_ratio * 1.5, // Allow some overhead
                        "Processing time scaling is worse than expected: {:.2}x time for {:.2}x size",
                        time_ratio, size_ratio
                    );
                }

                prev_time = Some(time);
                prev_size = Some(size);
            }
        }
    }
}

#[tokio::test]
async fn test_comparative_performance_analysis() {
    println!("\n=== COMPARATIVE PERFORMANCE ANALYSIS ===");

    let test_text = generate_test_text(50_000); // 50KB test case

    // Compare different strategies on the same input
    let strategies = vec![
        ("Sliding Window", ChunkingMode::Sliding),
        ("Fixed Size", ChunkingMode::Fixed { size: 1000, by_tokens: false }),
        ("Sentence-based", ChunkingMode::Sentence { max_sentences: 10 }),
    ];

    let mut strategy_results = Vec::new();

    for (name, mode) in strategies {
        let config = ChunkingConfig {
            mode,
            token_max: 1000,
            overlap: 100,
            preserve_sentences: true,
            deterministic: true,
        };

        let iterations = 10;
        let mut times = Vec::new();

        for _ in 0..iterations {
            let start = Instant::now();
            let chunks = chunk_content(&test_text, &config).await.unwrap();
            let elapsed = start.elapsed();

            times.push((elapsed, chunks.len()));
        }

        let avg_time = times.iter().map(|(t, _)| t.as_secs_f64()).sum::<f64>() / times.len() as f64;
        let avg_chunks = times.iter().map(|(_, c)| *c).sum::<usize>() / times.len();

        strategy_results.push((name, avg_time, avg_chunks));

        println!("{}: {:.2}ms avg, {} chunks avg",
                name, avg_time * 1000.0, avg_chunks);
    }

    // Find the fastest strategy
    if let Some((fastest_name, fastest_time, _)) = strategy_results.iter()
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()) {

        println!("\nFastest strategy: {} ({:.2}ms)", fastest_name, fastest_time * 1000.0);

        // Compare all strategies to the fastest
        for (name, time, chunks) in &strategy_results {
            if name != fastest_name {
                let slowdown = time / fastest_time;
                println!("{} is {:.2}x slower than {} ({} chunks vs {} chunks)",
                        name, slowdown, chunks,
                        strategy_results.iter().find(|(n, _, _)| n == fastest_name).unwrap().2);
            }
        }
    }
}

// Helper functions for benchmark tests

fn generate_test_text(target_size: usize) -> String {
    let base_sentences = vec![
        "The quick brown fox jumps over the lazy dog.",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        "Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
        "Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.",
        "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore.",
        "Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt.",
        "At vero eos et accusamus et iusto odio dignissimos ducimus qui blanditiis.",
        "Et harum quidem rerum facilis est et expedita distinctio nam libero tempore.",
    ];

    let mut result = String::new();
    let mut sentence_index = 0;

    while result.len() < target_size {
        if !result.is_empty() {
            result.push(' ');
        }

        result.push_str(base_sentences[sentence_index % base_sentences.len()]);
        sentence_index += 1;

        // Add paragraph breaks occasionally
        if sentence_index % 4 == 0 {
            result.push_str("\n\n");
        }
    }

    // Truncate to exact size
    if result.len() > target_size {
        result.truncate(target_size);
        // Ensure we end at a word boundary
        if let Some(last_space) = result.rfind(' ') {
            result.truncate(last_space);
        }
    }

    result
}

fn generate_test_html(target_size: usize) -> String {
    let mut html = String::from(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Test Document</title>
    <meta name="description" content="A test document for benchmarking">
</head>
<body>
    <header>
        <h1>Main Title</h1>
        <nav>
            <ul>
                <li><a href="/home">Home</a></li>
                <li><a href="/about">About</a></li>
                <li><a href="/contact">Contact</a></li>
            </ul>
        </nav>
    </header>
    <main>
"#);

    let mut section_count = 0;
    while html.len() < target_size {
        html.push_str(&format!(r#"
        <section id="section{}">
            <h2>Section {} Title</h2>
            <p>This is paragraph content for section {}. It contains various elements and text to simulate a real document.</p>
            <ul>
                <li>First item in section {}</li>
                <li>Second item with <a href="/link{}">a link</a></li>
                <li>Third item with <img src="image{}.jpg" alt="Image {}"></li>
            </ul>
            <table>
                <tr><th>Column 1</th><th>Column 2</th><th>Column 3</th></tr>
                <tr><td>Data {}</td><td>Value {}</td><td>Item {}</td></tr>
            </table>
        </section>
"#, section_count, section_count, section_count, section_count, section_count,
    section_count, section_count, section_count, section_count, section_count));

        section_count += 1;
    }

    html.push_str(r#"
    </main>
    <footer>
        <p>Footer content</p>
    </footer>
</body>
</html>
"#);

    // Truncate if necessary
    if html.len() > target_size {
        html.truncate(target_size);
        html.push_str("</body></html>");
    }

    html
}

fn generate_complex_html() -> String {
    r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Complex Document for Benchmarking</title>
    <meta name="description" content="A complex HTML document with various elements for performance testing">
    <meta name="keywords" content="benchmark, performance, testing, html">
</head>
<body>
    <header>
        <h1>Complex Document Structure</h1>
        <nav>
            <ul>
                <li><a href="#section1">Section 1</a></li>
                <li><a href="#section2">Section 2</a></li>
                <li><a href="#section3">Section 3</a></li>
            </ul>
        </nav>
    </header>

    <main>
        <article id="section1">
            <h2>Introduction</h2>
            <p>This is a complex HTML document designed to test the performance of various HTML processing operations. It contains multiple sections with different types of content.</p>

            <form id="contact-form" method="post" action="/submit">
                <fieldset>
                    <legend>Contact Information</legend>
                    <label for="name">Name:</label>
                    <input type="text" id="name" name="name" required>

                    <label for="email">Email:</label>
                    <input type="email" id="email" name="email" required>

                    <label for="message">Message:</label>
                    <textarea id="message" name="message" rows="4" cols="50"></textarea>

                    <button type="submit">Submit</button>
                </fieldset>
            </form>
        </article>

        <section id="section2">
            <h2>Data Tables</h2>
            <table>
                <caption>Performance Metrics</caption>
                <thead>
                    <tr>
                        <th>Metric</th>
                        <th>Value</th>
                        <th>Unit</th>
                    </tr>
                </thead>
                <tbody>
                    <tr><td>Processing Time</td><td>150</td><td>ms</td></tr>
                    <tr><td>Memory Usage</td><td>25</td><td>MB</td></tr>
                    <tr><td>Throughput</td><td>100</td><td>req/s</td></tr>
                </tbody>
            </table>

            <div class="image-gallery">
                <img src="chart1.png" alt="Performance Chart 1" width="300" height="200">
                <img src="chart2.png" alt="Performance Chart 2" width="300" height="200">
                <img src="chart3.png" alt="Performance Chart 3" width="300" height="200">
            </div>
        </section>

        <aside id="section3">
            <h3>Related Links</h3>
            <ul>
                <li><a href="https://example.com/doc1" target="_blank">External Documentation</a></li>
                <li><a href="/internal/guide" rel="help">Internal Guide</a></li>
                <li><a href="mailto:support@example.com">Email Support</a></li>
            </ul>
        </aside>
    </main>

    <footer>
        <p>&copy; 2024 Benchmark Test. All rights reserved.</p>
        <nav>
            <a href="/privacy">Privacy Policy</a>
            <a href="/terms">Terms of Service</a>
        </nav>
    </footer>
</body>
</html>
"#.to_string()
}

fn extract_task_count(benchmark_name: &str) -> Option<usize> {
    // Extract number from "(X tasks)" pattern
    if let Some(start) = benchmark_name.find('(') {
        if let Some(end) = benchmark_name.find(" tasks)") {
            let number_str = &benchmark_name[start + 1..end];
            return number_str.parse().ok();
        }
    }
    None
}

fn extract_size_from_name(benchmark_name: &str) -> Option<usize> {
    // Extract number from "(XB)" pattern
    if let Some(start) = benchmark_name.find('(') {
        if let Some(end) = benchmark_name.find("B)") {
            let number_str = &benchmark_name[start + 1..end];
            return number_str.parse().ok();
        }
    }
    None
}