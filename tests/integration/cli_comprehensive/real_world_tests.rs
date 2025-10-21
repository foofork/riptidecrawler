//! Comprehensive Real-World CLI Testing Framework
//!
//! This module provides extensive testing for all CLI commands against real-world URLs,
//! with automatic output storage, comparison utilities, and regression detection.

use anyhow::{Context, Result};
use assert_cmd::Command;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Test URL configuration with expected behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestUrl {
    pub id: String,
    pub url: String,
    pub category: String,
    pub expected: ExpectedResult,
    pub notes: String,
}

/// Expected test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedResult {
    pub min_content_length: Option<usize>,
    pub should_contain: Vec<String>,
    pub should_not_contain: Vec<String>,
    pub max_duration_ms: Option<u64>,
    pub expected_success: bool,
}

/// Test execution result with detailed metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub url: String,
    pub success: bool,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub stdout_length: usize,
    pub stderr_length: usize,
    pub stdout_preview: String,
    pub stderr_preview: String,
    pub content_stored_at: Option<PathBuf>,
    pub error: Option<String>,
    pub warnings: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: String,
}

/// Test session aggregating multiple test results
#[derive(Debug, Serialize, Deserialize)]
pub struct TestSession {
    pub session_id: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub total_duration_ms: u64,
    pub results: Vec<TestResult>,
    pub summary_stats: SummaryStats,
}

/// Aggregate statistics for test session
#[derive(Debug, Serialize, Deserialize)]
pub struct SummaryStats {
    pub avg_duration_ms: f64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub success_rate: f64,
    pub commands_tested: HashMap<String, usize>,
}

/// Main test harness for CLI testing
pub struct CliTestHarness {
    pub output_dir: PathBuf,
    pub binary_name: String,
    pub store_outputs: bool,
}

impl CliTestHarness {
    /// Create a new CLI test harness
    pub fn new(output_dir: PathBuf, binary_name: String) -> Self {
        Self {
            output_dir,
            binary_name,
            store_outputs: true,
        }
    }

    /// Run a CLI command and capture results
    pub fn run_command(
        &self,
        command: &str,
        args: &[&str],
    ) -> Result<(std::process::Output, Duration)> {
        let start = Instant::now();

        let output = Command::cargo_bin(&self.binary_name)?
            .arg(command)
            .args(args)
            .output()
            .context("Failed to execute CLI command")?;

        let duration = start.elapsed();

        Ok((output, duration))
    }

