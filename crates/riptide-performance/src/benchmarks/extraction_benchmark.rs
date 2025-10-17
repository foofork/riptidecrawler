//! Extraction pipeline performance benchmarks
//!
//! This module provides comprehensive benchmarking for the extraction pipeline,
//! measuring performance across different engines and scenarios.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Extraction engine type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExtractionEngine {
    Wasm,
    Headless,
    Stealth,
    Spider,
}

impl std::fmt::Display for ExtractionEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractionEngine::Wasm => write!(f, "WASM"),
            ExtractionEngine::Headless => write!(f, "Headless"),
            ExtractionEngine::Stealth => write!(f, "Stealth"),
            ExtractionEngine::Spider => write!(f, "Spider"),
        }
    }
}

/// Benchmark result for a single extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionBenchmark {
    pub engine: ExtractionEngine,
    pub url: String,
    pub duration: Duration,
    pub memory_peak_mb: f64,
    pub memory_average_mb: f64,
    pub cpu_percent: f64,
    pub success: bool,
    pub error: Option<String>,
    pub content_size_bytes: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Aggregated benchmark statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkStatistics {
    pub engine: ExtractionEngine,
    pub total_runs: usize,
    pub successful_runs: usize,
    pub failed_runs: usize,
    pub success_rate: f64,

    // Timing statistics
    pub avg_duration_ms: f64,
    pub min_duration_ms: f64,
    pub max_duration_ms: f64,
    pub p50_duration_ms: f64,
    pub p95_duration_ms: f64,
    pub p99_duration_ms: f64,

    // Memory statistics
    pub avg_memory_peak_mb: f64,
    pub max_memory_peak_mb: f64,

    // CPU statistics
    pub avg_cpu_percent: f64,
    pub max_cpu_percent: f64,

    // Throughput
    pub throughput_pages_per_sec: f64,
}

/// Comparative benchmark report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparativeBenchmarkReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub test_duration: Duration,
    pub engines_tested: Vec<ExtractionEngine>,
    pub statistics: HashMap<ExtractionEngine, BenchmarkStatistics>,
    pub performance_ranking: Vec<(ExtractionEngine, f64)>, // (engine, score)
    pub recommendations: Vec<String>,
}

/// Performance benchmark runner
pub struct ExtractionBenchmarkRunner {
    results: Vec<ExtractionBenchmark>,
    start_time: Option<Instant>,
}

impl ExtractionBenchmarkRunner {
    /// Create a new benchmark runner
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            start_time: None,
        }
    }

    /// Start benchmarking session
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.results.clear();
    }

    /// Record a benchmark result
    pub fn record(&mut self, benchmark: ExtractionBenchmark) {
        self.results.push(benchmark);
    }

    /// Calculate statistics for an engine
    fn calculate_statistics(&self, engine: ExtractionEngine) -> Option<BenchmarkStatistics> {
        let engine_results: Vec<&ExtractionBenchmark> =
            self.results.iter().filter(|r| r.engine == engine).collect();

        if engine_results.is_empty() {
            return None;
        }

        let total_runs = engine_results.len();
        let successful_runs = engine_results.iter().filter(|r| r.success).count();
        let failed_runs = total_runs - successful_runs;
        let success_rate = successful_runs as f64 / total_runs as f64;

        // Collect timing data
        let mut durations: Vec<f64> = engine_results
            .iter()
            .map(|r| r.duration.as_secs_f64() * 1000.0) // Convert to ms
            .collect();
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let avg_duration_ms = durations.iter().sum::<f64>() / durations.len() as f64;
        let min_duration_ms = *durations.first().unwrap_or(&0.0);
        let max_duration_ms = *durations.last().unwrap_or(&0.0);
        let p50_duration_ms = percentile(&durations, 0.50);
        let p95_duration_ms = percentile(&durations, 0.95);
        let p99_duration_ms = percentile(&durations, 0.99);

        // Memory statistics
        let avg_memory_peak_mb = engine_results.iter().map(|r| r.memory_peak_mb).sum::<f64>()
            / engine_results.len() as f64;
        let max_memory_peak_mb = engine_results
            .iter()
            .map(|r| r.memory_peak_mb)
            .fold(0.0_f64, |max, val| max.max(val));

        // CPU statistics
        let avg_cpu_percent =
            engine_results.iter().map(|r| r.cpu_percent).sum::<f64>() / engine_results.len() as f64;
        let max_cpu_percent = engine_results
            .iter()
            .map(|r| r.cpu_percent)
            .fold(0.0_f64, |max, val| max.max(val));

        // Throughput (pages per second)
        let total_time_sec: f64 = durations.iter().sum::<f64>() / 1000.0;
        let throughput_pages_per_sec = if total_time_sec > 0.0 {
            successful_runs as f64 / total_time_sec
        } else {
            0.0
        };

        Some(BenchmarkStatistics {
            engine,
            total_runs,
            successful_runs,
            failed_runs,
            success_rate,
            avg_duration_ms,
            min_duration_ms,
            max_duration_ms,
            p50_duration_ms,
            p95_duration_ms,
            p99_duration_ms,
            avg_memory_peak_mb,
            max_memory_peak_mb,
            avg_cpu_percent,
            max_cpu_percent,
            throughput_pages_per_sec,
        })
    }

    /// Generate comparative report
    pub fn generate_report(&self) -> ComparativeBenchmarkReport {
        let test_duration = self
            .start_time
            .map(|start| start.elapsed())
            .unwrap_or_default();

        // Get unique engines tested
        let mut engines_tested: Vec<ExtractionEngine> = self
            .results
            .iter()
            .map(|r| r.engine)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        engines_tested.sort_by_key(|e| format!("{:?}", e));

        // Calculate statistics for each engine
        let mut statistics = HashMap::new();
        for engine in &engines_tested {
            if let Some(stats) = self.calculate_statistics(*engine) {
                statistics.insert(*engine, stats);
            }
        }

        // Rank engines by performance score
        let performance_ranking = rank_engines(&statistics);

        // Generate recommendations
        let recommendations = generate_recommendations(&statistics, &performance_ranking);

        ComparativeBenchmarkReport {
            timestamp: chrono::Utc::now(),
            test_duration,
            engines_tested,
            statistics,
            performance_ranking,
            recommendations,
        }
    }

    /// Export results to JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let report = self.generate_report();
        serde_json::to_string_pretty(&report)
    }

    /// Export results to markdown
    pub fn export_markdown(&self) -> String {
        let report = self.generate_report();
        format_markdown_report(&report)
    }
}

