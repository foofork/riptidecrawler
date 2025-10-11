use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::cli_test_harness::{ExtractionResult, TestSession};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodComparison {
    pub test_id: String,
    pub url: String,
    pub methods: HashMap<String, MethodStats>,
    pub best_method: Option<String>,
    pub differences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodStats {
    pub success: bool,
    pub duration_ms: u64,
    pub content_length: usize,
    pub error: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonReport {
    pub session_id: String,
    pub timestamp: String,
    pub comparisons: Vec<MethodComparison>,
    pub summary: ComparisonSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonSummary {
    pub total_urls: usize,
    pub method_success_rates: HashMap<String, f64>,
    pub method_avg_duration: HashMap<String, f64>,
    pub method_avg_content_length: HashMap<String, f64>,
    pub best_overall_method: Option<String>,
    pub recommendations: Vec<String>,
}

pub struct ComparisonTool {
    pub output_dir: PathBuf,
}

impl ComparisonTool {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    pub fn load_session(&self, session_id: &str) -> Result<TestSession> {
        let path = self.output_dir.join(format!("{}.json", session_id));
        let content = fs::read_to_string(&path)
            .context(format!("Failed to read session file: {}", path.display()))?;
        let session: TestSession = serde_json::from_str(&content)
            .context("Failed to parse session JSON")?;
        Ok(session)
    }

    pub fn compare_methods(&self, session: &TestSession) -> Result<ComparisonReport> {
        let mut comparisons = Vec::new();
        let mut method_results: HashMap<String, Vec<&ExtractionResult>> = HashMap::new();

        // Group results by test_id
        let mut test_groups: HashMap<String, Vec<&ExtractionResult>> = HashMap::new();
        for result in &session.results {
            test_groups.entry(result.test_id.clone())
                .or_insert_with(Vec::new)
                .push(result);

            method_results.entry(result.method.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }

        // Compare methods for each test
        for (test_id, results) in test_groups {
            let mut methods = HashMap::new();
            let mut differences = Vec::new();

            // Collect stats for each method
            for result in &results {
                methods.insert(
                    result.method.clone(),
                    MethodStats {
                        success: result.success,
                        duration_ms: result.duration_ms,
                        content_length: result.content_length,
                        error: result.error.clone(),
                        warnings: result.warnings.clone(),
                    }
                );
            }

            // Find differences
            let success_counts: Vec<bool> = methods.values().map(|m| m.success).collect();
            if success_counts.iter().any(|&s| s) && success_counts.iter().any(|&s| !s) {
                differences.push("Different success rates across methods".to_string());
            }

            let content_lengths: Vec<usize> = methods.values().map(|m| m.content_length).collect();
            if !content_lengths.is_empty() {
                let max = content_lengths.iter().max().unwrap();
                let min = content_lengths.iter().min().unwrap();
                if max - min > max / 2 { // More than 50% difference
                    differences.push(format!(
                        "Large content length variation: {} - {} bytes",
                        min, max
                    ));
                }
            }

            let durations: Vec<u64> = methods.values().map(|m| m.duration_ms).collect();
            if !durations.is_empty() {
                let max = durations.iter().max().unwrap();
                let min = durations.iter().min().unwrap();
                if *max > min * 2 { // More than 2x difference
                    differences.push(format!(
                        "Performance variation: {} - {} ms",
                        min, max
                    ));
                }
            }

            // Determine best method
            let best_method = methods.iter()
                .filter(|(_, stats)| stats.success)
                .min_by_key(|(_, stats)| stats.duration_ms)
                .map(|(method, _)| method.clone());

            let url = results.first().map(|r| r.url.clone()).unwrap_or_default();

            comparisons.push(MethodComparison {
                test_id,
                url,
                methods,
                best_method,
                differences,
            });
        }

        // Calculate summary statistics
        let summary = self.calculate_summary(&method_results, comparisons.len());

        let report = ComparisonReport {
            session_id: format!("comparison-{}", chrono::Utc::now().timestamp()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            comparisons,
            summary,
        };

        // Save report
        let report_path = self.output_dir.join(format!("{}.json", report.session_id));
        let json = serde_json::to_string_pretty(&report)?;
        fs::write(&report_path, json)?;

        Ok(report)
    }

    fn calculate_summary(
        &self,
        method_results: &HashMap<String, Vec<&ExtractionResult>>,
        total_urls: usize,
    ) -> ComparisonSummary {
        let mut method_success_rates = HashMap::new();
        let mut method_avg_duration = HashMap::new();
        let mut method_avg_content_length = HashMap::new();
        let mut recommendations = Vec::new();

        for (method, results) in method_results {
            let success_count = results.iter().filter(|r| r.success).count();
            let success_rate = success_count as f64 / results.len() as f64;
            method_success_rates.insert(method.clone(), success_rate);

            let avg_duration = results.iter()
                .filter(|r| r.success)
                .map(|r| r.duration_ms)
                .sum::<u64>() as f64 / success_count.max(1) as f64;
            method_avg_duration.insert(method.clone(), avg_duration);

            let avg_content = results.iter()
                .filter(|r| r.success)
                .map(|r| r.content_length)
                .sum::<usize>() as f64 / success_count.max(1) as f64;
            method_avg_content_length.insert(method.clone(), avg_content);

            // Generate recommendations
            if success_rate < 0.5 {
                recommendations.push(format!(
                    "⚠️  {} has low success rate ({:.1}%) - consider alternative methods",
                    method, success_rate * 100.0
                ));
            }

            if avg_duration > 5000.0 {
                recommendations.push(format!(
                    "⏱️  {} is slow (avg {}ms) - may need optimization",
                    method, avg_duration as u64
                ));
            }
        }

        // Find best overall method
        let best_overall_method = method_success_rates.iter()
            .filter(|(_, &rate)| rate > 0.8)
            .min_by(|(method1, _), (method2, _)| {
                let dur1 = method_avg_duration.get(*method1).unwrap_or(&f64::MAX);
                let dur2 = method_avg_duration.get(*method2).unwrap_or(&f64::MAX);
                dur1.partial_cmp(dur2).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(method, _)| method.clone());

        if let Some(ref best) = best_overall_method {
            recommendations.push(format!(
                "🏆 {} is the best overall method (success: {:.1}%, avg: {}ms)",
                best,
                method_success_rates.get(best).unwrap_or(&0.0) * 100.0,
                method_avg_duration.get(best).unwrap_or(&0.0) as u64
            ));
        }

        ComparisonSummary {
            total_urls,
            method_success_rates,
            method_avg_duration,
            method_avg_content_length,
            best_overall_method,
            recommendations,
        }
    }

    pub fn print_report(&self, report: &ComparisonReport) {
        println!("\n📊 Comparison Report: {}", report.session_id);
        println!("=" .repeat(80));

        println!("\n📈 Summary:");
        println!("   Total URLs tested: {}", report.summary.total_urls);

        println!("\n   Success Rates:");
        for (method, rate) in &report.summary.method_success_rates {
            println!("      {} {:.1}%", method, rate * 100.0);
        }

        println!("\n   Average Duration (ms):");
        for (method, duration) in &report.summary.method_avg_duration {
            println!("      {} {}ms", method, *duration as u64);
        }

        println!("\n   Average Content Length (bytes):");
        for (method, length) in &report.summary.method_avg_content_length {
            println!("      {} {} bytes", method, *length as usize);
        }

        if let Some(ref best) = report.summary.best_overall_method {
            println!("\n   🏆 Best Overall Method: {}", best);
        }

        if !report.summary.recommendations.is_empty() {
            println!("\n💡 Recommendations:");
            for rec in &report.summary.recommendations {
                println!("   {}", rec);
            }
        }

        println!("\n🔍 Detailed Comparisons:");
        for comp in &report.comparisons {
            if !comp.differences.is_empty() {
                println!("\n   {} ({})", comp.test_id, comp.url);
                for diff in &comp.differences {
                    println!("      ⚠️  {}", diff);
                }
                if let Some(ref best) = comp.best_method {
                    println!("      ✅ Best method: {}", best);
                }
            }
        }

        println!("\n" + &"=".repeat(80));
    }

    pub fn diff_sessions(
        &self,
        session1_id: &str,
        session2_id: &str,
    ) -> Result<()> {
        let session1 = self.load_session(session1_id)?;
        let session2 = self.load_session(session2_id)?;

        println!("\n🔄 Comparing Sessions:");
        println!("   Session 1: {} ({} results)",
            session1.session_id, session1.results.len());
        println!("   Session 2: {} ({} results)",
            session2.session_id, session2.results.len());

        let mut improvements = 0;
        let mut regressions = 0;

        for result1 in &session1.results {
            if let Some(result2) = session2.results.iter().find(|r|
                r.test_id == result1.test_id && r.method == result1.method
            ) {
                // Compare success
                match (result1.success, result2.success) {
                    (false, true) => {
                        improvements += 1;
                        println!("   ✅ {} [{}]: Now succeeds!",
                            result1.test_id, result1.method);
                    }
                    (true, false) => {
                        regressions += 1;
                        println!("   ❌ {} [{}]: Now fails!",
                            result1.test_id, result1.method);
                    }
                    _ => {}
                }

                // Compare performance
                if result1.success && result2.success {
                    let perf_change = ((result2.duration_ms as f64 - result1.duration_ms as f64)
                        / result1.duration_ms as f64) * 100.0;

                    if perf_change.abs() > 20.0 {
                        if perf_change < 0.0 {
                            println!("   ⚡ {} [{}]: {}% faster",
                                result1.test_id, result1.method, perf_change.abs() as i64);
                        } else {
                            println!("   🐌 {} [{}]: {}% slower",
                                result1.test_id, result1.method, perf_change as i64);
                        }
                    }
                }
            }
        }

        println!("\n   Improvements: {}", improvements);
        println!("   Regressions:  {}", regressions);

        Ok(())
    }
}