    /// Run extraction test with various methods
    pub fn test_extract(
        &self,
        test_url: &TestUrl,
        method: &str,
        engine: &str,
    ) -> Result<TestResult> {
        let test_id = format!("{}_{}_{}", test_url.id, method, engine);
        let mut warnings = Vec::new();
        let mut metadata = HashMap::new();

        let args = vec![
            "--url",
            &test_url.url,
            "--method",
            method,
            "--engine",
            engine,
            "--local",
        ];

        let (output, duration) = self.run_command("extract", &args)?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        // Store output if enabled
        let content_path = if self.store_outputs && !stdout.is_empty() {
            let path = self
                .output_dir
                .join(format!("{}_output.txt", test_id));
            fs::write(&path, &stdout)?;
            Some(path)
        } else {
            None
        };

        // Validate expectations
        if let Some(min_length) = test_url.expected.min_content_length {
            if stdout.len() < min_length {
                warnings.push(format!(
                    "Content length {} below expected minimum {}",
                    stdout.len(),
                    min_length
                ));
            }
        }

        if let Some(max_duration) = test_url.expected.max_duration_ms {
            if duration.as_millis() as u64 > max_duration {
                warnings.push(format!(
                    "Duration {} ms exceeds maximum {} ms",
                    duration.as_millis(),
                    max_duration
                ));
            }
        }

        // Check required content
        for required in &test_url.expected.should_contain {
            if !stdout.contains(required) && !stderr.contains(required) {
                warnings.push(format!("Missing required content: {}", required));
            }
        }

        // Check prohibited content
        for prohibited in &test_url.expected.should_not_contain {
            if stdout.contains(prohibited) || stderr.contains(prohibited) {
                warnings.push(format!("Contains prohibited content: {}", prohibited));
            }
        }

        // Analyze content
        metadata.insert(
            "has_json_structure".to_string(),
            serde_json::json!(stdout.trim_start().starts_with("{")),
        );
        metadata.insert(
            "line_count".to_string(),
            serde_json::json!(stdout.lines().count()),
        );
        metadata.insert("method".to_string(), serde_json::json!(method));
        metadata.insert("engine".to_string(), serde_json::json!(engine));

        Ok(TestResult {
            test_id,
            command: "extract".to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            url: test_url.url.clone(),
            success,
            exit_code: output.status.code().unwrap_or(-1),
            duration_ms: duration.as_millis() as u64,
            stdout_length: stdout.len(),
            stderr_length: stderr.len(),
            stdout_preview: Self::preview(&stdout, 500),
            stderr_preview: Self::preview(&stderr, 200),
            content_stored_at: content_path,
            error: if !success {
                Some(stderr.clone())
            } else {
                None
            },
            warnings,
            metadata,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Test search command
    pub fn test_search(&self, query: &str, limit: u32) -> Result<TestResult> {
        let test_id = format!("search_{}", query.replace(' ', "_"));
        let mut metadata = HashMap::new();

        let args = vec![
            "--query",
            query,
            "--limit",
            &limit.to_string(),
        ];

        let (output, duration) = self.run_command("search", &args)?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        metadata.insert("query".to_string(), serde_json::json!(query));
        metadata.insert("limit".to_string(), serde_json::json!(limit));

        Ok(TestResult {
            test_id,
            command: "search".to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            url: String::new(),
            success,
            exit_code: output.status.code().unwrap_or(-1),
            duration_ms: duration.as_millis() as u64,
            stdout_length: stdout.len(),
            stderr_length: stderr.len(),
            stdout_preview: Self::preview(&stdout, 500),
            stderr_preview: Self::preview(&stderr, 200),
            content_stored_at: None,
            error: if !success { Some(stderr) } else { None },
            warnings: Vec::new(),
            metadata,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Test crawl command
    pub fn test_crawl(&self, url: &str, depth: u32, max_pages: u32) -> Result<TestResult> {
        let test_id = format!("crawl_{}_d{}", url.replace("https://", "").replace('/', "_"), depth);
        let mut metadata = HashMap::new();

        let output_dir = self.output_dir.join(&test_id);
        fs::create_dir_all(&output_dir)?;

        let args = vec![
            "--url",
            url,
            "--depth",
            &depth.to_string(),
            "--max-pages",
            &max_pages.to_string(),
            "--output-dir",
            output_dir.to_str().unwrap(),
        ];

        let (output, duration) = self.run_command("crawl", &args)?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        metadata.insert("depth".to_string(), serde_json::json!(depth));
        metadata.insert("max_pages".to_string(), serde_json::json!(max_pages));

        Ok(TestResult {
            test_id,
            command: "crawl".to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            url: url.to_string(),
            success,
            exit_code: output.status.code().unwrap_or(-1),
            duration_ms: duration.as_millis() as u64,
            stdout_length: stdout.len(),
            stderr_length: stderr.len(),
            stdout_preview: Self::preview(&stdout, 500),
            stderr_preview: Self::preview(&stderr, 200),
            content_stored_at: Some(output_dir),
            error: if !success { Some(stderr) } else { None },
            warnings: Vec::new(),
            metadata,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Run comprehensive test suite
    pub fn run_test_suite(&self, test_urls: &[TestUrl]) -> Result<TestSession> {
        let session_id = format!("session_{}", chrono::Utc::now().timestamp());
        let start_time = chrono::Utc::now().to_rfc3339();
        let mut results = Vec::new();

        println!("\n🧪 Starting Comprehensive CLI Test Suite");
        println!("   Session ID: {}", session_id);
        println!("   Test URLs: {}", test_urls.len());

        // Test extraction with different methods and engines
        let methods = vec!["auto", "article", "full"];
        let engines = vec!["auto", "raw", "wasm"];

        for test_url in test_urls {
            println!("\n📍 Testing: {} ({})", test_url.id, test_url.category);
            println!("   URL: {}", test_url.url);

            for method in &methods {
                for engine in &engines {
                    print!("   ⚙️  extract --method {} --engine {} ... ", method, engine);

                    match self.test_extract(test_url, method, engine) {
                        Ok(result) => {
                            if result.success {
                                println!("✅ OK ({} ms, {} bytes)", result.duration_ms, result.stdout_length);
                            } else {
                                println!("❌ FAILED");
                            }
                            results.push(result);
                        }
                        Err(e) => {
                            println!("💥 ERROR: {}", e);
                        }
                    }
                }
            }
        }

        // Test search
        println!("\n🔍 Testing search commands...");
        let search_queries = vec![
            ("rust programming", 10),
            ("web scraping", 5),
            ("artificial intelligence", 15),
        ];

        for (query, limit) in search_queries {
            print!("   🔎 search --query \"{}\" --limit {} ... ", query, limit);
            match self.test_search(query, limit) {
                Ok(result) => {
                    if result.success {
                        println!("✅ OK ({} ms)", result.duration_ms);
                    } else {
                        println!("❌ FAILED");
                    }
                    results.push(result);
                }
                Err(e) => {
                    println!("💥 ERROR: {}", e);
                }
            }
        }

        // Calculate statistics
        let end_time = chrono::Utc::now().to_rfc3339();
        let passed_tests = results.iter().filter(|r| r.success).count();
        let failed_tests = results.iter().filter(|r| !r.success).count();
        let total_tests = results.len();
        let total_duration_ms: u64 = results.iter().map(|r| r.duration_ms).sum();

        let durations: Vec<u64> = results.iter().map(|r| r.duration_ms).collect();
        let avg_duration_ms = if !durations.is_empty() {
            durations.iter().sum::<u64>() as f64 / durations.len() as f64
        } else {
            0.0
        };
        let min_duration_ms = *durations.iter().min().unwrap_or(&0);
        let max_duration_ms = *durations.iter().max().unwrap_or(&0);
        let success_rate = if total_tests > 0 {
            (passed_tests as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        let mut commands_tested = HashMap::new();
        for result in &results {
            *commands_tested.entry(result.command.clone()).or_insert(0) += 1;
        }

        let summary_stats = SummaryStats {
            avg_duration_ms,
            min_duration_ms,
            max_duration_ms,
            success_rate,
            commands_tested,
        };

        let session = TestSession {
            session_id: session_id.clone(),
            start_time,
            end_time: Some(end_time),
            total_tests,
            passed_tests,
            failed_tests,
            total_duration_ms,
            results,
            summary_stats,
        };

        // Save session results
        let results_path = self.output_dir.join(format!("{}.json", session_id));
        let json = serde_json::to_string_pretty(&session)?;
        fs::write(&results_path, json)?;

        // Print summary
        self.print_summary(&session);

        Ok(session)
    }

    /// Print test session summary
    fn print_summary(&self, session: &TestSession) {
        println!("\n" + &"=".repeat(80));
        println!("📊 TEST SESSION SUMMARY");
        println!("=".repeat(80));
        println!("Session ID:      {}", session.session_id);
        println!("Total Tests:     {}", session.total_tests);
        println!("Passed:          {} ({:.1}%)", session.passed_tests, session.summary_stats.success_rate);
        println!("Failed:          {}", session.failed_tests);
        println!("Total Duration:  {} ms", session.total_duration_ms);
        println!("Avg Duration:    {:.1} ms", session.summary_stats.avg_duration_ms);
        println!("Min Duration:    {} ms", session.summary_stats.min_duration_ms);
        println!("Max Duration:    {} ms", session.summary_stats.max_duration_ms);
        println!("\nCommands Tested:");
        for (cmd, count) in &session.summary_stats.commands_tested {
            println!("  • {}: {} tests", cmd, count);
        }
        println!("=".repeat(80));
    }

    /// Create preview of content (truncated)
    fn preview(content: &str, max_len: usize) -> String {
        if content.len() <= max_len {
            content.to_string()
        } else {
            format!("{}... ({} total chars)", &content[..max_len], content.len())
        }
    }

    /// Compare two test sessions
    pub fn compare_sessions(
        &self,
        session1: &TestSession,
        session2: &TestSession,
    ) -> Result<()> {
        println!("\n🔍 COMPARING TEST SESSIONS");
        println!("Session 1: {} ({} tests)", session1.session_id, session1.total_tests);
        println!("Session 2: {} ({} tests)", session2.session_id, session2.total_tests);

        let mut regressions = Vec::new();
        let mut improvements = Vec::new();

        for result1 in &session1.results {
            if let Some(result2) = session2
                .results
                .iter()
                .find(|r| r.test_id == result1.test_id)
            {
                // Check for success/failure changes
                if result1.success && !result2.success {
                    regressions.push(format!(
                        "{}: Now FAILING (was passing)",
                        result1.test_id
                    ));
                } else if !result1.success && result2.success {
                    improvements.push(format!(
                        "{}: Now PASSING (was failing)",
                        result1.test_id
                    ));
                }

                // Check for significant performance changes
                let duration_change = result2.duration_ms as i64 - result1.duration_ms as i64;
                if duration_change.abs() > 1000 {
                    let change_type = if duration_change > 0 { "slower" } else { "faster" };
                    let msg = format!(
                        "{}: {} by {} ms ({} ms → {} ms)",
                        result1.test_id,
                        change_type,
                        duration_change.abs(),
                        result1.duration_ms,
                        result2.duration_ms
                    );
                    if duration_change > 0 {
                        regressions.push(msg);
                    } else {
                        improvements.push(msg);
                    }
                }
            }
        }

        if !regressions.is_empty() {
            println!("\n❌ REGRESSIONS ({}):", regressions.len());
            for regression in &regressions {
                println!("  • {}", regression);
            }
        }

        if !improvements.is_empty() {
            println!("\n✅ IMPROVEMENTS ({}):", improvements.len());
            for improvement in &improvements {
                println!("  • {}", improvement);
            }
        }

        if regressions.is_empty() && improvements.is_empty() {
            println!("\n✨ No significant differences detected!");
        }

        Ok(())
    }
}

/// Load test URLs from configuration file
pub fn load_test_urls(path: &PathBuf) -> Result<Vec<TestUrl>> {
    let content = fs::read_to_string(path).context("Failed to read test URLs file")?;
    let urls: Vec<TestUrl> =
        serde_json::from_str(&content).context("Failed to parse test URLs JSON")?;
    Ok(urls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_creation() {
        let harness = CliTestHarness::new(
            PathBuf::from("/tmp/test_output"),
            "riptide".to_string(),
        );
        assert_eq!(harness.binary_name, "riptide");
        assert!(harness.store_outputs);
    }

    #[test]
    fn test_preview_function() {
        let short_text = "Hello World";
        assert_eq!(CliTestHarness::preview(short_text, 100), short_text);

        let long_text = "A".repeat(1000);
        let preview = CliTestHarness::preview(&long_text, 50);
        assert!(preview.contains("... (1000 total chars)"));
    }
}