impl Default for ExtractionBenchmarkRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate percentile from sorted data
fn percentile(sorted_data: &[f64], p: f64) -> f64 {
    if sorted_data.is_empty() {
        return 0.0;
    }

    let idx = ((sorted_data.len() as f64 - 1.0) * p) as usize;
    sorted_data[idx]
}

/// Rank engines by performance score
fn rank_engines(
    statistics: &HashMap<ExtractionEngine, BenchmarkStatistics>,
) -> Vec<(ExtractionEngine, f64)> {
    let mut scores: Vec<(ExtractionEngine, f64)> = statistics
        .iter()
        .map(|(engine, stats)| {
            // Composite performance score (lower is better)
            // Weights: success rate (40%), speed (30%), memory (20%), CPU (10%)
            let success_score = stats.success_rate * 100.0;
            let speed_score = if stats.avg_duration_ms > 0.0 {
                1000.0 / stats.avg_duration_ms // Inverse for "higher is better"
            } else {
                0.0
            };
            let memory_score = if stats.avg_memory_peak_mb > 0.0 {
                100.0 / stats.avg_memory_peak_mb // Inverse for "lower is better"
            } else {
                0.0
            };
            let cpu_score = if stats.avg_cpu_percent > 0.0 {
                100.0 / stats.avg_cpu_percent
            } else {
                0.0
            };

            let composite_score = (success_score * 0.4)
                + (speed_score * 0.3)
                + (memory_score * 0.2)
                + (cpu_score * 0.1);

            (*engine, composite_score)
        })
        .collect();

    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()); // Sort descending
    scores
}

/// Generate optimization recommendations
fn generate_recommendations(
    statistics: &HashMap<ExtractionEngine, BenchmarkStatistics>,
    ranking: &[(ExtractionEngine, f64)],
) -> Vec<String> {
    let mut recommendations = Vec::new();

    if let Some((best_engine, _)) = ranking.first() {
        recommendations.push(format!(
            "**Recommended Engine**: {} shows the best overall performance",
            best_engine
        ));
    }

    // Check for engines with poor success rates
    for (engine, stats) in statistics {
        if stats.success_rate < 0.9 {
            recommendations.push(format!(
                "**Reliability Issue**: {} has only {:.1}% success rate - investigate failures",
                engine,
                stats.success_rate * 100.0
            ));
        }

        if stats.avg_memory_peak_mb > 500.0 {
            recommendations.push(format!(
                "**Memory Optimization**: {} uses {:.1}MB peak memory - consider optimization",
                engine, stats.avg_memory_peak_mb
            ));
        }

        if stats.p95_duration_ms > 10000.0 {
            recommendations.push(format!(
                "**Performance Issue**: {} has P95 latency of {:.0}ms - optimize hot paths",
                engine, stats.p95_duration_ms
            ));
        }
    }

    if recommendations.len() == 1 {
        recommendations.push("All engines performing within acceptable parameters".to_string());
    }

    recommendations
}

