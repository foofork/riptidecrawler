//! Performance metrics and benchmarking for extraction strategies

use crate::strategies::ExtractionStrategy;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance metrics for strategy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub strategy_metrics: HashMap<String, StrategyMetrics>,
    pub session_start: std::time::SystemTime,
    pub total_extractions: usize,
    pub total_processing_time: Duration,
}

/// Metrics for a specific extraction strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetrics {
    pub strategy_name: String,
    pub total_runs: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub success_rate: f64,
    pub average_content_size: usize,
    pub quality_scores: Vec<f64>,
    pub average_quality: f64,
    pub throughput_per_second: f64,
    pub memory_usage: MemoryMetrics,
    pub error_count: usize,
    pub last_updated: std::time::SystemTime,
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub peak_memory_mb: f64,
    pub average_memory_mb: f64,
    pub memory_efficiency: f64,
}

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub test_content_sizes: Vec<usize>,
    pub strategies_to_test: Vec<ExtractionStrategy>,
    pub measure_memory: bool,
    pub detailed_timing: bool,
}

/// Benchmark result for comparison
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub strategy_name: String,
    pub content_size: usize,
    pub iterations: usize,
    pub total_time: Duration,
    pub average_time: Duration,
    pub median_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub standard_deviation: Duration,
    pub throughput_mb_per_sec: f64,
    pub success_rate: f64,
    pub memory_peak_mb: f64,
    pub quality_score: f64,
    pub confidence_interval_95: (Duration, Duration),
}

/// Performance comparison between strategies
#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyComparison {
    pub baseline_strategy: String,
    pub comparisons: Vec<StrategyComparison>,
    pub winner: String,
    pub performance_ratios: HashMap<String, f64>,
    pub quality_ratios: HashMap<String, f64>,
    pub recommendations: Vec<String>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            strategy_metrics: HashMap::new(),
            session_start: std::time::SystemTime::now(),
            total_extractions: 0,
            total_processing_time: Duration::new(0, 0),
        }
    }

    /// Record extraction performance
    pub fn record_extraction(
        &mut self,
        strategy: &ExtractionStrategy,
        duration: Duration,
        content_size: usize,
        _chunk_count: usize, // Ignored, chunking handled by other crates
    ) {
        let strategy_name = strategy_name(strategy);
        let metrics = self
            .strategy_metrics
            .entry(strategy_name.clone())
            .or_insert_with(|| StrategyMetrics::new(strategy_name));

        metrics.record_run(duration, content_size, true, 0.8); // Default quality

        self.total_extractions += 1;
        self.total_processing_time += duration;
    }

    /// Record extraction failure
    pub fn record_failure(&mut self, strategy: &ExtractionStrategy, duration: Duration) {
        let strategy_name = strategy_name(strategy);
        let metrics = self
            .strategy_metrics
            .entry(strategy_name.clone())
            .or_insert_with(|| StrategyMetrics::new(strategy_name));

        metrics.record_failure(duration);
    }

    /// Get performance summary
    pub fn get_summary(&self) -> PerformanceSummary {
        let mut best_strategy = String::new();
        let mut best_throughput = 0.0;

        for (name, metrics) in &self.strategy_metrics {
            if metrics.throughput_per_second > best_throughput {
                best_throughput = metrics.throughput_per_second;
                best_strategy = name.clone();
            }
        }

        PerformanceSummary {
            total_extractions: self.total_extractions,
            total_time: self.total_processing_time,
            average_time_per_extraction: if self.total_extractions > 0 {
                self.total_processing_time / self.total_extractions as u32
            } else {
                Duration::new(0, 0)
            },
            best_performing_strategy: best_strategy,
            best_throughput,
            strategies_tested: self.strategy_metrics.len(),
        }
    }

    /// Export metrics for analysis
    pub fn export_csv(&self) -> Result<String> {
        let mut csv = String::new();
        csv.push_str(
            "Strategy,Runs,AvgDuration,Throughput,SuccessRate,AvgQuality,MemoryPeak
",
        );

        for metrics in self.strategy_metrics.values() {
            csv.push_str(&format!(
                "{},{},{:.2},{:.2},{:.2},{:.2},{:.2}
",
                metrics.strategy_name,
                metrics.total_runs,
                metrics.average_duration.as_millis(),
                metrics.throughput_per_second,
                metrics.success_rate,
                metrics.average_quality,
                metrics.memory_usage.peak_memory_mb
            ));
        }

        Ok(csv)
    }
}

impl StrategyMetrics {
    pub fn new(strategy_name: String) -> Self {
        Self {
            strategy_name,
            total_runs: 0,
            total_duration: Duration::new(0, 0),
            average_duration: Duration::new(0, 0),
            min_duration: Duration::from_secs(u64::MAX),
            max_duration: Duration::new(0, 0),
            success_rate: 0.0,
            average_content_size: 0,
            quality_scores: Vec::new(),
            average_quality: 0.0,
            throughput_per_second: 0.0,
            memory_usage: MemoryMetrics::default(),
            error_count: 0,
            last_updated: std::time::SystemTime::now(),
        }
    }

