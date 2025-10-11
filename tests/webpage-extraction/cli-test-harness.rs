use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestUrl {
    pub id: String,
    pub url: String,
    pub category: String,
    pub expected: HashMap<String, serde_json::Value>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestUrls {
    pub test_urls: Vec<TestUrl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub test_id: String,
    pub method: String,
    pub url: String,
    pub success: bool,
    pub duration_ms: u64,
    pub content_length: usize,
    pub error: Option<String>,
    pub warnings: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub content_preview: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSession {
    pub session_id: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub total_tests: usize,
    pub successful_tests: usize,
    pub failed_tests: usize,
    pub results: Vec<ExtractionResult>,
}

pub struct TestHarness {
    pub output_dir: PathBuf,
    pub binary_path: PathBuf,
}

impl TestHarness {
    pub fn new(output_dir: PathBuf, binary_path: PathBuf) -> Self {
        Self {
            output_dir,
            binary_path,
        }
    }

    pub async fn load_test_urls(&self, path: &PathBuf) -> Result<TestUrls> {
        let content = fs::read_to_string(path)
            .context("Failed to read test URLs file")?;
        let urls: TestUrls = serde_json::from_str(&content)
            .context("Failed to parse test URLs JSON")?;
        Ok(urls)
    }

    pub async fn run_extraction(
        &self,
        method: &str,
        url: &str,
        timeout_secs: u64,
    ) -> Result<(String, Duration)> {
        let start = Instant::now();

        let output = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            Command::new(&self.binary_path)
                .arg("extract")
                .arg("--method")
                .arg(method)
                .arg("--url")
                .arg(url)
                .output()
        )
        .await
        .context("Extraction command timed out")??;

        let duration = start.elapsed();

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok((stdout, duration))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(anyhow::anyhow!("Extraction failed: {}", stderr))
        }
    }

    pub async fn test_url(
        &self,
        test_url: &TestUrl,
        method: &str,
    ) -> ExtractionResult {
        let mut warnings = Vec::new();
        let mut metadata = HashMap::new();

        let result = self.run_extraction(method, &test_url.url, 30).await;

        match result {
            Ok((content, duration)) => {
                let content_length = content.len();
                let content_preview = if content.len() > 500 {
                    format!("{}... ({} chars)", &content[..500], content.len())
                } else {
                    content.clone()
                };

                // Validate expectations
                if let Some(min_length) = test_url.expected.get("min_content_length") {
                    if let Some(min) = min_length.as_u64() {
                        if content_length < min as usize {
                            warnings.push(format!(
                                "Content length {} below expected minimum {}",
                                content_length, min
                            ));
                        }
                    }
                }

                // Check for common content indicators
                metadata.insert("has_html_tags".to_string(), serde_json::json!(content.contains("<")));
                metadata.insert("has_json_structure".to_string(), serde_json::json!(content.trim_start().starts_with("{")));
                metadata.insert("line_count".to_string(), serde_json::json!(content.lines().count()));

                ExtractionResult {
                    test_id: test_url.id.clone(),
                    method: method.to_string(),
                    url: test_url.url.clone(),
                    success: true,
                    duration_ms: duration.as_millis() as u64,
                    content_length,
                    error: None,
                    warnings,
                    metadata,
                    content_preview,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }
            }
            Err(e) => {
                ExtractionResult {
                    test_id: test_url.id.clone(),
                    method: method.to_string(),
                    url: test_url.url.clone(),
                    success: false,
                    duration_ms: 0,
                    content_length: 0,
                    error: Some(e.to_string()),
                    warnings,
                    metadata,
                    content_preview: String::new(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }
            }
        }
    }

    pub async fn run_test_suite(
        &self,
        test_urls: &TestUrls,
        methods: &[String],
    ) -> Result<TestSession> {
        let session_id = format!("test-session-{}", chrono::Utc::now().timestamp());
        let start_time = chrono::Utc::now().to_rfc3339();

        let mut results = Vec::new();
        let total_tests = test_urls.test_urls.len() * methods.len();

        println!("🧪 Starting test suite: {} URLs × {} methods = {} tests",
            test_urls.test_urls.len(), methods.len(), total_tests);

        for test_url in &test_urls.test_urls {
            println!("\n📍 Testing URL: {} ({})", test_url.id, test_url.url);

            for method in methods {
                print!("  ⚙️  Method: {} ... ", method);
                let result = self.test_url(test_url, method).await;

                if result.success {
                    println!("✅ OK ({} ms, {} chars)",
                        result.duration_ms, result.content_length);
                } else {
                    println!("❌ FAILED: {}",
                        result.error.as_ref().unwrap_or(&"Unknown error".to_string()));
                }

                if !result.warnings.is_empty() {
                    for warning in &result.warnings {
                        println!("    ⚠️  {}", warning);
                    }
                }

                results.push(result);
            }
        }

        let successful_tests = results.iter().filter(|r| r.success).count();
        let failed_tests = results.iter().filter(|r| !r.success).count();

        let session = TestSession {
            session_id: session_id.clone(),
            start_time,
            end_time: Some(chrono::Utc::now().to_rfc3339()),
            total_tests,
            successful_tests,
            failed_tests,
            results,
        };

        // Save session results
        let results_path = self.output_dir.join(format!("{}.json", session_id));
        let json = serde_json::to_string_pretty(&session)?;
        fs::write(&results_path, json)?;

        println!("\n📊 Test Session Complete!");
        println!("   Total:   {}", total_tests);
        println!("   Success: {} ({:.1}%)",
            successful_tests,
            (successful_tests as f64 / total_tests as f64) * 100.0);
        println!("   Failed:  {} ({:.1}%)",
            failed_tests,
            (failed_tests as f64 / total_tests as f64) * 100.0);
        println!("   Results: {}", results_path.display());

        Ok(session)
    }

    pub async fn compare_results(
        &self,
        session1: &TestSession,
        session2: &TestSession,
    ) -> Result<()> {
        println!("\n🔍 Comparing Sessions:");
        println!("   Session 1: {}", session1.session_id);
        println!("   Session 2: {}", session2.session_id);

        let mut differences = Vec::new();

        for result1 in &session1.results {
            if let Some(result2) = session2.results.iter().find(|r|
                r.test_id == result1.test_id && r.method == result1.method
            ) {
                let content_diff = (result2.content_length as i64 - result1.content_length as i64).abs();
                let duration_diff = (result2.duration_ms as i64 - result1.duration_ms as i64).abs();

                if result1.success != result2.success || content_diff > 100 || duration_diff > 1000 {
                    differences.push(format!(
                        "{}/{}: success:{}->{}, content:{}->{}, duration:{}ms->{}ms",
                        result1.test_id, result1.method,
                        result1.success, result2.success,
                        result1.content_length, result2.content_length,
                        result1.duration_ms, result2.duration_ms
                    ));
                }
            }
        }

        if differences.is_empty() {
            println!("✅ No significant differences found!");
        } else {
            println!("⚠️  Found {} differences:", differences.len());
            for diff in differences {
                println!("   • {}", diff);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_urls() {
        let harness = TestHarness::new(
            PathBuf::from("/tmp"),
            PathBuf::from("./target/debug/eventmesh-cli")
        );

        // Test URL loading would go here
    }

    #[tokio::test]
    async fn test_extraction_timeout() {
        // Test timeout handling
    }

    #[tokio::test]
    async fn test_error_handling() {
        // Test various error scenarios
    }
}