/// Format report as markdown
fn format_markdown_report(report: &ComparativeBenchmarkReport) -> String {
    let mut md = String::new();

    md.push_str("# Extraction Engine Performance Comparison\n\n");
    md.push_str(&format!(
        "**Report Date**: {}\n",
        report.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    md.push_str(&format!(
        "**Test Duration**: {:.2}s\n\n",
        report.test_duration.as_secs_f64()
    ));

    // Performance ranking
    md.push_str("## Performance Ranking\n\n");
    for (i, (engine, score)) in report.performance_ranking.iter().enumerate() {
        md.push_str(&format!("{}. {} (Score: {:.2})\n", i + 1, engine, score));
    }
    md.push_str("\n");

    // Detailed statistics table
    md.push_str("## Detailed Statistics\n\n");
    md.push_str("| Metric | ");
    for engine in &report.engines_tested {
        md.push_str(&format!("{} | ", engine));
    }
    md.push_str("\n|--------|");
    for _ in &report.engines_tested {
        md.push_str("------|");
    }
    md.push_str("\n");

    // Success rate
    md.push_str("| Success Rate | ");
    for engine in &report.engines_tested {
        if let Some(stats) = report.statistics.get(engine) {
            md.push_str(&format!("{:.1}% | ", stats.success_rate * 100.0));
        } else {
            md.push_str("N/A | ");
        }
    }
    md.push_str("\n");

    // Average duration
    md.push_str("| Avg Duration (ms) | ");
    for engine in &report.engines_tested {
        if let Some(stats) = report.statistics.get(engine) {
            md.push_str(&format!("{:.0} | ", stats.avg_duration_ms));
        } else {
            md.push_str("N/A | ");
        }
    }
    md.push_str("\n");

    // P95 duration
    md.push_str("| P95 Duration (ms) | ");
    for engine in &report.engines_tested {
        if let Some(stats) = report.statistics.get(engine) {
            md.push_str(&format!("{:.0} | ", stats.p95_duration_ms));
        } else {
            md.push_str("N/A | ");
        }
    }
    md.push_str("\n");

    // Memory peak
    md.push_str("| Peak Memory (MB) | ");
    for engine in &report.engines_tested {
        if let Some(stats) = report.statistics.get(engine) {
            md.push_str(&format!("{:.1} | ", stats.max_memory_peak_mb));
        } else {
            md.push_str("N/A | ");
        }
    }
    md.push_str("\n");

    // CPU usage
    md.push_str("| Avg CPU (%) | ");
    for engine in &report.engines_tested {
        if let Some(stats) = report.statistics.get(engine) {
            md.push_str(&format!("{:.1} | ", stats.avg_cpu_percent));
        } else {
            md.push_str("N/A | ");
        }
    }
    md.push_str("\n");

    // Throughput
    md.push_str("| Throughput (pages/s) | ");
    for engine in &report.engines_tested {
        if let Some(stats) = report.statistics.get(engine) {
            md.push_str(&format!("{:.2} | ", stats.throughput_pages_per_sec));
        } else {
            md.push_str("N/A | ");
        }
    }
    md.push_str("\n\n");

    // Recommendations
    md.push_str("## Recommendations\n\n");
    for (i, rec) in report.recommendations.iter().enumerate() {
        md.push_str(&format!("{}. {}\n", i + 1, rec));
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_runner() {
        let mut runner = ExtractionBenchmarkRunner::new();
        runner.start();

        // Add some test data
        runner.record(ExtractionBenchmark {
            engine: ExtractionEngine::Wasm,
            url: "https://example.com".to_string(),
            duration: Duration::from_millis(500),
            memory_peak_mb: 150.0,
            memory_average_mb: 120.0,
            cpu_percent: 45.0,
            success: true,
            error: None,
            content_size_bytes: 50000,
            timestamp: chrono::Utc::now(),
        });

        let report = runner.generate_report();
        assert!(!report.engines_tested.is_empty());
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_percentile_calculation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&data, 0.5), 3.0);
        assert_eq!(percentile(&data, 0.0), 1.0);
        assert_eq!(percentile(&data, 1.0), 5.0);
    }

    #[test]
    fn test_markdown_export() {
        let mut runner = ExtractionBenchmarkRunner::new();
        runner.start();

        runner.record(ExtractionBenchmark {
            engine: ExtractionEngine::Wasm,
            url: "https://example.com".to_string(),
            duration: Duration::from_millis(500),
            memory_peak_mb: 150.0,
            memory_average_mb: 120.0,
            cpu_percent: 45.0,
            success: true,
            error: None,
            content_size_bytes: 50000,
            timestamp: chrono::Utc::now(),
        });

        let markdown = runner.export_markdown();
        assert!(markdown.contains("# Extraction Engine Performance Comparison"));
        assert!(markdown.contains("Performance Ranking"));
    }
}