    /// Record a successful run
    pub fn record_run(
        &mut self,
        duration: Duration,
        content_size: usize,
        success: bool,
        quality_score: f64,
    ) {
        self.total_runs += 1;
        self.total_duration += duration;

        if success {
            self.min_duration = self.min_duration.min(duration);
            self.max_duration = self.max_duration.max(duration);
            self.quality_scores.push(quality_score);
        } else {
            self.error_count += 1;
        }

        // Update averages
        self.average_duration = self.total_duration / self.total_runs as u32;
        self.success_rate = (self.total_runs - self.error_count) as f64 / self.total_runs as f64;

        if !self.quality_scores.is_empty() {
            self.average_quality =
                self.quality_scores.iter().sum::<f64>() / self.quality_scores.len() as f64;
        }

        // Calculate throughput (MB/s)
        if self.average_duration.as_secs_f64() > 0.0 {
            let avg_size_mb = content_size as f64 / (1024.0 * 1024.0);
            self.throughput_per_second = avg_size_mb / self.average_duration.as_secs_f64();
        }

        self.average_content_size =
            ((self.average_content_size * (self.total_runs - 1)) + content_size) / self.total_runs;

        self.last_updated = std::time::SystemTime::now();
    }

    /// Record a failed run
    pub fn record_failure(&mut self, duration: Duration) {
        self.total_runs += 1;
        self.error_count += 1;
        self.total_duration += duration;

        self.average_duration = self.total_duration / self.total_runs as u32;
        self.success_rate = (self.total_runs - self.error_count) as f64 / self.total_runs as f64;

        self.last_updated = std::time::SystemTime::now();
    }
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        Self {
            peak_memory_mb: 0.0,
            average_memory_mb: 0.0,
            memory_efficiency: 0.0,
        }
    }
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            warmup_iterations: 10,
            test_content_sizes: vec![1024, 10240, 102400, 1048576], // 1KB to 1MB
            strategies_to_test: vec![ExtractionStrategy::Trek],
            measure_memory: true,
            detailed_timing: true,
        }
    }
}

/// Performance summary
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub total_extractions: usize,
    pub total_time: Duration,
    pub average_time_per_extraction: Duration,
    pub best_performing_strategy: String,
    pub best_throughput: f64,
    pub strategies_tested: usize,
}

/// Run comprehensive benchmarks
pub async fn run_benchmarks(config: BenchmarkConfig) -> Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    for strategy in &config.strategies_to_test {
        for &content_size in &config.test_content_sizes {
            let test_content = generate_test_content(content_size);
            let result = benchmark_strategy(strategy, &test_content, &config).await?;
            results.push(result);
        }
    }

    // Sort by performance
    results.sort_by(|a, b| a.average_time.cmp(&b.average_time));

    Ok(results)
}

/// Benchmark a specific strategy
async fn benchmark_strategy(
    strategy: &ExtractionStrategy,
    content: &str,
    config: &BenchmarkConfig,
) -> Result<BenchmarkResult> {
    let mut durations = Vec::new();
    let mut success_count = 0;
    let mut quality_scores = Vec::new();
    let mut peak_memory: f64 = 0.0;

    // Warmup runs
    for _ in 0..config.warmup_iterations {
        let _ = run_single_extraction(strategy, content).await;
    }

    // Actual benchmark runs
    for _ in 0..config.iterations {
        let start_memory = get_memory_usage_mb();
        let start_time = Instant::now();

        match run_single_extraction(strategy, content).await {
            Ok(result) => {
                success_count += 1;
                quality_scores.push(result.quality);
                durations.push(start_time.elapsed());
            }
            Err(_) => {
                durations.push(start_time.elapsed());
            }
        }

        if config.measure_memory {
            let current_memory = get_memory_usage_mb();
            peak_memory = peak_memory.max(current_memory - start_memory);
        }
    }

    // Calculate statistics
    durations.sort();
    let total_time: Duration = durations.iter().sum();
    let average_time = total_time / config.iterations as u32;
    let median_time = durations[config.iterations / 2];
    let min_time = durations[0];
    let max_time = durations[config.iterations - 1];

    let mean_ms = average_time.as_millis() as f64;
    let variance = durations
        .iter()
        .map(|d| {
            let diff = d.as_millis() as f64 - mean_ms;
            diff * diff
        })
        .sum::<f64>()
        / config.iterations as f64;
    let std_dev = Duration::from_millis(variance.sqrt() as u64);

    // Calculate confidence interval (95%)
    let t_value = 1.96; // Approximate for large sample sizes
    let std_err = std_dev.as_millis() as f64 / (config.iterations as f64).sqrt();
    let margin = Duration::from_millis((t_value * std_err) as u64);
    let confidence_interval = (average_time.saturating_sub(margin), average_time + margin);

    let success_rate = success_count as f64 / config.iterations as f64;
    let average_quality = if quality_scores.is_empty() {
        0.0
    } else {
        quality_scores.iter().sum::<f64>() / quality_scores.len() as f64
    };

    let content_size_mb = content.len() as f64 / (1024.0 * 1024.0);
    let throughput_mb_per_sec = if average_time.as_secs_f64() > 0.0 {
        content_size_mb / average_time.as_secs_f64()
    } else {
        0.0
    };

    Ok(BenchmarkResult {
        strategy_name: strategy_name(strategy),
        content_size: content.len(),
        iterations: config.iterations,
        total_time,
        average_time,
        median_time,
        min_time,
        max_time,
        standard_deviation: std_dev,
        throughput_mb_per_sec,
        success_rate,
        memory_peak_mb: peak_memory,
        quality_score: average_quality,
        confidence_interval_95: confidence_interval,
    })
}

/// Simple extraction result for benchmarking
struct ExtractionResult {
    quality: f64,
}

/// Run single extraction for benchmarking
async fn run_single_extraction(
    strategy: &ExtractionStrategy,
    _content: &str,
) -> Result<ExtractionResult> {
    // Simplified extraction for benchmarking
    match strategy {
        ExtractionStrategy::Trek => {
            // Trek extraction moved to riptide-html
            // Returning mock result for testing
            Ok(ExtractionResult { quality: 80.0 })
        }
    }
}

/// Generate test content of specified size
fn generate_test_content(size: usize) -> String {
    let base_content = r#"
    <html>
    <head>
        <title>Test Article for Benchmarking</title>
        <meta name="description" content="This is a test article for performance benchmarking.">
        <meta name="author" content="Test Author">
        <meta property="og:title" content="Test Article for Benchmarking">
        <meta property="og:description" content="This is a test article for performance benchmarking.">
    </head>
    <body>
        <article>
            <h1>Test Article for Benchmarking</h1>
            <p class="byline">By Test Author</p>
            <time datetime="2023-01-01">January 1, 2023</time>
            <div class="content">
    "#;

    let end_content = r#"
            </div>
        </article>
    </body>
    </html>
    "#;

    let mut content = String::from(base_content);
    let paragraph = "<p>This is a test paragraph with some content for benchmarking extraction strategies. It contains enough text to provide meaningful performance data while being representative of real-world content.</p>
";

    // Add paragraphs until we reach the desired size
    while content.len() + end_content.len() < size {
        content.push_str(paragraph);
    }

    content.push_str(end_content);
    content
}

/// Get current memory usage in MB (platform-specific)
fn get_memory_usage_mb() -> f64 {
    // Simple implementation - in production, this could use more accurate methods
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<f64>() {
                            return kb / 1024.0; // Convert KB to MB
                        }
                    }
                }
            }
        }
    }

    // Fallback: return 0 if we can't measure
    0.0
}

/// Get strategy name for metrics
fn strategy_name(strategy: &ExtractionStrategy) -> String {
    match strategy {
        ExtractionStrategy::Trek => "trek".to_string(),
    }
}

/// Compare strategies and provide recommendations
pub fn compare_strategies(results: &[BenchmarkResult]) -> StrategyComparison {
    if results.is_empty() {
        return StrategyComparison {
            baseline_strategy: "none".to_string(),
            comparisons: Vec::new(),
            winner: "none".to_string(),
            performance_ratios: HashMap::new(),
            quality_ratios: HashMap::new(),
            recommendations: vec!["No results to compare".to_string()],
        };
    }

    // Find the fastest strategy
    let fastest = results
        .iter()
        .min_by(|a, b| a.average_time.cmp(&b.average_time))
        .unwrap();

    let baseline_strategy = fastest.strategy_name.clone();
    let mut performance_ratios = HashMap::new();
    let mut quality_ratios = HashMap::new();
    let mut recommendations = Vec::new();

    for result in results {
        let perf_ratio = result.average_time.as_secs_f64() / fastest.average_time.as_secs_f64();
        let quality_ratio = result.quality_score / fastest.quality_score.max(0.01_f64);

        performance_ratios.insert(result.strategy_name.clone(), perf_ratio);
        quality_ratios.insert(result.strategy_name.clone(), quality_ratio);
    }

    // Generate recommendations
    recommendations.push(format!(
        "Fastest strategy: {} ({:.2}ms average)",
        fastest.strategy_name,
        fastest.average_time.as_millis()
    ));

    let highest_quality = results
        .iter()
        .max_by(|a, b| a.quality_score.partial_cmp(&b.quality_score).unwrap())
        .unwrap();

    if highest_quality.strategy_name != fastest.strategy_name {
        recommendations.push(format!(
            "Highest quality: {} ({:.2} quality score)",
            highest_quality.strategy_name, highest_quality.quality_score
        ));
    }

    // Efficiency recommendation
    let most_efficient = results
        .iter()
        .max_by(|a, b| {
            (a.quality_score / a.average_time.as_secs_f64())
                .partial_cmp(&(b.quality_score / b.average_time.as_secs_f64()))
                .unwrap()
        })
        .unwrap();

    recommendations.push(format!(
        "Most efficient (quality/time): {}",
        most_efficient.strategy_name
    ));

    StrategyComparison {
        baseline_strategy: baseline_strategy.clone(),
        comparisons: Vec::new(), // Simplified for now
        winner: baseline_strategy,
        performance_ratios,
        quality_ratios,
        recommendations,
    }
}
